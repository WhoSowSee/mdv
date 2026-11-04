# CI and Release Packaging

[build.yml](../../.github/workflows/build.yml) runs on tag pushes and manual
dispatches. Tags, with an optional `v` prefix, must match the version in
`Cargo.toml`. Manual runs build and check packages without publishing a release.

| Workflow | Responsibility |
|---|---|
| [build.yml](../../.github/workflows/build.yml) | Verify the publishable crate, call the build workflows, and publish after all checks pass. |
| [build-platforms.yml](../../.github/workflows/build-platforms.yml) | Linux, macOS, and Windows binaries, ZIP archives, Debian, RPM, and Arch packages. |
| [build-packages.yml](../../.github/workflows/build-packages.yml) | Snap and Nix builds and checks. |

The two build workflows use `workflow_call`. Publication requires `package`,
`platforms`, and `packages` to succeed and consumes artifacts from that same run.

## Pull request and branch checks

[ci.yml](../../.github/workflows/ci.yml) runs for pull requests targeting `main`
and pushes to `main`:

| Job | Checks |
|---|---|
| `Checks` on Linux | Formatting and Clippy for mdv and `vendor/minus`, plus all Python helper tests. |
| `Test (ubuntu-latest)` | mdv and pager tests, release build, and `mdv --version`. |
| `Test (windows-latest)` | mdv and pager tests. |
| `Test (macos-latest)` | mdv and pager tests. |

CI uses committed lockfiles and read-only repository permissions. The vendored
pager is a path dependency outside the workspace; its checks use
`vendor/minus/Cargo.toml` with `dynamic_output,search`. Test jobs run independently
on all three operating systems. CI does not publish packages.

## Build matrix

| Target | Runner | Builder | Artifacts |
|---|---|---|---|
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | Native Cargo | ZIP, DEB, RPM |
| `aarch64-unknown-linux-musl` | `ubuntu-latest` | Cross | ZIP, DEB |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native Cargo | ZIP, DEB, RPM, Arch Linux |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | Cross | ZIP, DEB |
| `i686-unknown-linux-gnu` | `ubuntu-latest` | Cross | ZIP, DEB |
| `riscv64gc-unknown-linux-gnu` | `ubuntu-latest` | Cross | ZIP, DEB |
| `sparc64-unknown-linux-gnu` | `ubuntu-latest` | Cross | ZIP, DEB |
| `aarch64-apple-darwin` | `macos-latest` | Native Cargo | ZIP |
| `x86_64-apple-darwin` | `macos-26-intel` | Native Cargo | ZIP |
| `x86_64-pc-windows-msvc` | `windows-latest` | Native Cargo | ZIP |
| `aarch64-pc-windows-msvc` | `windows-11-arm` | Native Cargo | ZIP |
| Snap `amd64` | `ubuntu-24.04` | Snapcraft, LXD, core24 | Snap |
| Snap `arm64` | `ubuntu-24.04-arm` | Snapcraft, LXD, core24 | Snap |
| Nix `x86_64-linux` | `ubuntu-latest` | Nix | Checked store output |
| Nix `aarch64-linux` | `ubuntu-24.04-arm` | Nix | Checked store output |

Native Cargo jobs run Rust tests and the release binary with `--version`; macOS
also verifies architecture with `lipo`. Cross jobs validate ELF architecture and
packages without executing foreign binaries. Snap jobs install and run their
packages; Nix jobs evaluate, build, and run the native package.

The `package` job checks formatting, Clippy, all Python helpers, the vendored
pager, and `cargo publish --dry-run --locked`.

## Debian runtime dependencies

[Cargo.toml](../../Cargo.toml) declares cargo-deb variants selected by the Linux
matrix:

```text
cargo deb --locked --no-build --variant VARIANT --target TARGET --output OUTPUT
```

