<i18n>
en:
  app: Phi Recorder

zh-CN:
  app: Phi Recorder

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
    <h1 class="app-title gradient-text" v-t="'app'"></h1>
    <h4 class="mt-n2 version-label text-glow">v{{ appVersion }}</h4>
    <v-btn class="github-btn hover-scale" prepend-icon="mdi-github" @click="open('https://github.com/2278535805/Phi-Recorder')">GitHub</v-btn>
    <p class="license-text text-gradient">Licensed by GPLv3</p>
  </div>
</template>

<style scoped>
.about-container {
  padding: 2rem;
  width: 100%;
  height: 100%;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.app-title {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.05em;
}

.version-label {
  font-size: 1.25rem;
  font-weight: 500;
  opacity: 0.8;
}

.github-btn {
  background: rgba(147, 147, 147, 0.2);
  padding: 12px 24px;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 4px 4px 6px rgba(0, 0, 0, 0.1);
}

.license-text {
  font-size: 0.9rem;
  opacity: 0.7;
}

.gradient-text {
  background: linear-gradient(45deg, #2196f3, #e91e63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.text-glow {
  text-shadow: 0 0 12px rgba(33, 150, 243, 0.4);
}

.hover-scale {
  transition: transform 0.3s ease;
}

.hover-scale:hover {
  transform: scale(1.05);
}

.text-gradient {
  background: linear-gradient(45deg, #4caf50, #ffeb3b);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
</style>