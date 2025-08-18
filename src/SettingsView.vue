<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();
const { t } = useI18n();

import { computed, nextTick, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { DEFAULT_CONFIG, type Config } from './model';
import { open } from '@tauri-apps/plugin-dialog';
import { toast, toastError, changeLocale } from './common';
import type { VForm } from 'vuetify/components';
import { useTheme } from 'vuetify';
import { SUPPORTED_LOCALES_NAME } from './main';
const theme = useTheme();

import TipSwitch from './components/TipSwitch.vue';
import { useStorage } from '@vueuse/core';

const form = ref<VForm>();
const loadingSave = ref(false);

const config = ref(DEFAULT_CONFIG);

async function updateConfig() {
  config.value = await invoke('read_config') as Config;
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
    if (typeof config.value[key as keyof Config] === 'string') {
      const val = config.value[key as keyof Config] as string;
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
  config.value = DEFAULT_CONFIG;
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
  let file = await open({
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

async function selectRpeDir() {
  let file = await selectDir(t('rpe-dir'));
  if (file) {
    try {
      await invoke('set_rpe_dir', { path: file, save: false });
      config.value.rpeDir = file;
    } catch (e) {
      toastError(e);
    }
  }
}

async function selectOutputDir() {
  config.value.outputDir = await selectDir(t('output-dir'))
}

async function openInFolder(path: string | null, isOutput: boolean = false) {
  if (!path && !isOutput) {
    toast(t('no-select'), 'error');
    return null;
  } else if (!path && isOutput) {
    await invoke('open_output_folder', { path });
    return;
  }
  try {
    await invoke('test_output_dir', { dir: path });
    await invoke('open_in_folder', { path });
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


</script>

<template>
  <div class="pa-8 w-100 h-90 d-flex flex-column align-center container fade-in" style="max-width: 1280px; gap: 1rem" :style="{ background: `${theme.current.value.colors.container}` }">
    <v-form ref="form" validateOn="eager" style="max-height: 60vh; overflow-x: hidden; overflow-y: auto; width: 100%;">
      <v-row>
        <h2 class="mt-1 mx-5">{{ t('setting') }}</h2>
      </v-row>
      <v-row no-gutters class="mt-5 mx-0">
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('rpe-dir')" v-model="config.rpeDir" append-inner-icon="mdi-folder-open" @click:append-inner="selectRpeDir" @contextmenu="openInFolder(config.rpeDir)"></v-text-field>
        </v-col>
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('output-dir')" v-model="config.outputDir" placeholder="/output/" append-inner-icon="mdi-folder-open" @click:append-inner="selectOutputDir" @contextmenu="openInFolder(config.outputDir, true)"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mt-3 mx-0">
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('encoder-avc')" v-model="config.encoderAvc" :append-inner-icon="getEncoderIcon" @click:append-inner="getEncoder(false)" @contextmenu="testEncoderAvc"></v-text-field>
        </v-col>
        <v-col cols="6">
          <v-text-field clearable class="mx-2" :label="t('encoder-hevc')" v-model="config.encoderHevc" :append-inner-icon="getEncoderIcon" @click:append-inner="getEncoder(true)" @contextmenu="testEncoderHevc"></v-text-field>
        </v-col>
      </v-row>
      <v-row no-gutters class="mx-0">
        <v-col cols="6">
          <TipSwitch class="mx-4" :tooltip="t('print-stderr-tip')" :label="t('print-stderr')" v-model="config.printStderr"></TipSwitch>
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
          <TipSwitch class="mx-4" :tooltip="t('list-expand-tip')" :label="t('list-expand')" v-model="listExpand"></TipSwitch>
        </v-col>
      </v-row>
      <v-row class="my-2" />
    </v-form>
    <div no-gutters class="mt-auto mx-2 d-flex flex-row" style="width: 100%">
      <v-btn @click="resetDialog = true" v-t="'reset'" color="btn-large" size="large"></v-btn>
      <div class="flex-grow-1"></div>
      <v-btn @click="saveConfig" :loading="loadingSave" v-t="'save'" color="btn-large" size="large"></v-btn>
    </div>
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