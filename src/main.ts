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
    en: {
      rules: {
        'not-empty': 'Must not be empty',
        'not-null': 'Must not be empty',
        'positive': 'Must be a positive number',
        'positive-int': 'Must be a positive integer',
        'resolution': "Must be like '1920x1080'",
        'sample-count': 'Must be a power of 2',
        'int': 'Must be a integer',
        'crf': 'Must be an integer between 0 and 51',
        'bitrate': 'Must be a valid bitrate',
        'non-spaces': 'Must not contain spaces',
        'not-zero': 'Must not be zero',
        'not-combo': 'Must not be COMBO',
        'long': 'Input is too long',
        'big': 'Input is too big',
        'invalid-path': 'Invalid path',
      },
      'has-error': 'There are errors in the configuration',
      'any-filter': 'All files',
      'theme': {
        'theme': 'Theme',
        'dark': 'Dark - Azure Whisper',
        'light': 'Light - Frost Glow',
        'deep-dark': 'Dark - Obsidian Glow',
        'light-blue': 'Light - Crystal Luminous'
      }
    },
    'zh-CN': {
      rules: {
        'not-empty': '不能为空',
        'not-null': '不能为空',
        'positive': '必须是正数',
        'positive-int': '必须是正整数',
        'resolution': "必须为 '宽x高'",
        'sample-count': '必须是 2 的幂',
        'int': '必须是整数',
        'crf': '必须是 0 到 51 之间的整数',
        'bitrate': '必须是有效的码率',
        'non-spaces': '不能包含空格',
        'not-zero': '不能为零',
        'not-combo': '不能为 COMBO',
        'long': '输入内容过长',
        'big': '输入内容过大',
        'invalid-path': '路径不合法',
      },
      'has-error': '配置中有错误',
      'any-filter': '所有文件',
      'theme': {
        'theme': '主题',
        'dark': '深色 - 浅澜微蓝',
        'light': '浅色 - 冰雾微光',
        'deep-dark': '深色 - 黑曜光辉',
        'light-blue': '浅色 - 璃光素澈'
      }
    },
  },
  legacy: false,
  missing(_locale, key) {
    if (key.startsWith('title-')) return '';
    return key;
  },
});
changeLocale(locale);

const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'Dark' : 'Light';
const savedTheme = localStorage.getItem("theme") || systemTheme;

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
