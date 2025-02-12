<i18n>
en:
  app: Phi Recorder
  check: Check Update
  new-version: New version available!
  non-version: It's the latest version
  err-version: Check update failed
  download: Download
  close: Close

zh-CN:
  app: Phi Recorder
  check: 检查更新
  new-version: 发现新版本!
  non-version: 已是最新版本
  err-version: 检查更新失败
  download: 下载
  close: 关闭

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();
const { t } = useI18n();

import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/api/shell';
//import { random } from 'mathjs';
//import { download as tauriDownload } from '@tauri-apps/plugin-upload';

const appVersion = await getVersion();

import { fetch } from '@tauri-apps/api/http';
import semver from 'semver';
import { ref } from 'vue';

import { os } from '@tauri-apps/api';

const platform = await os.type();
const isWindows = String(platform) === 'Windows_NT';
const isMacOS = String(platform) === 'Darwin';
const isLinux = String(platform) === 'Linux';

type Assets = {
  browser_download_url: string,
  name: string,
}

type Release = {
  id: number,
  assets: Assets[],
  tag_name: string,
};

async function checkForUpdates() {
  checking.value = true;
  try {
    const response = await fetch('https://api.github.com/repos/2278535805/Phi-Recorder/releases/latest', {
      method: 'GET',
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'Phi-Recorder',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });
    const release = response.data as Release;
    console.log(release);
    
    if (!release) {
      throw new Error('No tags found');
    }
    const latestVersion = release.tag_name;
    //const latestVersion = '0.3.0';
    console.log(latestVersion);
    updates.value = semver.gt(latestVersion, appVersion);
    if (updates.value) {
      dialog_update.value = true;
    } else {
      dialog_non.value = true;
    }
  } catch (error) {
    console.error('Error fetching tags:', error);
    updates.value = false;
    dialog_error.value = true;
  }
  checking.value = false;
}

const clamp = (num: number, lower: number, upper: number) => {
  return Math.min(Math.max(num, lower), upper);
};

async function download(url: string) {
  await open(url);
  //dialog_download.value = false;
  return;
}

async function getNewVersion() {
  //dialog_download.value = true;
  
  try {
    const response = await fetch('https://api.github.com/repos/2278535805/Phi-Recorder/releases/latest', {
      method: 'GET',
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'Phi-Recorder',
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });
    const release = response.data as Release;
    if (!release) {
      throw new Error('No tags found');
    }

    const assets = release.assets as Assets[];
    if (assets.length === 0) {
      throw new Error('No assets found');
    }
    const asset = assets.find((asset) => {
      if (isWindows) {
        return asset.name.endsWith('.exe');
      } else if (isMacOS) {
        return asset.name.endsWith('.dmg');
      } else if (isLinux) {
        return asset.name.endsWith('.AppImage');
      }
      return false;
    })
    //const subName = isWindows ? '.exe' : (isMacOS ? '.dmg' : '.AppImage');

    const link = (asset as Assets).browser_download_url;
    console.log(link);
    await download(link);
    
  } catch (error) {
    console.error('Error fetching tags:', error);
  }
}

const progress = ref(0.0);

const updates = ref(false);
const checking = ref(false);

const dialog_update = ref(false);
const dialog_non = ref(false);
const dialog_error = ref(false);
const dialog_download = ref(false);
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center" style="max-width: 1280px; gap: 1rem">
    <div class="about-container">
      <h1 class="app-title gradient-text text-glow" v-t="'app'"></h1>
      <h4 class="mt-n2 version-label text-glow">v{{ appVersion }}</h4>
      <v-btn class="github-btn hover-scale" prepend-icon="mdi-github" @click="open('https://github.com/2278535805/Phi-Recorder/releases')">GitHub</v-btn>
      <v-btn class="github-btn hover-scale" prepend-icon="mdi-update" :loading="checking" @click="checkForUpdates">{{ t('check') }}</v-btn>
      <p class="license-text text-gradient">Licensed by GPLv3</p>
    </div>
  </div>

  <v-dialog v-model="dialog_update" width="auto" min-width="400px">
    <v-card class="log-card">
      <v-card-title v-t="t('check')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ t('new-version') }}</pre>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn color="primary" variant="text" @click="dialog_update = false, getNewVersion()" v-t="t('download')"></v-btn>
        <v-btn color="primary" variant="text" @click="dialog_update = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="dialog_non" width="auto" min-width="400px">
    <v-card class="log-card">
      <v-card-title v-t="t('check')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ t('non-version') }}</pre>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn color="primary" variant="text" @click="dialog_non = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="dialog_error" width="auto" min-width="400px">
    <v-card class="log-card">
      <v-card-title v-t="t('check')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ t('err-version') }}</pre>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn color="primary" variant="text" @click="dialog_error = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="dialog_download" width="auto" min-width="400px">
    <v-card class="log-card">
      <v-card-title v-t="t('download')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ '111' }}</pre>
      </v-card-text>

      <v-progress-linear :model-value="progress * 100" rounded></v-progress-linear>

      <v-card-actions class="justify-end">
        <v-btn color="primary" variant="text" @click="dialog_download = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
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

.log-card {
  border-radius: 16px !important;
  background: rgba(0, 0, 0, 0.6) !important;
  backdrop-filter: blur(80px);
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>