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
    remove-select: Remove
    remove-after-render: Remove After Render
    post-select-render: Render

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
    remove-select: 移除
    remove-after-render: 渲染后移除
    post-select-render: 渲染

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


</i18n>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { event } from '@tauri-apps/api';

import { toastError, RULES, toast, anyFilter } from './common';
import { DEFAULT_CONFIG, type ChartInfo, type RenderConfig, type RenderChart, type Preset, type FileDropEvent } from './model';

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import moment from 'moment';
import * as dialog from "@tauri-apps/plugin-dialog"

import { listen } from "@tauri-apps/api/event";

import { useTheme } from 'vuetify';
const theme = useTheme();

const router = useRouter();

const charts = ref<RenderChart[]>([]);

const choosingChart = ref(false),
  parsingChart = ref(false);
async function chooseChart(folder?: boolean) {
  if (choosingChart.value) return;
  choosingChart.value = true;
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
    choosingChart.value = false;
    return;
  }

  // noexcept
  await loadCharts(file);

  choosingChart.value = false;
}

async function loadCharts(files: string[]) {
  for (let file of files) {
    try {
      parsingChart.value = true;
      let chartInfo: ChartInfo = (await invoke('parse_chart', { path: file }));
      charts.value.push({
        id: charts.value.length,
        path: file,
        isChosen: false,
        chartInfo: chartInfo,
      });
    } catch (e) {
      toastError(e);
    }
  }
  parsingChart.value = false;
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
  try {
    let params = await buildParams(chart.path, chart.chartInfo, preset.value.config);
    if (!params) return false;
    await invoke('post_render', { params });
    if (removeAfterRender.value) {
      charts.value.splice(charts.value.indexOf(chart), 1);
    }
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function postSelectRender() {
  for (let chart of charts.value) {
    if (chart.isChosen) {
      postRender(chart);
    }
  }
}

function removeSelectChart() {
  for (let i = charts.value.length - 1; i >= 0; i--) {
    if (charts.value[i].isChosen) {
      charts.value.splice(i, 1)
    }
  }
}

const loadingPreview = ref(false);
const removeAfterRender = ref(true);

const chartInfoDialog = ref(false);
const chartInfoSelect = ref(0);

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
    chart.isChosen = true
  })
}

function selectInvert() {
  charts.value.forEach(chart => {
    chart.isChosen = !chart.isChosen
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
      <v-combobox class="mt-2" style="flex: 4;" :label="t('presets')" :items="presets" item-title="name" item-value="config" v-model="preset"></v-combobox>
      <v-btn class="mx-1" :title="t('edit-preset')" icon="mdi-pencil" @click="editPreset"></v-btn>
      <v-spacer />
      <v-checkbox class="mt-2 mx-2" :label="t('choose.remove-after-render')" v-model="removeAfterRender"></v-checkbox>
      <v-btn class="mx-2" variant="tonal" @click="selectAll" >{{ t('choose.select-all') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="selectInvert" >{{ t('choose.select-invert') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="removeSelectChart" >{{ t('choose.remove-select') }}</v-btn>
      <v-btn class="mx-2" variant="tonal" @click="postSelectRender" >{{ t('choose.post-select-render') }}</v-btn>
    </v-toolbar>
    <div class="flex-grow-1 overflow-y-auto">
      <v-table fixed-header density="compact" style="position: absolute; top: 64px; left: 0px; right: 0px; bottom: 0px; background-color: transparent;">
        <thead>
          <tr>
            <th class="text-center" style="width: 3.3em; padding-left: 1.4em;">({{ charts.length }})</th>
            <th class="text-left" style="min-width: 5em;">{{ t('info.name') }}</th>
            <th class="text-left" style="min-width: 5em;">{{ t('info.level') }}</th>
            <th class="text-left" style="min-width: 5em;">{{ t('info.charter') }}</th>
            <th class="text-left" style="max-width: 25%;">{{ t('info.chart') }}</th>
            <th class="text-center" style="width: 7em;">{{ t('chart-info') }}</th>
            <th class="text-center" style="width: 5em;">{{ t('preview') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="chart in charts" :key="chart.id">
            <td><v-checkbox class="mt-2 ml-n1" v-model="chart.isChosen"></v-checkbox></td>
            <td style="max-width: 12em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.chartInfo.name">{{ chart.chartInfo.name }}</td>
            <td style="max-width: 8em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.chartInfo.level">{{ chart.chartInfo.level }}</td>
            <td style="max-width: 8em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.chartInfo.charter">{{ chart.chartInfo.charter }}</td>
            <td style="max-width: 11em; white-space: nowrap; text-overflow: ellipsis; overflow: hidden;" :title="chart.path">{{ chart.path }}</td>
            <td><v-btn variant="tonal" @click="chartInfoSelect = chart.id; chartInfoDialog = true">{{ t('edit') }}</v-btn></td>
            <td><v-btn variant="tonal" :loading="loadingPreview" @click="previewChart(chart)">{{ t('preview') }}</v-btn></td>
          </tr>
        </tbody>
      </v-table>
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
    <v-card class="log-card-only-window" style="background: rgba(40, 40, 80, 0.5) !important;">
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
