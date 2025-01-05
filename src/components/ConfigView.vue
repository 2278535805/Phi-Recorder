<i18n>
en:
  title:
    output: Output
    player: Player
    graphics: Graphics
    audio: Audio
    debug: Debug
    other: Other

  resolution: Resolution
  ffmpeg-preset: Preset
  fps: FPS

  hw-accel: Hardware Acceleration
  hw-accel-tips: Improve rendering speed, slightly reduce quality

  fxaa: FXAA
  fxaa-tips: FXAA, as a low-cost anti-aliasing method, will cause the picture to be blurred, and it is not recommended to turn it on

  hevc: HEVC encoding
  hevc-tips: Use HEVC encoding, which has higher compression rate and slower speed

  sample-count: Sample Count
  sample-count-tips: Must be a power of 2. A non-1 sample count enables MSAA, which can improve the quality of the picture while increasing the performance cost

  bitrate-control: Bitrate Control
  bitrate: Quantization parameters/Bitrate
  bitrate-tips: |
    CRF-CRF level.
    CBR-bitrate.

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
  aggressive-tips: Hide off-screen note, Improve rendering speed, but may cause some notes to disappear

  disable-particle: Disable Particle
  disable-effect: Disable Effect

  volume-music: Music Volume
  volume-sfx: SFX Volume
  compression-ratio: SFX Compression Ratio

  ending-length: Result Screen Duration
  disable-loading: Remove loading screen
  hires: Lossless Audio
  chart_debug: Debug Mode
  chart_ratio: Chart Zoom
  all_good: Force Good judgment
  watermark: Watermark
  roman: Roman Mode
  chinese: Chinese Mode
  combo: COMBO text
  difficulty: Custom Difficulty
  offset: Offset
  phiraMode: Phira Features
  phiraMode-tips: Hold cover using head position

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
    debug: 调试
    other: 其他

  resolution: 分辨率
  ffmpeg-preset: 预设
  fps: FPS

  hw-accel: 硬件加速
  hw-accel-tips: 提升渲染速度，略微降低质量

  fxaa: FXAA
  fxaa-tips: FXAA 以低成本实现抗锯齿，但会导致画面模糊，不建议开启

  hevc: HEVC编码
  hevc-tips: 使用 HEVC 编码，压缩率更高，渲染速度更慢

  sample-count: 采样数
  sample-count-tips: 非 1 的采样数(必须为 2 的幂)会启用 MSAA(若开头无画面请关闭此项)

  bitrate-control: 码率控制
  bitrate: 量化参数/码率
  bitrate-tips: |
    CRF-CRF级别
    CBR-码率

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
  aggressive-tips: 剔除屏幕外按键，提升渲染速度，但可能会导致部分音符消失

  disable-particle: 禁用粒子
  disable-effect: 禁用特效

  volume-music: 音乐音量
  volume-sfx: 音效音量
  compression-ratio: 音效压缩比

  ending-length: 结算画面时长
  disable-loading: 禁用加载
  hires: 无损音频
  chart_debug: 调试模式
  chart_ratio: 谱面缩放
  all_good: 强制Good
  watermark: 水印
  roman: 罗马模式
  chinese: 中文模式
  combo: COMBO文字
  difficulty: 自定义难度
  offset: 延时
  phiraMode: Phira模式
  phiraMode-tips: Hold 遮罩使用头部位置

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

const RESOLUTIONS = [ '1920x1080', '1280x720', '2560x1440', '3840x2160', '2844x1600', '2388x1668', '1600x1080'];
const ffmpegPresetPresetList = ['veryfast p1 speed', 'faster p2 speed','fast p3 balanced', 'medium p4 balanced', 'slow p5 balanced', 'slower p6 quality', 'veryslow p7 quality'];
const bitrateControlList = ['CRF','CBR'];
const fpsList = ['60', '120', '30'];

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
  ffmpegPreset = ref('medium p4 balanced'),
  fps = ref('60'),
  hwAccel = ref(true),
  hevc = ref(false);

const fxaa = ref(false),
  sampleCount = ref('2'),
  bitrateControl = ref('CRF'),
  bitrate = ref('28');

const playerAvatar = ref<string>(),
  playerName = ref(''),
  playerRks = ref('16.0');

const watermark = ref('');

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

const challengeColor = ref(t('challenge-colors').split(',')[5]),
  challengeRank = ref('3');

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
  aggressive = ref(true),
  disableParticle = ref(false),
  disableEffect = ref(false);

