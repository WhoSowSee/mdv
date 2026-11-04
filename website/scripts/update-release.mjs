import { writeFile } from 'node:fs/promises';

const repository = 'WhoSowSee/mdv';
const apiUrl = `https://api.github.com/repos/${repository}/releases/latest`;
const requiredPackages = {
  rpm: 'x86_64-unknown-linux-gnu.rpm',
  rpmArm64: 'aarch64-unknown-linux-gnu.rpm',
  arch: 'x86_64-unknown-linux-gnu.pkg.tar.zst',
};

export function packageRelease(release) {
  const tag = release?.tag_name;
  if (typeof tag !== 'string' || !/^v?\d+\.\d+\.\d+(?:[.-][A-Za-z0-9.-]+)?$/.test(tag)) {
    throw new Error('The latest GitHub release has an invalid version tag');
  }
  if (release.draft !== false || release.prerelease !== false || !Array.isArray(release.assets)) {
    throw new Error('A published stable release with package assets is required');
  }

  const packages = {};
  for (const [key, suffix] of Object.entries(requiredPackages)) {
    const filename = `mdv-${tag}-${suffix}`;
    const matches = release.assets.filter((asset) => asset.name === filename);
    if (matches.length !== 1) throw new Error(`Expected one release asset: ${filename}`);
    const url = `https://github.com/${repository}/releases/download/${tag}/${filename}`;
    if (matches[0].browser_download_url !== url) {
      throw new Error(`Unexpected download URL for ${filename}`);
    }
    packages[key] = { filename, url };
  }
  return { tag, packages };
}

export async function latestPackageRelease(request = fetch) {
  const headers = { Accept: 'application/vnd.github+json', 'User-Agent': 'mdv-website' };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  const response = await request(apiUrl, { headers, signal: AbortSignal.timeout(15000) });
  if (!response.ok) throw new Error(`GitHub release request failed: HTTP ${response.status}`);
  return packageRelease(await response.json());
}

if (import.meta.main) {
  const release = await latestPackageRelease();
  await writeFile(new URL('../src/data/package-release.json', import.meta.url), `${JSON.stringify(release, null, 2)}\n`);
  console.log(`Package installation commands: ${release.tag}`);
}
