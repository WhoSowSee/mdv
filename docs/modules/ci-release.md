# CI and Release Packaging

[build.yml](../../.github/workflows/build.yml) runs on tag pushes and manual dispatches. A manual
run builds and checks packages without publishing a GitHub release. Tag runs
require the tag, with an optional `v` prefix, to match `Cargo.toml`.

The release process is split across three files:

| Workflow | Responsibility |
|---|---|
| [build.yml](../../.github/workflows/build.yml) | Verify the publishable crate, call both build workflows, and publish the release after all checks pass. |
| [build-platforms.yml](../../.github/workflows/build-platforms.yml) | Linux, macOS, and Windows binary builds, tests, ZIP archives, and Debian packages. |
| [build-packages.yml](../../.github/workflows/build-packages.yml) | Snap and Nix builds and package checks. |

The two build workflows use `workflow_call` and run only through the caller.
Local workflow references use the caller's commit. Each called workflow defines
`PROJECT_NAME: mdv`, because the caller's workflow-level environment is not
inherited. All uploaded artifacts belong to the same run, so the release job
downloads them after `package`, `platforms`, and `packages` succeed.

## Pull request and branch checks

[ci.yml](../../.github/workflows/ci.yml) runs for pull requests targeting `main`
and pushes to `main`. It creates four jobs:

| Job | Checks |
|---|---|
| `Checks` on Linux | Formatting and Clippy for mdv and `vendor/minus`, plus Python tests for packaging, release notes, and star history. |
| `Test (ubuntu-latest)` | mdv and pager tests, release build, and `mdv --version`. |
| `Test (windows-latest)` | mdv and pager tests. |
| `Test (macos-latest)` | mdv and pager tests. |

Cargo compilation and test commands use the committed lockfiles. Pager tests use
its own manifest with `dynamic_output,search`, because it is not a workspace
member. All command steps use Bash, including Git Bash on Windows, so a failed
command stops the step. Matrix failures do not cancel the other platforms.

The workflow has read-only repository permissions and does not publish packages.
Cargo caches cover both target directories and are saved on pushes to `main`;
pull requests restore available caches. A new run cancels earlier runs for the
same pull request or branch.

## Build matrix

| Target | Runner | Builder | Artifacts |
|---|---|---|---|
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | Native Cargo | ZIP, DEB |
| `aarch64-unknown-linux-musl` | `ubuntu-latest` | Cross 0.2.5 | ZIP, DEB |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native Cargo | ZIP, DEB |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | Cross 0.2.5 | ZIP, DEB |
| `i686-unknown-linux-gnu` | `ubuntu-latest` | Cross 0.2.5 | ZIP, DEB |
| `riscv64gc-unknown-linux-gnu` | `ubuntu-latest` | Cross 0.2.5 | ZIP, DEB |
| `sparc64-unknown-linux-gnu` | `ubuntu-latest` | Cross 0.2.5 | ZIP, DEB |
| `aarch64-apple-darwin` | `macos-latest` | Native Cargo | ZIP |
| `x86_64-apple-darwin` | `macos-26-intel` | Native Cargo | ZIP |
| `x86_64-pc-windows-msvc` | `windows-latest` | Native Cargo | ZIP |
| `aarch64-pc-windows-msvc` | `windows-11-arm` | Native Cargo | ZIP |
| Snap `amd64` | `ubuntu-24.04` | Snapcraft, LXD, core24 | Snap |
| Snap `arm64` | `ubuntu-24.04-arm` | Snapcraft, LXD, core24 | Snap |
| Nix `x86_64-linux` | `ubuntu-latest` | Nixpkgs 26.05 | Checked store output |
| Nix `aarch64-linux` | `ubuntu-24.04-arm` | Nixpkgs 26.05 | Checked store output |

Native Cargo jobs run the Rust test suite and execute the release binary with
`--version`. macOS also checks the Mach-O architecture with `lipo`. Cross jobs
check ELF architecture and packaging, but do not execute foreign binaries.
Snap jobs install the resulting package and run it; each Nix job evaluates,
builds, and runs its native package. Together, the two jobs cover both systems.

The package job checks formatting, Clippy, Debian validator and release-note
tests, and `cargo publish --dry-run --locked`. Publishing waits for this job and every
platform job, including Nix. Only the release job receives `contents: write`.

The `package` job in `build.yml` also runs formatting, Clippy with `-D warnings`, and unit tests
against `vendor/minus/Cargo.toml` with `dynamic_output,search`. The fork is a
path dependency rather than a workspace member, so the root command does not
replace these checks.

## Debian runtime dependencies

Cargo.toml declares standard cargo-deb variants. The Linux matrix selects one
explicitly, and the workflow invokes cargo-deb directly:

```text
cargo deb --locked --no-build --variant VARIANT --target TARGET --output OUTPUT
```

| Variant | Targets | Runtime dependencies |
|---|---|---|
| `glibc-2-39` | Native GNU amd64 and arm64 | `libc6 (>= 2.39), libgcc-s1` |
| `glibc-2-18` | GNU i686 and sparc64 | `libc6 (>= 2.18), libgcc-s1` |
| `glibc-2-27` | GNU riscv64 | `libc6 (>= 2.27), libgcc-s1` |
| `musl` | Static amd64 and arm64 | None |

