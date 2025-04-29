<i18n>
en:
  save: Save
  reset: Reset
  rpe-dir: RPE Directory

zh-CN:
  save: 保存
  reset: 重置
  rpe-dir: RPE 目录

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();
const { t } = useI18n();

import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Config } from './model';

const DEFAULT_CONFIG: Config = {
  rpeDir: null,
}

const config = ref(DEFAULT_CONFIG);

async function updateConfig() {
  config.value = await invoke('read_config') as Config;
}
updateConfig();

async function saveConfig() {
  await invoke('save_config', { config: config.value });
  updateConfig();
}

async function resetConfig() {
  config.value = DEFAULT_CONFIG;
}

</script>

<template>
  <div class="pa-8 w-100 h-90 d-flex flex-column align-center about-container" style="max-width: 1280px; gap: 1rem">
    <v-form ref="form" style="max-height: 48vh; overflow-x: hidden; overflow-y: auto; width: 100%;">
      <div no-gutters class="mt-0 d-flex flex-row pt-0">
        <v-btn @click="resetConfig" v-t="'reset'" size="large"></v-btn>
        <div class="flex-grow-1"></div>
        <v-btn @click="saveConfig" v-t="'save'" size="large"></v-btn>

      </div>

      <v-row no-gutters class="mt-2">
        <v-col cols="3">
          <div>
            <v-text-field :label="t('rpe-dir')" v-model="config.rpeDir"></v-text-field>
          </div>
        </v-col>
      </v-row>
    </v-form>
  </div>

</template>

<style scoped>
.about-container {
  margin: 2rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>