const volumeMusic = ref(1.0),
  volumeSfx = ref(0.7),
  compressionRatio = ref(100.0);

const endingLength = ref('0.0');

const disableLoading = ref(true)
const hires = ref(true)

const chartDebug = ref(false)
const chartRatio = ref(1.0)
const allGood = ref(false)
const roman = ref(false)
const chinese = ref(false)
const combo = ref('AUTOPLAY')
const difficulty = ref('')
const phiraMode = ref(false)
//const offset = ref('0.0')

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
    hires: hires.value,
    chartDebug: chartDebug.value,
    chartRatio: chartRatio.value,
    fps: parseInt(fps.value),
    hardwareAccel: hwAccel.value,
    hevc: hevc.value,
    bitrateControl: bitrateControl.value,
    bitrate: bitrate.value,

    aggressive: aggressive.value,
    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value)],
    challengeRank: parseInt(challengeRank.value),
    disableEffect: disableEffect.value,
    doubleHint: doubleHint.value,
    fxaa: false, //Disable FXAA
    noteScale: noteScale.value,
    particle: !disableParticle.value,
    //offset: parseFloat(offset.value) / 1000,
    playerAvatar: playerAvatar.value ? (playerAvatar.value.length ? playerAvatar.value : null) : null,
    playerName: playerName.value,
    playerRks: parseFloat(playerRks.value),
    sampleCount: parseInt(sampleCount.value),
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
    compressionRatio: compressionRatio.value,
    allGood: allGood.value,
    watermark: watermark.value,
    roman: roman.value,
    chinese: chinese.value,
    combo: combo.value,
    difficulty: difficulty.value,
    phiraMode: phiraMode.value,
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
  hires.value = config.hires;
  chartDebug.value = config.chartDebug;
  chartRatio.value = config.chartRatio;
  allGood.value = config.allGood;
  fps.value = String(config.fps);
  hwAccel.value = config.hardwareAccel;
  hevc.value = config.hevc;
  bitrateControl.value = config.bitrateControl;
  bitrate.value = config.bitrate;

  aggressive.value = config.aggressive;
  challengeColor.value = t('challenge-colors').split(',')[STD_CHALLENGE_COLORS.indexOf(config.challengeColor)];
  challengeRank.value = String(config.challengeRank);
  disableEffect.value = config.disableEffect;
  doubleHint.value = config.doubleHint;
  //fxaa.value = config.fxaa;
  noteScale.value = config.noteScale;
  //offset.value = String(config.offset);
  disableParticle.value = !config.particle;
  playerAvatar.value = config.playerAvatar || undefined;
  playerName.value = config.playerName;
  playerRks.value = String(config.playerRks);
  sampleCount.value = String(config.sampleCount);
  respack.value = respacks.value.find((x) => x.path === config.resPackPath) || respacks.value[0];
  volumeMusic.value = config.volumeMusic;
  volumeSfx.value = config.volumeSfx;
  compressionRatio.value = config.compressionRatio;
  watermark.value = config.watermark;
  roman.value = config.roman;
  chinese.value = config.chinese;
  combo.value = config.combo;
  difficulty.value = config.difficulty;
  phiraMode.value = config.phiraMode;
}

