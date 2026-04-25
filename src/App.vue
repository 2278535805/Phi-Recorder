<script lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';

import { VSonner } from 'vuetify-sonner';
import { invoke,   } from '@tauri-apps/api/core';
import { gt } from 'semver';
import { getVersion } from '@tauri-apps/api/app';
import * as os from "@tauri-apps/plugin-os"
import { openUrl } from '@tauri-apps/plugin-opener';
import { useStorage } from '@vueuse/core';

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
import { fetch } from '@tauri-apps/plugin-http';
import type { Release, Assets } from './model';
import { open } from '@tauri-apps/plugin-shell';
import { useTheme } from 'vuetify';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { toast } from './common';
const appWindow = getCurrentWindow();
const theme = useTheme();

localStorage.removeItem('BatchView.ChartList')
localStorage.removeItem('BatchView.Preset')
const useSystemTheme = useStorage<boolean>('useSystemTheme', true);

let count = 0;
let timer: number | null = null;

document.addEventListener('contextmenu', (e) => {
  count++;
  if (timer) {
    window.clearTimeout(timer);
  }

  timer = window.setTimeout(() => {
    count = 0;
    timer = null;
  }, 2000);

  if (count >= 7) {
    count = 0;
    const rand = Math.floor(Math.random() * 101);
    toast(`${t('luck-value')}: ${rand}`, 'info');
    const zoomValue = 300 - rand * 2;
    const blurValue = (100 - rand) * 0.02;
    const hueValue = 270 - rand * 2.70;
    const saturateValue = rand;
    const contrastValue = 150 - rand * 0.5;
    document.body.style.zoom = `${zoomValue}%`;
    document.body.style.filter = `blur(${blurValue}px) hue-rotate(${hueValue}deg) saturate(${saturateValue}%) contrast(${contrastValue}%`;
  }
}, { passive: true });

const { t } = useI18n();

const route = useRoute(),
  router = useRouter();

function routerPush(name: string) {
  router.push({ name });
}

const icons = ref({
  render: 'mdi-auto-fix',
  batch: 'mdi-filmstrip-box-multiple',
  bind: 'mdi-bookshelf',
  tasks: 'mdi-server',
  settings: 'mdi-cog',
  about: 'mdi-information-outline',
});

window.goto = (name: string) => {
  router.push({ name });
};

function appClose() {
  appWindow.close();
}

function appMinimize() {
  appWindow.minimize();
}

function appMaximize() {
  appWindow.toggleMaximize();
}

const now_theme = useStorage<string>('theme', 'Light');
watch(now_theme, (val) => {
  if (!val) return;
  theme.global.name.value = val;
});

function switchTheme() {
  const themes = ['Light', 'Dark', 'DeepDark', 'LightBlue'];
  const current = theme.global.name.value;
  const nextIndex = (themes.indexOf(current) + 1) % themes.length;
  const nextTheme = themes[nextIndex];

  now_theme.value = nextTheme;
}

document.addEventListener("keydown", (event) => {
  if (event.key === "F2") {
    switchTheme();
  }
});

window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (event) => {
  if (useSystemTheme.value) {
    now_theme.value = event.matches ? "DeepDark" : "LightBlue";
  }
});



// FFmpeg
const platform = os.family();
const isWindows = String(platform) === 'windows';
const isLinux = String(platform) === 'linux';
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
        return asset.name.includes('ffmpeg-master-latest-win64-gpl.zip');
      } else if (isLinux) {
        return asset.name.includes('ffmpeg-master-latest-linux64-gpl.tar.xz');
      }
      return false;
    })

    const link = (asset as Assets).browser_download_url;
    console.log(link);
    await openUrl(link);
    
  } catch (error) {
    console.error('Error fetching tags:', error);
    await openUrl("https://github.com/BtbN/FFmpeg-Builds/releases");
  } finally {
    ffmpegGetNewVersionLoding.value = false;
  }
}

async function openAppFolder() {
  await invoke('open_app_folder');
}

