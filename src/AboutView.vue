<i18n>
en:
  app: Phigros Recorder

zh-CN:
  app: Phigros Recorder

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();

import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/api/shell';

const appVersion = await getVersion();
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center" style="max-width: 1280px; gap: 1rem">
    <h1 v-t="'app'"></h1>
    <h4 class="mt-n2">v{{ appVersion }}</h4>
    <v-btn prepend-icon="mdi-github" @click="open('https://github.com/2278535805/Phigros-Recorder')">GitHub</v-btn>
    <p>Licensed by GPLv3</p>
  </div>
</template>
