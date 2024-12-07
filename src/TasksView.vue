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

  details: Details
  error: Error
  output: Output

  show-output: Show Output
  show-in-folder: Show in Folder

zh-CN:
  empty: 空空如也

  status:
    pending: 等待中…
    loading: 加载中…
    mixing: 混音中…
    rendering: 渲染中（{ progress }%），{ fps } FPS，预计 { estimate } 结束
    done: 已完成，耗时 { duration }
    canceled: 已取消
    failed: 失败

  cancel: 取消
  confirm: 确定

  details: 详情
  error: 错误
  output: 输出

  show-output: 查看输出
  show-in-folder: 在文件夹中显示

</i18n>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';

import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import type { Task, TaskStatus } from './model';

import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';

import moment from 'moment';
import { toastError } from './common';

const tasks = ref<Task[]>();

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
  console.log(tasks.value[0]);
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));

function describeStatus(status: TaskStatus): string {
  switch (status.type) {
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
        estimate: status.estimate ? moment.duration(Math.ceil(status.estimate), 'seconds').humanize(true, { ss: 0, s: 60, m: 60 }) : '',
      });
    case 'done':
      return t('status.done', {
        duration: moment.duration(Math.ceil(status.duration), 'seconds').humanize(false, { ss: 0, s: 60, m: 60 }),
      });
    case 'canceled':
      return t('status.canceled');
    case 'failed':
      return t('status.failed');
  }
}

const errorDialog = ref(false),
  errorDialogMessage = ref('');

const outputDialog = ref(false),
  outputDialogMessage = ref('');

async function showInFolder(path: string) {
  try {
    await invoke('show_in_folder', { path });
  } catch (e) {
    toastError(e);
  }
}

async function showFolder() {
  try {
    await invoke('show_folder');
  } catch (e) {
    toastError(e);
  }
}
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column" style="max-width: 1280px; gap: 1rem">
    <v-btn variant="text" @click="showFolder()" v-t="'show-in-folder'"></v-btn>
    <h1 v-if="!tasks || !tasks.length" class="text-center font-italic text-disabled" v-t="'empty'"></h1>
    <v-card v-for="task in tasks" :key="task.id">
      <div class="d-flex flex-row align-stretch">
        <div class="d-flex flex-row align-center" style="width: 35%">
          <div
            style="width: 100%; height: 100%; max-height: 240px; background-position: center; background-repeat: no-repeat; background-size: cover"
            :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }"></div>
        </div>
        <div class="d-flex flex-column w-100">
          <v-card-title>{{ task.name }}</v-card-title>
          <v-card-subtitle class="mt-n2">{{ task.path }}</v-card-subtitle>
          <div class="w-100 pa-4 pb-2 pr-2 mt-2">
            <p class="mb-2 text-medium-emphasis">{{ describeStatus(task.status) }}</p>
            <template v-if="['loading', 'mixing', 'rendering'].includes(task.status.type)">
              <v-progress-linear
                :indeterminate="task.status.type !== 'rendering'"
                :model-value="task.status.type === 'rendering' ? task.status.progress * 100 : 0"></v-progress-linear>
              <div class="pt-4 d-flex justify-end">
                <v-btn variant="text" @click="invoke('cancel_task', { id: task.id })" v-t="'cancel'"></v-btn>
              </div>
            </template>
            <div v-if="task.status.type === 'failed'" class="pt-4 d-flex justify-end">
              <v-btn
                variant="text"
                @click="
                  () => {
                    if (task.status.type === 'failed') {
                      errorDialogMessage = task.status.error;
                      errorDialog = true;
                    }
                  }
                "
                v-t="'details'"></v-btn>
            </div>
            <div v-if="task.status.type === 'done'" class="pt-4 d-flex justify-end">
              <v-btn
                variant="text"
                @click="
                  () => {
                    if (task.status.type === 'done') {
                      outputDialogMessage = task.status.output;
                      outputDialog = true;
                    }
                  }
                "
                v-t="'show-output'"></v-btn>
              <v-btn variant="text" @click="showInFolder(task.output)" v-t="'show-in-folder'"></v-btn>
            </div>
          </div>
        </div>
      </div>
    </v-card>

    <v-dialog v-model="errorDialog" width="auto" min-width="400px">
      <v-card>
        <v-card-title v-t="'error'"> </v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ errorDialogMessage }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="text" @click="errorDialog = false" v-t="'confirm'"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="outputDialog" width="auto" min-width="400px">
      <v-card>
        <v-card-title v-t="'output'"> </v-card-title>
        <v-card-text>
          <pre class="block whitespace-pre overflow-auto" style="max-height: 60vh">{{ outputDialogMessage }}</pre>
        </v-card-text>
        <v-card-actions class="justify-end">
          <v-btn color="primary" variant="text" @click="outputDialog = false" v-t="'confirm'"></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>