async function checkFFmpegFilter() {
  try {
    if (!(await invoke('check_ffmpeg'))) {
      ffmpegDialog.value = true;
      return false;
    }
    ffmpegDialogFilter.value = !(await invoke("check_ffmpeg_filter", { path: null }));
    } catch (error) {
      console.error('Error running check_ffmpeg_filter:', error);
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
    const release = await response.json() as Release;
    console.log(release);
    
    if (!release) {
      throw new Error('No tags found');
    }
    const latestVersion = release.tag_name;
    //const latestVersion = '0.4.0';
    console.log(latestVersion);
    const updates = gt(latestVersion, await getVersion());
    if (updates) {
      return true;
    }
  } catch (error) {
    console.error('Error fetching tags:', error);  }
    return false;
}

const listExpand = ref(
  localStorage.getItem("listExpand") !== null
    ? JSON.parse(localStorage.getItem("listExpand") as string)
    : true
);

onMounted(async () => {
  checkFFmpegFilter();

  if (await checkForUpdates()) {
    icons.value.about = 'mdi-cloud-download';
    update.value = true;
  }
});
</script>

<template>
  <v-app id="phi-recorder" :style="{ background: `linear-gradient(45deg, ${theme.current.value.colors.bgLeft}, ${theme.current.value.colors.bgRight}` }">
    <v-sonner position="top-center" />
    <v-app-bar style="box-shadow: 0px -18px 20px 0px rgba(0, 0, 0, 0.5) !important" class="blur-background" height="60">
      <!--<v-app-bar-nav-icon @click="toggleNav" class="mx-1"></v-app-bar-nav-icon>-->
      <div class="gradient-text" style="position: absolute; pointer-events: none;">
        <v-app-bar-title class="mx-5 text-glow">Phi Recorder</v-app-bar-title>
      </div>
      <div @click="routerPush('about')" style="position: absolute; left: 152px; cursor: pointer;" v-if="update">
          <i class="mdi mdi-cloud-download"></i>&nbsp;&nbsp;{{t('update-available')}}
      </div>
      <div data-tauri-drag-region class="flex-grow-1" style="height: 100%; min-width: 10px;"></div>
      <div class="d-flex" style="position: fixed; right: 0;">
        <v-btn class="mx-2" size="small" color="grey" icon="mdi-circle" @click="appMinimize" @contextmenu="appMaximize"></v-btn>
        <v-btn class="mr-4" size="small" color="red" icon="mdi-circle" @click="appClose()"></v-btn>
      </div>
    </v-app-bar>
    <v-navigation-drawer v-model="drawer" width="170" :expand-on-hover="listExpand" rail permanent class="nav-drawer-border blur-background list-item">
      <v-list density="compact" nav class="v-list-none" v-if="listExpand">
        <v-list-item
          v-for="key in ['render', 'batch', 'bind', 'tasks', 'settings', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="routerPush(key)"
          @contextmenu="listExpand = !listExpand"
          slim
          class="list-item-hover"
        ></v-list-item>
      </v-list>
      <v-list density="compact" nav class="v-list-none" v-else>
        <v-list-item
          v-for="key in ['render', 'batch', 'bind', 'tasks', 'settings', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="routerPush(key)"
          @contextmenu="listExpand = !listExpand"
          slim
          class="list-item-hover-rail"
        ></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-main class="d-flex justify-center">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" :style="{ 'scrollbar-color': `${theme.current.value.colors.scrollbar} ${theme.current.value.colors.back}` }" />
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
          <div class="block whitespace-pre overflow-auto log-card-msg select wrap" style="max-height: 60vh; white-space: pre-wrap">{{ t('ffmpeg-not-found-detail') }}</div>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn variant="text" @click="getNewVersion" :loading="ffmpegGetNewVersionLoding" v-t="t('try-download')"></v-btn>
          <v-btn variant="text" @click="openUrl('https://github.com/BtbN/FFmpeg-Builds/releases')" v-t="t('open-download')"></v-btn>
          <v-btn variant="text" @click="openAppFolder" v-t="t('open-app-folder')"></v-btn>
          <v-btn color="btn" class="hover-scale" variant="text" @click="ffmpegDialog = false" v-t="t('confirm')"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="ffmpegDialogFilter" width="auto" min-width="400px" class="log-card-bg">
      <v-card class="log-card-only-window">
        <v-card-title v-t="t('ffmpeg-check')"> </v-card-title>
        <v-card-text>
          <div class="block whitespace-pre overflow-auto select wrap" style="max-height: 60vh; white-space: pre-wrap">{{ t('ffmpeg-check-detail') }}</div>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="btn" class="hover-scale" variant="text" @click="ffmpegDialogFilter = false" v-t="t('confirm')"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-app>
