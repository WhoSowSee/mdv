import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const website = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const arguments_ = process.argv.slice(2);
for (const [command, script] of [[process.execPath, 'scripts/capture-previews.mjs'], ['python', 'scripts/capture-interactive-previews.py']]) {
  const result = spawnSync(command, [script, ...arguments_], { cwd: website, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
