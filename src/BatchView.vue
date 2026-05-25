<script setup lang="ts">
import { ref, nextTick, onUnmounted, type Ref } from 'vue';
import { useStorage } from '@vueuse/core';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';

import { toastError, RULES, toast, anyFilter } from './common';
import { DEFAULT_RENDER_CONFIG, type ChartInfo, type RenderConfig, type RenderChart, type Preset, type FileDropEvent, type Task } from './model';
import router from './router';
import { AnsiUp } from 'ansi_up';
const ansi = new AnsiUp();

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import * as dialog from "@tauri-apps/plugin-dialog"

import { listen } from "@tauri-apps/api/event";

const form = ref<VForm>();
const charts = useStorage<RenderChart[]>('BatchView.ChartList', []);

const DEFAULT_PRESET: Preset = {
  name: t('default-preset'),
  key: 'default',
  config: DEFAULT_RENDER_CONFIG,
};

async function getPresets() {
  let result = [DEFAULT_PRESET];
  let pairs = (await invoke('get_presets')) as Record<string, RenderConfig>;
  for (let key of Object.keys(pairs).sort()) {
    result.push({
      name: key,
      key,
      config: pairs[key]!,
    });
  }
  return result;
}
const presets = ref([DEFAULT_PRESET]);
const preset = useStorage<Preset>('BatchView.Preset', DEFAULT_PRESET);
async function updatePresets() {
  presets.value = await getPresets();
  if (preset.value.key !== 'temp') {
    preset.value = presets.value.find((x) => x.key === preset.value.key) || presets.value[0];
  }
}
updatePresets();

const loadingChoosingChart = ref(false);
const loadingParsingChart = ref(false);
const loadingPreview = ref(false);
const loadingPostRender = ref(false);

const chartInfoDialog = ref(false);
const chartInfoSelect = ref(0);
const disSelectStartRender = useStorage<boolean>('BatchView.disSelectStartRender', true);
const removeStartRender = useStorage<boolean>('BatchView.removeStartRender', false);
const removeAfterRender = useStorage<boolean>('BatchView.removeAfterRender', false);
const autoChangeAspectRatio = useStorage<boolean>('BatchView.autoChangeAspectRatio', false);


async function chooseChart(folder?: boolean) {
  if (loadingChoosingChart.value) return;
  loadingChoosingChart.value = true;
  let file = folder
    ? await dialog.open({ directory: true, multiple: true })
    : await dialog.open({
        filters: [
          {
            name: t('choose.chart-file'),
            extensions: ['zip', 'pez'],
          },
          anyFilter(),
        ],
        multiple: true,
      });
  if (!file) {
    loadingChoosingChart.value = false;
    return;
  }

  // noexcept
  await loadCharts(file);

  loadingChoosingChart.value = false;
}

async function loadCharts(files: string[]) {
  loadingParsingChart.value = true;
  for (let file of files) {
    try {
      let chartInfo: ChartInfo = (await invoke('parse_chart', { path: file }));
      charts.value.push({
        id: charts.value.length,
        path: file,
        isSelect: true,
        taskId: null,
        output: '',
        status: { type: 'null' },
        chartInfo: chartInfo,
      });
    } catch (e) {
      toastError(e);
    }
  }
  loadingParsingChart.value = false;
}

const fileHovering = ref(false);
let files: string[] = [];
listen('tauri://drag-over', () => {
  if (loadingParsingChart.value) return;
  fileHovering.value = true
});
listen('tauri://drag-leave', () => {
  fileHovering.value = false
});
listen('tauri://drag-drop', async (event) => {
  fileHovering.value = false
  const newFiles = (event.payload as FileDropEvent).paths;
  files.push(...newFiles);
  if (loadingParsingChart.value) return;
  await loadCharts(files);
  files = [];
});

async function buildParams(chartPath: string, chartInfo: ChartInfo, config: RenderConfig) {
  checkInfo(chartInfo);
  return {
    path: chartPath,
    info: chartInfo,
    config,
  };
}

