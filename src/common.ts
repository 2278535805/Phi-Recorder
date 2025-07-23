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
  notEmpty: (value: string) => value.trim().length > 0 || i18n.global.t('rules.not-empty'),
  notNull: (value: string) => (value != null) || i18n.global.t('rules.not-null'),
  int: (value: string) => (isNumeric(value) && Math.abs(Number(value) - Math.round(Number(value))) < 1e-4) || i18n.global.t('rules.int'),
  positive: (value: string) => (isNumeric(value) && Number(value) > 0) || i18n.global.t('rules.positive'),
  positiveOrZero: (value: string) => (isNumeric(value) && Number(value) >= 0) || i18n.global.t('rules.positive'),
  less10000: (value: string) => {
    if (isNumeric(value)) {
      if (Number(value) > 100000) return i18n.global.t('rules.big');
    }
    return true;
  },
  less4000000000: (value: string) => {
    if (isNumeric(value)) {
      if (Number(value) > 4000000000) return i18n.global.t('rules.big');
    }
    return true;
  },
  nonSpaces: (value: string) => !/\s/.test(value) || i18n.global.t('rules.non-spaces'),
  notZero: (value: string) => (Number(value) != 0) || i18n.global.t('rules.not-zero'),
  notCOMBO: (value: string) => {
    const filteredValue = value.replace(/[^a-zA-Z0-9!"#$%&'()*+,\-./:;<=>?@\\\[\]^_`{|}~ΜΟΒСՕⅭОмвＣＯＭＢМⅯВ]/g, '').trim();
    if (value.length > 50) {
      return i18n.global.t('rules.long');
    }
    return !/^[CСⅭＣ][OՕΟ0ОＯ][MΜмＭМⅯ][BΒ8вＢВ][OՕΟ0ОＯ]$/.test(filteredValue) || i18n.global.t('rules.not-combo');
  },
};

export function isNumeric(num: any) {
  return (typeof num === 'number' || (typeof num === 'string' && num.trim() !== '')) && !isNaN(num as number);
}

export function setTitle(title: string) {
  document.title = title.length ? title + ' - Phi' : 'Phi';
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
      style: 'width: var(--width); user-select: text;',
    },
  });
}

export function toastError(error: any) {
  console.error(error);
  const msg = error instanceof Error ? error.message : String(error);
  if (msg.length) toast(msg, 'error');
}
