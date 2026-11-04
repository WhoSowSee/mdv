import { describe, expect, test } from 'bun:test';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import snapshot from '../src/data/render-previews.json';
import symbols from '../src/data/preview-symbols.json';
import { plainTerminalOutput, renderTerminalOutput } from '../src/scripts/terminal-preview';
import { mergeArguments, previewEnvironment } from './capture-previews.mjs';
import { catalog } from './previews/catalog.mjs';
import { fixtures } from './previews/fixtures.mjs';

describe('Documentation rendering previews', () => {
  test('captures every choice for both languages without machine-specific arguments', async () => {
    const cargo = await readFile(resolve(import.meta.dir, '../../Cargo.toml'), 'utf8');
    expect(snapshot.version).toBe(cargo.match(/^version\s*=\s*"([^"]+)"/m)[1]);
    expect(Object.keys(snapshot.previews).sort()).toEqual(catalog.map((entry) => entry.id).sort());
    for (const entry of catalog) {
      const preview = snapshot.previews[entry.id];
      expect(preview.option).toBe(entry.option);
      for (const locale of ['en', 'ru']) {
        const { source, variants } = preview[locale];
        expect(source).toBe(fixtures(locale)[entry.fixture] + '\n');
        expect(variants).toHaveLength(entry.variants.length);
        expect(new Set(variants.map((variant) => variant.ansi)).size, entry.id).toBe(variants.length);
        for (const variant of variants) {
          expect(variant.command).toStartWith('mdv --no-config ');
          expect(variant.command).toEndWith(' example.md');
          expect(variant.command).not.toContain('--config-file');
          expect(variant.ansi.length).toBeGreaterThan(0);
          expect(variant.rows).toBeGreaterThan(0);
          expect(variant.columns).toBeGreaterThan(0);
        }
      }
    }
  });

  test('places every preview in its CLI entry and the matching translated guide', async () => {
    for (const locale of ['en', 'ru']) {
      const folder = resolve(import.meta.dir, '../src/content/docs', locale === 'ru' ? 'ru' : '');
      const cli = await readFile(resolve(folder, 'cli.md'), 'utf8');
      for (const entry of catalog) {
        const marker = `<div data-mdv-preview="${entry.id}"></div>`;
        const section = [...cli.matchAll(/<section class="cli-option" aria-label="([^"]+)">([\s\S]*?)<\/section>/g)]
          .find((match) => match[1] === entry.option)[2];
        expect(section, entry.id).toContain(marker);
        expect(section).toContain('<details class="mdv-reference-preview">');
        if (entry.heading === null) continue;
        const guide = await readFile(resolve(folder, entry.page + '.md'), 'utf8');
        const headings = guide.split(/(?=^## )/m).slice(1);
        expect(headings[entry.heading], `${locale}: ${entry.id}`).toContain(marker);
        expect(guide.split(marker)).toHaveLength(2);
      }
    }
  });

  test('names tabs with CLI syntax or consistent localized states', () => {
    for (const entry of catalog) {
      for (const locale of ['en', 'ru']) {
        const variants = snapshot.previews[entry.id][locale].variants;
        expect(variants.map((variant) => variant.label)).toEqual(entry.variants.map((variant) => variant.label[locale]));
        const states = locale === 'en' ? ['Default', 'Off', 'On'] : ['По умолчанию', 'Выкл.', 'Вкл.'];
        for (const variant of variants) {
          if (states.includes(variant.label)) continue;
          expect(variant.args.some((argument) => argument === variant.label
            || argument.endsWith(`:${variant.label}`) || argument.endsWith(`;${variant.label}`)), `${entry.id}: ${variant.label}`).toBe(true);
        }
      }
    }
  });

  test('replaces only the specified capture arguments and preserves compound values', () => {
    expect(mergeArguments(['--cols', '64', '--wrap', 'char'], ['--cols', '40', '--table-borders'], ['--wrap', 'word']))
      .toEqual(['--cols', '40', '--wrap', 'word', '--table-borders']);
    expect(mergeArguments(['--line-numbers', 'source;separator'], ['--line-numbers']))
      .toEqual(['--line-numbers']);
    expect(() => mergeArguments(['README.md'])).toThrow('Expected an option');
  });

  test('isolates captured colors and configuration from the calling terminal', () => {
    expect(previewEnvironment({ NO_COLOR: '1', FORCE_COLOR: '0', CLICOLOR: '0', CLICOLOR_FORCE: '0', TERM: 'dumb', COLORTERM: '', MDV_THEME: 'nord', PATH: 'tools' }))
      .toEqual({ PATH: 'tools', TERM: 'xterm-256color', COLORTERM: 'truecolor' });
    const table = snapshot.previews['table-borders'].en.variants[1].ansi;
    expect(table).toContain('\x1b[38;2;137;180;250m');
  });

  test('uses the marker code points from mdv without swapping small and large sets', async () => {
    const source = await readFile(resolve(import.meta.dir, '../../src/list_marker.rs'), 'utf8');
    for (const [index, size] of [[2, 'SMALL'], [3, 'LARGE']]) {
      const line = source.split('\n').find((line) => line.startsWith(`const NERD_FONT_${size}_ICONS`));
      const icons = [...line.matchAll(/\\u\{([a-f0-9]+)\}/g)].map((match) => parseInt(match[1], 16));
      for (const locale of ['en', 'ru']) {
        const output = plainTerminalOutput(snapshot.previews['list-style'][locale].variants[index].ansi);
        for (const icon of icons) expect(output).toContain(String.fromCodePoint(icon));
      }
    }
    for (const [large, small] of [[0xf444, 0xf09de], [0xf445, 0xf0a13], [0xf4c3, 0xf14dc], [0xf51d, 0xf0a14]]) {
      const a = symbols.metrics[large].bounds, b = symbols.metrics[small].bounds;
      expect(a[3] - a[1]).toBeGreaterThan(b[3] - b[1]);
    }
    expect(symbols.sources.unicode.codepoints).toContain(0x1f836);
    for (const locale of ['en', 'ru']) {
      expect(plainTerminalOutput(snapshot.previews['definition-marker-style'][locale].variants[1].ansi)).toContain('🠶 ');
    }
  });

  test('retains the full-size Rust icon with padding cells on either side', () => {
    const rust = String.fromCodePoint(0xe68b);
    const bounds = symbols.metrics[0xe68b].bounds;
    expect(bounds[3] - bounds[1]).toBeGreaterThan(870);
    expect(symbols.sources.nerd.wideCodepoints).toContain(0xe68b);
    let occurrences = 0;
    for (const preview of Object.values(snapshot.previews)) {
      for (const locale of ['en', 'ru']) {
        for (const variant of preview[locale].variants) {
          const output = plainTerminalOutput(variant.ansi);
          for (const match of output.matchAll(new RegExp(rust, 'g'))) {
            expect(output[match.index - 1]).toBe(' ');
            expect(output[match.index + rust.length]).toBe(' ');
            occurrences++;
          }
        }
      }
    }
    expect(occurrences).toBeGreaterThan(0);
  });

  test('preserves the source stroke widths of vertical terminal borders', () => {
    for (const [codepoint, width] of [[0x2502, 100], [0x2503, 200], [0x2506, 100]]) {
      const [left, , right] = symbols.metrics[codepoint].bounds;
      expect(right - left).toBe(width);
    }
    for (const locale of ['en', 'ru']) {
      expect(plainTerminalOutput(snapshot.previews['callout-style'][locale].variants[1].ansi)).toStartWith('┃ ');
    }
  });

  test('retains full-size callout icons with space before and after them', () => {
    for (const codepoint of [0xf040, 0xef76]) {
      const icon = String.fromCodePoint(codepoint);
      const [left, bottom, right, top] = symbols.metrics[codepoint].bounds;
      expect(top - bottom).toBeGreaterThan(920);
      expect(right - left).toBeGreaterThan(690);
      expect(symbols.sources.nerd.wideCodepoints).toContain(codepoint);
      for (const locale of ['en', 'ru']) {
        const output = plainTerminalOutput(snapshot.previews['callout-icons'][locale].variants[2].ansi);
        expect(output).toContain(` ${icon} `);
      }
    }
  });

  test('escapes document text and OSC link destinations', () => {
    const html = renderTerminalOutput('<img src=x> & "test"');
    expect(html).not.toContain('<img');
    expect(html).toContain('&lt;');
    expect(html).toContain('&amp;');
    expect(html).toContain('&quot;');
    expect(renderTerminalOutput('\x1b]8;;javascript:alert(1)\x1b\\unsafe\x1b]8;;\x1b\\')).not.toContain('<a ');
    expect(renderTerminalOutput('\x1b]8;;https://example.org/?q="x"\x07link\x1b]8;;\x07')).toContain('href="https://example.org/?q=&quot;x&quot;"');
  });

  test('supports color depths, style resets and incomplete color sequences', () => {
    expect(renderTerminalOutput('\x1b[31mred')).toContain('color:#800000');
    expect(renderTerminalOutput('\x1b[38;5;123mcolor')).toContain('color:rgb(135,255,255)');
    expect(renderTerminalOutput('\x1b[48;2;11;16;23mbackground')).toContain('background-color:rgb(11,16,23)');
    const html = renderTerminalOutput('\x1b[1;3;4;38;2;137;180;250mblue\x1b[0mplain');
    expect(html).toContain('font-weight:700;font-style:italic;text-decoration:underline;color:rgb(137,180,250)');
    expect(html).toEndWith('</span>plain');
    expect(renderTerminalOutput('\x1b[38;2;10mtext')).not.toContain('rgb(');
    expect(renderTerminalOutput('\x1b[4;9mstyled\x1b[24mstrike')).toContain('text-decoration:underline line-through');
    expect(renderTerminalOutput('\x1b[4;9mstyled\x1b[24mstrike')).toContain('style="text-decoration:line-through"');
    expect(plainTerminalOutput('\x1b[31mA\x1b[0m\n\x1b]8;;https://example.org\x07B\x1b]8;;\x07')).toBe('A\nB');
  });

  test('keeps terminal cell contents and line breaks intact in every captured output', () => {
    const decode = (html) => html.replace(/<[^>]*>/g, '').replaceAll('&lt;', '<').replaceAll('&gt;', '>')
      .replaceAll('&quot;', '"').replaceAll('&#39;', "'").replaceAll('&amp;', '&');
    for (const preview of Object.values(snapshot.previews)) {
      for (const locale of ['en', 'ru']) {
        for (const variant of preview[locale].variants) {
          expect(decode(renderTerminalOutput(variant.ansi))).toBe(plainTerminalOutput(variant.ansi));
        }
      }
    }
    expect(renderTerminalOutput('A─B')).toBe('A─B');
    expect(symbols.codepoints).toContain(0x2500);
    expect(symbols.codepoints.some((codepoint) => codepoint >= 0xf0000)).toBe(true);
  });

  test('renders selectable text runs without individual character boxes', () => {
    expect(renderTerminalOutput('fn main() {\n    println!("hello, mdv");\n}'))
      .toBe('fn main() {\n    println!(&quot;hello, mdv&quot;);\n}');
    expect(renderTerminalOutput('\x1b[31mred\x1b[0m plain'))
      .toBe('<span style="color:#800000">red</span> plain');
    for (const preview of Object.values(snapshot.previews)) {
      for (const locale of ['en', 'ru']) {
        for (const variant of preview[locale].variants) {
          expect(renderTerminalOutput(variant.ansi)).not.toContain('mdv-preview-cell');
        }
      }
    }
  });
});