async function postRender(chart: RenderChart) {
  let config = preset.value.config;
  if (autoChangeAspectRatio.value) { applyAspectRatio(config.resolution, chart.chartInfo.aspectRatio); }
  let params = await buildParams(chart.path, chart.chartInfo, config);
  if (!params) return false;
  try {
    await invoke('post_render', { params });
    let tasks = await invoke<Task[]>('get_tasks');
    chart.taskId = tasks[0]!.id;
    chart.output = tasks[0]!.output;
  } catch (e) {
    toastError(e);
    return false;
  }
  if (disSelectStartRender.value) chart.isSelect = false;
  return true;
}

async function postSelectRender() {
  loadingPostRender.value = true;
  if (removeStartRender.value) {
    for (let i = charts.value.length - 1; i >= 0; i--) {
      if (!loadingPostRender.value) break;
      if (charts.value[i]!.isSelect) {
        await postRender(charts.value[i]!);
        charts.value.splice(i, 1);
      }
    }
  } else {
    for (let chart of charts.value) {
      if (!loadingPostRender.value) break;
      if (chart.isSelect) {
        await postRender(chart);
      }
    }
  }
  loadingPostRender.value = false;
}

function applyAspectRatio(resolution: number[], aspectRatio: number) {
  if (preset.value.key !== 'default') return;

  let h = resolution[1]!;

  if (aspectRatio < 1.0) {
    resolution = [h, h]
  } else {
    let w = Math.floor(h * aspectRatio);
    resolution = [w, h]
  }
}

function removeSelectChart() {
  for (let i = charts.value.length - 1; i >= 0; i--) {
    if (charts.value[i]!.isSelect) {
      charts.value.splice(i, 1)
    }
  }
}

async function previewChart(chart: RenderChart) {
  loadingPreview.value = true;
  try {
    let params = await buildParams(chart.path, chart.chartInfo, preset.value.config);
    if (!params) return false;
    await invoke('preview_chart', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  } finally {
    loadingPreview.value = false
  }
}

async function checkInfo(chartInfo: ChartInfo) {
  if (!(await form.value!.validate()).valid) {
    return false;
  }
  if (chartInfo.previewEnd) {
    if (chartInfo.previewEnd - chartInfo.previewStart > 15) {
      toast(t('error.preview-start-end-15s'), 'error');
      return false;
    }
  }
  if (chartInfo.previewEnd as string | null === '') {
    chartInfo.previewEnd = null;
  }
  if (!chartInfo.tip?.trim().length) chartInfo.tip = null;

  return true;
}

function selectAll() {
  charts.value.forEach(chart => {
    chart.isSelect = true
  })
}

function selectInvert() {
  charts.value.forEach(chart => {
    chart.isSelect = !chart.isSelect
  })
}

const configView = ref<typeof ConfigView>();
const presetDialog = ref(false);
async function editPreset() {
  presetDialog.value = true;
  await nextTick();
  await configView.value?.applyConfig(preset.value.config);
}

async function savePreset() {
  presetDialog.value = false;
  await updatePresets();
  let temp_preset: Preset = {
    name: t('temp-preset'),
    key: 'temp',
    config: await configView.value!.buildConfig(),
  };
  preset.value = temp_preset;
}

function sortChartsByKey(key: string) {
  switch (key) {
    case 'id':
      charts.value.sort((a, b) => a.id - b.id);
      break;
    case 'name':
      charts.value.sort((a, b) => a.chartInfo.name.localeCompare(b.chartInfo.name, undefined, { numeric: true, sensitivity: 'case', caseFirst: 'upper' }));
      break;
    case 'path':
      charts.value.sort((a, b) => a.path.localeCompare(b.path, undefined, { numeric: true, sensitivity: 'case', caseFirst: 'upper' }));
      break;
    case 'level':
      charts.value.sort((a, b) => a.chartInfo.level.localeCompare(b.chartInfo.level, undefined, { numeric: true, sensitivity: 'case', caseFirst: 'upper' }));
      break;
    case 'charter':
      charts.value.sort((a, b) => a.chartInfo.charter.localeCompare(b.chartInfo.charter, undefined, { numeric: true, sensitivity: 'case', caseFirst: 'upper' }));
      break;
  }
}

function sortChartsReverse() {
  charts.value.reverse();
}

const tasks = ref<Task[]>();

async function updateList() {
  if (!charts.value) return;
  tasks.value = await invoke<Task[]>('get_tasks');
  // console.log(tasks.value);
  for (let i = charts.value.length - 1; i >= 0; i--) {
    let chart = charts.value[i]!;
    let task = tasks.value.find((x) => x.id === chart.taskId);
    if (task) {
      chart.status = task.status;
      // console.log(chart.status);
    }
    if (removeAfterRender.value && chart.status.type === 'done') {
      charts.value.splice(i, 1);
    }
  }
}

async function cancelSelectTask() {
  for (let chart of charts.value) {
    if (chart.isSelect && chart.taskId !== null) {
      try {
        await invoke('cancel_task', { id: chart.taskId });
      } catch (e) {
        toastError(e);
      }
    }
  }
}

async function showOutputFolder() {
  try {
    await invoke('open_output_folder');
  } catch (e) {
    toastError(e);
  }
}

async function openFile(path: string) {
  try {
    await openPath(path);
  } catch (e) {
    toastError(e);
  }
}

async function showInFolder(path: string) {
  try {
    await revealItemInDir(path);
  } catch (e) {
    toastError(e);
  }
}

async function clearTasks() {
  loadingPostRender.value = false;
  charts.value = [];
  try {
    await invoke('clear_tasks');
  } catch (e) {
    toastError(e);
  }
  window.location.reload();
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));