</template>

<style>

*[data-tauri-drag-region] {
  app-region: drag;
}

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background-color: rgba(58, 58, 58, 0.1);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background-color: rgba(58, 58, 58, 0.4);
  border-radius: 4px;
  transition: background-color 0.3s;
}

::-webkit-scrollbar-thumb:hover {
  background-color: rgba(94, 94, 94, 0.7);
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

.v-input__details { /* 不让details占据布局空间 */
  /* display: none; */
  margin-bottom: -6px;
  min-height: 16px;
  padding-top: 2px
}

.v-form {
  scrollbar-width: thin;
  /* scrollbar-color: rgba(99, 102, 241, 0.5) rgba(54, 50, 98, 0.1); */
}

.scroll-mask {
  mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 0) 0%, rgba(0, 0, 0, 1) 5%, rgba(0, 0, 0, 1) 95%, rgba(0, 0, 0, 0) 100%);
  -webkit-mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 0) 0%, rgba(0, 0, 0, 1) 5%, rgba(0, 0, 0, 1) 95%, rgba(0, 0, 0, 0) 100%);
}

.v-overlay__scrim {
  background: rgba(255, 255, 255, 0.0);
}

.waitIn {
  visibility: hidden;
  animation: Up 0.5s 0.5s forwards;
}

.container {
  margin: 2rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0px 0px 12px rgba(0, 0, 0, 0.05);
}

.v-list-none {
  background-color: none !important;
  backdrop-filter: none !important;
}

.v-list {
  background-color: rgba(var(--v-theme-primary), 0.0) !important; /* This color cannot be apply in tag editor */
  backdrop-filter: blur(20px);
}

.log-card-bg {
  animation: blurFade 0.3s ease forwards;
}


.log-card-window {
  border-radius: 16px !important;
  background: rgba(var(--v-theme-dialog), 0.6) !important;
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.log-card-only-window {
  border-radius: 16px !important;
  background: rgba(var(--v-theme-dialog), 0.8) !important;
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.log-card-msg {
  user-select: text;
  border-radius: 12px !important;
  background: rgba(var(--v-theme-dialog), 0.8) !important;
  font-family: CascadiaMono, HarmonyOSSansSC, SarasaUiSC, system-ui !important;
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
  white-space: pre;
  padding: 1rem 1rem 1rem 1rem;
}

.v-btn {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--v-theme-primary);
}

.overlay {
  position: relative;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(var(--v-theme-overlay), 0.3);
  display: flex;
  justify-content: center;
  align-items: center;
  opacity: 0;
  transition: opacity 0.3s ease;
  cursor: pointer;
}

.overlay:hover {
  opacity: 1;
}

.icon {
  font-size: 250%;
}

.nav-drawer-border {
  border-right: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.list-item {
  transition: all 0.3s cubic-bezier(0.2, 0, 0.1, 1);
  box-shadow: 0px 0px 10px rgba(0, 0, 0, 0.1) !important;
}

.list-item-hover {
  transition: all 0.5s ease;
  margin: 8px 0 0 0;
  border-radius: 12px;
}

.list-item-hover:hover {
  background: rgba(255, 255, 255, 0.05) !important;
  margin: 8px 0 0 4px;
  filter: drop-shadow(0 0 8px #ffffff8a);
}

.list-item-hover:active {
  margin: 8px 0 0 6px;
}

.list-item-hover-rail {
  transition: all 0.3s ease;
  margin: 8px 0px;
  border-radius: 12px;
}

.list-item-hover-rail:hover {
  background: rgba(255, 255, 255, 0.05) !important;
  transform: translateY(-1px);
  filter: drop-shadow(0 0 8px #ffffff8a);
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

.blur-background {
  background: linear-gradient(300deg, rgba(var(--v-theme-topLeft), 0.4) 10%, rgba(var(--v-theme-topRight), 0.4) 90%) !important;
  backdrop-filter: blur(40px) saturate(180%);
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

.drop-zone-overlay {
  /* background: rgba(255, 255, 255, 0.15) !important; */
  /* backdrop-filter: blur(20px); */
  /* animation: all 0.3s ease; */
  animation: blurFade 0.4s ease forwards;
}

html {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

html::-webkit-scrollbar {
  display: none;
}
</style>