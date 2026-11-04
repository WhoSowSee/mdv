# mdv website

Static Astro site with TypeScript, Tailwind CSS and English/Russian documentation.

## Development

Requires Bun 1.4.4 or newer and Node.js 22.23.4 or newer on `PATH` for Astro.
The Bun minimum is declared in `package.json`; Astro declares its Node.js minimum.

```sh
cd website
bun install --frozen-lockfile
bun run dev
```

Open the printed URL under `/mdv/`. Run the remaining website commands from this directory.

## Checks and build

```sh
bun run test
bun run build
bun run preview
```

The build refreshes `src/data/package-release.json` from the latest stable GitHub
release, checks types and generates `dist/`. It requires GitHub API access and
fails if required package assets cannot be resolved. Set `GITHUB_TOKEN` when
needed. Development uses the stored release snapshot.

## Publishing

Set the repository's **Settings > Pages > Source** to **GitHub Actions** once.
[website.yml](../.github/workflows/website.yml) checks and builds the site for
pull requests targeting `main` and every push to `main`. Successful main builds
publish `dist/` to GitHub Pages. The site origin and `/mdv` base path are in
`astro.config.mjs`.

The workflow also refreshes the site after a successful tag-triggered
`Build binaries` run, updating package links from the published release.
It builds the default branch, not the tag. **Actions > Website > Run workflow**
can publish `main` manually; runs on other branches only check and build.
The built-in `GITHUB_TOKEN` supplies API access and Pages deployment permissions.

## Source files

- `src/content/docs/` contains English guides, with Russian translations in `ru/`. Update both languages together.
- `src/i18n/` contains translated interface text and locale-aware URLs.
- `src/data/site.ts` contains project links, theme samples and installation commands.
- `public/install.sh` and `public/install.ps1` install release binaries with SHA-256 verification.

Run installer tests from the repository root with Python 3.11 or newer.
Shell tests require `sh` and `unzip`, and Windows tests require PowerShell.

```sh
python -m unittest discover -s .github/scripts -p 'test_installers.py'
```

## Previews and fonts

The [preview guide](scripts/previews/README.md) covers recorded mdv output,
regeneration and snapshot checks.

The bundled MDV Pixel Text fonts derive from Monocraft 4.2. Original fonts in
`src/assets/fonts/` are inputs to regeneration. Licenses are in `public/third-party/`.

```sh
python -m pip install -r scripts/font-requirements.txt
python scripts/build-text-fonts.py
```

After changing the text fonts, regenerate the preview fonts as described in the guide.
