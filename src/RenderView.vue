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

  ffmpeg-not-found: You haven't installed ffmpeg yet. Please download FFmpeg.exe and put it in the specific folder.

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

  render-started: 视频已开始渲染！
  see-tasks: 查看任务列表

  ffmpeg-not-found: 您尚未安装 FFmpeg。请下载 FFmpeg.exe 并放置在指定文件夹内。

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
    offset_text.value = String(chartInfo.value.offset * 1000);
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
  if (event.key === 'Enter') {
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

async function postRender() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      await dialog.message(t('ffmpeg-not-found'));
      await invoke('open_app_folder');
      await shell.open('https://github.com/BtbN/FFmpeg-Builds/releases');
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

async function previewChart() {
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('preview_chart', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
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
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('preview_play', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
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
    if (await postRender()) {
      stepIndex.value++;
    }
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
    return width / height;
  } catch (e) {
    return undefined;
  }
}
</script>

<template>
  <div class="pa-8 w-100 h-100" style="max-width: 1280px">
    <v-stepper alt-labels v-model="stepIndex" hide-actions :items="steps.map((x) => t('steps.' + x))">
      <div v-if="step === 'config' || step === 'options'" class="d-flex flex-row pa-6 pb-4 pt-0">
        <v-btn @click="stepIndex && stepIndex--" variant="text" v-t="'prev-step'"></v-btn>
        <v-btn v-if="step === 'options'" variant="tonal" @click="previewTweakoffset" class="mr-2" v-t="'tweakoffset'"></v-btn>
        <div class="flex-grow-1"></div>
        <v-btn v-if="step === 'options'" variant="tonal" @click="previewPlay" class="mr-2" v-t="'play'"></v-btn>
        <v-btn v-if="step === 'options'" variant="tonal" @click="previewChart" class="mr-2" v-t="'preview'"></v-btn>
        <v-btn variant="tonal" @click="moveNext">{{ step === 'options' ? t('render') : t('next-step') }}</v-btn>
      </div>

      <template v-slot:item.1>
        <div class="mt-8 d-flex" style="gap: 1rem">
          <div class="flex-grow-1 d-flex align-center justify-center w-0 py-8">
            <v-btn class="w-75" style="overflow: hidden" size="large" color="primary" @click="chooseChart(false)" prepend-icon="mdi-folder-zip">{{ t('choose.archive') }}</v-btn>
          </div>
          <v-divider vertical></v-divider>
          <div class="flex-grow-1 d-flex align-center justify-center w-0">
            <v-btn class="w-75" size="large" color="primary" @click="chooseChart(true)" prepend-icon="mdi-folder">{{ t('choose.folder') }}</v-btn>
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
              <v-slider :label="t('dim')" thumb-label="always" :min="0" :max="1" :step="0.05" v-model="chartInfo.backgroundDim"></v-slider>
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
          <v-btn @click="router.push({ name: 'tasks' })" v-t="'see-tasks'"></v-btn>
        </div>
      </template>
    </v-stepper>
    <v-overlay v-model="fileHovering" contained class="align-center justify-center" persistent :close-on-content-click="false">
      <h1 v-t="'choose.drop'"></h1>
    </v-overlay>
  </div>
</template>

<style scoped>
.v-progress-linear,
.v-progress-linear__determinate {
  transition: none;
}
</style>
}
