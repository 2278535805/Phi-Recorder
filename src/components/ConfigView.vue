<script setup lang="ts">
import { ref, h, watch } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import { VDivider, VForm } from 'vuetify/components';

import { RULES, isNumeric, toast, anyFilter, toastError } from '../common';
import { DEFAULT_RENDER_CONFIG, type Preset, type RenderConfig } from '../model';

import TipSwitch from './TipSwitch.vue';
import TipSlider from './TipSlider.vue';
import TooltipIcon from './TooltipIcon.vue';

const form = ref<VForm>();
const page = ref(0);
const fileNameFormatDialog = ref(false);

const RESOLUTIONS = [ '1280x720', '1920x1080', '1620x1080', '1440x1080', '2560x1440', '2844x1600', '2388x1668', '3840x2160'],
  fpsList = ['30', '60', '120'],
  bitrateList = ['2M', '5M', '7M'],
  bitrateCrfList = ['24', '28', '35', '40'],
  encoderList = ref(t('encoder-list').split(','))

const
  endingLength = ref(String(DEFAULT_RENDER_CONFIG.endingLength)),
  chartDebugLine = ref(DEFAULT_RENDER_CONFIG.chartDebugLine),
  chartDebugNote = ref(DEFAULT_RENDER_CONFIG.chartDebugNote),
  chartRatio = ref(DEFAULT_RENDER_CONFIG.chartRatio),
  allGood = ref(DEFAULT_RENDER_CONFIG.allGood),
  allBad = ref(DEFAULT_RENDER_CONFIG.allBad),
  fps = ref(String(DEFAULT_RENDER_CONFIG.fps)),
  hwAccel = ref(DEFAULT_RENDER_CONFIG.hardwareAccel),
  dynamicBitrateControl = ref(true),
  bitrate = ref('28'),
  resolution = ref('1920x1080');
const encoder = ref(t('encoder-list').split(',')[0])

const challengeColor = ref(t('challenge-colors').split(',')[5]),
  challengeRank = ref(String(DEFAULT_RENDER_CONFIG.challengeRank)),
  noteScale = ref(DEFAULT_RENDER_CONFIG.noteScale),
  playerAvatar = ref<string>(),
  playerName = ref(DEFAULT_RENDER_CONFIG.playerName),
  playerRks = ref(String(DEFAULT_RENDER_CONFIG.playerRks)),
  sampleCount = ref(String(DEFAULT_RENDER_CONFIG.sampleCount))

const volumeMusic = ref(DEFAULT_RENDER_CONFIG.volumeMusic);
const volumeSfx = ref(DEFAULT_RENDER_CONFIG.volumeSfx);
const compressionRatio = ref(DEFAULT_RENDER_CONFIG.compressionRatio);
const limitThreshold = ref(DEFAULT_RENDER_CONFIG.limitThreshold);
const forceLimit = ref(DEFAULT_RENDER_CONFIG.forceLimit);
const hires = ref(DEFAULT_RENDER_CONFIG.hires);
const loudnessEqualization = ref(DEFAULT_RENDER_CONFIG.loudnessEqualization);
const audioMixOptimization = ref(DEFAULT_RENDER_CONFIG.audioMixOptimization);

watch(() => volumeSfx.value, (volume) => {
  if (forceLimit.value && volume > limitThreshold.value) {
    limitThreshold.value = volumeSfx.value;
  }
})
watch(() => limitThreshold.value, (limit) => {
  if (forceLimit.value && limit < volumeSfx.value) {
    volumeSfx.value = limitThreshold.value;
  }
})
watch(() => loudnessEqualization.value, (loudnessEq) => {
  if (loudnessEq) {
    volumeMusic.value = 1.0;
  }
})

const
  combo = ref(DEFAULT_RENDER_CONFIG.combo),
  difficulty = ref(DEFAULT_RENDER_CONFIG.difficulty),
  judgeOffset = ref(String(DEFAULT_RENDER_CONFIG.judgeOffset)),
  fileNameFormat = ref(DEFAULT_RENDER_CONFIG.fileNameFormat),
  bgBlurriness = ref(String(DEFAULT_RENDER_CONFIG.bgBlurriness)),
  watermark = ref(DEFAULT_RENDER_CONFIG.watermark)

const maxParticlesText = ref(t('max-particles-list').split(',')[0])
const maxParticles = ref(DEFAULT_RENDER_CONFIG.maxParticles)
const maxParticlesTextList = t('max-particles-list').split(',')
const maxParticlesList = [5000, 25000, 200000];

const playStartTime = ref(String(DEFAULT_RENDER_CONFIG.playStartTime)),
  playEndTime = ref('');