const DEFAULT_CONFIG: RenderConfig = {
  resolution: [1920, 1080],
  ffmpegPreset: 'medium p4 balanced',
  endingLength: 0.0,
  disableLoading: true,
  hires: true,
  chartDebug: false,
  chartRatio: 1,
  allGood: false,
  fps: 60,
  hardwareAccel: true,
  hevc: false,
  bitrateControl: 'CRF',
  bitrate: '28',

  aggressive: true,
  challengeColor: 'rainbow',
  challengeRank: 3,
  disableEffect: false,
  doubleHint: true,
  fxaa: false,
  noteScale: 1,
  //offset: 0.0,
  particle: true,
  playerAvatar: null,
  playerName: '',
  playerRks: 16.00,
  sampleCount: 2,
  resPackPath: null,
  speed: 1,
  volumeMusic: 1.0,
  volumeSfx: 0.7,
  compressionRatio: 100.0,
  watermark: '',
  roman: false,
  chinese: false,
  combo: 'AUTOPLAY',
  difficulty: '',
  phiraMode: false,
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
        <v-combobox @update:model-value="(val: Preset) => applyConfig(val.config)" class="mx-2" :label="t('presets')" :items="presets" item-title="name" v-model="preset"></v-combobox>
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
          <v-combobox :label="t('fps')" :items="fpsList" class="mx-2" type="number" :rules="[RULES.positiveInt]" v-model="fps"></v-combobox>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('hw-accel')" v-model="hwAccel"></TipSwitch> <!-- :tooltip="t('hw-accel-tips')" -->
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
          <v-autocomplete :label="t('bitrate-control')" :items="bitrateControlList" class="mx-2" :rules="[RULES.non_empty]" v-model="bitrateControl"></v-autocomplete>
        </v-col>
        <!--<v-col cols="3">
          <TipSwitch :label="t('fxaa')" :tooltip="t('fxaa-tips')" v-model="fxaa"></TipSwitch>
        </v-col>-->
        <v-col cols="3">
          <TipSwitch :label="t('hevc')" v-model="hevc"></TipSwitch>
        </v-col>
      </v-row>
    </div>
    <div class="mt-2">
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
      <v-row no-gutters class="mx-n2 mt-1">
        <v-col cols="4">
          <v-text-field class="mx-2" :label="t('player-rks')" :rules="[RULES.positive]" type="number" v-model="playerRks"></v-text-field>
        </v-col>
        <v-col cols="4">
          <v-autocomplete class="mx-2" :label="t('challenge-color')" :items="t('challenge-colors').split(',')" v-model="challengeColor" :rules="[RULES.non_empty]"></v-autocomplete>
        </v-col>
        <v-col cols="4">
          <v-text-field class="mx-2" :label="t('challenge-rank')" :rules="[RULES.positiveInt]" type="number" v-model="challengeRank"></v-text-field>
        </v-col>
      </v-row>
    </div>

    <div class="mt-2">
      <StickyLabel :title="t('title.graphics')"></StickyLabel>
      <v-row no-gutters class="mx-n2 mt-4 align-center">
        <v-col cols="8">
          <v-autocomplete class="mx-2" :label="t('respack')" :items="respacks" item-title="name" v-model="respack"></v-autocomplete>
        </v-col>
        <v-col cols="2" class="mt-n5 d-flex justify-center">
          <v-btn class="pa-1" size="large" @click="updateRespacks" v-t="'respack-refresh'"></v-btn>
        </v-col>
        <v-col cols="2" class="mt-n5 d-flex justify-center">
          <v-btn class="pa-1" size="large" @click="openRespackFolder" v-t="'respack-open'"></v-btn>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-4 align-center">
        <v-col cols="6" class="px-6">
          <v-slider :label="t('note-scale')" thumb-label="always" :min="0" :max="5" :step="0.05" v-model="noteScale"> </v-slider>
        </v-col>
        <v-col cols="6" class="px-6">
          <v-slider :label="t('chart_ratio')" thumb-label="always" :min="0.05" :max="1" :step="0.05" v-model="chartRatio"> </v-slider>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-2">
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
        <v-col cols="4">
          <v-slider :label="t('volume-music')" thumb-label="always" :min="0" :max="2" :step="0.05" v-model="volumeMusic"> </v-slider>
        </v-col>
        <v-col cols="4">
          <v-slider :label="t('volume-sfx')" thumb-label="always" :min="0" :max="2" :step="0.05" v-model="volumeSfx"> </v-slider>
        </v-col>
        <v-col cols="4">
          <v-slider :label="t('compression-ratio')" thumb-label="always" :min="1" :max="100" :step="0.1" v-model="compressionRatio"> </v-slider>
        </v-col>
      </v-row>
    </div>

    <div class="mt-2">
      <StickyLabel :title="t('title.other')"></StickyLabel>
      <v-row no-gutters class="align-center">
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('ending-length')" v-model="endingLength" :rules="[RULES.non_empty]"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('watermark')" v-model="watermark"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('combo')" v-model="combo"></v-text-field>
        </v-col>
        <v-col cols="3">
          <v-text-field class="mx-2" :label="t('difficulty')" v-model="difficulty"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="3">
          <TipSwitch :label="t('disable-loading')" v-model="disableLoading"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('chart_debug')" v-model="chartDebug"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('hires')" v-model="hires"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('all_good')" v-model="allGood"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-2">
        <v-col cols="3">
          <TipSwitch :label="t('roman')" v-model="roman"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('chinese')" v-model="chinese"></TipSwitch>
        </v-col>
        <v-col cols="3">
          <TipSwitch :label="t('phiraMode')" :tooltip="t('phiraMode-tips')" v-model="phiraMode"></TipSwitch>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-n2 mt-2">
        
      </v-row>
    </div>
  </v-form>
</template>
