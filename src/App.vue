<i18n>
en:
  render: Render
  rpe: RPE
  tasks: Tasks
  about: About

  ffmpeg-check: FFmpeg Check
  ffmpeg-check-detail: FFmpeg is incomplete, please install the full version of FFmpeg

  update-available: Update available!

  open-app-folder: Open app folder
  open-download: Open FFmpeg Download Page
  try-download: Try to download FFmpeg
  ffmpeg-not-found: FFmpeg not found!
  ffmpeg-not-found-detail: |
    Please download ffmpeg, Windows users usually only need to download "ffmpeg-master-latest-win64-gpl.zip"
    Place all files in the bin folder in the program folder or configure the ffmpeg environment variables
  
  confirm: Confirm
  cancel: Cancel


zh-CN:
  render: 渲染
  rpe: RPE
  tasks: 任务列表
  about: 关于

  ffmpeg-check: FFmpeg 检查
  ffmpeg-check-detail: FFmpeg 不完整, 请安装 full 版本的 FFmpeg

  update-available: 有新版本可用!

  open-app-folder: 打开程序文件夹
  open-download: 打开 FFmpeg 下载页
  try-download: 尝试下载 FFmpeg
  ffmpeg-not-found: 未找到 FFmpeg!
  ffmpeg-not-found-detail: |
    请下载 ffmpeg, Windows 用户一般只需下载 "ffmpeg-master-latest-win64-gpl.zip"
    将 bin 文件夹内的所有文件放置在程序文件夹内或配置 ffmpeg 环境变量

  confirm: 确定
  cancel: 取消

</i18n>

<script lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';

import { VSonner } from 'vuetify-sonner';
import { invoke, os, shell } from '@tauri-apps/api';
import semver from 'semver';
import { getVersion } from '@tauri-apps/api/app';

const onLoaded = ref<() => void>();
const component = ref();

watch(component, (comp) => {
  if (comp && onLoaded.value) onLoaded.value();
});

export function useOnLoaded() {
  return onLoaded;
}

declare global {
  interface Window {
    goto: (name: string) => void;
  }
}

export default {
  data() {
    return {
      drawer: true,
    };
  },
  methods: {
    toggleNav() {
      this.drawer = !this.drawer;
    },
  },

};
</script>

<script setup lang="ts">
import { fetch } from '@tauri-apps/api/http';
import type { Release, Assets } from './model';
import { open } from '@tauri-apps/api/shell';

const { t } = useI18n();

const route = useRoute(),
  router = useRouter();

const icons = ref({
  render: 'mdi-auto-fix',
  rpe: 'mdi-bookshelf',
  tasks: 'mdi-server',
  about: 'mdi-information-outline',
});

const rail = ref(true);
// 监听菜单/右键
document.addEventListener('contextmenu', (event) => {
  event.preventDefault();
  //console.log('Detected contextmenu', event);
});

// 监听中键
document.addEventListener('mousedown', (event) => {
  if (event.button === 1) {
    event.preventDefault();
    //console.log('Detected mousedown 1', event);
    rail.value = !rail.value;
  }
});

window.goto = (name: string) => {
  router.push({ name });
};


// FFmpeg
const platform = os.type();
const isWindows = String(platform) === 'Windows_NT';
const isLinux = String(platform) === 'Linux';
const ffmpegGetNewVersionLoding = ref(false);
const ffmpegDialog = ref(false);
const ffmpegDialogFilter = ref(false);

async function getNewVersion() {
  //dialog_download.value = true;
  ffmpegGetNewVersionLoding.value = true;
  try {
    const response = await fetch('https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/latest', {
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
        return asset.name.includes('ffmpeg-master-latest-win64-gpl.zip');
      } else if (isLinux) {
        return asset.name.includes('ffmpeg-master-latest-linux64-gpl.tar.xz');
      }
      return false;
    })

    const link = (asset as Assets).browser_download_url;
    console.log(link);
    await open(link);
    
  } catch (error) {
    console.error('Error fetching tags:', error);
    await open("https://github.com/BtbN/FFmpeg-Builds/releases");
  } finally {
    ffmpegGetNewVersionLoding.value = false;
  }
}

