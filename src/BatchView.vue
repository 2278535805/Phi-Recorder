<i18n>
en:
  prev-step: Previous
  next-step: Next

  choose:
    chart: Choose chart
    archive: Archive
    folder: Folder
    can-also-drop: You can also drag and drop the file to here
    select-or-drop: Select or drag and drop files
    drop: DROP CHART HERE
    filter-name: Chart file
    select-all: Select All
    select-invert: Invert
    cancel-select: Cancel
    remove-select: Remove
    dis-select-start-render: Deselect after rendering is initiated
    remove-start-render: Remove after rendering is initiated
    remove-after-render: Remove after rendering is completed
    post-select-render: Render
    auto-change-aspect-ratio: Auto Change Aspect Ratio

  info:
    name: Chart name
    difficulty: Difficulty
    level: Display Difficulty
    charter: Chart Design
    composer: Composer
    illustrator: Illustrator

    chart: Chart file
    music: Music file
    illustration: illustration File

    previewStart: Preview Start Time
    previewEnd: Preview End Time

    aspectRatio: Aspect Ratio
    backgroundDim: Background Dim
    lineLength: Line Length
    offset: Offset
    tip: Tip
    tip-placeholder: Leave empty to choose randomly
    tags: Tags
    tag-editor: Tag Editor
    tag-list: Regular, Troll, Plain, Visual

    intro: Introduction

    holdPartialCover: Hold Tail Cover

  error:
    preview-start-end-15s: Preview time cannot be greater than 15 seconds

  width: Width
  height: Height

  file:
    title: File
    chart: Chart file (empty for default)
    music: Music (empty for default)
    illustration: Illustration (empty for default)

  tweakoffset: Tweak Offset
  more: More
  chart-info: Chart Info
  preview: Preview
  render: Render
  play: Play
  edit: Edit
  sort-tip: Click to sort, right-click to reverse

  render-started: Rendering has started!
  see-tasks: See tasks
  
  confirm: Confirm
  close: Close
  save: Save
  save-success: Saved successfully
  read-success: Read successfully
  save-info: Save Info
  read-info: Read Info

  presets: Presets
  default-preset: Default
  edit-preset: Edit Preset
  temp-preset: (Edited)

  task:
    pending: Pending
    loading: Loading
    mixing: Mixing
    done: Completed
    canceled: Canceled
    failed: Failed
    output: Output
    show-output: Show Output
    show-folder: Open Output Folder
    open-file: Open File

zh-CN:
  prev-step: 上一步
  next-step: 下一步

  choose:
    chart: 选择谱面
    archive: 压缩包
    folder: 文件夹
    can-also-drop: 可拖放谱面至此处
    select-or-drop: 选择或拖放谱面
    drop: 拖放谱面至此处
    filter-name: 谱面文件
    select-all: 全选
    select-invert: 反选
    cancel-select: 取消
    remove-select: 移除
    dis-select-start-render: 发起渲染后取消选择
    remove-start-render: 发起渲染后移除
    remove-after-render: 渲染完成后移除
    post-select-render: 渲染
    auto-change-aspect-ratio: 自动调整宽高比

  info:
    name: 谱面名
    difficulty: 难度
    level: 显示难度
    charter: 谱面设计
    composer: 音乐制作
    illustrator: 插画设计

    chart: 谱面文件
    music: 音乐文件
    illustration: 曲绘文件

    previewStart: 预览开始时间
    previewEnd: 预览结束时间

    aspectRatio: 宽高比
    backgroundDim: 背景暗淡
    lineLength: 判定线长度
    offset: 偏移
    tip: Tip
    tip-placeholder: 留空则随机选择
    tags: 标签
    tag-editor: 标签编辑器
    tag-list: 常规,整活,纯配置,观赏

    intro: 简介

    holdPartialCover: Hold 尾部遮罩

  error:
    preview-start-end-15s: 预览时间不能大于15秒

  width: 宽
  height: 高

  tweakoffset: 调整延时
  more: 更多
  chart-info: 谱面信息
  preview: 预览
  render: 渲染
  play: 游玩
  edit: 编辑
  sort-tip: 点击排序 右键反转

  render-started: 视频已开始渲染!
  see-tasks: 查看任务列表

  confirm: 确定
  close: 关闭
  save: 保存
  save-success: 保存成功
  read-success: 读取成功
  save-info: 保存信息
  read-info: 读取信息

  presets: 预设配置
  default-preset: 默认
  edit-preset: 编辑预设
  temp-preset: (已编辑)

  task:
    pending: 待处理
    loading: 加载中
    mixing: 混音中
    done: 完成
    canceled: 取消
    failed: 失败
    output: 输出
    show-output: 查看输出
    show-folder: 打开输出文件夹
    open-file: 打开文件

