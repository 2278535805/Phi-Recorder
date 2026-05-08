export interface ChartInfo {
  name: string;
  difficulty: number;
  level: string;
  charter: string;
  composer: string;
  illustrator: string;

  chart: string;
  music: string;
  illustration: string;

  previewStart: number;
  previewEnd: number | null;
  aspectRatio: number;
  forceAspectRatio: boolean;
  backgroundDim: number;
  lineLength: number;
  offset: number;
  tip: string | null;
  tags: string[];

  intro: string,

  holdPartialCover: boolean;
  noteUniformScale: boolean;
  scoreTotal: number;
  holdParticleIntervalRatio: number;
  foldAnimation: boolean;
}

export type TaskStatus =
  | {
      type: 'null';
    }
  | {
      type: 'pending';
    }
  | {
      type: 'loading';
    }
  | {
      type: 'mixing';
    }
  | {
      type: 'mixing_sfx';
      progress: number;
    }
  | {
      type: 'rendering';
      progress: number;
      fps: number;
      estimate: number;
    }
  | {
      type: 'done';
      duration: number;
      output: string;
    }
  | {
      type: 'canceled';
      output: string;
    }
  | {
      type: 'failed';
      output: string;
    };

export interface Task {
  id: number;
  name: string;
  output: string;
  path: string;
  info: ChartInfo;
  config: RenderConfig;
  cover: string;
  status: TaskStatus;
}

export interface RenderConfig {
  resolution: number[];
  endingLength: number;
  renderLoading: boolean;
  hires: boolean;
  chartDebugLine: number;
  chartDebugNote: number;
  chartRatio: number;
  allGood: boolean;
  allBad: boolean;
  fps: number;
  hardwareAccel: boolean;
  hevc: boolean;
  mpeg4: boolean;
  customEncoder: string | null;
  dynamicBitrateControl: boolean;
  bitrate: string;

  aggressiveChart: boolean;
  aggressiveNote: boolean;
  aggressiveParticle: boolean;
  challengeColor: string;
  challengeRank: number;
  noteScale: number;
  particle: boolean;
  playerAvatar: string | null;
  playerName: string;
  playerRks: number;
  sampleCount: number;
  fxaa: boolean;
  resPackPath: string | null;
  speed: number;
  volumeMusic: number;
  volumeSfx: number;
  compressionRatio: number;
  watermark: string;
  roman: boolean;
  chinese: boolean;
  combo: string;
  difficulty: string;
  judgeOffset: number;
  forceLimit: boolean;
  limitThreshold: number;
  loudnessEqualization: boolean;
  audioMixOptimization: boolean;
  fileNameFormat: string;

  renderLine: boolean;
  renderLineExtra: boolean;
  renderNote: boolean;
  renderDoubleHint: boolean;
  renderUiPause: boolean;
  renderUiName: boolean;
  renderUiLevel: boolean;
  renderUiScore: boolean;
  renderUiCombo: boolean;
  renderUiBar: boolean;
  renderBg: boolean;
  renderBgDim: boolean;
  preserveFramebuffer: boolean;
  renderExtra: boolean;
  bgBlurriness: number;

  maxParticles: number;
  playStartTime: number;
  playEndTime: number | null;

  fade: number;
  alphaTint: boolean;
}

export const DEFAULT_RENDER_CONFIG: RenderConfig = {
  resolution: [1920, 1080],
  endingLength: 0.0,
  renderLoading: false,
  hires: false,
  chartDebugLine: 0.,
  chartDebugNote: 0.,
  chartRatio: 1,
  allGood: false,
  allBad: false,
  fps: 60,
  hardwareAccel: true,
  hevc: false,
  mpeg4: false,
  customEncoder: null,
  dynamicBitrateControl: true,
  bitrate: '28',

  aggressiveChart: false,
  aggressiveNote: false,
  aggressiveParticle: false,
  challengeColor: 'rainbow',
  challengeRank: 3,
  noteScale: 1,
  particle: true,
  playerAvatar: null,
  playerName: '',
  playerRks: 16.00,
  sampleCount: 8,
  fxaa: false,
  resPackPath: null,
  speed: 1,
  volumeMusic: 0.5,
  volumeSfx: 0.4,
  compressionRatio: 20.0,
  forceLimit: true,
  limitThreshold: 0.5,
  loudnessEqualization: false,
  audioMixOptimization: true,
  watermark: '',
  roman: false,
  chinese: false,
  combo: 'AUTOPLAY',
  difficulty: '',
  judgeOffset: 0,
  fileNameFormat: '%date% %time% %info.name%_%level_prefix%',
  renderLine: true,
  renderLineExtra: true,
  renderNote: true,
  renderDoubleHint: true,
  renderUiPause: true,
  renderUiName: true,
  renderUiLevel: true,
  renderUiScore: true,
  renderUiCombo: true,
  renderUiBar: true,
  renderBg: true,
  renderBgDim: true,
  preserveFramebuffer: false,
  renderExtra: true,
  bgBlurriness: 80,

  maxParticles: 5000,
  playStartTime: 0.0,
  playEndTime: null,

  fade: 0.0,
  alphaTint: false,
};

export interface AppConfig {
  rpeDir: string | null;
  outputDir: string | null;
  encoderAvc: string | null;
  encoderHevc: string | null;
  ffmpegPath: string | null;
  printStderr: boolean;
  showDetailedLog: boolean;
  fullscreenMode: boolean;
}

export const DEFAULT_APP_CONFIG: AppConfig = {
  rpeDir: null,
  outputDir: null,
  encoderAvc: null,
  encoderHevc: null,
  ffmpegPath: null,
  printStderr: false,
  showDetailedLog: true,
  fullscreenMode: false,
}

export interface RPEChart {
  name: string;
  id: string;
  path: string;
  illustration: string;
  charter: string;
}

export type Assets = {
  browser_download_url: string,
  name: string,
}

export type Release = {
  id: number,
  assets: Assets[],
  tag_name: string,
  body: string,
  message: string, // error message
};

export interface RenderChart {
  id: number;
  path: string;
  isSelect: boolean;
  chartInfo: ChartInfo;

  taskId: number | null;
  output: string;
  status: TaskStatus;
}

export interface Preset {
  name: string;
  key: string;
  config: RenderConfig;
}

export interface FileDropEvent {
  paths: string[];
  position: { x: number; y: number };
}
