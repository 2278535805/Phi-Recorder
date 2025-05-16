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
  backgroundDim: number;
  lineLength: number;
  offset: number;
  tip: string | null;
  tags: string[];

  intro: string,

  holdPartialCover: boolean;
}

export type TaskStatus =
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
  cover: string;
  status: TaskStatus;
}

export interface RenderConfig {
  resolution: number[];
  ffmpegPreset: string;
  endingLength: number;
  disableLoading: boolean;
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

  aggressive: boolean;
  challengeColor: string;
  challengeRank: number;
  disableEffect: boolean;
  doubleHint: boolean;
  fxaa: boolean;
  noteScale: number;
  //offset: number;
  particle: boolean;
  playerAvatar: string | null;
  playerName: string;
  playerRks: number;
  sampleCount: number;
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
  simpleFileName: boolean;

  renderLine: boolean;
  renderLineExtra: boolean;
  renderNote: boolean;
  renderUiPause: boolean;
  renderUiName: boolean;
  renderUiLevel: boolean;
  renderUiScore: boolean;
  renderUiCombo: boolean;
  renderUiBar: boolean;
  renderBg: boolean;
  renderBgDim: boolean;
  bgBlurriness: number;

  maxParticles: number;
  renderStartTime: number;
  renderEndTime: number | null;

  fade: number;
  alphaTint: boolean;
}

export interface Config {
  rpeDir: string | null;
  outputDir: string | null;
  encoderAvc: string | null;
  encoderHevc: string | null;
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