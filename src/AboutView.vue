<i18n>
en:
  app: Phi Recorder
  check: Check Update
  new-version: New version available!
  non-version: It's the latest version

zh-CN:
  app: Phi Recorder
  check: 检查更新
  new-version: 有新版本可用!
  non-version: 已是最新版本

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();
const { t } = useI18n();

import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/api/shell';

const appVersion = await getVersion();

import { fetch } from '@tauri-apps/api/http';
import semver from 'semver';
import { ref } from 'vue';

type Release = {
  id: number,
  tag_name: string,
};

async function checkForUpdates() {
  checking.value = true;
  try {
    const response = await fetch<Release | null>('https://api.github.com/repos/2278535805/Phi-Recorder/releases/latest', {
      method: 'GET',
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'Phi-Recorder',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });
    const release = response.data;
    if (!release) {
      throw new Error('No tags found');
    }
    const latestVersion = release.tag_name;
    updates.value = semver.gt(latestVersion, appVersion);
  } catch (error) {
    console.error('Error fetching tags:', error);
    updates.value = false;
  }
  check.value = true;
  checking.value = false;
}

const updates = ref(false);
const checking = ref(false);
const check = ref(false);
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center" style="max-width: 1280px; gap: 1rem">
    <div class="about-container">
      <h1 class="app-title gradient-text text-glow" v-t="'app'"></h1>
      <h4 class="mt-n2 version-label text-glow">v{{ appVersion }}</h4>
      <v-btn class="github-btn hover-scale" prepend-icon="mdi-github" @click="open('https://github.com/2278535805/Phi-Recorder/releases')">GitHub</v-btn>
      <v-btn v-if="!check" class="github-btn hover-scale" prepend-icon="mdi-update" :loading="checking" @click="checkForUpdates">{{ t('check') }}</v-btn>
      <p v-if="updates" class="mt-2 text-glow">{{ t('new-version') }}</p>
      <p v-if="!updates && check" class="mt-2 text-glow">{{ t('non-version') }}</p>
      <p class="license-text text-gradient">Licensed by GPLv3</p>
    </div>
  </div>
</template>

<style scoped>
.about-container {
  padding: 2rem;
  min-width: 600px;
  min-height: 300px;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  animation: fadeIn 0.5s cubic-bezier(0, 0, 0, 1) forwards;
  opacity: 0; /* 初始状态透明 */
}

@keyframes fadeIn {
  from {
    opacity: 0;
    scale: 0.8;
    transform: translateY(0px);
  }
  to {
    opacity: 1;
    scale: 1;
    transform: translateY(0px);
  }
}

.app-title {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.version-label {
  font-size: 1.25rem;
  font-weight: 500;
  opacity: 0.8;
}

.github-btn {
  background: rgba(147, 147, 147, 0.2);
  padding: 0px 24px;
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