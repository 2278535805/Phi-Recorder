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
    filter-name: Chart file

  info:
    name: Chart name
    difficulty: Difficulty
    level: Display Difficulty
    charter: Charter
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

    holdPartialCover: Hold Partial Cover

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
  phira-info: Phira Chart Info
  preview: Preview
  render: Render
  play: Play

  render-started: Rendering has started!
  see-tasks: See tasks
  
  confirm: Confirm
  close: Close
  save: Save
  save-success: Saved successfully
  read-success: Read successfully
  save-info: Save Info
  read-info: Read Info

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
    can-also-drop: 可直接拖放谱面至此处
    drop: 拖放谱面至此处
    filter-name: 谱面文件

  info:
    name: 谱面名
    difficulty: 难度
    level: 显示难度
    charter: 谱师
    composer: 曲师
    illustrator: 画师

    chart: 谱面文件
    music: 音乐文件
    illustration: 曲绘文件

    previewStart: 预览开始时间
    previewEnd: 预览结束时间

    aspectRatio: 宽高比
    backgroundDim: 背景亮度
    lineLength: 判定线长度
    offset: 偏移
    tip: 提示
    tip-placeholder: 留空则随机选择
    tags: 标签
    tag-editor: 标签编辑器
    tag-list: 常规,整活,纯配置,观赏

    intro: 简介

    holdPartialCover: Hold 遮罩位置

  error:
    preview-start-end-15s: 预览时间不能大于15秒

  width: 宽
  height: 高

  tweakoffset: 调整延时
  more: 更多
  phira-info: Phira 谱面信息
  preview: 预览
  render: 渲染
  play: 游玩

  render-started: 视频已开始渲染!
  see-tasks: 查看任务列表

  confirm: 确定
  close: 关闭
  save: 保存
  save-success: 保存成功
  read-success: 读取成功
  save-info: 保存信息
  read-info: 读取信息

</i18n>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { event } from '@tauri-apps/api';

import { toastError, RULES, toast, anyFilter, isString } from './common';
import type { ChartInfo } from './model';

import { VForm } from 'vuetify/components';

import ConfigView from './components/ConfigView.vue';

import moment from 'moment';
import * as dialog from "@tauri-apps/plugin-dialog"
import * as shell from "@tauri-apps/plugin-shell"

import { listen } from "@tauri-apps/api/event";
import { message, save, open } from '@tauri-apps/plugin-dialog';

import { useTheme } from 'vuetify';
const theme = useTheme();

