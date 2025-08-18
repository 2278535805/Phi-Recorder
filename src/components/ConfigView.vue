<script setup lang="ts">
import { ref, h } from 'vue';

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

const RESOLUTIONS = [ '1280x720', '1920x1080', '1620x1080', '1440x1080', '2560x1440', '2844x1600', '2388x1668', '3840x2160']
const ffmpegPresetPresetTextList = t('ffmpeg-preset-list').split(','),
  ffmpegPresetPresetList = ['veryfast p1 veryfast speed', 'faster p2 faster speed','fast p3 fast speed', 'medium p4 medium balanced', 'slow p5 slow quality', 'slower p6 slower quality', 'veryslow p7 veryslow quality'],
  ffmpegPresetText = ref(ffmpegPresetPresetTextList[3]),
  ffmpegPreset = ref(ffmpegPresetPresetList[3])

const
  endingLength = ref(String(DEFAULT_RENDER_CONFIG.endingLength)),
  chartDebugLine = ref(DEFAULT_RENDER_CONFIG.chartDebugLine),
  chartDebugNote = ref(DEFAULT_RENDER_CONFIG.chartDebugNote),
  chartRatio = ref(DEFAULT_RENDER_CONFIG.chartRatio),
  allGood = ref(DEFAULT_RENDER_CONFIG.allGood),
  allBad = ref(DEFAULT_RENDER_CONFIG.allBad),
  fpsList = ['30', '60', '120'],
  fps = ref(String(DEFAULT_RENDER_CONFIG.fps)),
  hwAccel = ref(DEFAULT_RENDER_CONFIG.hardwareAccel),
  dynamicBitrateControl = ref(true),
  bitrate = ref('28'),
  bitrateList = ['2M', '5M', '7M'],
  bitrateCrfList = ['24', '28', '35', '40'],
  resolution = ref('1920x1080')
const encoderList = ref(t('encoder-list').split(','))
const encoder = ref(t('encoder-list').split(',')[0])

const challengeColor = ref(t('challenge-colors').split(',')[5]),
  challengeRank = ref(String(DEFAULT_RENDER_CONFIG.challengeRank)),
  noteScale = ref(DEFAULT_RENDER_CONFIG.noteScale),
  playerAvatar = ref<string>(),
  playerName = ref(DEFAULT_RENDER_CONFIG.playerName),
  playerRks = ref(String(DEFAULT_RENDER_CONFIG.playerRks)),
  sampleCount = ref(String(DEFAULT_RENDER_CONFIG.sampleCount))

const volumeMusic = ref(DEFAULT_RENDER_CONFIG.volumeMusic),
  volumeSfx = ref(DEFAULT_RENDER_CONFIG.volumeSfx),
  compressionRatio = ref(DEFAULT_RENDER_CONFIG.compressionRatio),
  limitThreshold = ref(DEFAULT_RENDER_CONFIG.limitThreshold)


const renderList = ref(t('render-list').split(','))
const render = ref<string[]>([])
render.value.push(...renderList.value.slice(1, 15))

const expandList = ref(t('expand-list').split(','))
const expand = ref<string[]>([])

const audioList = ref(t('audio-list').split(','))
const audio = ref([audioList.value[0]])

const
  combo = ref(DEFAULT_RENDER_CONFIG.combo),
  difficulty = ref(DEFAULT_RENDER_CONFIG.difficulty),
  judgeOffset = ref(String(DEFAULT_RENDER_CONFIG.judgeOffset)),
  simpleFileName = ref(DEFAULT_RENDER_CONFIG.simpleFileName),
  bgBlurriness = ref(String(DEFAULT_RENDER_CONFIG.bgBlurriness)),
  watermark = ref(DEFAULT_RENDER_CONFIG.watermark)

const maxParticlesText = ref(t('max-particles-list').split(',')[0])
const maxParticles = ref(DEFAULT_RENDER_CONFIG.maxParticles)
const maxParticlesTextList = t('max-particles-list').split(',')
const maxParticlesList = [5000, 25000, 200000];