const judgeMode = ref(t('judge-modes').split(',')[0])
const fade = ref(String(DEFAULT_RENDER_CONFIG.fade))
const alphaTint = ref(DEFAULT_RENDER_CONFIG.alphaTint)

const renderLoading = ref(DEFAULT_RENDER_CONFIG.renderLoading);
const renderLine = ref(DEFAULT_RENDER_CONFIG.renderLine);
const renderLineExtra = ref(DEFAULT_RENDER_CONFIG.renderLineExtra);
const renderNote = ref(DEFAULT_RENDER_CONFIG.renderNote);
const renderUiPause = ref(DEFAULT_RENDER_CONFIG.renderUiPause);
const renderUiName = ref(DEFAULT_RENDER_CONFIG.renderUiName);
const renderUiLevel = ref(DEFAULT_RENDER_CONFIG.renderUiLevel);
const renderUiScore = ref(DEFAULT_RENDER_CONFIG.renderUiScore);
const renderUiCombo = ref(DEFAULT_RENDER_CONFIG.renderUiCombo);
const renderUiBar = ref(DEFAULT_RENDER_CONFIG.renderUiBar);
const renderBg = ref(DEFAULT_RENDER_CONFIG.renderBg);
const renderBgDim = ref(DEFAULT_RENDER_CONFIG.renderBgDim);
const particle = ref(DEFAULT_RENDER_CONFIG.particle);
const renderExtra = ref(DEFAULT_RENDER_CONFIG.renderExtra);
const renderDoubleHint = ref(DEFAULT_RENDER_CONFIG.renderDoubleHint);

const aggressive = ref(DEFAULT_RENDER_CONFIG.aggressive);
const roman = ref(DEFAULT_RENDER_CONFIG.roman);
const chinese = ref(DEFAULT_RENDER_CONFIG.chinese);


function parseResolution(resolution: string): [number, number] | null {
  let parts = resolution.split(/[xX]/g);
  if (parts.length !== 2) return null;
  let ws = parts[0]!.trim(),
    hs = parts[1]!.trim();
  if (!isNumeric(ws) || !isNumeric(hs)) return null;
  let w = parseInt(ws),
    h = parseInt(hs);
  if (w <= 0 || h <= 0) return null;
  return [w, h];
}
const resolutionRule = (value: string) => parseResolution(value) !== null || t('rules.resolution');
const sampleCountRule = (value: string) => (isNumeric(value) && Math.log2(Number(value)) % 1 === 0) || t('rules.sample-count');
const isCrf = (value: string) => (Number.isInteger(Number(value)) && Number(value) >= 0 && Number(value) <= 51) || t('rules.crf');
const isBitrate = (value: string) => {
    if (!value || value.trim() === '') {
      return t('rules.bitrate');
    }
    const regex = /^(\d+)(Kbps|Mbps|K|M)$/i;
    const match = value.match(regex);
    if (!match) return t('rules.bitrate');
    
    const number = Number(match[1]);
    const unit = match[2]!.toLowerCase();
  
    if ((unit === 'kbps' || unit === 'k') && number > 0 && number <= 1000000) return true;
    if ((unit === 'mbps' || unit === 'm') && number > 0 && number <= 1000) return true;
  
    return t('rules.bitrate');
  };


async function chooseAvatar() {
  let file = await open({
    filters: [
      {
        name: t('image-filter'),
        extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'],
      },
      anyFilter(),
    ],
  });
  if (file) {
    playerAvatar.value = file as string;
  }
}

interface Respack {
  name: string;
  path: string | null;
  index: number;
}
const DEFAULT_RESPACK: Respack = {
  name: t('respack-default'),
  path: null,
  index: 0,
};
async function getRespacks() {
  return [DEFAULT_RESPACK, ...((await invoke('get_respacks')) as { name: string; path: string }[])].map((obj, index) => ({
    name: obj.name,
    path: obj.path,
    index: index + 1,
  }));
}
const respacks = ref([DEFAULT_RESPACK]);
const respack = ref(DEFAULT_RESPACK);
async function updateRespacks() {
  respacks.value = await getRespacks();
  respack.value = respacks.value.find((x) => x.name === respack.value.name) || respacks.value[0]!;
}
updateRespacks();

function updateBitrate() {
  if (dynamicBitrateControl.value) {
    bitrate.value = bitrateCrfList[0]!;
  } else {
    bitrate.value = bitrateList[0]!;
  }
}