</i18n>

<script setup lang="ts">
import { ref, nextTick, onUnmounted, watch } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';

import { toastError, RULES, toast, anyFilter } from './common';
import { DEFAULT_CONFIG, type ChartInfo, type RenderConfig, type RenderChart, type Preset, type FileDropEvent, type Task } from './model';

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import * as dialog from "@tauri-apps/plugin-dialog"

import { listen } from "@tauri-apps/api/event";

const charts = ref<RenderChart[]>([]);
try {
  let raw = localStorage.getItem('batchViewChartList');
  if (raw) {
    let val: RenderChart[] = JSON.parse(raw)
    charts.value = val;
  }
} catch(e) {
  localStorage.removeItem('batchViewChartList')
  toastError(e);
}

watch(charts, (val) => {
  localStorage.setItem('batchViewChartList', JSON.stringify(val))
}, {
  deep: true,
  immediate: false
})

const loadingChoosingChart = ref(false);
const loadingParsingChart = ref(false);
const loadingPreview = ref(false);
const loadingPostRender = ref(false);

const chartInfoDialog = ref(false);
const chartInfoSelect = ref(0);
const disSelectStartRender = ref(true);
const removeStartRender = ref(false);
const removeAfterRender = ref(false);
const autoChangeAspectRatio = ref(false);

