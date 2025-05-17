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
import { open } from '@tauri-apps/plugin-shell';
//import { random } from 'mathjs';
//import { download as tauriDownload } from '@tauri-apps/plugin-upload';

const appVersion = await getVersion();

import { fetch } from '@tauri-apps/plugin-http';
import semver from 'semver';
import { onMounted, ref } from 'vue';
import MarkdownIt from 'markdown-it';
const md = new MarkdownIt();

import { useTheme } from 'vuetify';
const theme = useTheme();

import * as os from "@tauri-apps/plugin-os";

const platform = os.family();
const isWindows = String(platform) === 'windows';
const isMacOS = String(platform) === 'macos';
const isLinux = String(platform) === 'linux';

import type { Release, Assets } from './model';
async function checkForUpdates(dialog = true) {
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
    const release: Release = await response.json();
    console.log(release);
    
    if (!release) {
      throw new Error('No tags found');
    }
    const latestVersion = release.tag_name;
    //const latestVersion = '0.4.0';
    console.log(latestVersion);
    if (latestVersion) {
      updates.value = semver.gt(latestVersion, appVersion);
      updateBody.value = `${latestVersion}\n${release.body}`;
      if (updates.value) {
        dialog_update.value = true;
      } else if (dialog) {
        dialog_non.value = true;
      }
    } else {
      updateBody.value = `${release.message}`;
      updates.value = false;
      dialog_error.value = true;
    }
  } catch (error) {
    console.error('Error fetching tags:', error);
    updateBody.value = `${error}`;
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
    const release = await response.json() as Release;
    if (!release) {
      throw new Error('No tags found');
    }

    const assets = release.assets as Assets[];
    if (assets.length === 0) {
      throw new Error('No assets found');
    }
    const asset = assets.find((asset) => {
      if (isWindows) {
        return asset.name.endsWith('.msi');
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
    await open("https://github.com/2278535805/Phi-Recorder/releases/latest");
  }
}

const progress = ref(0.0);

const updates = ref(false);
const checking = ref(false);

const dialog_update = ref(false);
const dialog_non = ref(false);
const dialog_error = ref(false);
const dialog_download = ref(false);
const updateBody = ref('');

onMounted(() => {
  //setTimeout(() => {
  checkForUpdates(false);
  //}, 100);
});
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center" style="max-width: 1280px; gap: 1rem">
    <div class="about-container container fade-in" :style="{ background: `${theme.current.value.colors.container}` }">
      <h1 class="app-title gradient-text text-glow" v-t="'app'"></h1>
      <h4 class="mt-n2 version-label text-glow">v{{ appVersion }}</h4>
      <v-btn class="github-btn hover-scale" prepend-icon="mdi-github" @click="open('https://github.com/2278535805/Phi-Recorder/releases')">GitHub</v-btn>
      <v-btn class="github-btn hover-scale" prepend-icon="mdi-update" :loading="checking" @click="checkForUpdates">{{ t('check') }}</v-btn>
      <p class="license-text license-text-gradient">Licensed by GPLv3</p>
    </div>
  </div>

  <v-dialog v-model="dialog_update" theme="darkTheme" width="auto" min-width="400px" class="log-card-bg">
    <v-card class="log-card-only-window">
      <v-card-title v-t="t('check')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ t('new-version') }}</pre>
        <div class="block overflow-auto" style="max-height: 60vh" v-html="md.render(updateBody)"></div>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn color="btn" variant="text" @click="dialog_update = false, getNewVersion()" v-t="t('download')"></v-btn>
        <v-btn color="btn" variant="text" @click="dialog_update = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="dialog_non" theme="darkTheme" width="auto" min-width="400px" class="log-card-bg">
    <v-card class="log-card-only-window">
      <v-card-title v-t="t('check')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ t('non-version') }}</pre>
        <div class="block overflow-auto" style="max-height: 60vh" v-html="md.render(updateBody)"></div>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn color="btn" variant="text" @click="dialog_non = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="dialog_error" theme="darkTheme" width="auto" min-width="400px" class="log-card-bg">
    <v-card class="log-card-only-window">
      <v-card-title v-t="t('check')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ t('err-version') }}</pre>
        <pre class="block whitespace-pre overflow-auto select wrap" style="max-height: 60vh">{{ updateBody }}</pre>
      </v-card-text>
      <v-card-actions class="justify-end">
        <v-btn color="btn" variant="text" @click="dialog_error = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="dialog_download" theme="darkTheme" width="auto" min-width="400px" class="log-card-bg">
    <v-card class="log-card-only-window">
      <v-card-title v-t="t('download')"> </v-card-title>
      <v-card-text>
        <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ '111' }}</pre>
      </v-card-text>

      <v-progress-linear :model-value="progress * 100" rounded></v-progress-linear>

      <v-card-actions class="justify-end">
        <v-btn color="btn" variant="text" @click="dialog_download = false" v-t="t('close')"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.about-container {
  margin: 0px;
  padding: 2rem;
  min-width: none;
  min-height: 300px;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
}

@media (max-width: 720px) {
  .about-container {
    min-width: 350px;
    width: 100%;
    min-height: 300px;
    max-width: 1280px;
  }
}

@media (min-width: 721px) {
  .about-container {
    min-width: 600px;
    min-height: 300px;
    max-width: 1280px;
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

.gradient-text {
  background: linear-gradient(45deg, #2196f3, #e91e63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.license-text {
  font-size: 0.9rem;
  opacity: 0.7;
}

.license-text-gradient {
  background: linear-gradient(45deg, #4caf50, #ffeb3b);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
</style>