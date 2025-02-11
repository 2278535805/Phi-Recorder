<i18n>
en:
  already-running: Phi Recorder is already running

  prev-step: Previous
  next-step: Next
  steps:
    choose: 'Choose the chart'
    config: 'Configure chart'
    options: 'Render options'
    render: 'Render'

  choose:
    archive: Archive
    folder: Folder
    can-also-drop: You can also drag & drop the file to here
    drop: DROP CHART HERE

  chart-file: Chart file

  chart-name: Chart name
  chart-offset: Offset
  charter: Charter
  composer: Composer
  illustrator: Illustrator
  level: Level
  aspect: Aspect ratio
  dim: Background dim

  tip: Tip
  tip-placeholder: Leave empty to choose randomly

  width: Width
  height: Height

  file:
    title: File
    chart: Chart file (empty for default)
    music: Music (empty for default)
    illustration: Illustration (empty for default)

  tweakoffset: Tweak Offset
  preview: Preview
  render: Render
  play: Play

  render-started: Rendering has started!
  see-tasks: See tasks

  open-app-folder: Open app folder
  open-download: Download FFmpeg
  ffmpeg-not-found: FFmpeg not found!
  ffmpeg-not-found-detail: |
    Please download ffmpeg, Windows users usually only need to download "ffmpeg-master-latest-win64-gpl.zip"
    Place all files in the bin folder in the program folder or configure the ffmpeg environment variables
  
  confirm: Confirm

zh-CN:
  already-running: Phi Recorder 已经在运行

  prev-step: 上一步
  next-step: 下一步
  steps:
    choose: '选择谱面'
    config: '配置谱面'
    options: '渲染参数'
    render: '渲染视频'

  choose:
    archive: 压缩包
    folder: 文件夹
    can-also-drop: 也可以直接拖放谱面至此处
    drop: 拖放谱面至此处

  chart-file: 谱面文件

  chart-name: 谱面名
  chart-offset: 偏移
  charter: 谱师
  composer: 曲师
  illustrator: 画师
  level: 难度
  aspect: 宽高比
  dim: 背景亮度

  tip: Tip
  tip-placeholder: 留空则随机选择

  width: 宽
  height: 高

  tweakoffset: 调整延时
  preview: 预览
  render: 渲染
  play: 游玩

  render-started: 视频已开始渲染!
  see-tasks: 查看任务列表

  open-app-folder: 打开程序文件夹
  open-download: 下载 FFmpeg
  ffmpeg-not-found: 未找到 FFmpeg!
  ffmpeg-not-found-detail: |
    请下载 ffmpeg, Windows 用户一般只需下载 "ffmpeg-master-latest-win64-gpl.zip"
    将 bin 文件夹内的所有文件放置在程序文件夹内或配置 ffmpeg 环境变量

  confirm: 确定

</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke, event, dialog, shell } from '@tauri-apps/api';

import { toastError, RULES, toast, anyFilter, isString } from './common';
import type { ChartInfo } from './model';

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import moment from 'moment';

if (!(await invoke('is_the_only_instance'))) {
  await dialog.message(t('already-running'));
  await invoke('exit_program');
}

const router = useRouter();

const steps = ['choose', 'config', 'options', 'render'];
const stepIndex = ref(1),
  step = computed(() => steps[stepIndex.value - 1]);

const chartInfo = ref<ChartInfo>();

let chartPath = '';

const choosingChart = ref(false),
  parsingChart = ref(false);
