<i18n>
en:
  title:
    output: Output
    player: Player
    graphics: Graphics
    audio: Audio

  resolution: Resolution
  ffmpeg-preset: Preset
  fps: FPS

  hw-accel: Hardware Acceleration
  hw-accel-tips: If render fails, try to turn it off

  fxaa: FXAA
  fxaa-tips: FXAA, as a low-cost anti-aliasing method, will cause the picture to be blurred, and it is not recommended to turn it on

  sample-count: Sample Count
  sample-count-tips: Must be a power of 2. A non-1 sample count enables MSAA, which can improve the quality of the picture while increasing the performance cost

  bitrate-control: Bitrate Control
  bitrate: level/bitrate
  bitrate-tips: CRF: This is the CRF level<br>CBR: This is the bitrate

  player-avatar: Player Avatar
  player-name: Player Name
  player-rks: Player Rks.

  image-filter: Image

  challenge-color: Challenge Mode Color
  challenge-colors: White,Green,Blue,Red,Golden,Rainbow

  challenge-rank: Challenge Mode Rank

  respack: Resource Pack
  respack-default: '[Default]'
  respack-refresh: Refresh
  respack-open: Open Folder

  note-scale: Note Scale

  double-hint: Double Hit Hint

  aggressive: Aggressive Optimization
  aggressive-tips: Improve rendering speed, Hide off-screen note, but may cause some notes to disappear

  disable-particle: Disable Particle
  disable-effect: Disable Effect

  volume-music: Music Volume
  volume-sfx: SFX Volume

  ending-length: Result Screen Duration
  disable-loading: Remove loading screen
  chart_debug: Debug Mode
  chart_ratio: Chart Zoom

  presets: Presets
  preset-refresh: Refresh
  preset-create: Create
  preset-create-title: Preset name
  preset-created: Preset created
  preset-delete: Delete
  preset-deleted: Preset deleted
  preset-replace: Replace
  preset-replaced: Preset replaced
  preset-cannot-use-default: Cannot use 'default' as preset name
  default-preset: Default

zh-CN:
  title:
    output: 输出
    player: 玩家
    graphics: 图像
    audio: 音频

  resolution: 分辨率
  ffmpeg-preset: 预设
  fps: FPS

  hw-accel: 硬件加速
  hw-accel-tips: 如果渲染失败，请尝试关闭此选项

  fxaa: FXAA
  fxaa-tips: FXAA 以低成本实现抗锯齿，但会导致画面模糊，不建议开启

  sample-count: 采样数
  sample-count-tips: 非 1 的采样数(必须为 2 的幂)会启用 MSAA(若开头无画面请关闭此项)

  bitrate-control: 码率控制
  bitrate: 级别/码率
  bitrate-tips: CRF: 该项为CRF级别<br>CBR: 该项为码率

  player-avatar: 玩家头像
  player-name: 玩家名
  player-rks: 玩家 RKS

  image-filter: 图像

  challenge-color: 课题模式颜色
  challenge-colors: 白,绿,蓝,红,金,彩

  challenge-rank: 课题模式等级

  respack: 资源包
  respack-default: '[默认]'
  respack-refresh: 刷新
  respack-open: 打开文件夹

  note-scale: 音符缩放

  double-hint: 双押提示

  aggressive: 激进优化
  aggressive-tips: 提升渲染速度，隐藏屏幕外按键，但可能会导致部分音符消失

  disable-particle: 禁用粒子
  disable-effect: 禁用特效

  volume-music: 音乐音量
  volume-sfx: 音效音量

  ending-length: 结算画面时长
  disable-loading: 删除加载画面
  chart_debug: 调试模式
  chart_ratio: 谱面缩放

  presets: 预设配置
  preset-refresh: 刷新
  preset-create: 创建
  preset-create-title: 预设配置名
  preset-created: 预设配置已创建
  preset-delete: 删除
  preset-deleted: 预设配置已删除
  preset-replace: 替换
  preset-replaced: 预设配置已替换
  preset-cannot-use-default: 不能使用 'default' 作为配置名
  default-preset: 默认

