import { spawnSync } from 'node:child_process';
import { readFile, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { plainTerminalOutput } from '../src/scripts/terminal-preview.ts';
import { catalog } from './previews/catalog.mjs';
import { fixtures } from './previews/fixtures.mjs';

export function mergeArguments(...layers) {
  const groups = new Map();
  for (const args of layers) {
    for (let index = 0; index < args.length; index++) {
      const flag = args[index];
      if (!flag.startsWith('--')) throw new Error(`Expected an option: ${flag}`);
      const group = [flag];
      if (args[index + 1] !== undefined && !args[index + 1].startsWith('--')) group.push(args[++index]);
      groups.set(flag, group);
    }
  }
  return [...groups.values()].flat();
}

export function previewEnvironment(environment) {
  const colorVariables = new Set(['NO_COLOR', 'FORCE_COLOR', 'CLICOLOR', 'CLICOLOR_FORCE']);
  return {
    ...Object.fromEntries(Object.entries(environment).filter(([key]) => !key.startsWith('MDV_') && !colorVariables.has(key))),
    TERM: 'xterm-256color', COLORTERM: 'truecolor',
  };
}

async function capturePreviews(binary, check) {
  const project = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
  const environment = previewEnvironment(process.env);
  const execute = (args, input = '') => {
    const result = spawnSync(binary, args, { input, encoding: 'utf8', env: environment, maxBuffer: 4 * 1024 * 1024 });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`${args.join(' ')}: ${result.stderr}`);
    return result.stdout.replaceAll('\r\n', '\n');
  };
  const build = execute(['--version']).trim();
  const version = build.match(/Version:\s*([^\s]+)/)?.[1];
  const cargo = await readFile(join(project, 'Cargo.toml'), 'utf8');
  if (version !== cargo.match(/^version\s*=\s*"([^"]+)"/m)[1]) throw new Error('Build mdv from the current project before capturing previews.');
  const scratch = await mkdtemp(join(tmpdir(), 'mdv-preview-'));
  try {
    const previews = {};
    for (const entry of catalog) {
      const localized = {};
      for (const locale of ['en', 'ru']) {
        const source = fixtures(locale)[entry.fixture] + '\n';
        const variants = entry.variants.map((variant) => {
          const baseline = entry.id === 'preset' ? ['--cols', '64'] : ['--theme', 'catppuccin', '--cols', '64'];
          const args = mergeArguments(baseline, entry.base ?? [], variant.args);
          const captureArgs = mergeArguments(['--color', 'always', '--color-depth', 'truecolor'], args);
          const ansi = execute(['--no-config', '--config-file', scratch, ...captureArgs, '-'], source);
          const plain = plainTerminalOutput(ansi);
          const quote = (value) => /^[a-zA-Z0-9._/-]+$/.test(value) ? value : `'${value.replaceAll("'", "'\\''")}'`;
          return {
            label: variant.label[locale], args, ansi,
            command: `mdv --no-config ${args.map(quote).join(' ')} example.md`,
            rows: plain.trimEnd().split('\n').length,
            columns: Math.max(...plain.split('\n').map((line) => [...line].length)),
          };
        });
        localized[locale] = { source, variants };
      }
      previews[entry.id] = { title: entry.title, option: entry.option, ...(entry.description ? { description: entry.description } : {}), ...localized };
      const unique = new Set(localized.en.variants.map((variant) => variant.ansi));
      console.log(`${entry.id}: ${entry.variants.length} choices, ${unique.size} outputs`);
    }
    const destination = join(project, 'website/src/data/render-previews.json');
    const result = { schema: 1, version, build, previews };
    if (check) {
      const saved = JSON.parse(await readFile(destination, 'utf8'));
      if (JSON.stringify(saved) !== JSON.stringify(result)) throw new Error('Documentation previews differ from current mdv output. Regenerate the snapshot.');
      console.log('Every captured variant matches current mdv output in both languages.');
    } else {
      await writeFile(destination, JSON.stringify(result, null, 2) + '\n');
    }
  } finally {
    if (dirname(resolve(scratch)) !== resolve(tmpdir()) || !basename(scratch).startsWith('mdv-preview-')) throw new Error('Unexpected temporary directory.');
    await rm(scratch, { recursive: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const project = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
  const args = process.argv.slice(2).filter((arg) => arg !== '--check');
  if (args.length > 1) throw new Error('Expected an optional mdv binary path and --check.');
  const binary = args[0] ? resolve(args[0]) : join(project, 'target/release', process.platform === 'win32' ? 'mdv.exe' : 'mdv');
  await capturePreviews(binary, process.argv.includes('--check'));
}