async function chooseChart(folder?: boolean) {
  if (choosingChart.value) return;
  choosingChart.value = true;
  let file = folder
    ? await dialog.open({ directory: true })
    : await dialog.open({
        filters: [
          {
            name: t('choose.filter-name'),
            extensions: ['zip', 'pez'],
          },
          anyFilter(),
        ],
      });
  if (!file) {
    choosingChart.value = false;
    return;
  };

  // noexcept
  await loadChart(file as string);

  choosingChart.value = false;
}
async function loadChart(file: string) {
  try {
    parsingChart.value = true;
    chartPath = file;
    chartInfo.value = (await invoke('parse_chart', { path: file })) as ChartInfo;
    stepIndex.value++;
    offset_text.value = String(Math.floor(chartInfo.value.offset * 1000));
    aspectWidth.value = String(chartInfo.value.aspectRatio);
    aspectHeight.value = '1.0';
    for (let asp of [
      [16, 9],
      [4, 3],
      [8, 5],
      [3, 2],
    ]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.value.aspectRatio) < 1e-4) {
        aspectWidth.value = String(asp[0]);
        aspectHeight.value = String(asp[1]);
        break;
      }
    }
  } catch (e) {
    toastError(e);
  } finally {
    parsingChart.value = false;
  }
}

const aspectWidth = ref('0'),
  aspectHeight = ref('0');

const offset_text = ref('0')

const fileHovering = ref(false);
event.listen('tauri://file-drop-hover', (_event) => (fileHovering.value = step.value === 'choose'));
event.listen('tauri://file-drop-cancelled', (_event) => (fileHovering.value = false));
event.listen('tauri://file-drop', async (event) => {
  if (step.value === 'choose') {
    fileHovering.value = false;
    await loadChart((event.payload as string[])[0]);
  } else if (step.value === 'options') {
    fileHovering.value = false;
    stepIndex.value = 1;
    await loadChart((event.payload as string[])[0]);
  }
});

document.addEventListener('keydown', async (event) => {
  if (document.hasFocus() && event.key === 'Enter') {
    await moveNext();
  }
});

const form = ref<VForm>();

const configView = ref<typeof ConfigView>();
async function buildParams() {
  let config = await configView.value!.buildConfig();
  chartInfo.value!.offset = parseFloat(offset_text.value) / 1000;
  if (!config) return null;
  if (!chartInfo.value!.tip?.trim().length) chartInfo.value!.tip = null;
  return {
    path: chartPath,
    info: chartInfo.value,
    config,
  };
}