const
  renderStartTime = ref(String(DEFAULT_RENDER_CONFIG.renderStartTime)),
  renderEndTime = ref('');

const judgeMode = ref(t('judge-modes').split(',')[0])
const fade = ref(String(DEFAULT_RENDER_CONFIG.fade))
const alphaTint = ref(DEFAULT_RENDER_CONFIG.alphaTint)


function parseResolution(resolution: string): [number, number] | null {
  let parts = resolution.split(/[xX]/g);
  if (parts.length !== 2) return null;
  let ws = parts[0].trim(),
    hs = parts[1].trim();
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
    const unit = match[2].toLowerCase();
  
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
  respack.value = respacks.value.find((x) => x.name === respack.value.name) || respacks.value[0];
}
updateRespacks();

function updateBitrate() {
  if (dynamicBitrateControl.value) {
    bitrate.value = bitrateCrfList[0];
  } else {
    bitrate.value = bitrateList[0];
  }
}

function updateMaxParticles() {
  const index = maxParticlesTextList.indexOf(maxParticlesText.value);
  const textNum = Number(maxParticlesText.value);
  if (index >= 0 && index < maxParticlesTextList.length) {
    maxParticles.value = maxParticlesList[index];
  } else if (Number.isInteger(textNum) && textNum > 0) {
    maxParticles.value = parseInt(maxParticlesText.value);
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
  updateList(ffmpegPreset, ffmpegPresetText, ffmpegPresetPresetList, ffmpegPresetPresetTextList);

  return {
    resolution: (() => {
      let parts = resolution.value.split('x');
      return [parseInt(parts[0]), parseInt(parts[1])];
    })(),
    ffmpegPreset: ffmpegPreset.value,
    endingLength: parseFloat(endingLength.value),
    chartDebugLine: chartDebugLine.value,
    chartDebugNote: chartDebugNote.value,
    chartRatio: chartRatio.value,
    fps: parseInt(fps.value),
    hardwareAccel: hwAccel.value,
    hevc: encoder.value === encoderList.value[1],
    mpeg4: encoder.value === encoderList.value[2],
    customEncoder: encoderList.value.includes(encoder.value) ? null : encoder.value,
    dynamicBitrateControl: encoder.value === encoderList.value[2] || dynamicBitrateControl.value,
    bitrate: encoder.value === encoderList.value[2] ? '7' : bitrate.value,

    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value)],
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
    simpleFileName: simpleFileName.value,
    bgBlurriness: parseFloat(bgBlurriness.value),
    
    disableLoading: !render.value.includes(renderList.value[0]),
    renderLine: render.value.includes(renderList.value[1]),
    renderLineExtra: render.value.includes(renderList.value[2]),
    renderNote: render.value.includes(renderList.value[3]),
    renderUiPause: render.value.includes(renderList.value[4]),
    renderUiName: render.value.includes(renderList.value[5]),
    renderUiLevel: render.value.includes(renderList.value[6]),
    renderUiScore: render.value.includes(renderList.value[7]),
    renderUiCombo: render.value.includes(renderList.value[8]),
    renderUiBar: render.value.includes(renderList.value[9]),
    renderBg: render.value.includes(renderList.value[10]),
    renderBgDim: render.value.includes(renderList.value[11]),
    particle: render.value.includes(renderList.value[12]),
    renderExtra: render.value.includes(renderList.value[13]),
    renderDoubleHint: render.value.includes(renderList.value[14]),

    aggressive: expand.value.includes(expandList.value[0]),
    roman: expand.value.includes(expandList.value[1]),
    chinese: expand.value.includes(expandList.value[2]),

    forceLimit: audio.value.includes(audioList.value[0]),
    hires: audio.value.includes(audioList.value[1]),
    loudnessEqualization: audio.value.includes(audioList.value[2]),

    maxParticles: maxParticles.value,
    renderStartTime: parseFloat(renderStartTime.value),
    renderEndTime: parseFloat(renderEndTime.value),

    fade: parseFloat(fade.value),
    alphaTint: alphaTint.value,
  };
}