</i18n>

<script setup lang="ts">
import { ref, h } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api';
import { open } from '@tauri-apps/api/dialog';

import { VDivider, VForm } from 'vuetify/components';

import { RULES, isNumeric, toast, anyFilter, toastError } from '../common';
import type { RenderConfig } from '../model';

import TipSwitch from './TipSwitch.vue';
import TipTextField from './TipTextField.vue';

const props = defineProps<{ initAspectRatio?: number }>();

const RESOLUTIONS = [ '1280x720','1920x1080', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080'];
const ffmpegPresetPresetList = ['ultrafast', 'superfast', 'veryfast', 'faster', 'fast', 'medium', 'slow', 'slower', 'veryslow', 'placebo'];
const bitrateControlList = ['CRF','CBR'];

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

const form = ref<VForm>();

const resolution = ref('1920x1080'),
  ffmpegPreset = ref('fast'),
  fps = ref('60'),
  hwAccel = ref(true);

const fxaa = ref(false),
  sampleCount = ref('1'),
  bitrateControl = ref('CRF'),
  bitrate = ref('26');

const playerAvatar = ref<string>(),
  playerName = ref(''),
  playerRks = ref('15.0');

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

const challengeColor = ref(t('challenge-colors').split(',')[4]),
  challengeRank = ref('45');

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

const noteScale = ref(1);

const doubleHint = ref(true),
  aggressive = ref(false),
  disableParticle = ref(false),
  disableEffect = ref(false);

const volumeMusic = ref(1),
  volumeSfx = ref(1);

const endingLength = ref('-2.0');

const disableLoading = ref(true)

const chartDebug = ref(false)
const chartRatio = ref(1.0)

const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

async function buildConfig(): Promise<RenderConfig | null> {
  if (!(await form.value!.validate()).valid) {
    toast(t('has-error'), 'error');
    return null;
  }
  return {
    resolution: (() => {
      let parts = resolution.value.split('x');
      return [parseInt(parts[0]), parseInt(parts[1])];
    })(),
    ffmpegPreset: ffmpegPreset.value,
    endingLength: parseFloat(endingLength.value),
    disableLoading: disableLoading.value,
    chartDebug: chartDebug.value,
    chartRatio: chartRatio.value,
    fps: parseInt(fps.value),
    hardwareAccel: hwAccel.value,
    bitrateControl: bitrateControl.value,
    bitrate: bitrate.value,

    aggressive: aggressive.value,
    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value)],
    challengeRank: parseInt(challengeRank.value),
    disableEffect: disableEffect.value,
    doubleHint: doubleHint.value,
    fxaa: fxaa.value,
    noteScale: noteScale.value,
    particle: !disableParticle.value,
    playerAvatar: playerAvatar.value ? (playerAvatar.value.length ? playerAvatar.value : null) : null,
    playerName: playerName.value,
    playerRks: parseFloat(playerRks.value),
    sampleCount: parseInt(sampleCount.value),
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
  };
}

function onEnter() {
  if (preset.value.key !== 'default') return;
  resolution.value = RESOLUTIONS[1];
  if (props.initAspectRatio) {
    for (let res of RESOLUTIONS) {
      let [w, h] = parseResolution(res)!;
      if (Math.abs(w / h - props.initAspectRatio) < 0.01) {
        resolution.value = res;
        break;
      }
    }
  }
}

defineExpose({ buildConfig, onEnter });

function StickyLabel(props: { title: string }) {
  return h('div', { class: 'mb-4 bg-surface', style: 'position: sticky; top: 0; z-index: 2' }, [h('h3', { class: 'pa-1' }, props.title), h(VDivider)]);
}

