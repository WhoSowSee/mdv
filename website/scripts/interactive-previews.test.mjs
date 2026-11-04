import { describe, expect, test } from 'bun:test';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import snapshot from '../src/data/interactive-previews.json';
import rendering from '../src/data/render-previews.json';
import symbols from '../src/data/preview-symbols.json';
import { plainTerminalOutput, renderTerminalOutput } from '../src/scripts/terminal-preview';

describe('Interactive documentation previews', () => {
  test('keeps punctuation attached to words in pager preview paragraphs', () => {
    for (const locale of ['en', 'ru']) {
      for (const variant of snapshot.previews['builtin-pager'][locale].variants) {
        for (const line of plainTerminalOutput(variant.ansi).split('\n')) {
          const text = line.includes('│') ? line.slice(line.indexOf('│') + 1) : line;
          expect(text.trim(), `${locale}: ${variant.label}`).not.toMatch(/^[.,;:!?]+$/);
        }
      }
    }
  });

  test('covers printable ASCII and every captured character with the terminal font', () => {
    const coverage = new Set(symbols.textCodepoints);
    for (let code = 32; code < 127; code++) expect(coverage.has(code)).toBe(true);
    for (const preview of Object.values(snapshot.previews)) {
      for (const locale of ['en', 'ru']) {
        for (const variant of preview[locale].variants) {
          const characters = new Set([...plainTerminalOutput(variant.ansi)].filter((c) => c !== '\n'));
          for (const character of characters) expect(coverage.has(character.codePointAt(0))).toBe(true);
        }
      }
    }
  });

  test('preserves a continuous cell background across rows and clears it after SGR reset', () => {
    const html = renderTerminalOutput('\x1b[48;2;36;36;36mHelp\nNext row\x1b[49mplain');
    expect(html).toContain('--terminal-background:rgb(36,36,36)');
    expect(html).toContain('Help\nNext row</span>plain');
    expect(html.match(/--terminal-background/g)).toHaveLength(1);
  });

  test('stores distinct complete terminal screens in both languages', () => {
    expect(snapshot.version).toBe(rendering.version);
    for (const [id, count] of [['document-browser', 3], ['builtin-pager', 7]]) {
      for (const locale of ['en', 'ru']) {
        const { source, variants } = snapshot.previews[id][locale];
        expect(source.length).toBeGreaterThan(0);
        expect(variants).toHaveLength(count);
        expect(new Set(variants.map(({ ansi }) => ansi)).size).toBe(count);
        for (const variant of variants) {
          const lines = plainTerminalOutput(variant.ansi).split('\n');
          expect(lines).toHaveLength(variant.rows);
          expect(lines.every((line) => [...line].length === variant.columns)).toBe(true);
          expect(variant.ansi.replace(/\x1b\[[0-9;]*m/g, '')).not.toContain('\x1b');
          expect(variant.ansi).not.toMatch(/[A-Z]:\\|mdv-interactive-preview-|Users\//);
          expect(variant.command).toStartWith('mdv --no-config --theme catppuccin ');
          expect(variant.command).not.toContain('--config-file');
          expect(variant.description.length).toBeGreaterThan(0);
          expect(renderTerminalOutput(variant.ansi)).not.toContain('\x1b');
        }
      }
    }
  });

  test('captures the requested browser and pager modes rather than ordinary output', () => {
    for (const locale of ['en', 'ru']) {
      const browser = snapshot.previews['document-browser'][locale].variants;
      expect(plainTerminalOutput(browser[0].ansi)).toContain('8 documents');
      expect(plainTerminalOutput(browser[1].ansi)).toContain('Find: install');
      expect(plainTerminalOutput(browser[1].ansi)).toContain('guides/installation.md');
      expect(plainTerminalOutput(browser[2].ansi)).toContain('close help');
      const pager = snapshot.previews['builtin-pager'][locale].variants;
      expect(plainTerminalOutput(pager[1].ansi)).toContain('Find: mdv');
      expect(plainTerminalOutput(pager[2].ansi)).toContain('01');
      expect(plainTerminalOutput(pager[2].ansi)).toContain('│');
      expect(renderTerminalOutput(pager[3].ansi)).toContain('background-color:');
      expect(pager[3].ansi).not.toBe(pager[0].ansi);
      expect(plainTerminalOutput(pager[4].ansi)).toMatch(/\n\s*1\s/);
      expect(plainTerminalOutput(pager[5].ansi)).toContain(':16');
      expect(plainTerminalOutput(pager[6].ansi)).toContain('close help');
    }
  });

  test('places each preview in the translated guide and corresponding CLI entry', async () => {
    for (const locale of ['en', 'ru']) {
      const folder = resolve(import.meta.dir, '../src/content/docs', locale === 'ru' ? 'ru' : '');
      const cli = await readFile(resolve(folder, 'cli.md'), 'utf8');
      for (const [id, page, flag] of [['document-browser', 'browser', '--interactive'], ['builtin-pager', 'pager', '--pager']]) {
        const marker = `<div data-mdv-preview="${id}"></div>`;
        const guide = await readFile(resolve(folder, page + '.md'), 'utf8');
        expect(guide.split(marker)).toHaveLength(2);
        const section = [...cli.matchAll(/<section class="cli-option" aria-label="([^"]+)">([\s\S]*?)<\/section>/g)]
          .find((match) => match[1] === flag)[2];
        expect(section).toContain(marker);
        expect(section).toContain('<details class="mdv-reference-preview">');
      }
    }
  });
});
