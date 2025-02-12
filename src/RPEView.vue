<i18n>
en:
  not-binded: You have not binded RPE yet
  bind: Bind RPE
  binded: Binded successfully
  unbind: Unbind RPE
  unbinded: Unbinded successfully
  rpe-folder: Please select RPE's folder

  render: Render

zh-CN:
  not-binded: 你还没有绑定 RPE
  bind: 绑定 RPE
  binded: 绑定成功
  unbind: 解绑 RPE
  unbinded: 解绑成功
  rpe-folder: 请选择 RPE 所在文件夹

  render: 渲染

</i18n>

<script setup lang="ts">
import { ref } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

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
              :style="{ 'background-image': 'url(' + convertFileSrc(chart.illustration) + ')' }"></div>
          </div>
          <div class="d-flex flex-column w-100 chart-content">
            <v-card-title class="chart-name">{{ chart.name }}</v-card-title>
            <v-card-subtitle class="mt-n2 chart-id">{{ chart.id }}</v-card-subtitle>
            <v-card-subtitle class="chart-id">{{ chart.charter }}</v-card-subtitle>
            <div class="w-100 mt-2">
              <div class="pt-4 d-flex justify-end">
                <v-btn class="render-btn hover-scale" color="primary" @click="router.push({ name: 'render', query: { chart: chart.path } })" v-t="'render'"></v-btn>
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

.fade-in {
  animation: fadeIn 0.5s cubic-bezier(0, 0, 0, 0.75) forwards;
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

.chart-card {
  border-radius: 12px;
  overflow: hidden;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
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

.chart-cover {
  width: 35%;
  min-height: 100px;
  background: rgba(0, 0, 0, 0.1);
}

.cover-image {
  width: 100%;
  height: 100%;
}

.chart-content {
  width: 65%;
  padding: 1rem;
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

.hover-scale {
  transition: transform 0.3s ease;
}

.hover-scale:hover {
  transform: scale(1.05);
}

.text-gradient {
  background: linear-gradient(45deg, #2196f3, #e91e63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

@keyframes fadeUp {
  from {
    opacity: 0;
    scale: 0.8;
    transform: translateY(100px);
  }
  to {
    opacity: 1;
    scale: 1;
    transform: translateY(0);
  }
}

@keyframes hoverUp {
  from {
    opacity: 1;
    scale: 1;
    transform: translateY(0px);
  }
  to {
    opacity: 1;
    scale: 1.05;
    transform: translateY(-4px);
  }
}

</style>