const outputDialog = ref(false);
const outputDialogMessage = ref('');

function openOutputDialog(output: string) {
  outputDialogMessage.value = ansi.ansi_to_html(output);
  filteredOutputDialogMessage.value = outputDialogMessage.value;
  filter.value = [];
  outputDialog.value = true;
}

const filteredOutputDialogMessage = ref('');

const filterItems: string[] = ["INFO", "DEBUG", "WARN", "ERROR", "! INFO", "! DEBUG", "! WARN", "! ERROR"];
const filter: Ref<string[]> = ref([]);

function filterText(
  rawText: string,
  filters: string[]
): string {
  const include: string[] = [];
  const exclude: string[] = [];

  for (let f of filters) {
    f = f.trim();
    if (!f) continue;
    if (f.startsWith('!')) {
      const kw = f.slice(1).trim();
      if (kw) exclude.push(kw);
    } else {
      include.push(f);
    }
  }

  if (include.length === 0 && exclude.length === 0) {
    return rawText;
  }

  const resultLines = rawText
    .split(/\r?\n/)
    .filter(line => {
      if (include.length > 0) {
        const hitInclude = include.some(kw => line.includes(kw));
        if (!hitInclude) return false;
      }
      const hitExclude = exclude.some(kw => line.includes(kw));
      if (hitExclude) return false;

      return true;
    });

  return resultLines.join('\n');
}

</script>

