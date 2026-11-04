import { describe, expect, test } from 'bun:test';
import { readFile, readdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createSatteriMarkdownProcessor } from '@astrojs/markdown-satteri';
import docsSearchBlocks from './docs-search-blocks.mjs';
import { searchDocumentation } from '../src/scripts/docs-search-model';

const root = resolve(import.meta.dir, '../..');
const docsRoot = resolve(root, 'website/src/content/docs');
const groups = ['Getting started', 'Reading', 'Formatting', 'Customization', 'Reference'];
const processor = await createSatteriMarkdownProcessor({ shikiConfig: { theme: 'catppuccin-mocha' }, hastPlugins: [docsSearchBlocks] });

async function readPages(locale) {
  const directory = locale === 'en' ? docsRoot : resolve(docsRoot, 'ru');
  const files = (await readdir(directory)).filter((name) => name.endsWith('.md'));
  const pages = [];
  for (const file of files) {
    const source = (await readFile(resolve(directory, file), 'utf8')).replaceAll('\r\n', '\n');
    const [, frontmatter, body] = source.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
    const meta = Object.fromEntries(frontmatter.split('\n').map((line) => {
      const [, key, value] = line.match(/^(\w+):\s*(.*)$/);
      return [key, value.startsWith('"') ? JSON.parse(value) : value];
    }));
    const rendered = await processor.render(body);
    const path = `/mdv/${locale === 'ru' ? 'ru/' : ''}docs/${meta.slug ? `${meta.slug}/` : ''}`;
    pages.push({ source, body, meta, path, ...rendered });
  }
  return pages;
}

const catalog = { en: await readPages('en'), ru: await readPages('ru') };

