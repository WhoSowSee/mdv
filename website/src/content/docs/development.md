---
title: "Development"
description: "Build, test and work on mdv and its website."
slug: "development"
group: "Reference"
order: 44
---

## Build and validate

See [CONTRIBUTING.md](https://github.com/WhoSowSee/mdv/blob/main/CONTRIBUTING.md) for reporting issues, preparing pull requests, and contribution guidelines.

For AI-assisted contributions, read the [AI Policy](https://github.com/WhoSowSee/mdv/blob/main/AI_POLICY.md).

Local development uses [rust-toolchain.toml](https://github.com/WhoSowSee/mdv/blob/main/rust-toolchain.toml) to select stable Rust with `rustfmt` and `clippy`. Formatting settings are in [rustfmt.toml](https://github.com/WhoSowSee/mdv/blob/main/rustfmt.toml).

CI checks, package builds, and release publishing are described in the [CI and release documentation](https://github.com/WhoSowSee/mdv/blob/main/docs/modules/ci-release.md).

The optional [justfile](https://github.com/WhoSowSee/mdv/blob/main/justfile) provides development commands. Install [just](https://just.systems/man/en/) 1.56.0 or newer with `cargo install --locked just`.

```text
just --list
just check-all
just fmt
just run-dev README.md
```

Windows requires PowerShell 7.4+ (`pwsh.exe` on `PATH`); `check-all` also requires Python 3.11+. See [development and testing documentation](https://github.com/WhoSowSee/mdv/blob/main/docs/modules/testing.md) for all commands and details.

### Website

The project website lives in [website/](https://github.com/WhoSowSee/mdv/blob/main/website/README.md) and uses Astro,
TypeScript, and Tailwind CSS.

- Bun 1.4.4 or newer is the minimum declared in [website/package.json](https://github.com/WhoSowSee/mdv/blob/main/website/package.json).
- Astro also requires Node.js 22.23.4 or newer on `PATH`.

```sh
cd website
bun install --frozen-lockfile
bun run dev
```

Open the printed local URL under `/mdv/`.

```sh
bun run test
bun run build
bun run preview
```

`bun run build` first refreshes package links from the latest stable GitHub release,
then checks types and generates `website/dist/`. It requires GitHub API access;
set `GITHUB_TOKEN` when needed. Development uses the stored release snapshot.

GitHub Actions checks pull requests targeting `main` and publishes successful
pushes to `main`. It also refreshes the site after the release workflow succeeds
for a tag. Manual publication is available through **Actions > Website > Run workflow**
on `main`; other branches only check and build. Publishing requires
**Settings > Pages > Source > GitHub Actions**. See the
[website workflow](https://github.com/WhoSowSee/mdv/blob/main/.github/workflows/website.yml).

The website includes complete user guides and CLI/YAML references at `/mdv/docs/`, with navigation, full-text search, and page outlines.

- Its Markdown sources live in `website/src/content/docs/`.
- The language icon switches the current page between English and Russian.
- Russian pages start at `/mdv/ru/`, with documentation at `/mdv/ru/docs/`.