<template>
  <v-card color="transparent" class="d-flex flex-column fade-in" width="100%" style="border-radius: 0px; box-shadow: none;">
    <v-toolbar color="transparent" class="px-1" style="position: sticky; top: 0px;">
      <v-menu :close-on-content-click="false">
        <template v-slot:activator="{ props }">
          <v-btn icon="mdi-menu" variant="text" v-bind="props"></v-btn>
        </template>
        <v-list>
          <v-list-item>
            <v-row no-gutters class="justify-center">
              <v-btn variant="text" @click="showOutputFolder">{{ t('task.show-folder') }}</v-btn>
            </v-row>
            <v-row no-gutters class="justify-center">
              <v-btn variant="text" @click="clearTasks">{{ t('clear-tasks') }}</v-btn>
            </v-row>
            <VDivider class="my-2"></VDivider>
            <v-row no-gutters>
              <v-checkbox :label="t('dis-select-start-render')" v-model="disSelectStartRender" ></v-checkbox>
            </v-row>
            <v-row no-gutters>
              <v-checkbox :label="t('remove-start-render')" v-model="removeStartRender"></v-checkbox>
            </v-row>
            <v-row no-gutters>
              <v-checkbox :label="t('remove-after-render')" v-model="removeAfterRender"></v-checkbox>
            </v-row>
            <v-row no-gutters>
              <v-checkbox :label="t('auto-change-aspect-ratio')" v-model="autoChangeAspectRatio"></v-checkbox>
            </v-row>
          </v-list-item>
        </v-list>
      </v-menu>
      <div v-if="charts.length === 0" class="d-flex align-center" style="flex: 1; margin-left: -70px;">
        <v-spacer />
        <v-btn class="mx-8" variant="tonal" style="width: 15em;" :title="t('select-or-drop')" @click="chooseChart(false)" prepend-icon="mdi-folder-zip">{{ t('choose.archive') }}</v-btn>
        <v-btn class="mx-8" variant="tonal" style="width: 15em;" :title="t('select-or-drop')" @click="chooseChart(true)" prepend-icon="mdi-folder">{{ t('choose.folder') }}</v-btn>
        <v-spacer />
      </div>
      <div v-else class="d-flex align-center" style="flex: 1">
        <v-combobox class="mx-2 mt-2" style="flex: 4;" :label="t('presets')" :items="presets" item-title="name" item-value="config" v-model="preset"></v-combobox>
        <v-btn class="" :title="t('edit-preset')" icon="mdi-pencil" @click="editPreset"></v-btn>
        <v-spacer />
        <v-btn class="mx-2" variant="tonal" @click="selectAll" >{{ t('select-all') }}</v-btn>
        <v-btn class="mx-2" variant="tonal" @click="selectInvert" >{{ t('select-invert') }}</v-btn>
        <v-btn class="mx-2" variant="tonal" @click="removeSelectChart" >{{ t('remove-select') }}</v-btn>
        <v-btn class="mx-2" variant="tonal" @click="cancelSelectTask" >{{ t('cancel-select') }}</v-btn>
        <v-btn class="mx-2" variant="tonal" @click="postSelectRender" :loading="loadingPostRender">{{ t('post-select-render') }}</v-btn>
      </div>
    </v-toolbar>
    <div class="flex-grow-1 overflow-y-auto" style="font-size: 0.9em;">
      <v-row no-gutters class="d-flex align-center batch-title">
        <v-col cols="1" class="justify-center text-center" style="cursor: pointer; max-width: 60px;" @click="sortChartsByKey('id')" @contextmenu="sortChartsReverse" :title="t('sort-tip')">({{ charts.length }})</v-col>
        <v-col cols="3" style="cursor: pointer;" @click="sortChartsByKey('name')" @contextmenu="sortChartsReverse" :title="t('sort-tip')">{{ t('info.name') }}</v-col>
        <v-col cols="2" style="cursor: pointer;" @click="sortChartsByKey('level')" @contextmenu="sortChartsReverse" :title="t('sort-tip')">{{ t('info.level') }}</v-col>
        <v-col cols="2" style="cursor: pointer;" @click="sortChartsByKey('charter')" @contextmenu="sortChartsReverse" :title="t('sort-tip')">{{ t('info.charter') }}</v-col>
        <v-col style="cursor: pointer;" @click="sortChartsByKey('path')" @contextmenu="sortChartsReverse" :title="t('sort-tip')">{{ t('info.chart') }}</v-col>
        <v-col cols="1" class="d-flex justify-center" style="min-width: 80px; max-width: 100px;">{{ t('chart-info') }}</v-col>
        <v-col cols="1" class="d-flex justify-center" style="min-width: 80px; max-width: 100px;">{{ t('preview') }}</v-col>
      </v-row>
      <VDivider></VDivider>
      <v-virtual-scroll
        :items="charts"
        height="calc(100vh - 170px)"
        item-key="id"
        v-if="!loadingParsingChart"
        style="overflow-y: scroll;"
      >
        <template v-slot:default="{ item }">
          <v-row no-gutters class="d-flex align-center" style="height: 5em;">
            <v-col cols="1" class="d-flex justify-center" style="max-width: 60px;"><v-checkbox class="mt-2 ml-n1" v-model="item.isSelect"></v-checkbox></v-col>
            <v-col cols="3" style="white-space: nowrap; text-overflow: ellipsis; overflow: hidden; padding-right: 10px; cursor: pointer;"
              :title="`${item.chartInfo.name}\n${t('chart-open-tip')}`"
              @click="router.push({ name: 'render', query: { chart: item.path, info: JSON.stringify(item.chartInfo), config: JSON.stringify(preset.config) } })"
              @contextmenu="showInFolder(item.path)"
            >{{ item.chartInfo.name }}</v-col>

            <v-col cols="2" v-if="item.status.type === 'pending'">{{ t('task.pending') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'loading'">{{ t('task.loading') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'mixing'">{{ t('task.mixing') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'mixing_sfx'">{{ t('task.mixing') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'rendering'">{{ (item.status.progress * 100).toFixed(2) }}%</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'done'">{{ t('task.done') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'canceled'">{{ t('task.canceled') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'failed'">{{ t('task.failed') }}</v-col>
            <v-col cols="2" v-else style="white-space: nowrap; text-overflow: ellipsis; overflow: hidden; padding-right: 10px;" :title="item.chartInfo.level">{{ item.chartInfo.level }}</v-col>

            <v-col cols="2" v-if="item.status.type === 'pending'">-</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'loading'">-</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'mixing'">-</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'mixing_sfx'">-</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'rendering'">{{ item.status.fps }} FPS</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'done'" style="cursor: pointer;" @click="openFile(item.output)">{{ t('task.open-file') }}</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'canceled'">-</v-col>
            <v-col cols="2" v-else-if="item.status.type === 'failed'">-</v-col>
            <v-col cols="2" v-else style="white-space: nowrap; text-overflow: ellipsis; overflow: hidden; padding-right: 10px;" :title="item.chartInfo.charter">{{ item.chartInfo.charter }}</v-col>

            <v-col v-if="item.status.type === 'pending'" style="cursor: pointer;" @contextmenu="showInFolder(item.path)">-</v-col>
            <v-col v-else-if="item.status.type === 'loading'" style="cursor: pointer;" @contextmenu="showInFolder(item.path)">-</v-col>
            <v-col v-else-if="item.status.type === 'mixing'" style="cursor: pointer;" @contextmenu="showInFolder(item.path)">-</v-col>
            <v-col v-else-if="item.status.type === 'mixing_sfx'" style="cursor: pointer;" @contextmenu="showInFolder(item.path)">-</v-col>
            <v-col v-else-if="item.status.type === 'rendering'" style="cursor: pointer;" @contextmenu="showInFolder(item.path)">{{ item.status.estimate.toFixed(0) }} s</v-col>
            <v-col v-else-if="item.status.type === 'done'" style="cursor: pointer;" @click="openOutputDialog(item.status.output)" @contextmenu="showInFolder(item.path)">{{ t('task.show-output') }}</v-col>
            <v-col v-else-if="item.status.type === 'canceled'" style="cursor: pointer;" @click="openOutputDialog(item.status.output)" @contextmenu="showInFolder(item.path)">{{ t('task.show-output') }}</v-col>
            <v-col v-else-if="item.status.type === 'failed'" style="cursor: pointer;" @click="openOutputDialog(item.status.output)" @contextmenu="showInFolder(item.path)">{{ t('task.show-output') }}</v-col>
            <v-col v-else style="white-space: nowrap; text-overflow: ellipsis; overflow: hidden; padding-right: 10px; cursor: pointer;" @click="showInFolder(item.path)" @contextmenu="showInFolder(item.path)" :title="`${item.path}\n${t('file-open-tip')}`">{{ item.path }}</v-col>

            <v-col cols="1" class="d-flex justify-center" style="min-width: 80px; max-width: 100px;"><v-btn variant="tonal" @click="chartInfoSelect = item.id; chartInfoDialog = true">{{ t('edit') }}</v-btn></v-col>
            <v-col cols="1" class="d-flex justify-center" style="min-width: 80px; max-width: 100px;"><v-btn variant="tonal" :loading="loadingPreview" @click="previewChart(item)">{{ t('preview') }}</v-btn></v-col>
          </v-row>
        </template>
      </v-virtual-scroll>
      <v-overlay v-model="loadingParsingChart" class="align-center justify-center" persistent noClickAnimation :close-on-content-click="false">
        <v-progress-circular indeterminate style="filter: none;"> </v-progress-circular>
      </v-overlay>
    </div>
  </v-card>

  <v-dialog v-model="chartInfoDialog" width="auto" class="log-card-bg">
    <v-card class="log-card-only-window" style="background: rgba(var(--v-theme-dialog), 0.4) !important;">
      <v-card-title v-t="'chart-info'"> </v-card-title>
      <v-card-text>

        <v-form v-if="charts[chartInfoSelect]" validateOn="eager">
          <v-row>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.name')" v-model="charts[chartInfoSelect]!.chartInfo.name"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.charter')" v-model="charts[chartInfoSelect]!.chartInfo.charter"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.composer')" v-model="charts[chartInfoSelect]!.chartInfo.composer"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.illustrator')" v-model="charts[chartInfoSelect]!.chartInfo.illustrator"></v-text-field>
            </v-col>
          </v-row>

          <v-row>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.positiveOrZero, RULES.notZero]" :label="t('info.aspectRatio')"
              v-model="charts[chartInfoSelect]!.chartInfo.aspectRatio" @update:modelValue="charts[chartInfoSelect]!.chartInfo.aspectRatio = parseFloat($event)"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.level')" v-model="charts[chartInfoSelect]!.chartInfo.level"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.notEmpty]" :label="t('info.offset')"
              v-model="charts[chartInfoSelect]!.chartInfo.offset" @update:modelValue="charts[chartInfoSelect]!.chartInfo.offset = parseFloat($event)"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.less10000, RULES.notEmpty]" :label="t('info.lineLength')"
              v-model="charts[chartInfoSelect]!.chartInfo.lineLength" @update:modelValue="charts[chartInfoSelect]!.chartInfo.lineLength = parseFloat($event)"></v-text-field>
            </v-col>
          </v-row>

          <v-row>
            <v-col cols="3">
              <v-text-field :label="t('info.score-total')" type="number" :rules="[RULES.less4000000000]"
              v-model="charts[chartInfoSelect]!.chartInfo.scoreTotal" @update:modelValue="charts[chartInfoSelect]!.chartInfo.scoreTotal = parseInt($event)"></v-text-field>
            </v-col>
            <v-col cols="6">
              <v-slider class="my-3" :label="t('info.backgroundDim')" thumb-label="always" :min="0" :max="1" :step="0.05" color="btn" v-model="charts[chartInfoSelect]!.chartInfo.backgroundDim"> </v-slider>
            </v-col>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.positive]" :label="t('info.hold-particle-interval-ratio')" v-model="charts[chartInfoSelect]!.chartInfo.holdParticleIntervalRatio" @update:modelValue="charts[chartInfoSelect]!.chartInfo.holdParticleIntervalRatio = parseFloat($event)"></v-text-field>
            </v-col>
          </v-row>

          <v-row>
            <v-col cols="9">
              <v-text-field type="text" class="" :label="t('info.tip')" v-model="charts[chartInfoSelect]!.chartInfo.tip"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-switch class="d-flex justify-center ml-n2" v-model="charts[chartInfoSelect]!.chartInfo.holdPartialCover" :label="t('info.hold-partial-cover')" color="btn" :title="t('info.hold-partial-cover-tip')"></v-switch>
            </v-col>
          </v-row>

          <v-row>
            <v-col cols="3">
              <v-switch class="d-flex justify-center ml-n2" :label="t('info.force-aspect-ratio')" color="btn" v-model="charts[chartInfoSelect]!.chartInfo.forceAspectRatio"></v-switch>
            </v-col>
            <v-col cols="3">
              <v-switch class="d-flex justify-center ml-n2" v-model="charts[chartInfoSelect]!.chartInfo.noteUniformScale" :label="t('info.note-uniform-scale')" color="btn" :title="t('info.note-uniform-scale-tip')"></v-switch>
            </v-col>
            <v-col cols="3">
              <v-switch class="d-flex justify-center ml-n2" :label="t('info.fold-animation')" color="btn" v-model="charts[chartInfoSelect]!.chartInfo.foldAnimation"></v-switch>
            </v-col>
            <v-col cols="3">
              <v-switch class="d-flex justify-center ml-n2" v-model="charts[chartInfoSelect]!.chartInfo.negativeLengthHold" :label="t('info.negative-length-hold')" color="btn"></v-switch>
            </v-col>
          </v-row>

        </v-form>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn class="hover-scale" variant="text" @click="chartInfoDialog = false;" v-t="'close'"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="presetDialog" width="850px" class="log-card-bg">
    <v-card class="log-card-only-window" style="background: rgba(var(--v-theme-dialog), 0.4) !important;">
      <v-card-title v-t="'presets'"> </v-card-title>
      <v-card-text>
        <template v-slot>
          <ConfigView ref="configView"></ConfigView>
        </template>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn class="hover-scale" variant="text" @click="savePreset" v-t="'save'"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="outputDialog" class="log-card-bg">
    <v-card class="log-card-window">
      <v-card-title style="margin-bottom: -16px;" v-t="'output'"></v-card-title>
      <v-card-text style="padding-bottom: 0px;">
        <div
          class="block whitespace-pre overflow-auto log-card-msg user-select"
          style="height: calc(100vh - 240px);"
          v-html="filteredOutputDialogMessage"
        ></div>
        <v-combobox
          class="mt-4"
          variant="outlined"
          v-model="filter"
          :items="filterItems"
          clearable
          multiple
          placeholder="Filter (comma separated)"
          @update:model-value="(val: string[]) => {
            filteredOutputDialogMessage = filterText(outputDialogMessage, val);
          }"
        ></v-combobox>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn class="hover-scale" variant="text" @click="outputDialog = false" v-t="'close'"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-overlay v-model="fileHovering" contained class="align-center justify-center drop-zone-overlay" persistent :close-on-content-click="false">
    <div class="drop-pulse">
      <h1 v-t="'choose.drop'"></h1>
    </div>
  </v-overlay>

</template>

<style scoped>

.v-progress-linear,
.v-progress-linear__determinate {
  transition: none;
}

.batch-title {
  background-color: rgba(var(--v-theme-container), 0.05);
  height: 40px;
  padding-right: 8px;
}

.v-text-field :deep(.v-field--focused) {
  border-color: #6366f1 !important;
  box-shadow: 0 0 0 2px rgb(99 102 241 / 0.2);
}

h2 {
  font-weight: 600;
  letter-spacing: -0.025em;
  background: linear-gradient(45deg, #3b82f6, #6366f1);
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

:deep(span) {
  user-select: text;
}

:deep(.v-stepper-header__item) .v-stepper-header__title {
  font-weight: 500;
  color: #64748b;
}

:deep(.v-stepper-header__item--active) .v-stepper-header__title {
  color: #6366f1;
  font-weight: 600;
}

::v-deep(.v-stepper-header) {
  box-shadow: none;
}

::v-deep(.v-stepper-window) {
  margin: 0rem 1.5rem 0.5rem 1.5rem;
}

::v-deep(.v-window__container .v-stepper-window-item) {
  transition: 0.5s cubic-bezier(0.2, 0.8, 0.25, 1);
}

.drop-pulse {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.05); }
  100% { transform: scale(1); }
}

:deep(.v-slider__thumb) {
  background: #6366f1 !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2) !important;
}

:deep(.v-slider__track-fill) {
  background: linear-gradient(90deg, #6366f1, #8b5cf6) !important;
}

</style>