async function chooseChart(folder?: boolean) {
  if (loadingChoosingChart.value) return;
  loadingChoosingChart.value = true;
  let file = folder
    ? await dialog.open({ directory: true, multiple: true })
    : await dialog.open({
        filters: [
          {
            name: t('choose.filter-name'),
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
        isSelect: false,
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

listen('tauri://drag-over', () => (fileHovering.value = true));
listen('tauri://drag-leave', () => (fileHovering.value = false));
listen('tauri://drag-drop', async (event) => {
  fileHovering.value = false;
  const files = (event.payload as FileDropEvent).paths;
  await loadCharts(files);
});

const form = ref<VForm>();

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
  if (autoChangeAspectRatio.value) {
    applyAspectRatio(config.resolution, chart.chartInfo.aspectRatio);
  }
  let params = await buildParams(chart.path, chart.chartInfo, config);
  if (!params) return false;
  try {
    await invoke('post_render', { params });
  } catch (e) {
    toastError(e);
    return false;
  }
  if (removeStartRender.value) {
    charts.value.splice(charts.value.indexOf(chart), 1);
  }
  let tasks = await invoke<Task[]>('get_tasks');
  chart.taskId = tasks[0].id;
  chart.output = tasks[0].output;
  if (disSelectStartRender.value) chart.isSelect = false;
  return true;
}

async function postSelectRender() {
  loadingPostRender.value = true;
  for (let chart of charts.value) {
    if (chart.isSelect) {
      await postRender(chart);
    }
  }
  loadingPostRender.value = false;
}

function applyAspectRatio(resolution: number[], aspectRatio: number) {
  if (preset.value.key !== 'default') return;

  let h = resolution[1];

  if (aspectRatio < 1.0) {
    resolution = [h, h]
  } else {
    let w = Math.floor(h * aspectRatio);
    resolution = [w, h]
  }
}

function removeSelectChart() {
  for (let i = charts.value.length - 1; i >= 0; i--) {
    if (charts.value[i].isSelect) {
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

const DEFAULT_PRESET: Preset = {
  name: t('default-preset'),
  key: 'default',
  config: DEFAULT_CONFIG,
};

async function getPresets() {
  let result = [DEFAULT_PRESET];
  let pairs = (await invoke('get_presets')) as Record<string, RenderConfig>;
  for (let key of Object.keys(pairs).sort()) {
    result.push({
      name: key,
      key,
      config: pairs[key],
    });
  }
  return result;
}
const presets = ref([DEFAULT_PRESET]);
const preset = ref(DEFAULT_PRESET);
async function updatePresets() {
  presets.value = await getPresets();
  preset.value = presets.value.find((x) => x.key === preset.value.key) || presets.value[0];
}
updatePresets();

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
    let chart = charts.value[i];
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
    await invoke('open_file', { path });
  } catch (e) {
    toastError(e);
  }
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));

const outputDialog = ref(false),
  outputDialogMessage = ref('');

</script>

<template>
  <v-card color="transparent" class="d-flex flex-column fade-in" width="100%" style="border-radius: 0px; box-shadow: none;">
    <v-toolbar v-if="charts.length === 0" color="transparent" class="px-1" style="position: sticky; top: 0px;">
      <v-spacer />
      <v-btn class="mx-8" variant="tonal" style="width: 15em;" :title="t('choose.select-or-drop')" @click="chooseChart(false)" prepend-icon="mdi-folder-zip">{{ t('choose.archive') }}</v-btn>
      <v-btn class="mx-8" variant="tonal" style="width: 15em;" :title="t('choose.select-or-drop')" @click="chooseChart(true)" prepend-icon="mdi-folder">{{ t('choose.folder') }}</v-btn>
      <v-spacer />
    </v-toolbar>
    <v-toolbar v-else color="transparent" class="px-1" style="position: sticky; top: 0px;">
      <v-menu>
        <template v-slot:activator="{ props }">
          <v-btn icon="mdi-menu" variant="text" v-bind="props"></v-btn>
        </template>
        <v-list>
          <v-list-item class="mr-2">
            <v-btn class="mx-2" variant="text" @click="showOutputFolder">{{ t('task.show-folder') }}</v-btn>
            <v-checkbox :label="t('choose.dis-select-start-render')" v-model="disSelectStartRender" ></v-checkbox>
            <v-checkbox :label="t('choose.remove-start-render')" v-model="removeStartRender"></v-checkbox>
            <v-checkbox :label="t('choose.remove-after-render')" v-model="removeAfterRender"></v-checkbox>
            <v-checkbox :label="t('choose.auto-change-aspect-ratio')" v-model="autoChangeAspectRatio"></v-checkbox>
          </v-list-item>
        </v-list>
      </v-menu>
      <v-combobox class="mx-2 mt-2" style="flex: 4;" :label="t('presets')" :items="presets" item-title="name" item-value="config" v-model="preset"></v-combobox>
      <v-btn class="" :title="t('edit-preset')" icon="mdi-pencil" @click="editPreset"></v-btn>
      <v-spacer />
      <v-btn class="mx-2" variant="tonal" @click="selectAll" >{{ t('choose.select-all') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="selectInvert" >{{ t('choose.select-invert') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="removeSelectChart" >{{ t('choose.remove-select') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="cancelSelectTask" >{{ t('choose.cancel-select') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="postSelectRender" :loading="loadingPostRender">{{ t('choose.post-select-render') }}</v-btn>
    </v-toolbar>
    <div class="flex-grow-1 overflow-y-auto">
      <v-table fixed-header density="compact" style="position: absolute; top: 64px; left: 0px; right: 0px; bottom: 0px; background-color: transparent;">
        <thead>
          <tr :title="t('sort-tip')" @contextmenu="sortChartsReverse">
            <th class="text-center" @click="sortChartsByKey('id')" style="width: 3.3em; padding-left: 1.4em;">({{ charts.length }})</th>
            <th class="text-left" @click="sortChartsByKey('name')" style="min-width: 5em;">{{ t('info.name') }}</th>
            <th class="text-left" @click="sortChartsByKey('level')" style="min-width: 5em;">{{ t('info.level') }}</th>
            <th class="text-left" @click="sortChartsByKey('charter')" style="min-width: 5em;">{{ t('info.charter') }}</th>
            <th class="text-left" @click="sortChartsByKey('path')" style="max-width: 25%;">{{ t('info.chart') }}</th>
            <th class="text-center" style="width: 7em;">{{ t('chart-info') }}</th>
            <th class="text-center" style="width: 5em;">{{ t('preview') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="chart in charts" :key="chart.id">
            <td><v-checkbox class="mt-2 ml-n1" v-model="chart.isSelect"></v-checkbox></td>
            <td style="max-width: 12em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.chartInfo.name">{{ chart.chartInfo.name }}</td>

            <td v-if="chart.status.type === 'pending'">{{ t('task.pending') }}</td>
            <td v-else-if="chart.status.type === 'loading'">{{ t('task.loading') }}</td>
            <td v-else-if="chart.status.type === 'mixing'">{{ t('task.mixing') }}</td>
            <td v-else-if="chart.status.type === 'rendering'">{{ (chart.status.progress * 100).toFixed(2) }}%</td>
            <td v-else-if="chart.status.type === 'done'">{{ t('task.done') }}</td>
            <td v-else-if="chart.status.type === 'canceled'">{{ t('task.canceled') }}</td>
            <td v-else-if="chart.status.type === 'failed'">{{ t('task.failed') }}</td>
            <td v-else style="max-width: 8em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.chartInfo.level">{{ chart.chartInfo.level }}</td>

            <td v-if="chart.status.type === 'pending'">-</td>
            <td v-else-if="chart.status.type === 'loading'">-</td>
            <td v-else-if="chart.status.type === 'mixing'">-</td>
            <td v-else-if="chart.status.type === 'rendering'">{{ chart.status.fps }} FPS</td>
            <td v-else-if="chart.status.type === 'done'" @click="openFile(chart.output)">{{ t('task.open-file') }}</td>
            <td v-else-if="chart.status.type === 'canceled'">-</td>
            <td v-else-if="chart.status.type === 'failed'">-</td>
            <td v-else style="max-width: 8em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.chartInfo.charter">{{ chart.chartInfo.charter }}</td>

            <td v-if="chart.status.type === 'pending'">-</td>
            <td v-else-if="chart.status.type === 'loading'">-</td>
            <td v-else-if="chart.status.type === 'mixing'">-</td>
            <td v-else-if="chart.status.type === 'rendering'">{{ chart.status.estimate.toFixed(0) }} s</td>
            <td v-else-if="chart.status.type === 'done'" @click="outputDialogMessage = chart.status.output; outputDialog = true;">{{ t('task.show-output') }}</td>
            <td v-else-if="chart.status.type === 'canceled'" @click="outputDialogMessage = chart.status.output; outputDialog = true;">{{ t('task.show-output') }}</td>
            <td v-else-if="chart.status.type === 'failed'" @click="outputDialogMessage = chart.status.output; outputDialog = true;">{{ t('task.show-output') }}</td>
            <td v-else style="max-width: 11em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.path">{{ chart.path }}</td>

            <td><v-btn variant="tonal" @click="chartInfoSelect = chart.id; chartInfoDialog = true">{{ t('edit') }}</v-btn></td>
            <td><v-btn variant="tonal" :loading="loadingPreview" @click="previewChart(chart)">{{ t('preview') }}</v-btn></td>
          </tr>
        </tbody>
      </v-table>
      <v-overlay v-model="loadingParsingChart" contained class="align-center justify-center" persistent scroll-strategy="none" :close-on-content-click="false">
          <v-progress-circular indeterminate> </v-progress-circular>
        </v-overlay>
    </div>
  </v-card>

  <v-dialog v-model="chartInfoDialog" width="auto" class="log-card-bg">
    <v-card class="log-card-only-window">
      <v-card-title v-t="'chart-info'"> </v-card-title>
      <v-card-text>

        <v-form v-if="charts[chartInfoSelect]" validateOn="eager">
          <v-row>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.name')" v-model="charts[chartInfoSelect].chartInfo.name"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.charter')" v-model="charts[chartInfoSelect].chartInfo.charter"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.composer')" v-model="charts[chartInfoSelect].chartInfo.composer"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.illustrator')" v-model="charts[chartInfoSelect].chartInfo.illustrator"></v-text-field>
            </v-col>
          </v-row>

          <v-row>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.positive, RULES.nonZero]" :label="t('info.aspectRatio')"
              v-model="charts[chartInfoSelect].chartInfo.aspectRatio" @update:modelValue="charts[chartInfoSelect].chartInfo.aspectRatio = parseFloat($event)"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="text" class="" :label="t('info.level')" v-model="charts[chartInfoSelect].chartInfo.level"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.non_empty]" :label="t('info.offset')"
              v-model="charts[chartInfoSelect].chartInfo.offset" @update:modelValue="charts[chartInfoSelect].chartInfo.offset = parseFloat($event)"></v-text-field>
            </v-col>
            <v-col cols="3">
              <v-text-field type="number" class="" :rules="[RULES.positive10000, RULES.non_empty]" :label="t('info.lineLength')"
              v-model="charts[chartInfoSelect].chartInfo.lineLength" @update:modelValue="charts[chartInfoSelect].chartInfo.lineLength = parseFloat($event)"></v-text-field>
            </v-col>
          </v-row>

          <v-row>
            <v-col cols="6">
              <v-text-field type="text" class="" :label="t('info.tip')" v-model="charts[chartInfoSelect].chartInfo.tip"></v-text-field>
            </v-col>
            <v-col cols="6">
              <v-slider class="my-3" :label="t('info.backgroundDim')" thumb-label="always" :min="0" :max="1" :step="0.05" v-model="charts[chartInfoSelect].chartInfo.backgroundDim"> </v-slider>
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

  <v-dialog v-model="outputDialog" width="auto" min-width="400px" class="log-card-bg">
    <v-card class="log-card-window">
      <v-card-title v-t="'task.output'"> </v-card-title>
      <v-card-text>
        <div class="block whitespace-pre overflow-auto log-card-msg" style="max-height: 60vh">{{ outputDialogMessage }}</div>
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

.gradient-primary {
  background: linear-gradient(45deg, #6366f1, #8b5cf6) !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2);
  transition: all 0.4s ease;
  color: white !important;
}

.gradient-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 15px -3px rgb(99 102 241 / 0.3);
}

.elevated-stepper {
  border-radius: 16px !important;
  box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1) !important;
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
