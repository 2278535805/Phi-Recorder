<i18n>
en:
  empty: Nothing here

  status:
    pending: Pending…
    loading: Loading…
    mixing: Mixing…
    rendering: Rendering ({ progress }%), { fps } FPS, estimated to end { estimate }
    done: Done, took { duration }
    canceled: Canceled
    failed: Failed

  cancel: Cancel
  confirm: Confirm
  close: Close

  details: Details
  output: Output
  remove-task: Remove Task
  remove-task-confirm: Confirm to remove task?

  show-output: Show Output
  show-folder: Open Output Folder
  show-in-folder: Show in Folder
  open-file: Open File
  duration: 
    hours: h
    minutes: m
    seconds: s

zh-CN:
  empty: 空空如也

  status:
    pending: 等待中…
    loading: 加载中…
    mixing: 混音中…
    rendering: 渲染中 ({ progress }%), { fps } FPS, 预计 { estimate } 结束
    done: 已完成，耗时 { duration }
    canceled: 已取消
    failed: 失败

  cancel: 取消
  confirm: 确定
  close: 关闭

  details: 详情
  output: 输出
  remove-task: 删除任务
  remove-task-confirm: 确认删除任务?

  show-output: 查看输出
  show-folder: 打开输出文件夹
  show-in-folder: 在文件夹中显示
  open-file: 打开文件
  
  duration: 
    hours: 时
    minutes: 分
    seconds: 秒
  
</i18n>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import type { Task, TaskStatus } from './model';

import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

import moment from 'moment';
import { toastError } from './common';

import router from './router';

const tasks = ref<Task[]>();

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
  //console.log(tasks.value[0]);
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));


function formatDuration(seconds: number) {
  const duration = moment.duration(Math.ceil(seconds), 'seconds');
  const hours = Math.floor(duration.asHours());
  const minutes = duration.minutes();
  const secs = duration.seconds();

  if (hours > 0) {
    return `${hours} ${t('duration.hours')} ${minutes} ${t('duration.minutes')} ${secs} ${t('duration.seconds')}`;
  } else if (minutes > 0) {
    return `${minutes} ${t('duration.minutes')} ${secs} ${t('duration.seconds')}`;
  } else if (secs > 0) {
    return `${secs} ${t('duration.seconds')}`;
  } else {
    return '';
  }
}


function describeStatus(status: TaskStatus): string {
  switch (status.type) {
    case 'null':
      return ''
    case 'pending':
      return t('status.pending');
    case 'loading':
      return t('status.loading');
    case 'mixing':
      return t('status.mixing');
    case 'rendering':
      return t('status.rendering', {
        progress: (status.progress * 100).toFixed(2),
        fps: status.fps,
        estimate: status.estimate ? formatDuration(status.estimate) : '',// status.estimate ? moment.duration(Math.ceil(status.estimate), 'seconds').humanize(true, { ss: 0, s: 120, m: 120, h: 120 })
      });
    case 'done':
      return t('status.done', {
        duration: status.duration ? formatDuration(status.duration) : '',
      });
    case 'canceled':
      return t('status.canceled');
    case 'failed':
      return t('status.failed');
  }
}

const outputDialog = ref(false),
  outputDialogMessage = ref('');

async function showInFolder(path: string) {
  try {
    await invoke('show_in_folder', { path });
  } catch (e) {
    toastError(e);
  }
}

async function openFile(path: string) {
  try {
    await invoke('open_file', { path });
  } catch (e) {
    toastError(e);
  }
}

async function showOutputFolder() {
  try {
    await invoke('open_output_folder');
  } catch (e) {
    toastError(e);
  }
}

