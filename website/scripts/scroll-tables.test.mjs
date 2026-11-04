import { expect, test } from 'bun:test';
import { createSatteriMarkdownProcessor } from '@astrojs/markdown-satteri';
import scrollTables from './scroll-tables.mjs';

async function render(markdown, plugins) {
  const processor = await createSatteriMarkdownProcessor({ hastPlugins: plugins });
  return processor.render(markdown);
}

test('makes wide Markdown tables keyboard-accessible without changing their content or alignment', async () => {
  const source = '## Settings\n\n| Key | Default |\n| :--- | ---: |\n| `theme` | [terminal](../themes/) |';
  const [before, after] = await Promise.all([render(source, []), render(source, [scrollTables])]);
  expect(after.code).toContain('class="table-frame" tabindex="0" role="region" aria-label="Key, Default"');
  expect(after.code.replace(/<div class="table-frame"[^>]*>/, '').replace('</table></div>', '</table>')).toBe(before.code);
  expect(after.metadata.headings).toEqual(before.metadata.headings);
});

test('wraps nested tables once and leaves ordinary inline content unchanged', async () => {
  const source = '> | Name | Value |\n> | --- | --- |\n> | width | 80 |\n\nUse `width`.';
  const result = await render(source, [scrollTables, scrollTables]);
  expect(result.code.match(/class="table-frame"/g)).toHaveLength(1);
  expect(result.code).toContain('<p>Use <code>width</code>.</p>');
});
