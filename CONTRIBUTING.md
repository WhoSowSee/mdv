# Contributing to mdv

Check the [issues](https://github.com/WhoSowSee/mdv/issues) and [open pull requests](https://github.com/WhoSowSee/mdv/pulls) for related work before opening a new one.

AI-assisted contributions must follow the [AI Policy](AI_POLICY.md).

For feature requests, [use the feature request form](https://github.com/WhoSowSee/mdv/issues/new?template=feature.yml) with the problem you want to solve and an example of how you'd use the feature. Discuss large features and changes to existing behavior before implementing them.

## Reporting a bug

[Use the bug report form](https://github.com/WhoSowSee/mdv/issues/new?template=bug.yml) with:

- Your mdv version (`mdv --version`), operating system, and terminal.
- The command and configuration, theme, or preset settings needed to reproduce the bug.
- A minimal Markdown input, pasted as text.
- What you expected and what happened, including errors.

For layout problems, include the terminal width and a screenshot if useful.

## Working locally

Install Rust through [rustup](https://rustup.rs/). The repository's [rust-toolchain.toml](rust-toolchain.toml) selects `stable` with `rustfmt` and `clippy`. The minimum supported Rust version is declared in [Cargo.toml](Cargo.toml).

Fork [mdv](https://github.com/WhoSowSee/mdv), then:

```sh
git clone https://github.com/YOUR-USERNAME/mdv.git
cd mdv
git remote add upstream https://github.com/WhoSowSee/mdv.git
git switch -c your-change
```

Try the sample:

```sh
cargo run -- docs/examples/sample.md
```

## Making a change

See the [module docs](docs/modules/README.md) for code organization and architecture.

Keep your PR focused on one problem and follow the structure of the surrounding code. Write comments and public API documentation in English.

For code fixes, add or extend a test that catches the bug. Put CLI tests in the existing integration-test topic modules. Reusable Markdown fixtures go in `tests/files/` with lowercase, hyphenated filenames; one-off input stays in the test. Choose additional checks from the [testing guide](docs/modules/testing.md#minimum-checks-by-change).

Keep these files in sync:

- [README.md](README.md) and [README-RU.md](README-RU.md).
- [docs/examples/config.yaml](docs/examples/config.yaml) and [assets/config/config.yaml](assets/config/config.yaml). Check configuration changes against these examples.

When moving files or changing public interfaces or state ownership, update the module docs and [file index](docs/modules/file-index.md).

The maintainer keeps the changelog. Leave changelog updates out of your PR unless asked to include them.

## Checking your work

Run these commands before submitting a code change:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

With [just](https://just.systems/man/en/), use `just check` for the same validation with a formatting check instead of automatic formatting.

Changes to `vendor/minus` also need `just check-pager`; Cargo's root checks don't cover that crate's own tests. The [testing guide](docs/modules/testing.md#commands) has the Cargo equivalents, Windows prerequisites, and details of `just check-all`.

For documentation changes, check links, examples, and accuracy against the implementation.

## Opening a pull request

Write commit subjects in the past tense: `Fixed table wrapping`, `Added a preset`, `Updated the docs`.

```sh
git add -- path/to/file
git commit -m "Fixed table wrapping"
```

After committing, rebase your branch onto the latest `upstream/main`:

```sh
git fetch upstream
git rebase upstream/main
```

Rerun the relevant checks, then push:

```sh
git push -u origin your-change
```

For a branch already pushed to your fork, use `git push --force-with-lease origin your-change` after rebasing.

Open a pull request to [WhoSowSee/mdv](https://github.com/WhoSowSee/mdv/pulls) and fill in the [PR template](.github/pull_request_template.md). Describe the problem, the change, and the checks you ran. Link the related issue; add an input/output example for rendering changes.

If you couldn't run a check, say which one and why. Push review fixes to the same branch.