function applyConfig(config: RenderConfig) {
  resolution.value = config.resolution.join('x');
  ffmpegPreset.value = config.ffmpegPreset;
  endingLength.value = String(config.endingLength);
  disableLoading.value = config.disableLoading;
  chartDebug.value = config.chartDebug;
  chartRatio.value = config.chartRatio;
  fps.value = String(config.fps);
  hwAccel.value = config.hardwareAccel;
  bitrateControl.value = config.bitrateControl;
  bitrate.value = config.bitrate;

  aggressive.value = config.aggressive;
  challengeColor.value = t('challenge-colors').split(',')[STD_CHALLENGE_COLORS.indexOf(config.challengeColor)];
  challengeRank.value = String(config.challengeRank);
  disableEffect.value = config.disableEffect;
  doubleHint.value = config.doubleHint;
  fxaa.value = config.fxaa;
  noteScale.value = config.noteScale;
  disableParticle.value = !config.particle;
  playerAvatar.value = config.playerAvatar || undefined;
  playerName.value = config.playerName;
  playerRks.value = String(config.playerRks);
  sampleCount.value = String(config.sampleCount);
  respack.value = respacks.value.find((x) => x.path === config.resPackPath) || respacks.value[0];
  volumeMusic.value = config.volumeMusic;
  volumeSfx.value = config.volumeSfx;
}

