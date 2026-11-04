import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';
import { satteri } from '@astrojs/markdown-satteri';
import pixelCodeBlocks from './scripts/pixel-code-blocks.mjs';
import scrollTables from './scripts/scroll-tables.mjs';
import docsSearchBlocks from './scripts/docs-search-blocks.mjs';

export default defineConfig({
  site: 'https://whosowsee.github.io',
  base: '/mdv',
  output: 'static',
  trailingSlash: 'always',
  i18n: { defaultLocale: 'en', locales: ['en', 'ru'], routing: { prefixDefaultLocale: false } },
  devToolbar: { enabled: false },
  markdown: { shikiConfig: { theme: 'catppuccin-mocha' }, processor: satteri({ hastPlugins: [pixelCodeBlocks, scrollTables, docsSearchBlocks] }) },
  vite: { plugins: [tailwindcss()] },
});
