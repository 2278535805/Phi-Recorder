<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();
const { t } = useI18n();

import { computed, nextTick, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { DEFAULT_APP_CONFIG, type AppConfig } from './model';
import * as dialog from '@tauri-apps/plugin-dialog';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
import { toast, toastError, changeLocale, anyFilter } from './common';
import type { VForm } from 'vuetify/components';
import { useTheme } from 'vuetify';
import { SUPPORTED_LOCALES_NAME } from './main';
const theme = useTheme();

import TipSwitch from './components/TipSwitch.vue';
import { useStorage } from '@vueuse/core';

const form = ref<VForm>();
const loadingSave = ref(false);

const config = ref(DEFAULT_APP_CONFIG);

async function updateConfig() {
  config.value = await invoke('read_config') as AppConfig;
}
updateConfig();

async function saveConfig() {
  loadingSave.value = true;
  if (!(await form.value!.validate()).valid) {
    toast(t('has-error'), 'error');
    loadingSave.value = false;
    return null;
  }
  if (config.value.outputDir) {
    try {
      await invoke('test_output_dir', { dir: config.value.outputDir });
    } catch (e) {
      toastError(e);
      loadingSave.value = false;
      return null;
    }
  }
  for (const key in config.value) {
    if (typeof config.value[key as keyof AppConfig] === 'string') {
      const val = config.value[key as keyof AppConfig] as string;
      if (val.trim() === "") {
        (config.value as any)[key] = null;
      }
    }
  }

  try {
    await invoke('save_config', { config: config.value });
  } catch (e) {
    toastError(e);
  }

  updateConfig();
  loadingSave.value = false;
  toast(t('save-success'), 'success');
}

const resetDialog = ref(false);
async function resetConfig(all: boolean) {
  config.value = DEFAULT_APP_CONFIG;
  nextTick(() => {
    form.value?.resetValidation();
  });
  if (all) {
    await saveConfig();
    localStorage.clear();
    window.location.reload();
  }
  resetDialog.value = false;
}

async function selectDir(title: string) {
  let file = await dialog.open({
    directory: true,
    title
  });
  if (!file) {
    toast(t('no-select'), 'error');
    return null;
  } else {
    //toast(t('select'), 'success');
    return file;
  }
}

async function selectFile(title: string) {
  let file = await dialog.open({
    filters: [
      {
        name: t('choose.program'),
        extensions: ['exe', ''],
      },
      anyFilter(),
    ],
    multiple: false,
  });
  if (!file) {
    toast(t('no-select'), 'error');
    return null;
  } else {
    //toast(t('select'), 'success');
    return file;
  }
}

async function selectRpeDir() {
  let path = await selectDir(t('rpe-dir'));
  if (path) {
    try {
      let valid = await invoke('check_rpe_dir', { path: path });
      if (valid) {
        config.value.rpeDir = path;
      } else {
        toast(t('not-valid-rpe'), 'error');
      }
    } catch (e) {
      toastError(e);
    }
  }
}

async function selectOutputDir() {
  let path = await selectDir(t('output-dir'));
  if (path) {
    try {
      config.value.outputDir = path;
    } catch (e) {
      toastError(e);
    }
  }
}

async function selectFFmpegFile() {
  let path = await selectFile(t('ffmpeg-path'));
  if (path) {
    try {
      if (!(await invoke('check_ffmpeg_filter', { ffmpeg: path }))) {
        toast(t('ffmpeg-not-found'), 'error');
        return;
      }
      config.value.ffmpegPath = path;
    } catch (e) {
      toastError(e);
    }
  }
}

async function openInFolder(path: string | null, isOutput: boolean = false) {
  try {
    if (!path && !isOutput) {
      toast(t('no-select'), 'error');
      return null;
    } else if (!path && isOutput) {
      await invoke('open_output_folder', { path });
      return;
    }

    await invoke('test_output_dir', { dir: path });
    await openPath(path!);
  } catch (e) {
    toastError(e);
  }
}

async function showInFolder(path: string | null) {
  try {
    if (!path) {
      toast(t('no-select'), 'error');
      return null;
    }
    await revealItemInDir(path!);
  } catch (e) {
    toastError(e);
  }
}

const getEncoderIcon = ref('mdi-auto-fix');
async function getEncoder(hevc: boolean) {
  getEncoderIcon.value = 'mdi-loading mdi-spin';
  try {
    const encoder = await invoke('get_encoder', { hevc });
    if (encoder && hevc) {
      config.value.encoderHevc = encoder as string;
    } else if (encoder) {
      config.value.encoderAvc = encoder as string;
    } else {
      toast(t('no-encoder'), 'error');
    }
  } catch (e) {
    toastError(e);
  } finally {
    getEncoderIcon.value = 'mdi-auto-fix';
  }
}

async function testEncoderHevc() {
  if (config.value.encoderHevc === '' || config.value.encoderHevc === null) {
    toast(t('no-select'), 'error');
    return null;
  }
  getEncoderIcon.value = 'mdi-loading mdi-spin';
  try {
    if (await invoke('test_encoder', { encoder: config.value.encoderHevc })) {
      toast(t('encoder-ok'), 'success');
    } else {
      toast(t('encoder-error'), 'error');
    }
  } catch (e) {
    toastError(e);
  } finally {
    getEncoderIcon.value = 'mdi-auto-fix';
  }
}

async function testEncoderAvc() {
  if (config.value.encoderAvc === '' || config.value.encoderAvc === null) {
    toast(t('no-select'), 'error');
    return null;
  }
  getEncoderIcon.value = 'mdi-loading mdi-spin';
  try {
    if (await invoke('test_encoder', { encoder: config.value.encoderAvc })) {
      toast(t('encoder-ok'), 'success');
    } else {
      toast(t('encoder-error'), 'error');
    }
  } catch (e) {
    toastError(e);
  } finally {
    getEncoderIcon.value = 'mdi-auto-fix';
  }
}


const locale = useStorage<string>('locale', 'en');
watch(locale, (val) => {
  if (!val) return;
  changeLocale(val);
});

const listExpand = ref(
  localStorage.getItem("listExpand") !== null
    ? JSON.parse(localStorage.getItem("listExpand") as string)
    : true
);

watch(listExpand, (val) => {
  localStorage.setItem("listExpand", JSON.stringify(val))
});

const now_theme = useStorage<string>('theme', 'Light');
watch(now_theme, (val) => {
  if (!val) return;
  theme.global.name.value = val;
});

const SUPPORTED_THEME_NAME = computed(() => [
  { name: t("theme.light"), code: "Light" },
  { name: t("theme.dark"), code: "Dark" },
  { name: t("theme.deep-dark"), code: "DeepDark" },
  { name: t("theme.light-blue"), code: "LightBlue" },
]);

const useSystemTheme = useStorage<boolean>('useSystemTheme', true);


</script>

<template>
  <div class="pa-8 w-100 h-90 d-flex flex-column align-center container fade-in" style="max-width: 1280px" :style="{ background: `${theme.current.value.colors.container}` }">
    <v-form ref="form" validateOn="eager" style="overflow-x: hidden; overflow-y: auto; width: 100%;">
      <v-row no-gutters>
        <v-col class="mx-2 align-self-end">
          <h2 class="mt-1 mx-3">{{ t('setting') }}</h2>
        </v-col>
        <v-col cols="auto" class="mx-2 align-self-end">
          <v-btn @click="resetDialog = true" v-t="'reset'"></v-btn>
        </v-col>
        <v-col cols="auto" class="mx-2 align-self-end">
          <v-btn @click="saveConfig" :loading="loadingSave" v-t="'save'"></v-btn>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-4 mx-0">
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('rpe-dir')" v-model="config.rpeDir" :title="t('folder-tip')" append-inner-icon="mdi-folder-open" @click:append-inner="selectRpeDir" @contextmenu="openInFolder(config.rpeDir)"></v-text-field>
        </v-col>
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('output-dir')" v-model="config.outputDir" :title="t('folder-tip')" placeholder="/output/" append-inner-icon="mdi-folder-open" @click:append-inner="selectOutputDir" @contextmenu="openInFolder(config.outputDir, true)"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-3 mx-0">
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('encoder-avc')" v-model="config.encoderAvc" :title="t('encoder-tip')" :append-inner-icon="getEncoderIcon" @click:append-inner="getEncoder(false)" @contextmenu="testEncoderAvc"></v-text-field>
        </v-col>
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('encoder-hevc')" v-model="config.encoderHevc" :title="t('encoder-tip')" :append-inner-icon="getEncoderIcon" @click:append-inner="getEncoder(true)" @contextmenu="testEncoderHevc"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-4 mx-0">
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('ffmpeg-path')" v-model="config.ffmpegPath" :placeholder="t('auto-detect')" append-inner-icon="mdi-folder-open" @click:append-inner="selectFFmpegFile" @contextmenu="showInFolder(config.ffmpegPath)"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-0">
        <v-col cols="6">
          <TipSwitch class="mx-4" :tooltip="t('print-stderr-tip')" :label="t('print-stderr')" v-model="config.printStderr"></TipSwitch>
        </v-col>
        <v-col cols="6">
          <TipSwitch class="mx-4" :tooltip="t('show-detailed-log-tip')" :label="t('show-detailed-log')" v-model="config.showDetailedLog"></TipSwitch>
        </v-col>
      </v-row>

      <div class="mt-1 mx-2">
        <VDivider />
      </div>

      <v-row no-gutters class="mt-2 mx-0">
        <v-col cols="6">
          <v-autocomplete class="mx-2" :label="t('theme.theme')" :items="SUPPORTED_THEME_NAME" item-title="name" item-value="code" v-model="now_theme"></v-autocomplete>
        </v-col>
        <v-col cols="6">
          <v-autocomplete class="mx-2" :label="t('lang')" :items="SUPPORTED_LOCALES_NAME" item-title="name" item-value="code" v-model="locale"></v-autocomplete>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-2 mx-0">
        <v-col cols="6">
          <TipSwitch class="mx-4" :label="t('use-system-theme')" v-model="useSystemTheme"></TipSwitch>
        </v-col>
        <v-col cols="6">
          <TipSwitch class="mx-4" :tooltip="t('list-expand-tip')" :label="t('list-expand')" v-model="listExpand"></TipSwitch>
        </v-col>
      </v-row>
      <v-row class="my-2" />
    </v-form>
  </div>

  <v-dialog v-model="resetDialog" width="auto" min-width="300px" class="log-card-bg">
    <v-card class="log-card-only-window">
      <v-card-title v-t="'reset-confirm'"> </v-card-title>
      <v-card-text>
        <div>
          <p v-t="'reset-confirm-desc'"></p>
        </div>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn @click="resetConfig(false)" color="btn" v-t="'reset-only'" ></v-btn>
        <v-btn @click="resetConfig(true)" color="error" v-t="'reset-all'" ></v-btn>
      </v-card-actions>

    </v-card>
  </v-dialog>


</template>

<style scoped>
</style>