| Variant | Targets | Runtime dependencies |
|---|---|---|
| `glibc-2-39` | Native GNU amd64 and arm64 | `libc6 (>= 2.39), libgcc-s1` |
| `glibc-2-18` | GNU i686 and sparc64 | `libc6 (>= 2.18), libgcc-s1` |
| `glibc-2-27` | GNU riscv64 | `libc6 (>= 2.27), libgcc-s1` |
| `musl` | Static amd64 and arm64 | None |

Explicit dependencies avoid host-dependent `dpkg-shlibdeps` results for foreign
architectures. The glibc minimum must cover the binary's required symbol versions.
[verify_deb.py](../../.github/scripts/verify_deb.py) checks ELF compatibility,
dependencies, metadata, executable permissions, and binary identity. Musl
packages must be static; unsupported libraries or ABI versions fail validation.
If a toolchain or runner update raises the required glibc version, review the
compatibility baseline before changing the variant.

## RPM and Arch Linux packages

RPM packages for amd64 and arm64, and the Arch package for x86_64, reuse the
native GNU release binaries. [linux_packages.py](../../.github/scripts/linux_packages.py)
prepares and validates recipes using
[mdv.spec.in](../../.github/packaging/mdv.spec.in) and
[PKGBUILD.in](../../.github/packaging/PKGBUILD.in). Validation covers metadata,
runtime dependencies, binary identity and permissions, and the MIT license.
Both formats use the `glibc-2-39` compatibility baseline and preserve the
already stripped executable.

RPMs are installed and tested in `fedora:44`; the Arch package is built as an
unprivileged user, installed, and tested in `archlinux:base-devel`. Only validated
packages are uploaded. Arch Linux ARM packages are outside the build matrix.

## Build tools

Workflow files are the source of truth for runner and tool versions. Rust
`stable`, Action major tags, and Snapcraft `9.x/stable` can resolve to newer
releases. Cross is pinned; ARM64 installs cargo-deb from crates.io.

[snapcraft.yaml](../../snap/snapcraft.yaml) uses `core24` and the Rust plugin's
defaults. Before adding Cargo arguments, check the resolved plugin's generated
command to avoid duplicating flags such as `--locked`.

[flake.nix](../../flake.nix) follows `nixos-26.05`. No flake lock is committed, so
the Nix input revision can change between builds.

## Release notes and verification

Publish a new `mdv-minus` version before releasing mdv when the vendored pager
API changes. Update the fork manifests and the root version requirement together.
Cargo removes the path dependency when packaging mdv and verifies against the
crates.io release; local success with the vendored source is not sufficient.

[release-notes.sh](../../.github/scripts/release-notes.sh) selects the matching
changelog version, then the tag annotation or commit message if that section is
absent or empty. [compose_release_notes.py](../../.github/scripts/compose_release_notes.py)
combines this source with GitHub's generated notes in `release-notes.md`:

- PR-related changelog items form one collapsed PR block with the original
  title, author, number, descriptions, and commit links. Ordinary commits stay
  in their sections; emptied sections are removed.
- PRs without changelog descriptions retain their generated title and author.
- PRs with any item under a `Breaking Changes` changelog heading, including
  nested headings, or in that generated category appear first under
  `Breaking Changes`. Other PRs use `Other Changes` only when breaking PRs exist.
- `New Contributors` and the comparison link are preserved. A horizontal rule
  appears directly before `Full Changelog`.

An explicit PR reference takes priority over commit association: use `PR #3`,
`(#3)`, or `[#3](https://github.com/WhoSowSee/mdv/pull/3)`. References inside inline
code or fenced examples are ignored. Plain issue numbers do not assign PRs.
Otherwise, commit links for this repository identify merged PRs included in the
generated notes. Explicit references missing from that list are also grouped
when the PR is merged and at least one of its commits appears in the release
comparison. The comparison comes from GitHub's `Full Changelog` link and must
end at the requested tag. Unmerged PRs and PRs outside the comparison stay
in their original sections.
An item matching multiple PRs requires an explicit reference;
ambiguity or API failures stop publication. The source `CHANGELOG.md` remains unchanged.

