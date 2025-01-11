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
import { VStepper, VStepperActions, VStepperHeader, VStepperItem } from 'vuetify/labs/VStepper';

import 'vuetify/styles';

import theme from './theme';

export const SUPPORTED_LOCALES = ['en', 'zh-CN', 'zh-TW'];

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
        'non-empty': 'Must not be empty',
        'positive': 'Must be a positive number',
        'positive-int': 'Must be a positive integer',
        'resolution': "Must be like '1920x1080'",
        'sample-count': 'Must be a power of 2',
        'int': 'Must be a integer',
        'crf': 'Must be an integer between 0 and 51',
        'bitrate': 'Must be a valid bitrate',
      },
      'has-error': 'There are errors in the configuration',
      'any-filter': 'All files',
    },
    'zh-CN': {
      rules: {
        'non-empty': '不能为空',
        'positive': '必须是正数',
        'positive-int': '必须是正整数',
        'resolution': "必须为 '宽x高'",
        'sample-count': '必须是 2 的幂',
        'int': '必须是整数',
        'crf': '必须是 0 到 51 之间的整数',
        'bitrate': '必须是有效的码率',
      },
      'has-error': '配置中有错误',
      'any-filter': '所有文件',
    },
  },
  legacy: false,
  missing(_locale, key) {
    if (key.startsWith('title-')) return '';
    return key;
  },
});
changeLocale(locale);

const vuetify = createVuetify({
  components: {
    VStepper,
    VStepperActions,
    VStepperHeader,
    VStepperItem,
    ...components,
  },
  directives,
  theme: {
    defaultTheme: 'customTheme',
    themes: {
      customTheme: theme,
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
