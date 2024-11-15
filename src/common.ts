import { toast as sonnerToast } from 'vuetify-sonner';

import { SUPPORTED_LOCALES, i18n } from './main';

import moment from 'moment';

import 'moment/dist/locale/zh-cn';
import 'moment/dist/locale/zh-hk';

export function anyFilter() {
  return {
    name: i18n.global.t('any-filter'),
    extensions: ['*'],
  };
}

export function isString(s: unknown): s is string {
  return typeof s === 'string';
}

export const RULES = {
  non_empty: (value: string) => value.trim().length > 0 || i18n.global.t('rules.non-empty'),
  positive: (value: string) => (isNumeric(value) && Number(value) >= 0) || i18n.global.t('rules.positive'),
  positiveInt: (value: string) => (isNumeric(value) && Math.abs(Number(value) - Math.round(Number(value))) < 1e-4 && Number(value) > 0) || i18n.global.t('rules.positive-int'),
};

export function isNumeric(num: any) {
  return (typeof num === 'number' || (typeof num === 'string' && num.trim() !== '')) && !isNaN(num as number);
}

export function setTitle(title: string) {
  document.title = title.length ? title + ' - Phigros' : 'Phigros';
}

export function changeLocale(locale: string) {
  if (locale.startsWith('en')) locale = 'en';
  if (!SUPPORTED_LOCALES.includes(locale)) locale = 'en';
  i18n.global.locale.value = (locale === 'zh-TW' ? 'zh-CN' : locale) as typeof i18n.global.locale.value;
  localStorage.setItem('locale', locale);
  const momentLocale =
    {
      'zh-CN': 'zh-cn',
      'zh-TW': 'zh-hk',
      en: 'en-us',
    }[locale] ?? 'en-us';
  moment.locale(momentLocale);
}

export function toast(message: string, kind?: 'success' | 'info' | 'warning' | 'error') {
  sonnerToast(message, {
    duration: 2000,
    cardProps: {
      color: kind,
      // @ts-ignore
      style: 'width: var(--width)',
    },
  });
}

export function toastError(error: any) {
  console.error(error);
  const msg = error instanceof Error ? error.message : String(error);
  if (msg.length) toast(msg, 'error');
}
