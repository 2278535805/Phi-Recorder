import { nextTick } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';

import AboutView from '../AboutView.vue';
import RenderView from '../RenderView.vue';
import RPEView from '../RPEView.vue';
import TasksView from '../TasksView.vue';

import { useOnLoaded } from '../App.vue';

const onLoaded = useOnLoaded();

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'render', component: RenderView },
    { path: '/rpe', name: 'rpe', component: RPEView },
    { path: '/tasks', name: 'tasks', component: TasksView },
    { path: '/about', name: 'about', component: AboutView },
  ],
  scrollBehavior(_to, from, savedPosition) {
    return new Promise((resolve) => {
      onLoaded.value = () => {
        resolve(savedPosition ? savedPosition : { top: 0 });
      };
    });
  },
});

import { i18n } from '../main';
import { setTitle } from '../common';

router.afterEach((to) => {
  nextTick(() => {
    const title = i18n.global.t('title-' + ((to.name as string | undefined | null) || 'default'));
    setTitle(title);
  });
});

export default router;