describe('Complete website documentation', () => {
  test('indexes actual matching blocks in both languages with targets present in the rendered pages', () => {
    for (const pages of Object.values(catalog)) {
      for (const page of pages) {
        const blocks = page.metadata.frontmatter.searchBlocks;
        expect(blocks.length).toBeGreaterThan(0);
        expect(new Set(blocks.map((block) => block.id)).size).toBe(blocks.length);
        for (const block of blocks) expect(page.code).toContain(`data-doc-search-id="${block.id}"`);
      }
      const results = searchDocumentation(pages.map((page) => ({ title: page.meta.title, url: page.path, blocks: page.metadata.frontmatter.searchBlocks })), 'kanag');
      expect(results).toHaveLength(2);
      for (const result of results) expect(result.snippet).toContain('kanagawa');
      expect(results.some((result) => result.block.kind === 'code')).toBe(true);
    }
  });

  test('closes fenced examples and includes a custom-theme example', () => {
    for (const pages of Object.values(catalog)) {
      for (const page of pages) {
        let fence;
        for (const line of page.body.split('\n')) {
          const match = line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
          if (!match) continue;
          const [, marker, suffix] = match;
          if (!fence) fence = marker;
          else if (marker[0] === fence[0] && marker.length >= fence.length && !suffix.trim()) fence = undefined;
        }
        expect(fence, `${page.path}: unclosed code example`).toBeUndefined();
      }
      const guide = pages.find((page) => page.meta.slug === 'themes');
      expect(guide.body).toContain('name: warm');
    }
  });

  test('documents every CLI option declared by the implementation in both languages', async () => {
    const source = await readFile(resolve(root, 'src/cli.rs'), 'utf8');
    const flags = new Set([...source.matchAll(/long\s*=\s*"([a-z][a-z-]+)"/g)].map((match) => match[1]));
    const aliases = [...source.matchAll(/#\[arg\(([\s\S]*?)\)\]\s*pub\s+\w+/g)].flatMap((match) => {
      const short = match[1].match(/short\s*=\s*'([a-zA-Z])'/)?.[1];
      const long = match[1].match(/long\s*=\s*"([a-z][a-z-]+)"/)?.[1];
      return short && long ? [{ short, long }] : [];
    });
    flags.add('help');
    flags.add('version');
    for (const [locale, pages] of Object.entries(catalog)) {
      const reference = pages.find((page) => page.meta.slug === 'cli').body;
      const entries = [...reference.matchAll(/^### `([^`]+)`/gm)].map((match) => match[1]);
      for (const flag of flags) {
        expect(entries.some((entry) => new RegExp(`--${flag}(?![a-z-])`).test(entry)), `${locale}: --${flag}`).toBe(true);
      }
      for (const { short, long } of aliases) expect(reference, `${locale}: -${short}`).toContain(`-${short}, --${long}`);
      expect(reference, locale).not.toContain('mdv [OPTIONS]');
      expect(reference, locale).not.toContain('cat <FILE> | mdv');
      const rendered = pages.find((page) => page.meta.slug === 'cli');
      expect(rendered.code.match(/<section class="cli-option"/g), locale).toHaveLength(entries.length);
      expect(rendered.metadata.headings.filter((heading) => heading.depth === 3), locale).toHaveLength(entries.length);
    }
  });

  test('includes every YAML key and the unchanged canonical configuration template', async () => {
    const template = (await readFile(resolve(root, 'docs/examples/config.yaml'), 'utf8')).replaceAll('\r\n', '\n').trim();
    const defaults = [...template.matchAll(/^([a-z_]+):[ \t]+([^#\n]+)/gm)].map((match) => [match[1], match[2].trim()]);
    for (const [locale, pages] of Object.entries(catalog)) {
      const reference = pages.find((page) => page.meta.slug === 'configuration-reference').body;
      expect(reference, locale).toContain(`\n\`\`\`yaml\n${template}\n\`\`\``);
      const rows = new Map([...reference.matchAll(/^\| `([a-z_]+)` \| `([^`]+)` \|/gm)].map((match) => [match[1], match[2]]));
      expect(rows.size, locale).toBe(defaults.length);
      for (const [key, value] of defaults) expect(rows.get(key), `${locale}: ${key}`).toBe(value);
    }
  });

  test('keeps documented runtime minimums aligned with the manifest and installed Astro', async () => {
    const manifest = JSON.parse(await readFile(resolve(root, 'website/package.json'), 'utf8'));
    const astro = JSON.parse(await readFile(resolve(root, 'website/node_modules/astro/package.json'), 'utf8'));
    const bun = manifest.engines.bun.match(/^>=(\d+\.\d+\.\d+)$/)?.[1];
    const node = astro.engines.node.match(/^>=(\d+\.\d+\.\d+)$/)?.[1];
    expect(bun).toBeDefined();
    expect(node).toBeDefined();
    const readme = await readFile(resolve(root, 'website/README.md'), 'utf8');
    for (const pages of Object.values(catalog)) {
      const development = pages.find((page) => page.meta.slug === 'development').body;
      expect(development).toContain(`Bun ${bun}`);
      expect(development).toContain(`Node.js ${node}`);
    }
    expect(readme).toContain(`Bun ${bun}`);
    expect(readme).toContain(`Node.js ${node}`);
    for (const name of ['README.md', 'README-RU.md']) {
      expect(await readFile(resolve(root, 'website/scripts/previews', name), 'utf8')).toContain(`Bun ${bun}`);
    }
  });

  test('uses section-search examples whose text exists in the named README', async () => {
    for (const pages of Object.values(catalog)) {
      const usage = pages.find((page) => page.meta.slug === 'usage').body;
      const examples = [...usage.matchAll(/^mdv --from '([^']+)' (README(?:-RU)?\.md)$/gm)];
      expect(examples).toHaveLength(2);
      for (const [, query, file] of examples) {
        const text = await readFile(resolve(root, file), 'utf8');
        expect(text, `${file}: ${query}`).toContain(query.replace(/:\d+$/, ''));
      }
    }
  });

  test('keeps translations, navigation and heading-based language switching aligned', () => {
    expect(catalog.en.length).toBe(catalog.ru.length);
    for (const locale of ['en', 'ru']) {
      const pages = catalog[locale];
      for (const slug of ['', 'installation', 'usage', 'themes', 'presets', 'monitoring', 'browser', 'pager', 'rendering', 'configuration', 'cli']) {
        expect(pages.some((page) => page.meta.slug === slug), `${locale}: preserved ${slug}`).toBe(true);
      }
      expect(new Set(pages.map((page) => page.meta.slug)).size).toBe(pages.length);
      expect(new Set(pages.map((page) => page.meta.order)).size).toBe(pages.length);
      for (const page of pages) {
        expect(groups, page.path).toContain(page.meta.group);
        const alternate = catalog[locale === 'en' ? 'ru' : 'en'].find((entry) => entry.meta.slug === page.meta.slug);
        expect(alternate, page.path).toBeDefined();
        expect(alternate.meta.group, page.path).toBe(page.meta.group);
        expect(alternate.meta.order, page.path).toBe(page.meta.order);
        const outline = (entry) => entry.metadata.headings.filter((heading) => heading.depth === 2 || heading.depth === 3).map((heading) => heading.depth);
        expect(outline(alternate), page.path).toEqual(outline(page));
        expect(page.source, page.path).not.toContain('\u2014');
      }
    }
  });

  test('resolves documentation links and section anchors within the website', () => {
    const pagesByPath = new Map([...catalog.en, ...catalog.ru].map((page) => [page.path, page]));
    for (const page of pagesByPath.values()) {
      for (const [, href] of page.code.matchAll(/href="([^"]+)"/g)) {
        const target = new URL(href.replaceAll('&amp;', '&'), `https://example.test${page.path}`);
        if (target.origin !== 'https://example.test' || !/^\/mdv\/(ru\/)?docs\//.test(target.pathname)) continue;
        const linkedPage = pagesByPath.get(target.pathname);
        expect(linkedPage, `${page.path}: ${href}`).toBeDefined();
        if (target.hash) {
          expect(linkedPage.metadata.headings.map((heading) => heading.slug), `${page.path}: ${href}`).toContain(decodeURIComponent(target.hash.slice(1)));
        }
      }
    }
  });
});