function updateMaxParticles() {
  const index = maxParticlesTextList.indexOf(maxParticlesText.value!);
  const textNum = Number(maxParticlesText.value);
  if (index >= 0 && index < maxParticlesTextList.length) {
    maxParticles.value = maxParticlesList[index]!;
  } else if (Number.isInteger(textNum) && textNum > 0) {
    maxParticles.value = parseInt(maxParticlesText.value!);
  } else {
    toast(t('max-particles-error'), 'error');
    maxParticles.value = 100000;
    return false;
  }
  return true;
}

function updateList(value: any, text: any, list: any, textList: any) {
  const index = textList.indexOf(text.value);
  if (index >= 0 && index < textList.length) {
    value.value = list[index];
  } else {
    value.value = text.value;
  }
}

function setList(config: any, text: any, textList: any) {
  const index = textList.indexOf(text.value);
  if (index >= 0 && index < textList.length) {
    text.value = textList[index];
  } else {
    text.value = config;
  }
}

const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

async function buildConfig(): Promise<RenderConfig | null> {
  if (!(await form.value!.validate()).valid) {
    toast(t('has-error'), 'error');
    return null;
  }

  if (!updateMaxParticles()) {
    return null;
  }

  return {
    resolution: (() => {
      let parts = resolution.value.split('x');
      return [parseInt(parts[0]!), parseInt(parts[1]!)];
    })(),
    endingLength: parseFloat(endingLength.value),
    chartDebugLine: chartDebugLine.value,
    chartDebugNote: chartDebugNote.value,
    chartRatio: chartRatio.value,
    fps: parseInt(fps.value),
    hardwareAccel: hwAccel.value,
    hevc: encoder.value === encoderList.value[1],
    mpeg4: encoder.value === encoderList.value[2],
    customEncoder: encoderList.value.includes(encoder.value!) ? null : encoder.value!,
    dynamicBitrateControl: encoder.value === encoderList.value[2] || dynamicBitrateControl.value,
    bitrate: encoder.value === encoderList.value[2] ? '7' : bitrate.value,

    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value!)]!,
    challengeRank: parseInt(challengeRank.value),
    noteScale: noteScale.value,
    playerAvatar: playerAvatar.value ? (playerAvatar.value.length ? playerAvatar.value : null) : null,
    playerName: playerName.value,
    playerRks: parseFloat(playerRks.value),
    sampleCount: parseInt(sampleCount.value),
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
    compressionRatio: compressionRatio.value,
    limitThreshold: limitThreshold.value,
    allGood: judgeMode.value === t('judge-modes').split(',')[1] ? true : false,
    allBad: false,
    watermark: watermark.value,
    combo: combo.value,
    difficulty: difficulty.value,
    judgeOffset: parseInt(judgeOffset.value) / 1000,
    fileNameFormat: fileNameFormat.value,
    bgBlurriness: parseFloat(bgBlurriness.value),

    maxParticles: maxParticles.value,
    playStartTime: parseFloat(playStartTime.value),
    playEndTime: parseFloat(playEndTime.value),

    fade: parseFloat(fade.value),
    alphaTint: alphaTint.value,

    renderLoading: renderLoading.value,
    renderLine: renderLine.value,
    renderLineExtra: renderLineExtra.value,
    renderNote: renderNote.value,
    renderUiPause: renderUiPause.value,
    renderUiName: renderUiName.value,
    renderUiLevel: renderUiLevel.value,
    renderUiScore: renderUiScore.value,
    renderUiCombo: renderUiCombo.value,
    renderUiBar: renderUiBar.value,
    renderBg: renderBg.value,
    renderBgDim: renderBgDim.value,
    particle: particle.value,
    renderExtra: renderExtra.value,
    renderDoubleHint: renderDoubleHint.value,
    
    aggressive: aggressive.value,
    roman: roman.value,
    chinese: chinese.value,

    forceLimit: forceLimit.value,
    hires: hires.value,
    loudnessEqualization: loudnessEqualization.value,
    audioMixOptimization: audioMixOptimization.value,
  };
}

function applyAspectRatio(aspectRatio: number) {
  let h = parseInt(resolution.value.split('x')[1]!);

  if (aspectRatio <= 1.0) {
    resolution.value = `${ h }x${ h }`
  } else {
    let w = Math.floor(h * aspectRatio);
    resolution.value = `${ w }x${ h }`
  }
}

function applyResolution(w?: number, h?: number) {
  if (w) {
    let width = parseInt(resolution.value.split('x')[0]!);
    let ratio = w / width;
    let height = Math.floor(parseInt(resolution.value.split('x')[1]!) * ratio);
    
    resolution.value = `${ w }x${ height }`
  }
  if (h) {
    let height = parseInt(resolution.value.split('x')[1]!);
    let ratio = h / height;
    let width = Math.floor(parseInt(resolution.value.split('x')[0]!) * ratio);

    resolution.value = `${ width }x${ h }`
  }
}