if (!(await invoke('is_the_only_instance'))) {
  await dialog.message(t('already-running'));
  await invoke('exit_program', { code: 0 });
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
      [16, 10],
      [5, 4],
      [7, 5],
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

const aspectWidth = ref('16'),
  aspectHeight = ref('9');

const offset_text = ref('0');

const fileHovering = ref(false);

interface FileDropEvent {
  paths: string[];
  position: { x: number; y: number };
}

listen('tauri://drag-over', (_event) => (fileHovering.value = step.value === 'choose'));
listen('tauri://drag-leave', (_event) => (fileHovering.value = false));
listen('tauri://drag-drop', async (event) => {
  const files = (event.payload as FileDropEvent).paths;
  
  if (step.value === 'choose') {
    fileHovering.value = false;
    await loadChart(files[0]);
  } else if (step.value === 'config' || step.value === 'options' || step.value === 'render') {
    fileHovering.value = false;
    stepIndex.value = 1;
    await loadChart(files[0]);
  }
});

document.addEventListener('keydown', async (event) => {
  if (document.hasFocus() && event.key === 'Enter' && !moreInfo.value) {
    await moveNext();
  }
  if (document.hasFocus() && event.key === 'Escape' && stepIndex && !moreInfo.value) {
    stepIndex.value--;
  }
});

const form = ref<VForm>();

function syncInfo() {
  chartInfo.value!.offset = parseFloat(offset_text.value) / 1000;
  if (!chartInfo.value!.tip?.trim().length) chartInfo.value!.tip = null;
}

const configView = ref<typeof ConfigView>();
async function buildParams() {
  syncInfo();
  let config = await configView.value!.buildConfig();
  if (!config) return null;
  return {
    path: chartPath,
    info: chartInfo.value,
    config,
  };
}

async function postRender() {
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('post_render', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

const loadingNext = ref(false);
const loadingPreview = ref(false);
const loadingPlay = ref(false);
const loadingTweakoffset = ref(false);

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
  loadingTweakoffset.value = true;
  try {
    let params = await buildParams();
    if (!params) return false;
    let offset = await invoke('preview_tweakoffset', { params });
    if (offset != null) {
      chartInfo.value!.offset = offset as number;
      offset_text.value = String(Math.round(chartInfo.value!.offset * 1000));
    }
    if (stepIndex.value === 3) {
      stepIndex.value--;
    }
    return true;
  } catch (e) {
    toastError(e);
    return false;
  } finally {
    loadingTweakoffset.value = false
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
      tryParseAspect();
      await nextTick();
      configView.value!.applyAspectRatio(chartInfo.value!.aspectRatio);
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

const moreInfo = ref(false);
const tagEditor = ref(false);
async function checkInfo() {
  if (!(await form.value!.validate()).valid) {
    return false;
  }
  if (chartInfo.value?.previewEnd && chartInfo.value?.previewEnd - chartInfo.value?.previewStart > 15) {
    toast(t('error.preview-start-end-15s'), 'error');
    return false;
  }
  if (chartInfo.value!.previewEnd as string | null === '') {
    chartInfo.value!.previewEnd = null;
  }
  if (chartInfo.value!.tip as string | null === '') {
    chartInfo.value!.tip = null;
  }

  return true;
}

async function saveInfo() {
  let check = checkInfo();
  syncInfo();
  if (!check) return;
  let outputPath = await save({ title: t('output-folder'), filters: [{ name: 'Phira Chart Info File', extensions: ['yml'] }], defaultPath: 'info.yml' });
  if (!outputPath) return;
  try {
    await invoke('save_info', { path: outputPath, info: chartInfo.value });
    message(t('save-success'), { title: t('save-info') })
} catch (e) {
    toastError(e);
  }
}

async function readInfo() {
  let inputPath = await open({ title: t('rpe-folder'), filters: [{ name: 'Phira Chart Info File', extensions: ['yml'] }] });
  if (!inputPath) return;
  try {
    let info = await invoke('read_info', { path: inputPath }) as ChartInfo;
    chartInfo.value = info;
    message(t('read-success'), { title: t('read-info') })
} catch (e) {
    toastError(e);
  }
}

const tagListText = t('info.tag-list').split(','); // wtf bro
const tagList = ['Regular', 'Troll', 'Plain', 'Visual']
const tagitems = [
    tagList[0],
    tagList[1],
    tagList[2],
    tagList[3],
]

watch(() => chartInfo.value?.tags ?? [], (newVal, oldVal) => {
  let addedPreset = newVal.find(item =>
    tagList.some(tag => tag.toLowerCase() === item.toLowerCase()) &&
    !oldVal.some(tag => tag === item) // 比较旧值
  );

  if (addedPreset) {
    const matchedTag = tagList.find(tag => tag.toLowerCase() === addedPreset?.toLowerCase());
    if (matchedTag) {
      addedPreset = matchedTag;
    }

    const customItems = newVal.filter(item =>
      !tagList.some(tag => tag.toLowerCase() === item.toLowerCase())
    );
    
    chartInfo.value!.tags = [...customItems, addedPreset];
  }
});

</script>

<template>
  <div class="pa-8 w-100 h-100" style="max-width: 1280px">
    <v-stepper v-model="stepIndex" hide-actions :items="steps.map((x) => t('steps.' + x))" class="elevated-stepper fade-in" :style="{ background: `${theme.current.value.colors.container}` }">
      <div v-if="step === 'config' || step === 'options' || step === 'render'" class="d-flex flex-row pa-6 pb-4 pt-0">
        <v-btn variant="text" @click="stepIndex && stepIndex--">{{ t('prev-step') }}</v-btn>
        <v-btn v-if="step === 'options'" :loading="loadingTweakoffset" variant="text" @click="previewTweakoffset" class="mr-2">{{ t('tweakoffset') }}</v-btn>
        <div class="flex-grow-1"></div>
        <v-btn v-if="step === 'config'" variant="text" @click="moreInfo = true; tryParseAspect(); syncInfo();" class="mr-2">{{ t('more') }}</v-btn>
        <v-btn v-if="step === 'options'" :loading="loadingPlay" variant="text" @click="previewPlay" class="mr-2">{{ t('play') }}</v-btn>
        <v-btn v-if="step === 'options'" :loading="loadingPreview" variant="text" @click="previewChart" class="mr-2">{{ t('preview') }}</v-btn>
        <v-btn v-if="step !== 'render'" :loading="loadingNext" variant="tonal" @click="moveNext" class="gradient-primary">{{ step === 'options' ? t('render') : t('next-step') }}</v-btn>
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
          <v-row no-gutters class="my-2">
            <v-col cols="6">
              <v-text-field class="mx-2" :label="t('info.name')" v-model="chartInfo.name"></v-text-field>
            </v-col>
            <v-col cols="2">
              <v-text-field class="mx-2" :label="t('info.offset')" type="number" :rules="[RULES.int]" v-model="offset_text"></v-text-field>
            </v-col>
            <v-col cols="4">
              <v-text-field class="mx-2" :label="t('info.level')" v-model="chartInfo.level"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mt-1 my-2 pt-2">
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('info.charter')" v-model="chartInfo.charter"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('info.composer')" v-model="chartInfo.composer"></v-text-field>
            </v-col>
            <v-col cols="12" sm="4">
              <v-text-field class="mx-2" :label="t('info.illustrator')" v-model="chartInfo.illustrator"></v-text-field>
            </v-col>
          </v-row>

          <p class="text-caption mx-3" v-t="'info.aspectRatio'"></p>
          <v-row no-gutters class="mt-1 my-2 align-center">
            <v-col cols="4">
              <div class="mx-2 d-flex flex-column">
                <div class="d-flex flex-row align-center justify-center">
                  <v-text-field type="number" class="mr-2" :rules="[RULES.positive, RULES.nonZero]" :label="t('width')" v-model="aspectWidth"></v-text-field>
                  <p>:</p>
                  <v-text-field type="number" class="ml-2" :rules="[RULES.positive, RULES.nonZero]" :label="t('height')" v-model="aspectHeight"></v-text-field>
                </div>
              </div>
            </v-col>
            <v-col cols="8">
              <v-text-field class="mx-2" :label="t('info.tip')" :placeholder="t('info.tip-placeholder')" v-model="chartInfo.tip"></v-text-field>
            </v-col>
          </v-row>

          <v-row no-gutters class="mt-1 my-2 align-center">
            <v-col cols="8" class="px-6 py-6">
              <v-slider :label="t('info.backgroundDim')" thumb-label="always" color="btn" :min="0" :max="1" :step="0.01" v-model="chartInfo.backgroundDim"></v-slider>
            </v-col>
            <v-col cols="4">
              <v-switch class="mx-2" v-model="chartInfo.holdPartialCover" :label="t('info.holdPartialCover')"></v-switch>
            </v-col>
          </v-row>

          <v-row no-gutters class="mt-1 my-2">
          </v-row>
        </v-form>
      </template>

      <v-dialog v-model="moreInfo" theme="darkTheme" width="auto" min-width="90%" class="log-card-bg">
        <v-card class="log-card-only-window">
          <v-card-title v-t="'phira-info'"> </v-card-title>
          <v-card-text>

            <v-form v-if="chartInfo">
              <v-row>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.name')" v-model="chartInfo.name"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.charter')" v-model="chartInfo.charter"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.composer')" v-model="chartInfo.composer"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.illustrator')" v-model="chartInfo.illustrator"></v-text-field>
                </v-col>
              </v-row>

              <v-row>
                <v-col cols="3">
                  <v-text-field type="number" class="" :rules="[RULES.positive, RULES.nonZero]" :label="t('info.aspectRatio')"
                  v-model="chartInfo.aspectRatio" @update:modelValue="chartInfo.aspectRatio = parseFloat($event)"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.level')" v-model="chartInfo.level"></v-text-field>
                </v-col>
                <v-col cols="6">
                  <v-slider class="my-3" :label="t('info.difficulty')" thumb-label="always" :min="0" :max="20" :step="0.1" v-model="chartInfo.difficulty"> </v-slider>
                </v-col>
              </v-row>

              <v-row>
                <v-col cols="3">
                  <v-text-field type="number" class="" :rules="[RULES.positive]" :label="t('info.previewStart')"
                  v-model="chartInfo.previewStart" @update:modelValue="chartInfo.previewStart = parseFloat($event)"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="number" class="" :rules="[RULES.positiveNull]" :label="t('info.previewEnd')"
                  v-model="chartInfo.previewEnd" @update:modelValue="chartInfo.previewEnd = parseFloat($event)"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="number" class="" :rules="[RULES.non_empty]" :label="t('info.offset')"
                  v-model="offset_text"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="number" class="" :rules="[RULES.positive10000, RULES.non_empty]" :label="t('info.lineLength')"
                  v-model="chartInfo.lineLength" @update:modelValue="chartInfo.lineLength = parseFloat($event)"></v-text-field>
                </v-col>
              </v-row>
              
              <v-row>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.chart')" v-model="chartInfo.chart"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.music')" v-model="chartInfo.music"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.illustration')" v-model="chartInfo.illustration"></v-text-field>
                </v-col>
                <v-col cols="3" class="d-flex align-center justify-center">
                  <v-btn class="" color="#414047" size="large" @click="tagEditor = true">{{ t('info.tag-editor') }}</v-btn>
                </v-col>
              </v-row>

              <v-row>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.tip')" v-model="chartInfo.tip"></v-text-field>
                </v-col>
                <v-col cols="3">
                  <v-text-field type="text" class="" :label="t('info.intro')" v-model="chartInfo.intro"></v-text-field>
                </v-col>
                <v-col cols="6">
                  <v-slider class="my-3" :label="t('info.backgroundDim')" thumb-label="always" :min="0" :max="1" :step="0.05" v-model="chartInfo.backgroundDim"> </v-slider>
                </v-col>
              </v-row>

            </v-form>
          </v-card-text>
          <v-card-actions class="justify-end">
            <v-btn class="hover-scale" variant="text" @click="readInfo" v-t="'read-info'"></v-btn>
            <v-btn class="hover-scale" variant="text" @click="saveInfo" v-t="'save-info'"></v-btn>
            <v-btn class="hover-scale" variant="text" @click="moreInfo = false" v-t="'close'"></v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <v-dialog v-model="tagEditor" theme="darkTheme" width="auto" min-width="90%" class="log-card-bg">
        <v-card class="log-card-only-window">
          <v-card-title v-t="'info.tag-editor'"> </v-card-title>
          <v-card-text>

            <v-form v-if="chartInfo">
              <v-row>
                <v-col cols="12">
                  <v-combobox
                    v-model="chartInfo.tags"
                    :items="tagitems"
                    multiple
                    chips
                    :clearable="true"
                    :label="t('info.tags')"
                    :item-props="true"
                    allow-custom
                  ></v-combobox>
                </v-col>
                
              </v-row>

            </v-form>
          </v-card-text>
          <v-card-actions class="justify-end">
            <v-btn class="hover-scale" variant="text" @click="tagEditor = false" v-t="'close'"></v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <template v-slot:item.3>
        <ConfigView ref="configView"></ConfigView>
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

:deep(.v-slider__thumb) {
  background: #6366f1 !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2) !important;
}

:deep(.v-slider__track-fill) {
  background: linear-gradient(90deg, #6366f1, #8b5cf6) !important;
}

</style>
