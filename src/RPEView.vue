<i18n>
en:
  not-binded: You have not binded RPE yet
  bind: Bind RPE
  binded: Binded successfully
  unbind: Unbind RPE
  unbinded: Unbinded successfully
  rpe-folder: Please select RPE's folder
  search: Search
  export: Export
  export-success: Exported successfully
  delete: Delete
  delete-chart: Delete Chart
  cancel: Cancel
  delete-confirm: Are you sure you want to delete this chart?
  delete-confirm-info: will be lost forever! (A long time!)
  delete-autosave: Delete Autosave
  delete-autosave-confirm: Are you sure you want to delete this chart's autosave file?
  delete-success: Deleted successfully
  output-folder: Please select output folder
  show-folder: Open Folder
  sort: Sort
  sort-option-list: Modified Time,Name,Charter,ID

  render: Render

zh-CN:
  not-binded: 你还没有绑定 RPE
  bind: 绑定 RPE
  binded: 绑定成功
  unbind: 解绑 RPE
  unbinded: 解绑成功
  rpe-folder: 请选择 RPE 所在文件夹
  search: 搜索
  export: 导出
  export-success: 导出成功
  delete: 删除
  delete-chart: 删除谱面
  cancel: 取消
  delete-confirm: 你确定要刪除这个谱面吗？
  delete-confirm-info: 将会永久消失！（真的很久！）
  delete-autosave: 删除自动保存
  delete-autosave-confirm: 确定要刪除这个谱面的自动保存文件吗？
  delete-success: 删除成功
  output-folder: 请选择输出文件夹
  show-folder: 打开文件夹
  sort: 排序
  sort-option-list: 修改时间,名称,谱师,ID

  render: 渲染

</i18n>

<script setup lang="ts">
import { computed, ref } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open, save, confirm, message } from '@tauri-apps/plugin-dialog';

import { toast, toastError } from './common';
import type { RPEChart } from './model';
import router from './router';

async function getRPECharts() {
  return (await invoke('get_rpe_charts')) as RPEChart[] | null;
}
const charts = ref(await getRPECharts());
const searchQuery = ref('');
const sortOptionList = t('sort-option-list').split(',');
const sortOption = ref(sortOptionList[0]);
const filteredCharts = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return charts.value;

  return charts.value?.filter(chart =>
    chart.name.toLowerCase().includes(query) ||
    chart.charter.toLowerCase().includes(query) ||
    chart.id.toString().includes(query)
  );
});

async function sortCharts() {
  charts.value = await getRPECharts();
  if (sortOption.value === sortOptionList[1]) {
    charts.value?.sort((a, b) => a.name.localeCompare(b.name));
  } else if (sortOption.value === sortOptionList[2]) {
    charts.value?.sort((a, b) => a.charter.localeCompare(b.charter));
  } else if (sortOption.value === sortOptionList[3]) {
    charts.value?.sort((a, b) => parseInt(a.id) - parseInt(b.id));
  }
}