defineExpose({ buildConfig, applyAspectRatio, applyConfig });

function StickyLabel(props: { title: string }) {
  return h('div', { class: 'mb-4 bg-surface sticky-label', style: 'z-index: 2' }, [h('h3', { class: 'pa-1' }, props.title), h(VDivider)]);
}

/*function applyCrf() { // not working in combo box
  if (bitrateControl.value === bitrateControlList[0]) {
    bitrate.value = bitrateCrfList[0];
  } else if (bitrateControl.value === bitrateControlList[1]) {
    bitrate.value = bitrateList[0];
  }
}*/

function applyConfig(config: RenderConfig) {
  resolution.value = config.resolution.join('x');
  endingLength.value = String(config.endingLength);
  chartDebugLine.value = config.chartDebugLine;
  chartDebugNote.value = config.chartDebugNote;
  chartRatio.value = config.chartRatio;
  fps.value = String(config.fps);
  hwAccel.value = config.hardwareAccel;
  if (config.hevc) {
    encoder.value = encoderList.value[1];
  } else if (config.mpeg4) {
    encoder.value = encoderList.value[2];
  } else if (config.customEncoder) {
    encoder.value = config.customEncoder;
  } else {
    encoder.value = encoderList.value[0];
  }
  dynamicBitrateControl.value = config.mpeg4 || config.dynamicBitrateControl;
  bitrate.value = config.bitrate;

  challengeColor.value = t('challenge-colors').split(',')[STD_CHALLENGE_COLORS.indexOf(config.challengeColor)];
  challengeRank.value = String(config.challengeRank);
  noteScale.value = config.noteScale;
  playerAvatar.value = config.playerAvatar || undefined;
  playerName.value = config.playerName;
  playerRks.value = String(config.playerRks);
  sampleCount.value = String(config.sampleCount);
  respack.value = respacks.value.find((x) => x.path === config.resPackPath) || respacks.value[0]!;
  volumeMusic.value = config.volumeMusic;
  volumeSfx.value = config.volumeSfx;
  compressionRatio.value = config.compressionRatio;
  limitThreshold.value = config.limitThreshold;
  watermark.value = config.watermark;
  combo.value = config.combo;
  difficulty.value = config.difficulty;
  judgeOffset.value = String(config.judgeOffset * 1000);

  allGood.value = judgeMode.value === t('judge-modes').split(',')[1] ? true : false;
  allBad.value = false;
  if (config.allGood) judgeMode.value = t('judge-modes').split(',')[1]
  else if (config.allBad) judgeMode.value = t('judge-modes').split(',')[2]
  else judgeMode.value = t('judge-modes').split(',')[0];

  maxParticles.value = config.maxParticles;
  const index = maxParticlesList.indexOf(maxParticles.value);
  if (index >= 0 && index < maxParticlesTextList.length) {
    maxParticlesText.value = maxParticlesTextList[index];
  } else {
    maxParticlesText.value = String(maxParticles.value);
  }
  fade.value = String(config.fade);
  alphaTint.value = config.alphaTint;

  renderLoading.value = config.renderLoading;
  renderLine.value = config.renderLine;
  renderLineExtra.value = config.renderLineExtra;
  renderNote.value = config.renderNote;
  renderUiPause.value = config.renderUiPause;
  renderUiName.value = config.renderUiName;
  renderUiLevel.value = config.renderUiLevel;
  renderUiScore.value = config.renderUiScore;
  renderUiCombo.value = config.renderUiCombo;
  renderUiBar.value = config.renderUiBar;
  renderBg.value = config.renderBg;
  renderBgDim.value = config.renderBgDim;
  particle.value = config.particle;
  renderExtra.value = config.renderExtra;
  renderDoubleHint.value = config.renderDoubleHint;
}

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
const preset = ref(DEFAULT_PRESET);
async function updatePresets() {
  presets.value = await getPresets();
  preset.value = presets.value.find((x) => x.key === preset.value.key) || presets.value[0]!;
}
updatePresets();

async function openRespackFolder() {
  try {
    await invoke('open_respack_folder');
  } catch (e) {
    toastError(e);
  }
}