const removeDialog = ref(false);
const removeTaskIndex = ref(0);
function fromBackIndex(length: number, indexFromBack: number): number | null {
    if (indexFromBack >= length) return null;
    return length - indexFromBack - 1;
}
function removeTask(index: number) {
  removeDialog.value = false;
  let realIndex = fromBackIndex(tasks.value!.length, index);
  invoke('remove_task', { index: realIndex })
    .catch((e) => {
      toastError(e);
    });
}
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1rem">
    <v-form class="text-center fade-in" ref="form" style="max-height: 48vh;">
      <v-row>
        <v-col cols="12" style="margin: -20px 0px;">
          <v-btn size="large" class="hover-scale margin-btn" @click="showOutputFolder()" v-t="'show-folder'"></v-btn>
        </v-col>
      </v-row>
    </v-form>
    <h1 v-if="!tasks || !tasks.length" class="text-center font-italic text-disabled fade-in" v-t="'empty'"></h1>
    <v-lazy v-for="(task, index) in tasks" :key="task.id" :min-height="150" transition="fade-transition">
      <v-card class="task-card">
        <div class="d-flex flex-row align-stretch">
          <div class="d-flex flex-row align-center img-cover" style="width: 30%">
            <div
              style="width: 100%; height: 100%; max-height: 240px; background-position: center; background-repeat: no-repeat; background-size: cover"
              :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }"
            >
              <div 
                class="overlay"
                @click="router.push({ name: 'render', query: { chart: task.path, info: JSON.stringify(task.info), config: JSON.stringify(task.config) } })"
              >
                <i class="mdi mdi-reload icon"></i>
              </div>
            </div>
          </div>
          <div class="d-flex flex-column w-100 name-cover">
            <v-card-title class="select">{{ task.name }}</v-card-title>
            <v-card-subtitle class="mt-n2 select">{{ task.path }}</v-card-subtitle>
            <div class="w-100 pa-4 pb-2 pr-2 mt-2">
              <p class="mb-2 text-medium-emphasis">{{ describeStatus(task.status) }}</p>
              <template v-if="['loading', 'mixing', 'rendering'].includes(task.status.type)">
                <v-progress-linear
                  v-if="task.status.type !== 'rendering'"
                  :indeterminate="true"
                  class="glow-spinner"
                ></v-progress-linear>
                <v-progress-linear
                  v-else
                  :model-value="task.status.progress * 100"
                  rounded
                ></v-progress-linear>
              </template>
              <div class="pt-4 d-flex justify-end" v-if="['pending','loading', 'mixing', 'rendering'].includes(task.status.type)">
                <v-btn class="hover-scale"
                  prepend-icon="mdi-cancel"
                  variant="text"
                  @click="invoke('cancel_task', { id: task.id })"
                  v-t="'cancel'"></v-btn>
              </div>
              <div v-if="task.status.type === 'failed' || task.status.type === 'canceled'" class="pt-4 d-flex justify-end">
                <v-btn
                  variant="flat"
                  @click="removeDialog = true; removeTaskIndex =index"
                  v-t="'remove-task'"
                  class="hover-scale"></v-btn>
                <v-btn
                  variant="flat"
                  prepend-icon="mdi-alert-circle-outline"
                  @click="() => {
                      if (task.status.type === 'failed' || task.status.type === 'canceled') {
                        outputDialogMessage = task.status.output;
                        outputDialog = true;
                      }
                    }"
                  @contextmenu="removeDialog = true; removeTaskIndex = task.id"
                  v-t="'details'"
                  class="hover-scale"></v-btn>
              </div>
              <div v-if="task.status.type === 'done'" class="pt-4 d-flex justify-end">
                <v-btn variant="text" @click="openFile(task.output)" v-t="'open-file'" class="hover-scale"></v-btn>
                <v-btn 
                  variant="flat"
                  prepend-icon="mdi-folder-open-outline" 
                  @click="showInFolder(task.output)"
                  v-t="'show-in-folder'"
                  class="hover-scale"></v-btn>
                <v-btn
                  variant="flat"
                  prepend-icon="mdi-text-box-outline"
                  @click="
                    () => {
                      if (task.status.type === 'done') {
                        outputDialogMessage = task.status.output;
                        outputDialog = true;
                      }
                    }
                  "
                  v-t="'show-output'"
                  class="hover-scale"></v-btn>
              </div>
            </div>
          </div>
        </div>
      </v-card>
    </v-lazy>

    <v-dialog v-model="outputDialog" width="auto" min-width="400px" class="log-card-bg">
      <v-card class="log-card-window">
        <v-card-title v-t="'output'"> </v-card-title>
        <v-card-text>
          <div class="block whitespace-pre overflow-auto log-card-msg" style="max-height: 60vh">{{ outputDialogMessage }}</div>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn class="hover-scale" variant="text" @click="outputDialog = false" v-t="'close'"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="removeDialog" width="auto" min-width="400px" class="log-card-bg">
      <v-card class="log-card-only-window">
        <v-card-title v-t="'remove-task'"> </v-card-title>
        <v-card-text>
          <div class="block whitespace-pre overflow-auto">{{ t('remove-task-confirm') }}</div>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn class="hover-scale" variant="text" @click="removeDialog = false" v-t="'cancel'"></v-btn>
          <v-btn class="hover-scale" variant="text" @click="removeTask(removeTaskIndex)" v-t="'confirm'"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>


.task-card {
  border-radius: 16px !important;
  background: rgba(var(--v-theme-container), 0.04);
  margin: 5px;
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0px 0px 12px rgba(0, 0, 0, 0.05);
  animation: fadeUp 0.5s cubic-bezier(0, 0, 0, 1) forwards;
  opacity: 0; /* 初始状态透明 */
}

.task-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 0px 24px rgba(0, 0, 0, 0.2) !important;
  background: rgba(var(--v-theme-container), 0.02);
}

.margin-btn {
  margin-bottom: 14px !important;
}

.glass-background {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.text-gradient {
  background: linear-gradient(45deg, #2196F3, #E91E63);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

pre {
  background: rgba(0, 0, 0, 0.3) !important;
  padding: 16px !important;
  border-radius: 8px;
  font-family: 'Fira Code', monospace;
}

.animated-form {
  transition: opacity 0.1s ease, transform 0.1s ease;
}

.v-slide-y-transition-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}

.v-btn {
  background: rgba(127, 127, 127, 0.10);
  padding: 8px 14px;
  margin: 4px 8px;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 4px 4px 6px rgba(0, 0, 0, 0.1);
}

.icon {
  font-size: 250%;
}

@media (max-width: 600px) {
  .img-cover {
    min-width: none;
    max-width: 0%;
  }

  .name-cover {
    min-width: 100%;
    max-width: none;
  }
}

@media (min-width: 601px) and (max-width: 1065px) {
  .img-cover {
    min-width: 30%;
    max-width: none;
  }

  .name-cover {
    min-width: none;
    max-width: 70%;
  }
}

@media (min-width: 1065px) {
  .img-cover {
    min-width: 280px;
    max-width: none;
  }

  .name-cover {
    min-width: none;
    max-width: none;
  }
}

</style>