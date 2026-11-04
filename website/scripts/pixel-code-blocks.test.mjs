import { describe, expect, test } from 'bun:test';
import { createSatteriMarkdownProcessor } from '@astrojs/markdown-satteri';
import pixelCodeBlocks from './pixel-code-blocks.mjs';

async function render(markdown, plugins = [pixelCodeBlocks]) {
  const processor = await createSatteriMarkdownProcessor({
    shikiConfig: { theme: 'catppuccin-mocha' }, hastPlugins: plugins,
  });
  return processor.render(markdown);
}

describe('Pixel code block frames', () => {
  test('preserves highlighted code, whitespace, accessibility and heading metadata', async () => {
    const markdown = '# Example\n\n```yaml\ntheme: catppuccin\n  nested: true\n```\n';
    const [before, after] = await Promise.all([render(markdown, []), render(markdown)]);
    expect(after.code.replaceAll('<div class="code-frame">', '').replaceAll('</pre></div>', '</pre>')).toBe(before.code);
    expect(after.code).toContain('tabindex="0"');
    expect(after.metadata.headings).toEqual(before.metadata.headings);
  });

  test('frames code blocks nested in quotes and lists', async () => {
    const markdown = '> ```sh\n> mdv README.md\n> ```\n\n- Item\n\n  ```sh\n  mdv --monitor notes.md\n  ```';
    const result = await render(markdown);
    expect(result.code.match(/class="code-frame"/g)).toHaveLength(2);
    expect(result.code.match(/<pre\b/g)).toHaveLength(2);
  });

  test('keeps inline code and surrounding content unchanged', async () => {
    const markdown = 'Use `--monitor` with a file.\n\n## Next\n\n[Docs](./usage/)';
    const [before, after] = await Promise.all([render(markdown, []), render(markdown)]);
    expect(after).toEqual(before);
  });

  test('does not nest duplicate frames on repeated transforms', async () => {
    const result = await render('```sh\nmdv README.md\n```', [pixelCodeBlocks, pixelCodeBlocks]);
    expect(result.code.match(/class="code-frame"/g)).toHaveLength(1);
  });
});