const ffmpegDialog = ref(false);
async function postRender() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      ffmpegDialog.value = true;
      //await dialog.message(t('ffmpeg-not-found'));
      return false;
    }
    let params = await buildParams();
    if (!params) return false;
    await invoke('post_render', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function openAppFolder() {
  await invoke('open_app_folder');
}

async function openDownload() {
  await shell.open('https://github.com/BtbN/FFmpeg-Builds/releases');
}

const loadingNext = ref(false);
const loadingPreview = ref(false);
const loadingPlay = ref(false);

async function previewChart() {
  loadingPreview.value = true;
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('preview_chart', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  } finally {
    setTimeout(() => (loadingPreview.value = false), 1000)
  }
}

async function previewTweakoffset() {
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('preview_tweakoffset', { params });
    stepIndex.value--;
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function previewPlay() {
  loadingPlay.value = true;
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('preview_play', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  } finally {
    setTimeout(() => (loadingPlay.value = false), 1000)
  }
}

const renderMsg = ref(''),
  renderProgress = ref<number>(),
  renderDuration = ref<number>();
event.listen('render-msg', (msg) => (renderMsg.value = msg.payload as string));
event.listen('render-progress', (msg) => {
  let payload = msg.payload as { progress: number; fps: number; estimate: number };
  renderMsg.value = t('render-status', {
    progress: (payload.progress * 100).toFixed(2),
    fps: payload.fps,
    estimate: moment.duration(payload.estimate, 'seconds').humanize(true, { ss: 1 }),
  });
  renderProgress.value = payload.progress * 100;
  console.log(renderProgress.value);
});
event.listen('render-done', (msg) => {
  stepIndex.value++;
  renderDuration.value = Math.round(msg.payload as number);
});

async function moveNext() {
  if (step.value === 'config') {
    if ((await form.value!.validate()).valid) {
      stepIndex.value++;
      configView.value!.onEnter();
    } else {
      toast(t('has-error'), 'error');
    }
    return;
  }
  if (step.value === 'options') {
    loadingNext.value = true;
    if (await postRender()) {
      stepIndex.value++;
    }
    loadingNext.value = false;
    return;
  }
}

let chartInQuery = router.currentRoute.value.query.chart;
if (isString(chartInQuery)) {
  onMounted(() => loadChart(chartInQuery as string));
}

function tryParseAspect(): number | undefined {
  try {
    let width = parseFloat(aspectWidth.value);
    let height = parseFloat(aspectHeight.value);
    if (isNaN(width) || isNaN(height)) return undefined;
    chartInfo.value!.aspectRatio = width / height;
    return width / height;
  } catch (e) {
    return undefined;
  }
}
</script>

<template>
  <div class="pa-8 w-100 h-100" style="max-width: 1280px">
    <v-stepper v-model="stepIndex" hide-actions :items="steps.map((x) => t('steps.' + x))" class="elevated-stepper v-stepper">
      <div v-if="step === 'config' || step === 'options'" class="d-flex flex-row pa-6 pb-4 pt-0">
        <v-btn @click="stepIndex && stepIndex--" variant="text" v-t="'prev-step'"></v-btn>
        <v-btn v-if="step === 'options'" variant="tonal" @click="previewTweakoffset" class="mr-2" v-t="'tweakoffset'"></v-btn>
        <div class="flex-grow-1"></div>
        <v-btn v-if="step === 'options'" :loading="loadingPlay" variant="text" @click="previewPlay" class="mr-2">{{ t('play') }}</v-btn>
        <v-btn v-if="step === 'options'" :loading="loadingPreview" variant="text" @click="previewChart" class="mr-2">{{ t('preview') }}</v-btn>
        <v-btn :loading="loadingNext" variant="tonal" @click="moveNext" class="gradient-primary">{{ step === 'options' ? t('render') : t('next-step') }}</v-btn>
      </div>

      <template v-slot:item.1>
        <div class="mt-8 d-flex" style="gap: 1rem">
          <div class="flex-grow-1 d-flex align-center justify-center w-0 py-8">
            <v-btn class="w-75 gradient-primary" style="overflow: hidden" size="large" color="primary" @click="chooseChart(false)" prepend-icon="mdi-folder-zip">{{ t('choose.archive') }}</v-btn>
          </div>
          <v-divider vertical></v-divider>
          <div class="flex-grow-1 d-flex align-center justify-center w-0">
            <v-btn class="w-75 gradient-primary" size="large" color="primary" @click="chooseChart(true)" prepend-icon="mdi-folder">{{ t('choose.folder') }}</v-btn>
          </div>
        </div>
        <p class="mb-8 w-100 text-center mt-2 text-disabled" v-t="'choose.can-also-drop'"></p>
        <v-overlay v-model="parsingChart" contained class="align-center justify-center" persistent :close-on-content-click="false">
          <v-progress-circular indeterminate> </v-progress-circular>
        </v-overlay>
      </template>

      <template v-slot:item.2>
        <v-form ref="form" v-if="chartInfo">
          <v-row no-gutters class="mx-n2">
            <v-col cols="6">
              <v-text-field class="mx-2" :label="t('chart-name')" v-model="chartInfo.name"></v-text-field>
            </v-col>
            <v-col cols="2">
              <v-text-field class="mx-2" :label="t('chart-offset')" type="number" :rules="[RULES.int]" v-model="offset_text"></v-text-field>
            </v-col>
            <v-col cols="4">
              <v-text-field class="mx-2" :label="t('level')" v-model="chartInfo.level"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1">
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('charter')" v-model="chartInfo.charter"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('composer')" v-model="chartInfo.composer"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('illustrator')" v-model="chartInfo.illustrator"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1 align-center">
            <v-col cols="4">
              <div class="mx-2 d-flex flex-column">
                <p class="text-caption" v-t="'aspect'"></p>
                <div class="d-flex flex-row align-center justify-center">
                  <v-text-field type="number" class="mr-2" :rules="[RULES.positive]" :label="t('width')" v-model="aspectWidth"></v-text-field>
                  <p>:</p>
                  <v-text-field type="number" class="ml-2" :rules="[RULES.positive]" :label="t('height')" v-model="aspectHeight"></v-text-field>
                </div>
              </div>
            </v-col>
            <v-col cols="8" class="px-6">
              <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.01" v-model="chartInfo.backgroundDim"></v-slider>
            </v-col>
          </v-row>

          <v-row no-gutters class="mx-n2 mt-1">
            <v-col cols="12">
              <v-text-field class="mx-2" :label="t('tip')" :placeholder="t('tip-placeholder')" v-model="chartInfo.tip"></v-text-field>
            </v-col>
          </v-row>
        </v-form>
      </template>

      <template v-slot:item.3>
        <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()"></ConfigView>
      </template>

      <template v-slot:item.4>
        <div class="d-flex flex-column justify-center align-center mb-2" style="gap: 1rem">
          <span style="font-size: 84px">🎉</span>
          <h2>{{ t('render-started') }}</h2>
          <v-btn class="gradient-primary" @click="router.push({ name: 'tasks' })" v-t="'see-tasks'"></v-btn>
        </div>
      </template>
    </v-stepper>
    <v-overlay v-model="fileHovering" contained class="align-center justify-center drop-zone-overlay" persistent :close-on-content-click="false">
      <div class="drop-pulse">
        <h1 v-t="'choose.drop'"></h1>
      </div>
    </v-overlay>
  </div>

  <v-dialog v-model="ffmpegDialog" width="auto" min-width="400px">
      <v-card>
        <v-card-title v-t="t('ffmpeg-not-found')"> </v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh; white-space: pre-wrap">{{ t('ffmpeg-not-found-detail') }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn variant="text" @click="openDownload" v-t="t('open-download')"></v-btn>
          <v-btn variant="text" @click="openAppFolder" v-t="t('open-app-folder')"></v-btn>
          <v-btn color="primary" variant="text" @click="ffmpegDialog = false" v-t="t('confirm')"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
</template>

<style scoped>
.v-progress-linear,
.v-progress-linear__determinate {
  transition: none;
}

.gradient-primary {
  background: linear-gradient(45deg, #6366f1, #8b5cf6) !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2);
  transition: transform 0.2s, box-shadow 0.2s;
}

.gradient-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 15px -3px rgb(99 102 241 / 0.3);
}

.elevated-stepper {
  border-radius: 16px !important;
  box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1) !important;
  background: rgba(255, 255, 255, 0.03) !important;
  backdrop-filter: blur(8px);
}

.v-text-field :deep(.v-field--focused) {
  border-color: #6366f1 !important;
  box-shadow: 0 0 0 2px rgb(99 102 241 / 0.2);
}

.v-stepper {
  font-family: 'Inter var', system-ui, sans-serif;
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

.drop-zone-overlay {
  background: rgba(99, 102, 241, 0.15) !important;
  backdrop-filter: blur(4px);
}

.drop-pulse {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.05); }
  100% { transform: scale(1); }
}

.v-progress-circular {
  color: #6366f1;
  --v-progress-circular-size: 48px;
}

:deep(.v-slider__thumb) {
  background: #6366f1 !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2) !important;
}

:deep(.v-slider__track-fill) {
  background: linear-gradient(90deg, #6366f1, #8b5cf6) !important;
}

.v-text-field.type-number {
  --v-field-padding-start: 1rem;
  --v-field-border-radius: 8px;
}

.v-stepper__content {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.v-stepper__content-leave-active {
  position: absolute;
}

.v-stepper__content-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

  .v-stepper__content-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.v-btn {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
  
</style>
