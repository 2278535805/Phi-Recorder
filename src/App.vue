<i18n>
en:
  render: Render
  render1: Fast Render
  rpe: RPE
  tasks: Tasks
  about: About

zh-CN:
  render: 渲染
  render1: 快速渲染
  rpe: RPE
  tasks: 任务列表
  about: 关于

</i18n>

<script lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { useI18n } from 'vue-i18n';

import { VSonner } from 'vuetify-sonner';

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
const { t } = useI18n();

const route = useRoute(),
  router = useRouter();

const icons = {
  render: 'mdi-auto-fix',
  render1: 'mdi-auto-fix',
  rpe: 'mdi-bookshelf',
  tasks: 'mdi-server',
  about: 'mdi-information-outline',
};

window.goto = (name: string) => {
  router.push({ name });
};
</script>

<template>
  <v-app id="phigros-recorder">
    <v-sonner position="top-center" />
    <v-app-bar :elevation="0">
      <!--<v-app-bar-nav-icon @click="toggleNav" class="mx-1"></v-app-bar-nav-icon>-->
      <v-app-bar-title class="mx-5">Phigros Recorder</v-app-bar-title>
    </v-app-bar>
    <v-navigation-drawer v-model="drawer" expand-on-hover rail permanent>
      <v-list density="compact" nav>
        <v-list-item
          v-for="key in ['render', 'render1', 'rpe', 'tasks', 'about']"
          :active="route.name === key"
          :key="key"
          :prepend-icon="icons[key as keyof typeof icons]"
          :title="t(key)"
          @click="router.push({ name: key })"></v-list-item>
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
              <v-progress-circular indeterminate size="large" />
            </div>
          </template>
        </Suspense>
      </router-view>
    </v-main>
  </v-app>
</template>