async function bindRPE() {
  let file = await open({ directory: true, title: t('rpe-folder') });
  if (!file) return;
  try {
    await invoke('set_rpe_dir', { path: file, save: true });
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

const moreLoading = ref(false);
async function exportPez(chartPath: string, chartName: string) {
  moreLoading.value = true;
  try {
    const outputName = chartName.replace(/[\\/:*?"<>|]/g, "_") + '.pez';
    const outputPath = await save({ title: t('output-folder'), filters: [{ name: 'RPE Chart File', extensions: ['pez', 'zip'] }], defaultPath: outputName });
    if (!outputPath) return;
    await invoke('export_pez', { chartPath, outputPath });
    message(t('export-success'), { title: t('export') })
  } catch (e) {
    toastError(e);
  } finally {
    moreLoading.value = false;
  }
}

async function deleteChart(chartName: string, chartPath: string) {
  confirm(`"${chartName}" ${t('delete-confirm-info')}`, { title: t('delete-confirm'), kind: 'warning' })
    .then(async (result) => {
      if (!result) return;
      try {
        moreLoading.value = true;
        await invoke('delete_path', { path: chartPath });
        message(t('delete-success'), { title: t('delete-chart') })
      } catch (e) {
        toastError(e);
      } finally {
        moreLoading.value = false;
        charts.value = await getRPECharts();
      }
    })
    .catch((e) => {
      toast(`${t('delete-cancel')}: ${e}`, 'info');
    })
}

async function deleteAutoSave(chartName: string, chartPath: string) {
  confirm(`"${chartName}" ${t('delete-autosave-confirm')}`, { title: t('delete-autosave'), kind: 'warning' })
    .then(async (result) => {
      if (!result) return;
      try {
        moreLoading.value = true;
        await invoke('delete_autosave', { path: chartPath });
        message(t('delete-success'), { title: t('delete-chart') })
      } catch (e) {
        toastError(e);
      } finally {
        moreLoading.value = false;
      }
    })
    .catch((e) => {
      toast(`${t('delete-cancel')}: ${e}`, 'info');
    })
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
          <v-col cols="8" style="margin: -10px 0px 5px 0px;">
            <v-text-field clearable class="ml-2 justify-center hover-scale-text" prepend-inner-icon="mdi-magnify" :label="t('search')" v-model="searchQuery"></v-text-field>
            <!-- <v-btn size="large" class="italic v-btn hover-scale" @click="unbindRPE" style="width: fit-content" v-t="'unbind'"></v-btn> -->
          </v-col>
          <v-col cols="4" style="margin: -10px 0px 5px 0px;">
            <v-select class="mr-2 hover-scale-text" :items="sortOptionList" v-model="sortOption" :label="t('sort')" append-icon="mdi-sort" @click:append="sortCharts"></v-select>
          </v-col>
        </v-row>
      </v-form>

      <v-lazy v-for="(chart, index) in filteredCharts" :key="chart.id" :min-height="150"> <!--transition="fade-transition"-->
        <v-card class="chart-card">
          <div class="d-flex flex-row align-stretch">
            <div class="d-flex flex-row align-center chart-cover" style="width: 35%">
              <div
                class="cover-image"
                style="width: 100%; height: 100%; max-height: 240px; background-position: center; background-repeat: no-repeat; background-size: cover"
                :style="{ 'background-image': 'url(' + convertFileSrc(chart.illustration) + ')' }"
              >
                <div 
                  class="overlay"
                  @click="router.push({ name: 'render', query: { chart: chart.path } })"
                >
                  <i class="mdi mdi-play icon"></i>
                </div>
              </div>
            </div>
            <div class="d-flex flex-column w-100 chart-content">
              <v-card-title class="chart-name select">{{ chart.name }}</v-card-title>
              <v-card-subtitle class="mt-n2 chart-id select">{{ chart.id }}</v-card-subtitle>
              <v-card-subtitle class="chart-id select">{{ chart.charter }}</v-card-subtitle>
              <div class="w-100 mt-2">
                <div class="pt-4 d-flex justify-end">
                  <v-menu>
                    <template v-slot:activator="{ props }">
                      <v-btn class="open-btn mx-2" v-bind="props" :loading="moreLoading">
                        <i class="mdi mdi-cog" />
                      </v-btn>
                    </template>
                    <v-list>
                      <v-list-item @click="exportPez(chart.path, chart.name)" v-t="'export'" />
                      <v-list-item @click="openInFolder(chart.path)" v-t="'show-folder'" />
                      <v-list-item @click="deleteChart(chart.name, chart.path)" v-t="'delete-chart'" />
                      <v-list-item @click="deleteAutoSave(chart.name, chart.path)" v-t="'delete-autosave'" />
                    </v-list>
                  </v-menu>
                  <v-btn class="render-btn mx-2" @click="router.push({ name: 'render', query: { chart: chart.path } })" v-t="'render'" />
                </div>
              </div>
            </div>
          </div>
        </v-card>
      </v-lazy>
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