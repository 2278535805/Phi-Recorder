<i18n>
en:
  not-binded: You have not binded RPE yet
  bind: Bind RPE
  binded: Binded successfully
  unbind: Unbind RPE
  unbinded: Unbinded successfully
  rpe-folder: Please select RPE's folder
  show-folder: Open Folder

  render: Render

zh-CN:
  not-binded: 你还没有绑定 RPE
  bind: 绑定 RPE
  binded: 绑定成功
  unbind: 解绑 RPE
  unbinded: 解绑成功
  rpe-folder: 请选择 RPE 所在文件夹
  show-folder: 打开文件夹

  render: 渲染

</i18n>

<script setup lang="ts">
import { ref } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

import { toast, toastError } from './common';
import type { RPEChart } from './model';
import router from './router';

async function getRPECharts() {
  return (await invoke('get_rpe_charts')) as RPEChart[] | null;
}
const charts = ref(await getRPECharts());

async function bindRPE() {
  let file = await open({ directory: true, title: t('rpe-folder') });
  if (!file) return;
  try {
    await invoke('set_rpe_dir', { path: file });
    toast(t('binded'), 'success');
    charts.value = await getRPECharts();
  } catch (e) {
    toastError(e);
  }
}
async function unbindRPE() {
  try {
    await invoke('unset_rpe_dir');
    toast(t('unbinded'), 'success');
    charts.value = null;
  } catch (e) {
    toastError(e);
  }
}

async function openInFolder(path: string) {
  try {
    await invoke('open_in_folder', { path });
  } catch (e) {
    toastError(e);
  }
}
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1rem">
    <template v-if="!charts">
      <h1 class="text-center font-italic text-disabled unbinded-title text-gradient fade-in" v-t="'not-binded'"></h1>
      <v-form class="text-center fade-in" ref="form" style="max-height: 48vh;">
        <v-row>
          <v-col cols="12" style="margin: -20px 0px;">
            <v-btn size="large" class="italic mt-2 v-btn hover-scale" @click="bindRPE" style="width: fit-content" v-t="'bind'"></v-btn>
          </v-col>
        </v-row>
      </v-form>
    </template>
    <template v-if="charts">
      <v-form class="text-center fade-in" ref="form" style="max-height: 48vh;">
        <v-row>
          <v-col cols="12" style="margin: -20px 0px;">
            <v-btn size="large" class="italic v-btn hover-scale" @click="unbindRPE" style="width: fit-content" v-t="'unbind'"></v-btn>
          </v-col>
        </v-row>
      </v-form>
      <v-card v-for="(chart, index) in charts" :key="chart.id" class="chart-card" :style="{ animationDelay: index * 0.1 + 's' }">
        <div class="d-flex flex-row align-stretch">
          <div class="d-flex flex-row align-center chart-cover" style="width: 35%">
            <div
              class="cover-image"
              style="width: 100%; height: 100%; max-height: 240px; background-position: center; background-repeat: no-repeat; background-size: cover"
              :style="{ 'background-image': 'url(' + convertFileSrc(chart.illustration) + ')' }">
              <div 
              class="overlay"
              @click="router.push({ name: 'render', query: { chart: chart.path } })"
              >
              <i class="mdi mdi-play icon">
              </i>
            </div>
            </div>
          </div>
          <div class="d-flex flex-column w-100 chart-content">
            <v-card-title class="chart-name">{{ chart.name }}</v-card-title>
            <v-card-subtitle class="mt-n2 chart-id">{{ chart.id }}</v-card-subtitle>
            <v-card-subtitle class="chart-id">{{ chart.charter }}</v-card-subtitle>
            <div class="w-100 mt-2">
              <div class="pt-4 d-flex justify-end">
                <v-btn class="open-btn mx-4" @click="openInFolder(chart.path)" v-t="'show-folder'"></v-btn>
                <v-btn class="render-btn" @click="router.push({ name: 'render', query: { chart: chart.path } })" v-t="'render'"></v-btn>
              </div>
            </div>
          </div>
        </div>
      </v-card>
    </template>
  </div>
</template>

<style scoped>
.rpe-container {
  padding: 2rem;
  width: 100%;
  height: 100%;
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.unbinded-title {
  font-size: 2rem;
  font-weight: 700;
  text-align: center;
  margin-bottom: 1.5rem;
}

.v-btn {
  background: rgba(255, 255, 255, 0.05);
  font-weight: 600;
  padding: 12px 24px;
  margin-bottom: 12px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.chart-card {
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  margin: 5px;
  box-shadow: 0px 0px 12px rgba(0, 0, 0, 0.1);
  animation: fadeUp 0.5s cubic-bezier(0, 0, 0, 1) forwards;
  opacity: 0; /* 初始状态透明 */
}

.chart-card:hover {
  background: rgba(255, 255, 255, 0.06);
  box-shadow: 0px 0px 24px rgba(0, 0, 0, 0.3);
}

.chart-name {
  font-size: 1.5rem;
  font-weight: 600;
}

.chart-id {
  font-size: 0.9rem;
  opacity: 0.7;
}

.render-btn {
  font-weight: 600;
  padding: 8px 16px;
  background: linear-gradient(45deg, #6366f1, #8b5cf6) !important;
  box-shadow: 0 4px 6px -1px rgb(99 102 241 / 0.2);
  transition: transform 0.2s, box-shadow 0.2s;
}

.render-btn:hover {
  font-weight: 700;
  padding: 8px 16px;
  transform: translateY(-1px);
  box-shadow: 0 10px 15px -3px rgb(99 102 241 / 0.3);
}

.open-btn {
  font-weight: 600;
  padding: 8px 16px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}

.open-btn:hover {
  font-weight: 700;
  padding: 8px 16px;
  transform: translateY(-1px);
  box-shadow: 0 10px 15px -2px rgba(0, 0, 0, 0.2);
}

.text-gradient {
  background: linear-gradient(45deg, #2196f3, #e91e63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

@media (max-width: 600px) {
  .cover-image {
    width: 100%;
    height: 100%;
  }

  .chart-cover {
    max-width: 0%;
    min-height: 100px;
    background: rgba(0, 0, 0, 0.1);
  }

  .chart-content {
    max-width: 100%;
    width: 65%;
    padding: 1rem;
  }
}

@media (min-width: 601px) and (max-width: 1336px) {
  .cover-image {
    width: 100%;
    height: 100%;
  }

  .chart-cover {
    max-width: 35%;
    min-height: 100px;
    background: rgba(0, 0, 0, 0.1);
  }

  .chart-content {
    max-width: 69%;
    width: 65%;
    padding: 1rem;
  }
}

@media (min-width: 1336px) {
  .cover-image {
    width: 100%;
    height: 100%;
  }

  .chart-cover {
    min-width: 310px;
    max-width: 35%;
    min-height: 100px;
    background: rgba(0, 0, 0, 0.1);
  }

  .chart-content {
    max-width: 69%;
    width: 65%;
    padding: 1rem;
  }
}

</style>