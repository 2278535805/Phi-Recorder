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
  int: (value: string) => (isNumeric(value) && Math.abs(Number(value) - Math.round(Number(value))) < 1e-4) || i18n.global.t('rules.int'),
  // greaterThanZero: (value: string) => (isNumeric(value) && Number(value) >= 0) || i18n.global.t('rules.greater-than-zero'),
  crf: (value: string) => (Number.isInteger(Number(value)) && Number(value) >= 1 && Number(value) <= 51) || i18n.global.t('rules.crf'),
  bitrate: (value: string) => {
    if (!value || value.trim() === '') {
      return i18n.global.t('rules.bitrate');
    }
    const regex = /^(\d+)(Kbps|Mbps|K|M)$/i;
    const match = value.match(regex);
    if (!match) return i18n.global.t('rules.bitrate');
    
    const number = Number(match[1]);
    const unit = match[2].toLowerCase();
  
    if ((unit === 'kbps' || unit === 'k') && number > 0 && number <= 1000000) return true;
    if ((unit === 'mbps' || unit === 'm') && number > 0 && number <= 1000) return true;
  
    return i18n.global.t('rules.bitrate');
  },
  nonSpaces: (value: string) => !/\s/.test(value) || i18n.global.t('rules.non-spaces'),
  nonCOMBO: (value: string) => {
    const filteredValue = value.replace(/[^a-zA-Z0-9!"#$%&'()*+,\-./:;<=>?@[\\\]^_`{|}~ΜΟΒСՕ]/g, '').trim();
    if (value.length > 50) {
      return i18n.global.t('rules.long');
    }
    return !/^[CС][OՕΟ][MΜ][BΒ][OՕΟ]$/.test(filteredValue) || i18n.global.t('rules.combo');
  }
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
      style: 'width: var(--width)',
    },
  });
}

export function toastError(error: any) {
  console.error(error);
  const msg = error instanceof Error ? error.message : String(error);
  if (msg.length) toast(msg, 'error');
}
