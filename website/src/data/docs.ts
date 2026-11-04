import type { MarkdownInstance } from 'astro';
import { siteLinks, type Locale } from '../i18n';
import type { SearchBlock, SearchPage } from '../scripts/docs-search-model';

export const documentationGroups = ['Getting started', 'Reading', 'Formatting', 'Customization', 'Reference'] as const;

interface DocFrontmatter {
  title: string;
  description: string;
  slug: string;
  group: (typeof documentationGroups)[number];
  order: number;
  searchBlocks: SearchBlock[];
}

export type DocPage = MarkdownInstance<DocFrontmatter>;

const documentationByLocale = {
  en: Object.values(import.meta.glob<DocPage>('../content/docs/*.md', { eager: true })),
  ru: Object.values(import.meta.glob<DocPage>('../content/docs/ru/*.md', { eager: true })),
};

export function getDocumentation(locale: Locale) {
  return [...documentationByLocale[locale]].sort((left, right) => left.frontmatter.order - right.frontmatter.order);
}

export function documentationUrl(slug: string, locale: Locale) {
  const { docs } = siteLinks(locale);
  return slug ? `${docs}${slug}/` : docs;
}

export function documentationSearchIndex(locale: Locale): SearchPage[] {
  return getDocumentation(locale).map((page) => ({
    title: page.frontmatter.title,
    url: documentationUrl(page.frontmatter.slug, locale),
    blocks: [
      { id: 'doc-search-title', section: '', text: page.frontmatter.title, kind: 'page' },
      { id: 'doc-search-description', section: '', text: page.frontmatter.description, kind: 'page' },
      ...page.frontmatter.searchBlocks,
    ],
  }));
}