The common `package.metadata.deb.name` keeps every variant's Debian package name
as `mdv`. The glibc bounds come from the released ELF binaries and native build
environment; they are explicit compatibility requirements, not compiler defaults.

This avoids host-dependent `dpkg-shlibdeps` results: a foreign binary can
otherwise produce an empty `Depends`, and i686 binaries analysed on amd64 can
incorrectly depend on `libc6-i386` and `lib32gcc-s1`.

After packaging, `.github/scripts/verify_deb.py` checks the ELF class, byte order,
machine, shared libraries, Debian metadata, and payload. It reads the payload
archive directly, requiring one regular `usr/bin/mdv` file with mode `0755`
and the same hash as the already stripped Cargo release binary. Permissions
come from the archive rather than the extraction user's filesystem access.
The declared glibc minimum must cover every required glibc symbol version.
Musl binaries must have neither shared libraries nor an ELF interpreter.
Unrecognised runtime libraries or ABI versions fail validation.

Packaging and validation do not modify Cargo.toml or Cargo.lock. If a compiler
or runner update raises the binary's glibc requirement, validation fails: review
the new compatibility baseline and adjust the variant/matrix or build environment.
Run the validator regressions with:

```text
python3 .github/scripts/test_verify_deb.py
```

## Build tools

The workflow files are the source of truth for tool references. Action major tags and
the Rust `stable` reference receive upstream updates automatically. Cross uses
its explicit stable release. The ARM64 cargo-deb installer builds from crates.io
because the selected release has no ARM64 binary asset.

Snapcraft 9.0.1's released `uv.lock` fixes Craft Parts to 2.33.0. For this ordinary
Cargo package, that plugin generates `cargo install -f --locked --path . --root
INSTALL_DIR`; `--locked` is already present. `rust-cargo-parameters` appends
arguments and must not add it again. The plugin supplies its own build packages;
mdv does not require OpenSSL.

Nixpkgs follows the stable 26.05 branch. No flake lock is committed, so separate
builds can resolve different stable updates; the resolved revision appears in
the Nix job log.

Primary references: [cargo-deb 3.8.0](https://github.com/kornelski/cargo-deb/releases/tag/v3.8.0),
[Snapcraft 9 migration](https://github.com/canonical/snapcraft/blob/9.0.1/docs/release-notes/snapcraft-9-0.rst),
[Snapcraft lockfile](https://github.com/canonical/snapcraft/blob/9.0.1/uv.lock),
[Rust plugin](https://github.com/canonical/craft-parts/blob/2.33.0/craft_parts/plugins/rust_plugin.py),
[release action](https://github.com/softprops/action-gh-release/blob/v3.0.3/src/github.ts),
[runner images](https://github.com/actions/runner-images#available-images).

## Release notes and verification

Publish a new `mdv-minus` version before releasing mdv when the vendored pager
API changes. Update the fork manifests and the root version requirement together.
Cargo removes the path dependency when packaging mdv and uses the crates.io
release during verification. The dry run blocks publication if that release
does not contain the APIs used by mdv, even when local binary builds succeed.

mdv 6.0.1 requires the published `mdv-minus` 6.0.1 patch with the Clippy 1.99.0
fix. The Unix-only `terminfo` 0.9.0 dependency reads terminfo files in Rust
and adds no ncurses linkage or mandatory runtime database package.

`.github/scripts/release-notes.sh` writes the custom release body directly to a
file, preserving the existing changelog, tag annotation, and commit-message
priority. It selects only the changelog section matching the release tag,
stopping before the next version heading. Regression tests in
`.github/scripts/test_release_notes.py` cover section boundaries and the
existing tag/commit message priority.
The release action's `generate_release_notes` option appends GitHub's
generated notes on both creation and update. The script does not call the GitHub
API or require a token. Multiline text is never passed through a fixed
`GITHUB_OUTPUT` delimiter.

The artifact contract is 11 ZIP archives, seven Debian packages, and two snaps.
Each upload has a unique name; the release prefixes filenames with the tag.
The Nix outputs gate publication but are not uploaded as release assets.

Local Windows/WSL checks do not substitute for hosted macOS, Windows ARM64,
Cross/Docker, LXD/Snapcraft, or Nix builds. Before releasing, run the modified
workflow through `workflow_dispatch` on a branch containing the changes and
require every job to pass. Rerunning an existing tag uses that tag's commit,
not later fixes in the working tree.

## README star history

`.github/workflows/star-history.yml` refreshes the English and Russian light and dark SVGs in
`.github/assets/` every Monday and Thursday at 06:17 UTC, or by manual dispatch.
The chart style follows [makerspet/oomwoo](https://github.com/makerspet/oomwoo).
`.github/scripts/gen_star_history.py` reads GitHub's aggregate daily star history
and renders SVGs using only Python's standard library. The job uses its own
`GITHUB_TOKEN` and commits only the four charts when their contents change.
Both README languages select the chart for the reader's color scheme using
absolute `raw.githubusercontent.com` URLs. This also works on crates.io, which
rewrites relative `img src` URLs but leaves `source srcset` URLs unchanged.
README-RU uses the `star-history-light-ru.svg` and `star-history-dark-ru.svg`
variants with Russian titles, star counts, and month labels. The SVGs provide
their own headings, so the README files do not repeat them.
The generator's pagination, cumulative counts, sampling, and empty/single-day
rendering are checked by `.github/scripts/test_gen_star_history.py`.