const DEFAULT_CONFIG: RenderConfig = {
  resolution: [1920, 1080],
  ffmpegPreset: 'fast',
  endingLength: -2.0,
  disableLoading: true,
  chartDebug: false,
  chartRatio: 1,
  fps: 60,
  hardwareAccel: true,
  bitrateControl: 'CRF',
  bitrate: '26',

  aggressive: false,
  challengeColor: 'golden',
  challengeRank: 45,
  disableEffect: false,
  doubleHint: true,
  fxaa: false,
  noteScale: 1,
  particle: true,
  playerAvatar: null,
  playerName: '',
  playerRks: 15,
  sampleCount: 1,
  resPackPath: null,
  speed: 1,
  volumeMusic: 1,
  volumeSfx: 1,
};
interface Preset {
  name: string;
  key: string;
  config: RenderConfig;
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
  if (name === 'default') {
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
  <v-form ref="form" style="max-height: 48vh; overflow-x: hidden; overflow-y: scroll">
    <v-row no-gutters class="mx-n2 align-center">
      <v-col cols="8">
        <v-combobox @update:model-value="(val) => applyConfig(val.config)" class="mx-2" :label="t('presets')" :items="presets" item-title="name" v-model="preset"></v-combobox>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2" v-t="'preset-refresh'" @click="updatePresets"></v-btn>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2" v-t="'preset-create'" @click="createPreset"></v-btn>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2" v-t="'preset-delete'" :disabled="preset.key === 'default'" @click="deletePreset"></v-btn>
      </v-col>
      <v-col cols="1" class="mt-n4">
        <v-btn class="px-2" v-t="'preset-replace'" :disabled="preset.key === 'default'" @click="replacePreset"></v-btn>
      </v-col>
    </v-row>

    <div>
      <StickyLabel :title="t('title.output')"></StickyLabel>
      <v-row no-gutters class="mx-n2">
        <v-col cols="3">
          <v-combobox :label="t('resolution')" :items="RESOLUTIONS" class="mx-2" :rules="[resolutionRule]" v-model="resolution"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-combobox :label="t('ffmpeg-preset')" :items="ffmpegPresetPresetList" class="mx-2" :rules="[RULES.non_empty]" v-model="ffmpegPreset"></v-combobox>
        </v-col>
        <v-col cols="3">
          <v-text-field :label="t('fps')" class="mx-2" type="number" :rules="[RULES.positiveInt]" v-model="fps"></v-text-field>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('hw-accel')" :tooltip="t('hw-accel-tips')" v-model="hwAccel"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-1">
        <v-col cols="3">
          <TipTextField :label="t('sample-count')" class="mx-2" type="number" :rules="[sampleCountRule]" v-model="sampleCount" :tooltip="t('sample-count-tips')"></TipTextField>
        </v-col>
        <v-col cols="3">
          <TipTextField :label="t('bitrate')" class="mx-2" :rules="[RULES.non_empty]" v-model="bitrate" :tooltip="t('bitrate-tips')"></TipTextField>
        </v-col>
        <v-col cols="3">
          <v-combobox :label="t('bitrate-control')" :items="bitrateControlList" class="mx-2" :rules="[RULES.non_empty]" v-model="bitrateControl"></v-combobox>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('fxaa')" :tooltip="t('fxaa-tips')" v-model="fxaa"></TipSwitch>
        </v-col>
      </v-row>
    </div>
    <div class="mt-2 px-6">
      <StickyLabel :title="t('title.player')"></StickyLabel>
      <v-row no-gutters class="mx-n2">
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
      <v-row no-gutters class="mx-n2">
        <v-col cols="4">
          <v-text-field class="mx-2" :label="t('player-rks')" :rules="[RULES.positive]" type="number" v-model="playerRks"></v-text-field>
        </v-col>
        <v-col cols="4">
          <v-combobox class="mx-2" :label="t('challenge-color')" :items="t('challenge-colors').split(',')" v-model="challengeColor" :rules="[RULES.non_empty]"></v-combobox>
        </v-col>
        <v-col cols="4">
          <v-text-field class="mx-2" :label="t('challenge-rank')" :rules="[RULES.positiveInt]" type="number" v-model="challengeRank"></v-text-field>
        </v-col>
      </v-row>
    </div>

    <div class="mt-2 px-6">
      <StickyLabel :title="t('title.graphics')"></StickyLabel>
      <v-row no-gutters class="mx-n2 align-center">
        <v-col cols="8">
          <v-combobox class="mx-2" :label="t('respack')" :items="respacks" item-title="name" v-model="respack"></v-combobox>
        </v-col>
        <v-col cols="2" class="mt-n5 d-flex justify-center">
          <v-btn class="pa-1" size="large" @click="updateRespacks" v-t="'respack-refresh'"></v-btn>
        </v-col>
        <v-col cols="2" class="mt-n5 d-flex justify-center">
          <v-btn class="pa-1" size="large" @click="openRespackFolder" v-t="'respack-open'"></v-btn>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-8 align-center">
        <v-col cols="12" class="px-6">
          <v-slider :label="t('note-scale')" thumb-label="always" :min="0" :max="5" :step="0.05" v-model="noteScale"> </v-slider>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-1 px-6">
        <v-col cols="3">
          <TipSwitch :label="t('double-hint')" v-model="doubleHint"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('aggressive')" :tooltip="t('aggressive-tips')" v-model="aggressive"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('disable-particle')" v-model="disableParticle"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('disable-effect')" v-model="disableEffect"></TipSwitch>
        </v-col>
      </v-row>
    </div>

    <div class="mt-2">
      <StickyLabel :title="t('title.audio')"></StickyLabel>
      <v-row no-gutters class="mx-n2 mt-8 align-center px-6">
        <v-col cols="6">
          <v-slider :label="t('volume-music')" thumb-label="always" :min="0" :max="2" :step="0.05" v-model="volumeMusic"> </v-slider>
        </v-col>
        <v-col cols="6">
          <v-slider :label="t('volume-sfx')" thumb-label="always" :min="0" :max="2" :step="0.05" v-model="volumeSfx"> </v-slider>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-8 align-center px-6">
        <v-col cols="12">
          <v-text-field :label="t('ending-length')" v-model="endingLength" type="number" :rules="[RULES.non_empty]"></v-text-field>
        </v-col>
      <!--</v-row>
      <v-row no-gutters class="mx-n2 mt-8 align-center px-6">-->
        <v-col cols="3">
          <TipSwitch :label="t('disable-loading')" v-model="disableLoading"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('chart_debug')" v-model="chartDebug"></TipSwitch>
        </v-col>
        <v-col cols="6">
          <v-slider :label="t('chart_ratio')" thumb-label="always" :min="0.05" :max="1" :step="0.05" v-model="chartRatio"> </v-slider>
        </v-col>
      </v-row>
    </div>
  </v-form>
</template>
