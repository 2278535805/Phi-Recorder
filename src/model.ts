export interface ChartInfo {
  name: string;
  level: string;
  charter: string;
  composer: string;
  illustrator: string;

  tip: string | null;
  offset: number;

  aspectRatio: number;
  backgroundDim: number;
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
    }
  | {
      type: 'failed';
      error: string;
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
  chartDebug: boolean;
  chartRatio: number;
  allGood: boolean;
  allBad: boolean;
  fps: number;
  hardwareAccel: boolean;
  hevc: boolean;
  mpeg4: boolean;
  customEncoder: string | null;
  bitrateControl: string;
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
  fade: number;
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
};