import './assets/main.css';
import '@mdi/font/css/materialdesignicons.css';

import { createApp } from 'vue';
import { createI18n } from 'vue-i18n';

import { changeLocale } from './common';

import App from './App.vue';
import router from './router';

import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { aliases, mdi } from 'vuetify/iconsets/mdi';

import 'vuetify/styles';

import { Dark, DeepDark, Light, LightBlue } from './theme';

import mainEn from './locales/en/main.json';
import aboutEn from './locales/en/about.json';
import appEn from './locales/en/app.json';
import batchEn from './locales/en/batch.json';
import configEn from './locales/en/config.json';
import renderEn from './locales/en/render.json';
import rpeEn from './locales/en/rpe.json';
import settingEn from './locales/en/setting.json';
import taskEn from './locales/en/task.json';

import mainZhCN from './locales/zh-CN/main.json';
import aboutzhCN from './locales/zh-CN/about.json';
import appzhCN from './locales/zh-CN/app.json';
import batchzhCN from './locales/zh-CN/batch.json';
import configzhCN from './locales/zh-CN/config.json';
import renderzhCN from './locales/zh-CN/render.json';
import rpezhCN from './locales/zh-CN/rpe.json';
import settingzhCN from './locales/zh-CN/setting.json';
import taskzhCN from './locales/zh-CN/task.json';

export const SUPPORTED_LOCALES: string[] = ['en', 'zh-CN', 'zh-TW'];
export const SUPPORTED_LOCALES_NAME: { name: string, code: string }[] = [
  { code: "en", name: "English" },
  { code: "zh-CN", name: "简体中文" },
  // { code: "zh-TW", name: "繁體中文" }, // TODO
];

let locale = localStorage.getItem('locale');
if (!locale) {
  locale = 'en';
  for (const alt of navigator.languages) {
    if (SUPPORTED_LOCALES.includes(alt)) {
      locale = alt;
      break;
    }
  }
}
const i18n = createI18n({
  locale: 'en',
  fallbackLocale: 'en',
  messages: {
    'en': {
      ...mainEn,
      ...aboutEn,
      ...appEn,
      ...batchEn,
      ...configEn,
      ...renderEn,
      ...rpeEn,
      ...settingEn,
      ...taskEn,
    },
    'zh-CN': {
      ...mainZhCN,
      ...aboutzhCN,
      ...appzhCN,
      ...batchzhCN,
      ...configzhCN,
      ...renderzhCN,
      ...rpezhCN,
      ...settingzhCN,
      ...taskzhCN,
    },
  },
  legacy: false,
  missing(_locale, key) {
    if (key.startsWith('title-')) return '';
    return key;
  },
});
changeLocale(locale);

const useSystemTheme = localStorage.getItem('useSystemTheme') === 'true';
const savedTheme = useSystemTheme
  ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'DeepDark' : 'LightBlue')
  : (localStorage.getItem('theme') ?? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'DeepDark' : 'LightBlue'));
localStorage.setItem('theme', savedTheme);

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: savedTheme,
    themes: {
      Dark,
      DeepDark,
      Light,
      LightBlue,
    },
  },
  icons: {
    defaultSet: 'mdi',
    aliases,
    sets: {
      mdi,
    },
  },
});

const app = createApp(App);
app.use(i18n).use(router).use(vuetify);

app.mount('#app');

export { i18n };
