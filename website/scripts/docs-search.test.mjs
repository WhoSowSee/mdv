import { expect, test } from 'bun:test';
import { createSatteriMarkdownProcessor } from '@astrojs/markdown-satteri';
import blocksPlugin from './docs-search-blocks.mjs';
import pixelCodeBlocks from './pixel-code-blocks.mjs';
import scrollTables from './scroll-tables.mjs';
import { searchDocumentation, searchSnippet, searchTextMatches, primarySearchMatch, searchResultUrl, documentationSearchDestination } from '../src/scripts/docs-search-model';

const processor = await createSatteriMarkdownProcessor({
  shikiConfig: { theme: 'catppuccin-mocha' }, hastPlugins: [pixelCodeBlocks, scrollTables, blocksPlugin],
});
const source = '## Built-in themes\n\nChoose **kanagawa** or `nord`.\n\n- One option.\n  - Nested option.\n\n| Name | Kind |\n| --- | --- |\n| kanagawa | dark |\n\n<details>\n<summary>Full template</summary>\n\n```yaml\ntheme: kanagawa\npager:\n  transparent: true\n```\n\n</details>\n';

test('indexes rendered blocks with stable targets while preserving normal heading anchors', async () => {
  const result = await processor.render(source);
  const blocks = result.metadata.frontmatter.searchBlocks;
  expect(result.metadata.headings[0].slug).toBe('built-in-themes');
  expect(blocks.find((block) => block.kind === 'paragraph').text).toBe('Choose kanagawa or nord.');
  expect(blocks.find((block) => block.kind === 'code').text).toBe('theme: kanagawa pager: transparent: true');
  expect(blocks.some((block) => block.kind === 'table' && block.text === 'kanagawa dark')).toBe(true);
  expect(new Set(blocks.map((block) => block.id)).size).toBe(blocks.length);
  for (const block of blocks) expect(result.code).toContain(`data-doc-search-id="${block.id}"`);
  const changed = await processor.render('A new introduction.\n\n' + source);
  expect(changed.metadata.frontmatter.searchBlocks.slice(1).map((block) => block.id)).toEqual(blocks.map((block) => block.id));
});

test('returns the matching block and excerpt instead of a fixed page description', async () => {
  const { metadata } = await processor.render(source);
  const pages = [{ title: 'Themes', url: '/mdv/docs/themes/', blocks: metadata.frontmatter.searchBlocks }];
  const result = searchDocumentation(pages, 'kanag')[0];
  expect(result.title).toBe('Themes');
  expect(result.snippet.toLowerCase()).toContain('kanagawa');
  expect(result.block.section).toBe('Built-in themes');
  expect(result.block.kind).not.toBe('heading');
  expect(searchDocumentation(pages, 'KANAG')).toEqual(searchDocumentation(pages, 'kanag'));
  expect(searchDocumentation(pages, 'not-present')).toEqual([]);
});

test('chooses context around a precise match in long configuration blocks', () => {
  const text = `${'An unrelated configuration property. '.repeat(80)}theme: kanagawa ${'Other fields. '.repeat(50)}`;
  const excerpt = searchSnippet(text, 'kanag');
  expect(excerpt).toContain('theme: kanagawa');
  expect(excerpt.length).toBeLessThan(200);
  expect(excerpt).toStartWith('…');
  expect(primarySearchMatch('theme theme theme separate kanagawa', 'theme kanag').start).toBe(27);
  expect(searchSnippet('x'.repeat(1000) + 'kanagawa', 'kanag').length).toBeLessThan(200);
});

test('matches literal flags, Unicode text and query punctuation without regex interpretation', () => {
  expect(searchTextMatches('Use [MODE] and --color-depth', '[MODE]')).toEqual([{ start: 4, end: 10 }]);
  expect(searchTextMatches('ТЕМА: канagawa', 'тема')).toEqual([{ start: 0, end: 4 }]);
  expect(searchTextMatches('<script>alert(1)</script>', '<script>')[0]).toEqual({ start: 0, end: 8 });
});

test('builds a search destination without a native hash jump and validates its target', () => {
  const id = 'doc-search-0123456789abcdef-1';
  const result = searchResultUrl('/mdv/ru/docs/themes/', id, '  kanag  ', 'https://example.org/mdv/ru/docs/');
  expect(result.hash).toBe('');
  expect(result.searchParams.get('q')).toBe('kanag');
  expect(documentationSearchDestination(result.href, 'https://example.org/mdv/ru/docs/').id).toBe(id);
  expect(documentationSearchDestination('https://elsewhere.org/?q=test&match=' + id, result.href)).toBeUndefined();
  expect(documentationSearchDestination('?q=test&match=invalid', result.href)).toBeUndefined();
  expect(documentationSearchDestination(result.href + '#ordinary-section', result.href)).toBeUndefined();
});