async function openAppFolder() {
  await invoke('open_app_folder');
}

async function openDownload() {
  await shell.open('https://github.com/BtbN/FFmpeg-Builds/releases');
}

async function testFFmpegFilter() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      ffmpegDialog.value = true;
      return false;
    }
    ffmpegDialogFilter.value = !(await invoke("test_ffmpeg_filter"));
    } catch (error) {
      console.error('Error running test_ffmpeg_filter:', error);
    }
}

// Update
const update = ref(false);
async function checkForUpdates(dialog = true): Promise<boolean> {
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
    //const latestVersion = '0.4.0';
    console.log(latestVersion);
    const updates = semver.gt(latestVersion, await getVersion());
    if (updates) {
      return true;
    }
  } catch (error) {
    console.error('Error fetching tags:', error);  }
    return false;
}


onMounted(async () => {
  //setTimeout(() => {
  testFFmpegFilter();

  if (await checkForUpdates()) {
    icons.value.about = 'mdi-cloud-download';
    update.value = true;
  }
  //}, 100);
});
</script>

<template>
  <v-app id="phi-recorder" class="dark-theme">
    <v-sonner position="top-center" />
    <v-app-bar :elevation="0" class="app-bar-shadow blur-background">
      <!--<v-app-bar-nav-icon @click="toggleNav" class="mx-1"></v-app-bar-nav-icon>-->
      <div class="gradient-text">
        <v-app-bar-title class="mx-5 text-glow">Phi Recorder</v-app-bar-title>
      </div>
      <div v-if="update">
        <i class="mdi mdi-cloud-download"></i>&nbsp;&nbsp;{{t('update-available')}}
      </div>
    </v-app-bar>
    <v-navigation-drawer v-model="drawer" :expand-on-hover="rail" rail permanent class="nav-drawer-border blur-background list-item">
      <v-list density="compact" nav>
        <v-list-item
          v-for="key in ['render', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key })"
          class="list-item-hover"
          v-if="rail"
          ></v-list-item>

          <v-list-item
          v-for="key in ['render', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key })"
          class="list-item-hover-rail"
          v-if="!rail"
          ></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-main class="d-flex justify-center">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" />
          </template>
          <template #fallback>
            <div class="flex justify-center pa-8">
              <v-progress-circular class="waitIn" indeterminate size="large" />
            </div>
          </template>
        </Suspense>
      </router-view>
    </v-main>
    <v-dialog v-model="ffmpegDialog" width="auto" min-width="400px" class="log-card-bg">
      <v-card class="log-card-window">
        <v-card-title v-t="t('ffmpeg-not-found')"> </v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto log-card-msg" style="max-height: 60vh; white-space: pre-wrap">{{ t('ffmpeg-not-found-detail') }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn variant="text" @click="getNewVersion" :loading="ffmpegGetNewVersionLoding" v-t="t('try-download')"></v-btn>
          <v-btn variant="text" @click="openDownload" v-t="t('open-download')"></v-btn>
          <v-btn variant="text" @click="openAppFolder" v-t="t('open-app-folder')"></v-btn>
          <v-btn color="primary" class="hover-scale" variant="text" @click="ffmpegDialog = false" v-t="t('confirm')"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="ffmpegDialogFilter" width="auto" min-width="400px" class="log-card-bg">
      <v-card class="log-card-window">
        <v-card-title v-t="t('ffmpeg-check')"> </v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto log-card-msg" style="max-height: 60vh; white-space: pre-wrap">{{ t('ffmpeg-check-detail') }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" class="hover-scale" variant="text" @click="ffmpegDialogFilter = false" v-t="t('confirm')"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-app>
</template>

<style>

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background-color: rgba(54, 50, 98, 0.1);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background-color: rgba(99, 102, 241, 0.5);
  border-radius: 4px;
  transition: background-color 0.3s;
}

::-webkit-scrollbar-thumb:hover {
  background-color: rgba(139, 92, 246, 0.7);
}

