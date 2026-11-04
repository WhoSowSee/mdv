import { links } from '../data/site';
import { en } from './en';
import { ru } from './ru';

export type Locale = 'en' | 'ru';
const base = import.meta.env.BASE_URL.replace(/\/$/, '');

export function getLocale(pathname: string): Locale {
  return pathname === `${base}/ru` || pathname.startsWith(`${base}/ru/`) ? 'ru' : 'en';
}

export function localizedPath(path: string, locale: Locale) {
  return `${base}${locale === 'ru' ? '/ru' : ''}${path}`;
}

export function alternatePath(pathname: string, locale: Locale) {
  const path = pathname.slice(base.length).replace(/^\/ru(?=\/|$)/, '') || '/';
  return localizedPath(path, locale);
}

export function siteLinks(locale: Locale) {
  return { ...links, home: localizedPath('/', locale), docs: localizedPath('/docs/', locale) };
}

export function getCopy(locale: Locale) {
  return { en, ru }[locale];
}
