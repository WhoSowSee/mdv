import { describe, expect, test } from 'bun:test';
import { latestPackageRelease, packageRelease } from './update-release.mjs';

function releaseFixture(tag = 'v6.1.0') {
  return {
    tag_name: tag, draft: false, prerelease: false,
    assets: ['x86_64-unknown-linux-gnu.rpm', 'aarch64-unknown-linux-gnu.rpm', 'x86_64-unknown-linux-gnu.pkg.tar.zst'].map((suffix) => ({
      name: `mdv-${tag}-${suffix}`,
      browser_download_url: `https://github.com/WhoSowSee/mdv/releases/download/${tag}/mdv-${tag}-${suffix}`,
    })),
  };
}

describe('Package release metadata', () => {
  test('selects the exact published packages for the latest version', () => {
    const result = packageRelease(releaseFixture('v7.2.3'));
    expect(result.tag).toBe('v7.2.3');
    expect(result.packages.arch.filename).toBe('mdv-v7.2.3-x86_64-unknown-linux-gnu.pkg.tar.zst');
    expect(result.packages.rpmArm64.url).toEndWith('/mdv-v7.2.3-aarch64-unknown-linux-gnu.rpm');
  });

  test('rejects absent and duplicate assets', () => {
    const missing = releaseFixture();
    missing.assets.pop();
    expect(() => packageRelease(missing)).toThrow('Expected one release asset');
    const duplicate = releaseFixture();
    duplicate.assets.push(duplicate.assets[0]);
    expect(() => packageRelease(duplicate)).toThrow('Expected one release asset');
  });

  test('rejects unsafe tags, download origins and unpublished releases', () => {
    for (const tag of ['../main', 'v6.1.0/other', 'latest']) {
      expect(() => packageRelease(releaseFixture(tag))).toThrow('invalid version tag');
    }
    const unexpected = releaseFixture();
    unexpected.assets[0].browser_download_url = 'https://example.com/package.rpm';
    expect(() => packageRelease(unexpected)).toThrow('Unexpected download URL');
    expect(() => packageRelease({ ...releaseFixture(), draft: true })).toThrow('published stable release');
    expect(() => packageRelease({ ...releaseFixture(), prerelease: true })).toThrow('published stable release');
  });

  test('does not publish stale commands when GitHub fails', async () => {
    await expect(latestPackageRelease(async () => new Response('', { status: 403 }))).rejects.toThrow('HTTP 403');
    await expect(latestPackageRelease(async () => { throw new Error('Network unavailable'); })).rejects.toThrow('Network unavailable');
  });

  test('reads release data from the GitHub latest-release API', async () => {
    const result = await latestPackageRelease(async (url, options) => {
      expect(url).toBe('https://api.github.com/repos/WhoSowSee/mdv/releases/latest');
      expect(options.headers.Accept).toBe('application/vnd.github+json');
      return Response.json(releaseFixture());
    });
    expect(result.tag).toBe('v6.1.0');
  });
});