The release job grants `contents: write` and `pull-requests: read` and supplies
`GITHUB_TOKEN` to the composer. The release action uses `body_path` with
`generate_release_notes: false` to publish the composed text without appending
another PR list.

The artifact contract is 23 files: 11 ZIP archives, seven Debian packages, two
RPM packages, one Arch Linux package, and two snaps. Asset names include the
release tag; Nix outputs gate publication but are not uploaded.

Before releasing CI or packaging changes, run `build.yml` through
`workflow_dispatch` and require every job to pass. Local checks do not replace
the platform and package jobs. Rerunning a tag checks out that tag's commit.

## Binary installers

The website publishes [install.sh](../../website/public/install.sh) and
[install.ps1](../../website/public/install.ps1). They query the GitHub Releases
API for the latest release or an exact tag and select
`mdv-<tag>-<target>.zip`. Each matching asset must be unique and expose a valid
`sha256:` digest. There is no checksum bypass or source-build fallback.

The sh installer supports x86_64/ARM64 Linux with static musl archives and
Intel/Apple Silicon macOS archives. The PowerShell installer supports x64/ARM64
Windows. Extraction reads only the root `mdv` or `mdv.exe` entry; verified
binaries are staged in the destination directory before replacement. Errors
before replacement preserve the existing executable. Windows adds the install
directory to the user and session PATH unless `-NoModifyPath` is set; sh prints
PATH instructions without modifying shell profiles.

[test_installers.py](../../.github/scripts/test_installers.py) covers local release
fixtures, platform selection, pinned tags, installation and replacement, missing
or duplicate assets, checksum failure, archive contents, and temporary-file
cleanup. Shell tests need `sh` and `unzip`; Windows-specific tests run only on
Windows. These tests are included in the helper suite. Native execution of
downloaded macOS/Linux binaries still requires those operating systems.

The installers use existing release assets, so adding them does not change the
release workflow or its 23-file artifact contract. Public one-line installation
commands become usable after the site is published to GitHub Pages.

The website's package-manager tabs use exact RPM/Arch filenames from
`website/src/data/package-release.json`. `bun run build` refreshes this snapshot
from the latest stable GitHub release before checking types and generating the
site. Missing assets, duplicate assets, unexpected download URLs, or API errors
stop the build. Development uses the stored snapshot without network requests.

## Website publication

[website.yml](../../.github/workflows/website.yml) runs the website tests and
`bun run build` on pull requests targeting `main` and every push to `main`.
It installs Node.js 24 and the Bun version declared in `website/package.json`,
using `website/bun.lock` with `--frozen-lockfile`.

Successful main builds upload `website/dist/` and deploy it to the `github-pages`
environment. Pull requests and manual runs on other branches only check and
build. Set **Settings > Pages > Source** to **GitHub Actions** once. No custom
token or committed build output is required.

A successful tag-triggered `Build binaries` run also starts website publication
through `workflow_run`. This handles releases created with `GITHUB_TOKEN`, whose
`release` events do not trigger another workflow. The site is rebuilt from the
default branch after release assets have been published; manual package checks
and unsuccessful runs do not deploy it. Production runs share a concurrency
group without cancelling an active deployment. Manual publication is available
through **Actions > Website > Run workflow** on `main`.

## README star history

[star-history.yml](../../.github/workflows/star-history.yml) refreshes four
English/Russian light/dark charts every Monday and Thursday at 06:17 UTC, or by
manual dispatch on a branch. [gen_star_history.py](../../.github/scripts/gen_star_history.py)
uses the read-only `GITHUB_TOKEN` to fetch star history.

Changed charts are committed with a dedicated GitHub App token. The workflow
allows only the four SVGs to be staged. Configure `STAR_HISTORY_APP_CLIENT_ID`
as a repository variable and `STAR_HISTORY_APP_PRIVATE_KEY` as a secret. The
publisher App needs `Contents: write` and a bypass for required checks on the
protected branch. Its token is scoped to this repository; App pushes trigger CI.

Both READMEs select charts by language and color scheme using absolute image URLs
that work on GitHub and crates.io.