function applyAspectRatio(aspectRatio: number) {
  if (preset.value.key !== 'default') return;

  let h = parseInt(resolution.value.split('x')[1]);

  if (aspectRatio <= 1.0) {
    resolution.value = `${ h }x${ h }`
  } else {
    let w = Math.floor(h * aspectRatio);
    resolution.value = `${ w }x${ h }`
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
  setList(config.ffmpegPreset, ffmpegPresetText, ffmpegPresetPresetTextList);
  //ffmpegPresetText.value = ffmpegPresetPresetTextList[ffmpegPresetPresetList.indexOf(config.ffmpegPreset)];
  endingLength.value = String(config.endingLength);
  chartDebugLine.value = config.chartDebugLine;
  chartDebugNote.value = config.chartDebugNote;
  chartRatio.value = config.chartRatio;
  fps.value = String(config.fps);
  hwAccel.value = config.hardwareAccel;
  if (config.hevc) {
    encoder.value = encoderList.value[1];
  }
  if (config.mpeg4) {
    encoder.value = encoderList.value[2];
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
  respack.value = respacks.value.find((x) => x.path === config.resPackPath) || respacks.value[0];
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

  render.value = [];
  if (!config.disableLoading) render.value.push(renderList.value[0]);
  if (config.renderLine) render.value.push(renderList.value[1]);
  if (config.renderLineExtra) render.value.push(renderList.value[2]);
  if (config.renderNote) render.value.push(renderList.value[3]);
  if (config.renderUiPause) render.value.push(renderList.value[4]);
  if (config.renderUiName) render.value.push(renderList.value[5]);
  if (config.renderUiLevel) render.value.push(renderList.value[6]);
  if (config.renderUiScore) render.value.push(renderList.value[7]);
  if (config.renderUiCombo) render.value.push(renderList.value[8]);
  if (config.renderUiBar) render.value.push(renderList.value[9]);
  if (config.renderBg) render.value.push(renderList.value[10]);
  if (config.renderBgDim) render.value.push(renderList.value[11]);
  if (config.particle) render.value.push(renderList.value[12]);
  if (config.renderExtra) render.value.push(renderList.value[13]);
  if (config.renderDoubleHint) render.value.push(renderList.value[14]);
  

  expand.value = [];
  if (config.aggressive) expand.value.push(expandList.value[0]);
  if (config.roman) expand.value.push(expandList.value[1]);
  if (config.chinese) expand.value.push(expandList.value[2]);

  audio.value = [];
  if (config.forceLimit) audio.value.push(audioList.value[0]);
  if (config.hires) audio.value.push(audioList.value[1]);
  if (config.loudnessEqualization) audio.value.push(audioList.value[2]);

  maxParticles.value = config.maxParticles;
  const index = maxParticlesList.indexOf(maxParticles.value);
  if (index >= 0 && index < maxParticlesTextList.length) {
    maxParticlesText.value = maxParticlesTextList[index];
  } else {
    maxParticlesText.value = String(maxParticles.value);
  }
  fade.value = String(config.fade);
  alphaTint.value = config.alphaTint;
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
    preset.value = presets.value.find((x) => x.key === name) || presets.value[0];
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

  <v-form ref="form" validateOn="eager" class="scroll-mask" style="max-height: 48vh; overflow-x: hidden; overflow-y: auto; margin-top: 0px;">
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
          <v-select v-model="render" :items="renderList" :label="t('render')" multiple>
            <template v-slot:selection="{ index }">
              <span v-if="index === 0" class="text-caption">
                ({{ render.length }} {{ t('selects') }})
              </span>
            </template>
          </v-select>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('ending-length')" v-model="endingLength" type="number" :rules="[RULES.notEmpty]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('player-rks')" :rules="[RULES.positiveOrZero]" type="number" v-model="playerRks"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('challenge-rank')" :rules="[RULES.int, RULES.positiveOrZero]" type="number" v-model="challengeRank"></v-text-field>
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
        <v-col cols="3" v-show="encoder !== encoderList[2]">
          <v-combobox class="mx-2" :label="t('ffmpeg-preset')" :items="ffmpegPresetPresetTextList" :rules="[RULES.nonSpaces, RULES.notEmpty]" v-model="ffmpegPresetText"></v-combobox>
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
          <v-text-field class="mx-2" :label="t('ending-length')" v-model="endingLength" type="number" :rules="[RULES.notEmpty]" v-show="renderEndTime === null || renderEndTime === ''"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('render-start-time')" v-model="renderStartTime" type="number" :rules="[RULES.positiveOrZero]" v-show="!render.includes(renderList[0])"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('render-end-time')" v-model="renderEndTime" type="number" :rules="[RULES.positiveOrNull]" v-show="parseFloat(endingLength) === 0.0"></v-text-field>
        </v-col>
        <v-col></v-col>
        <v-col cols="1" class="mx-2 justify-right" @click="dynamicBitrateControl = true; encoder = encoderList[1]; bitrate = '40'">
          <TooltipIcon :tooltip="t('output-tip')"></TooltipIcon>
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
          <v-btn class="pa-1 text-caption" color="btn-large" size="large" @click="updateRespacks" v-t="'respack-refresh'" style="flex: .9;"></v-btn>
        </v-col>
        <v-col cols="2" class="d-flex justify-center">
          <v-btn class="pa-1 text-caption" color="btn-large" size="large" @click="openRespackFolder" v-t="'respack-open'" style="flex: .9;"></v-btn>
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
      <v-row no-gutters class="mx-n2 mt-6">
        <v-col cols="6" class="px-2">
          <v-select v-model="render" :items="renderList" :label="t('renders')" multiple>
            <template v-slot:selection="{ item, index }">
              <v-chip size="small" v-if="index < 1" :text="item.title"></v-chip>
              <span v-if="index === 1" class="text-grey text-caption align-self-center">
                (+{{ render.length - 1 }} {{ t('others') }})
              </span>
            </template>
          </v-select>
        </v-col>
        <v-col cols="6" class="px-2">
          <v-select v-model="expand" :items="expandList" :label="t('expand')" multiple>
            <template v-slot:selection="{ item, index }">
              <v-chip size="small" v-if="index < 1" :text="item.title"></v-chip>
              <span v-if="index === 1" class="text-grey text-caption align-self-center">
                (+{{ expand.length - 1 }} {{ t('others') }})
              </span>
            </template>
          </v-select>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('bg-blurriness')" v-model="bgBlurriness" type="number" :rules="[RULES.positive, RULES.less10000]"></v-text-field>
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
      <v-row no-gutters class="mt-2 align-center">
        <v-col cols="12" class="">
          <v-select v-model="audio" :items="audioList" :label="t('audio-expand')" chips multiple></v-select>
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
          <v-slider v-if="!audio.includes(audioList[0])" :label="t('compression-ratio')" color="btn" thumb-label="always" :min="1" :max="20" :step="1" v-model="compressionRatio"> </v-slider>
          <v-slider v-if="audio.includes(audioList[0])" :label="t('limit-threshold')" color="btn" thumb-label="always" :min="0.1" :max="2" :step="0.01" v-model="limitThreshold"> </v-slider>
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
          <v-switch class="mx-4" :label="t('alpha-tint')" color="btn" :title="t('alpha-tint-tip')" v-model="alphaTint"></v-switch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2" />
    </div>
  </v-form>
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

.v-btn {
  background: rgba(54, 50, 98, 1);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

</style>