html {
  scroll-behavior: smooth;
}

/* make all text unselectable */
* {
  -webkit-user-select: none;
  /* Safari */
  -moz-user-select: none;
  /* Firefox */
  -ms-user-select: none;
  /* IE and Edge */
  user-select: none;
  /* Non-prefixed version, currently supported by Chrome, Opera, and Edge */
}

.v-input__details {
  /* 不让details占据布局空间 */
  display: none;
}

.v-form {
  scrollbar-width: thin;
  /* Firefox */
  scrollbar-color: rgba(99, 102, 241, 0.5) rgba(54, 50, 98, 0.1);
  /* Firefox */
}

.v-overlay-container {
  z-index: 1000;
  /* 设置遮罩层的背景颜色,使其与主题一致 */
  background-color: rgba(56, 56, 146, 0.8);
}

.waitIn {
  visibility: hidden;
  animation: Up 0.5s 0.5s forwards;
}

@keyframes Up {
  to {
    visibility: visible;
  }
}

.dark-theme {
  background: linear-gradient(45deg, #292364, #302b63, #24243e);
}

.app-bar-shadow {
  box-shadow: 0px 10px 10px rgba(0, 0, 0, 0.2) !important;
}

.nav-drawer-border {
  border-right: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.gradient-text {
  background: linear-gradient(30deg, #04c9ff, #ff13d4);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.list-item {
  transition: all 0.3s cubic-bezier(0.2, 0, 0.1, 1);
  box-shadow: 0px 0px 10px rgba(0, 0, 0, 0.2) !important;
}

.list-item-hover {
  transition: all 0.3s ease;
  margin: 8px 0px;
  border-radius: 12px;
}

.list-item-hover:hover {
  background: rgba(255, 255, 255, 0.05) !important;
  margin: 8px 4px;
  transform: translateX(4px);
  filter: drop-shadow(0 0 8px #ffffff8a);
}

.list-item-hover-rail {
  transition: all 0.3s ease;
  margin: 8px 0px;
  border-radius: 12px;
}

.active-item {
  background: linear-gradient(45deg, rgba(33, 150, 243, 0.2), transparent) !important;
}

.active-item:hover {
  background: linear-gradient(45deg, rgba(33, 150, 243, 0.2), transparent) !important;
}

.glow-spinner {
  filter: drop-shadow(0 0 8px #2196F3);
}

.animated-background {
  position: relative;
  overflow: hidden;
}

.animated-background::before {
  content: '';
  position: absolute;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    45deg,
    rgba(255, 255, 255, 0.01) 25%,
    transparent 25%,
    transparent 50%,
    rgba(255, 255, 255, 0.01) 50%,
    rgba(255, 255, 255, 0.01) 75%,
    transparent 75%,
    transparent
  );
  animation: animateFlow 2s linear infinite;
  opacity: 0.1;
}

@keyframes animateFlow {
  0% { transform: translate(-25%, -25%) rotate(0deg); }
  100% { transform: translate(-25%, -25%) rotate(360deg); }
}

.blur-background {
  backdrop-filter: blur(40px) saturate(180%);
  background: linear-gradient(45deg, rgba(122, 98, 168, 0.4), rgba(107, 66, 182, 0.4)) !important;
  transform: translateZ(0);
  position: relative;
  z-index: 1;
}
  
/* 新增的过渡动画样式 */
.fade-blur-enter-active,
.fade-blur-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: absolute;
  width: 100%;
}

.fade-blur-enter-from {
  opacity: 0;
  filter: blur(10px);
  transform: translateY(-20px);
}

.fade-blur-leave-to {
  opacity: 0;
  filter: blur(10px);
  transform: translateY(20px);
}

/* 新增的加载遮罩层样式 */
.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  backdrop-filter: blur(20px);
  background: rgba(16, 16, 36, 0.8);
  z-index: 999;
  display: flex;
  align-items: center;
  justify-content: center;
}

html {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

html::-webkit-scrollbar {
  display: none;
}
</style>