async function createPreset() {
  let config = await buildConfig();
  if (!config) return;
  let name = prompt(t('preset-create-title'));
  if (!name || !name.length) return;
  if (name === 'default' || name === t('default-preset')) {
    toast(t('preset-cannot-use-default'), 'error');
    return;
  }
  try {
    await invoke('add_preset', { name, config });
    await updatePresets();
    preset.value = presets.value.find((x) => x.key === name) || presets.value[0]!;
    toast(t('preset-created'), 'success');
  } catch (e) {
    toastError(e);
  }
}
async function deletePreset() {
  try {
    await invoke('remove_preset', { name: preset.value.key });
    await updatePresets();
    toast(t('preset-deleted'), 'success');
  } catch (e) {
    toastError(e);
  }
}
async function replacePreset() {
  let config = await buildConfig();
  if (!config) return;
  try {
    await invoke('remove_preset', { name: preset.value.key });
    await invoke('add_preset', { name: preset.value.key, config });
    await updatePresets();
    toast(t('preset-replaced'), 'success');
  } catch (e) {
    toastError(e);
  }
}

function setConfigForSocial() {
  applyResolution(undefined, 720);
  encoder.value = encoderList.value[1];
  dynamicBitrateControl.value = true;
  bitrate.value = '40';
}
function setConfigForQuality() {
  applyResolution(undefined, 1600);
  encoder.value = encoderList.value[1];
  dynamicBitrateControl.value = true;
  bitrate.value = '24';
  hires.value = true;
}
</script>

