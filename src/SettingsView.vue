<i18n>
en:
  save: Save
  save-success: Save successfully
  reset: Reset
  select: Selected
  no-select: No selection
  rpe-dir: RPE Directory
  output-dir: Output Directory

zh-CN:
  save: 保存
  save-success: 保存成功
  reset: 重置
  select: 已选择
  no-select: 没有选择
  rpe-dir: RPE 目录
  output-dir: 输出目录

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();
const { t } = useI18n();

import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Config } from './model';
import { open } from '@tauri-apps/plugin-dialog';
import { toast, toastError, RULES } from './common';
import type { VForm } from 'vuetify/components';

const form = ref<VForm>();
const loadingSave = ref(false);

const DEFAULT_CONFIG: Config = {
  rpeDir: null,
  outputDir: null,
}

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
    if (config.value[key as keyof Config]?.trim() === "") {
      config.value[key as keyof Config] = null;
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

async function resetConfig() {
  config.value = DEFAULT_CONFIG;
}

async function selectDir() {
  let file = await open({ directory: true, title: t('output-dir') });
  if (!file) {
    toast(t('no-select'), 'error');
    return null;
  } else {
    //toast(t('select'), 'success');
    return file;
  }
}

async function selectRpeDir() {
  let file = await selectDir();
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
  config.value.outputDir = await selectDir()
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

</script>

<template>
  <div class="pa-8 w-100 h-90 d-flex flex-column align-center container fade-in" style="max-width: 1280px; gap: 1rem">
    <v-form ref="form" style="max-height: 48vh; overflow-x: hidden; overflow-y: auto; width: 100%;">
      <div no-gutters class="mt-2 mx-2 d-flex flex-row">
        <v-btn @click="resetConfig" v-t="'reset'" size="large"></v-btn>
        <div class="flex-grow-1"></div>
        <v-btn @click="saveConfig" :loading="loadingSave" v-t="'save'" size="large"></v-btn>

      </div>

      <v-row no-gutters class="mt-3 mx-0">
        <v-col cols="6">
          <div>
            <v-text-field clearable class="mx-2" :label="t('rpe-dir')" :rules="[RULES.isPath]" v-model="config.rpeDir" append-inner-icon="mdi-folder-open" @click:append-inner="selectRpeDir" @contextmenu="openInFolder(config.rpeDir)"></v-text-field>
          </div>
        </v-col>
        <v-col cols="6">
          <div>
            <v-text-field clearable class="mx-2" :label="t('output-dir')" :rules="[RULES.isPath]" v-model="config.outputDir" placeholder="/output/" append-inner-icon="mdi-folder-open" @click:append-inner="selectOutputDir" @contextmenu="openInFolder(config.outputDir, true)"></v-text-field>
          </div>
        </v-col>
      </v-row>
    </v-form>
  </div>

</template>

<style scoped>
</style>