<template>
  <v-layout style="height: 56px;">
    <v-bottom-navigation 
      class="navigation"
      v-model="page"
      horizontal
    >
      <v-btn class="navigation">
        <v-icon>mdi-star-box</v-icon>
        {{ t('title.common') }}
      </v-btn>
      <v-btn class="navigation">
        <v-icon>mdi-video-box</v-icon>
        {{ t('title.output') }}
      </v-btn>
      <v-btn class="navigation">
        <v-icon>mdi-account</v-icon>
        {{ t('title.player') }}
      </v-btn>
      <v-btn class="navigation">
        <v-icon>mdi-image-area</v-icon>
        {{ t('title.graphics') }}
      </v-btn>
      <v-btn class="navigation">
        <v-icon>mdi-music</v-icon>
        {{ t('title.audio') }}
      </v-btn>
      <v-btn class="navigation">
        <v-icon>mdi-toolbox</v-icon>
        {{ t('title.other') }}
      </v-btn>
    </v-bottom-navigation>
  </v-layout>
  <VDivider />

  <v-form ref="form" validateOn="eager" class="scroll-mask" style="max-height: calc(100vh - 320px); overflow-x: hidden; overflow-y: auto; margin-top: 0px;">
    <div v-show="page === 0 || page === undefined"
      style="padding: 10px 0; display: flex; flex-direction: row; align-items: center; gap: 8px;">
      <v-combobox @update:model-value="(val: Preset) => applyConfig(val.config)" :label="t('presets')" :items="presets"
        item-title="name" v-model="preset" style="flex: 1;"></v-combobox>
      <v-btn class="text-caption" color="btn-large" v-t="'preset-refresh'" size="large" @click="updatePresets" style="flex: .2;"></v-btn>
      <v-btn class="text-caption" color="btn-large" v-t="'preset-create'" size="large" @click="createPreset" style="flex: .2;"></v-btn>
      <v-btn class="text-caption" color="btn-large" v-t="'preset-delete'" size="large" :disabled="preset.key === 'default'"
        @click="deletePreset" style="flex: .2;"></v-btn>
      <v-btn class="text-caption" color="btn-large" v-t="'preset-replace'" size="large" :disabled="preset.key === 'default'"
        @click="replacePreset" style="flex: .2;"></v-btn>
    </div>

    <div v-show="page === 0">
      <StickyLabel :title="t('title.common')"></StickyLabel>
      <v-row no-gutters class="mx-n2">
        <v-col cols="3">
          <v-combobox :label="t('resolution')" :items="RESOLUTIONS" class="mx-2" :rules="[resolutionRule]" v-model="resolution"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-combobox v-if="dynamicBitrateControl && encoder !== encoderList[2]" :label="t('bitrate-crf')" :items="bitrateCrfList" :title="t('bitrate-crf-tip')" class="mx-2" type="number" :rules="[isCrf]" v-model="bitrate"></v-combobox>
          <v-combobox v-if="!dynamicBitrateControl && encoder !== encoderList[2]" :label="t('bitrate')" :items="bitrateList" class="mx-2" :rules="[isBitrate]" v-model="bitrate"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-text-field
            readonly
            class="mx-2"
            accept="image/*"
            :label="t('player-avatar')"
            @click="chooseAvatar"
            @click.clear="playerAvatar = undefined"
            clearable
            :model-value="playerAvatar ? playerAvatar.split('\\').pop()!.split('/').pop() : ''"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('player-name')" v-model="playerName"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 my-2">
        <v-col cols="3" class="px-2">
          <v-combobox v-model="encoder" :items="encoderList" :label="t('encoder')"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-autocomplete class="mx-2" :label="t('challenge-color')" :rules="[RULES.notEmpty]" :items="t('challenge-colors').split(',')" v-model="challengeColor"></v-autocomplete>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('player-rks')" :rules="[RULES.positiveOrZero]" type="number" v-model="playerRks"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('challenge-rank')" :rules="[RULES.int, RULES.positiveOrZero]" type="number" v-model="challengeRank"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 my-2">
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('ending-length')" v-model="endingLength" type="number" :rules="[RULES.notEmpty]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('render-start-time')" v-model="playStartTime" type="number" :rules="[RULES.positiveOrZero]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('render-end-time')" v-model="playEndTime" type="number" :rules="[RULES.positiveOrNull]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <TipSwitch class="ml-n1" :label="t('render-loading')" color="btn" v-model="renderLoading"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2">
        <v-col cols="1" class="mx-2" @click="setConfigForSocial">
          <TooltipIcon :tooltip="t('output-ref-0')"></TooltipIcon>
        </v-col>
        <v-col cols="1" class="mx-2" @click="setConfigForQuality">
          <TooltipIcon :tooltip="t('output-ref-1')"></TooltipIcon>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2" />
    </div>

    <div v-show="page === 1 || page === undefined">
      <StickyLabel :title="t('title.output')"></StickyLabel>

      <v-row no-gutters class="mx-n2 my-2">
        <v-col cols="3">
          <v-combobox :label="t('resolution')" :items="RESOLUTIONS" class="mx-2" :rules="[resolutionRule]" v-model="resolution"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-combobox :label="t('fps')" :items="fpsList" class="mx-2" type="number" :rules="[RULES.int, RULES.positive]" v-model="fps"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-text-field :label="t('sample-count')" class="mx-2" type="number" :rules="[sampleCountRule]" v-model="sampleCount" :title="t('sample-count-tips')"></v-text-field>
        </v-col>
        <v-col cols="3" v-if="encoder !== encoderList[2]">
          <TipSwitch :label="t('hw-accel')" color="btn" v-model="hwAccel" :title="t('hw-accel-tips')"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 my-2">
        <v-col cols="3" class="px-2">
          <v-combobox v-model="encoder" :items="encoderList" :label="t('encoder')"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('file-name-format')" v-model="fileNameFormat" type="text" :rules="[RULES.notEmpty, RULES.notNull]" append-inner-icon="mdi-help-circle-outline" @click:append-inner="fileNameFormatDialog = true"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-combobox class="mx-2" v-if="dynamicBitrateControl && encoder !== encoderList[2]" :label="t('bitrate-crf')" :items="bitrateCrfList" :title="t('bitrate-crf-tip')" type="number" :rules="[isCrf]" v-model="bitrate"></v-combobox>
          <v-combobox class="mx-2" v-if="!dynamicBitrateControl && encoder !== encoderList[2]" :label="t('bitrate')" :items="bitrateList" :rules="[isBitrate]" v-model="bitrate"></v-combobox>
        </v-col>
        <v-col cols="3">
          <TipSwitch v-if="encoder !== encoderList[2]" :label="t('dynamic-bitrate-control')" color="btn" @change="updateBitrate" v-model="dynamicBitrateControl"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2">
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('ending-length')" v-model="endingLength" type="number" :rules="[RULES.notEmpty]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('render-start-time')" v-model="playStartTime" type="number" :rules="[RULES.positiveOrZero]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('render-end-time')" v-model="playEndTime" type="number" :rules="[RULES.positiveOrNull]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('render-loading')" color="btn" v-model="renderLoading"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2">
      </v-row>
    </div>
    <div v-show="page === 2 || page === undefined">
      <StickyLabel :title="t('title.player')"></StickyLabel>
      <v-row no-gutters class="mx-n2 my-2">
        <v-col cols="4">
          <v-text-field
            readonly
            class="mx-2"
            accept="image/*"
            :label="t('player-avatar')"
            @click="chooseAvatar"
            @click.clear="playerAvatar = undefined"
            clearable
            :model-value="playerAvatar ? playerAvatar.split('\\').pop()!.split('/').pop() : ''"></v-text-field>
        </v-col>
        <v-col cols="8">
          <v-text-field class="mx-2" :label="t('player-name')" v-model="playerName"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-1">
        <v-col cols="4">
          <v-text-field class="mx-2" :label="t('player-rks')" :rules="[RULES.positiveOrZero]" type="number" v-model="playerRks"></v-text-field>
        </v-col>
        <v-col cols="4">
          <v-autocomplete class="mx-2" :label="t('challenge-color')" :rules="[RULES.notEmpty]" :items="t('challenge-colors').split(',')" v-model="challengeColor"></v-autocomplete>
        </v-col>
        <v-col cols="4">
          <v-text-field class="mx-2" :label="t('challenge-rank')" :rules="[RULES.int, RULES.positiveOrZero]" type="number" v-model="challengeRank"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2" />
    </div>

    <div v-show="page === 3 || page === undefined">
      <StickyLabel :title="t('title.graphics')"></StickyLabel>
      <v-row no-gutters class="mr-1 mt-4 align-center">
        <v-col cols="8">
          <v-combobox class="mr-1" :label="t('respack')" :rues="[RULES.notEmpty]" :items="respacks" item-title="name" v-model="respack"></v-combobox>
        </v-col>
        <v-col cols="2" class="d-flex justify-center">
          <v-btn class="config-btn pa-1 text-caption" color="btn-large" size="large" @click="updateRespacks" v-t="'respack-refresh'" style="flex: .9;"></v-btn>
        </v-col>
        <v-col cols="2" class="d-flex justify-center">
          <v-btn class="config-btn pa-1 text-caption" color="btn-large" size="large" @click="openRespackFolder" v-t="'respack-open'" style="flex: .9;"></v-btn>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-6 align-center">
        <v-col cols="6" class="pl-6">
          <TipSlider :label="t('chart-debug-line')" :tooltip="t('chart-debug-line-tip')" color="btn" thumb-label="always" :min="0" :max="1" :step="0.01" v-model="chartDebugLine"></TipSlider>
        </v-col>
        <v-col cols="6" class="pl-4 pr-4">
          <TipSlider :label="t('chart-ratio')" :tooltip="t('chart-ratio-tip')" color="btn" thumb-label="always" :min="0.05" :max="1" :step="0.01" v-model="chartRatio"> </TipSlider>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-6 align-center">
        <v-col cols="6" class="pl-6">
          <TipSlider :label="t('chart-debug-note')" :tooltip="t('chart-debug-note-tip')" color="btn" thumb-label="always" :min="0" :max="1" :step="0.01" v-model="chartDebugNote"> </TipSlider>
        </v-col>
        <v-col cols="6" class="pl-4 pr-4">
          <TipSlider :label="t('note-scale')" :tooltip="t('note-scale-tip')" color="btn" thumb-label="always" :min="0" :max="5" :step="0.01" v-model="noteScale"> </TipSlider>
        </v-col>
      </v-row>

      <v-row no-gutters class="px-n2 mt-2">
        <v-col cols="12" class="px-1">
          <v-expansion-panels>
            <v-expansion-panel :title="t('render-item')">
              <v-expansion-panel-text>
                <v-row no-gutters class="mx-n2 mt-2">
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-line')" color="btn" v-model="renderLine"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-line-extra')" color="btn" v-model="renderLineExtra"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-note')" color="btn" v-model="renderNote"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-double-hint')" color="btn" v-model="renderDoubleHint"></v-checkbox>
                  </v-col>
                </v-row>
                <v-row no-gutters class="mx-n2 mt-2">
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-ui-pause')" color="btn" v-model="renderUiPause"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-ui-name')" color="btn" v-model="renderUiName"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-ui-level')" color="btn" v-model="renderUiLevel"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-ui-score')" color="btn" v-model="renderUiScore"></v-checkbox>
                  </v-col>
                </v-row>
                <v-row no-gutters class="mx-n2 mt-2">
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-ui-combo')" color="btn" v-model="renderUiCombo"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-ui-bar')" color="btn" v-model="renderUiBar"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-background')" color="btn" v-model="renderBg"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-background-dim')" color="btn" v-model="renderBgDim"></v-checkbox>
                  </v-col>
                </v-row>
                <v-row no-gutters class="mx-n2 mt-2">
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-particle')" color="btn" v-model="particle"></v-checkbox>
                  </v-col>
                  <v-col cols="3" class="px-2">
                    <v-checkbox :label="t('render-extra')" color="btn" v-model="renderExtra"></v-checkbox>
                  </v-col>
                </v-row>
              </v-expansion-panel-text>
            </v-expansion-panel>
          </v-expansion-panels>
        </v-col>
      </v-row>

      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="4" class="px-2">
          <TipSwitch :label="t('aggressive')" color="btn" v-model="aggressive"></TipSwitch>
        </v-col>
        <v-col cols="4" class="px-2">
          <TipSwitch :label="t('roman')" color="btn" v-model="roman"></TipSwitch>
        </v-col>
        <v-col cols="4" class="px-2">
          <TipSwitch :label="t('chinese')" color="btn" v-model="chinese"></TipSwitch>
        </v-col>
      </v-row>

      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('bg-blurriness')" v-model="bgBlurriness" type="number" :rules="[RULES.positiveOrZero, RULES.less10000]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('watermark')" v-model="watermark"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('combo')" :rules="[RULES.notCOMBO]" v-model="combo"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-combobox class="mx-2" :label="t('max-particles')" :rules="[RULES.notEmpty]" :title="t('max-particles-tip')" :items="maxParticlesTextList" v-model="maxParticlesText"></v-combobox>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2" />
    </div>

    <div v-show="page === 4 || page === undefined">
      <StickyLabel :title="t('title.audio')"></StickyLabel>
      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="3" class="px-2">
          <TipSwitch :label="t('force-limit')" color="btn" v-model="forceLimit"></TipSwitch>
        </v-col>
        <v-col cols="3" class="px-2">
          <TipSwitch :label="t('hires')" color="btn" v-model="hires"></TipSwitch>
        </v-col>
        <v-col cols="3" class="px-2">
          <TipSwitch :label="t('loudness-equalization')" color="btn" v-model="loudnessEqualization"></TipSwitch>
        </v-col>
        <v-col cols="3" class="px-2">
          <TipSwitch :label="t('audio-mix-optimization')" color="btn" v-model="audioMixOptimization"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-6 align-center px-6">
        <v-col cols="4">
          <v-slider :label="t('volume-music')" color="btn" thumb-label="always" :min="0" :max="2" :step="0.01" v-model="volumeMusic"> </v-slider>
        </v-col>
        <v-col cols="4">
          <v-slider :label="t('volume-sfx')" color="btn" thumb-label="always" :min="0" :max="2" :step="0.01" v-model="volumeSfx"> </v-slider>
        </v-col>
        <v-col cols="4">
          <v-slider v-if="!forceLimit" :label="t('compression-ratio')" color="btn" thumb-label="always" :min="1" :max="20" :step="1" v-model="compressionRatio"> </v-slider>
          <v-slider v-if="forceLimit" :label="t('limit-threshold')" color="btn" thumb-label="always" :min="0.1" :max="2" :step="0.01" v-model="limitThreshold"> </v-slider>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2" />
    </div>

    <div v-show="page === 5 || page === undefined">
      <StickyLabel :title="t('title.other')"></StickyLabel>
      <v-row no-gutters class="mx-n2 align-center">
        <v-col cols="3">
          <v-autocomplete class="mx-2" :label="t('judge-mode')" :rules="[RULES.notEmpty]" :items="t('judge-modes').split(',')" v-model="judgeMode"></v-autocomplete>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('judgeOffset')" v-model="judgeOffset" type="number" :rules="[RULES.notEmpty]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('difficulty')" v-model="difficulty"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('fade')" :title="t('fade-tip')" v-model="fade" type="number" :rules="[RULES.notEmpty]"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="3">
          <v-switch class="text-center justify-center mr-2 d-flex" :label="t('alpha-tint')" color="btn" :title="t('alpha-tint-tip')" v-model="alphaTint"></v-switch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2" />
    </div>
  </v-form>

  <v-dialog v-model="fileNameFormatDialog" width="850px" class="log-card-bg">
    <v-card class="log-card-only-window" style="background: rgba(var(--v-theme-dialog), 0.4) !important;">
      <v-card-title v-t="'file-name-format'"> </v-card-title>
      <v-card-text>
        <v-text-field class="" :label="t('file-name-format')" v-model="fileNameFormat" type="text" :rules="[RULES.notEmpty, RULES.notNull]"></v-text-field>
        <pre class="mx-2">{{ t('file-name-format-tip') }}</pre>

      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn class="hover-scale" variant="text" @click="fileNameFormatDialog = false" v-t="'close'"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.gradient-primary {
  background: linear-gradient(45deg, #6366f1, #8b5cf6) !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2);
  transition: transform 0.2s, box-shadow 0.2s;
}

.elevated-stepper {
  border-radius: 16px !important;
  box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1) !important;
  background: rgba(23, 9, 99, 0.8) !important;
  backdrop-filter: blur(8px);
}

.sticky-label {
  background: rgba(54, 50, 98, 0) !important;
}

.navigation {
  background: rgba(54, 50, 98, 0) !important;
  /* box-shadow: 0 !important; */
}

.config-btn {
  background: rgba(54, 50, 98, 1);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

</style>