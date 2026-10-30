# Testing

Unit tests protect local contracts. The integration harness in
[tests/integration.rs](../../tests/integration.rs) runs the compiled CLI through
`assert_cmd`. Use the [file index](file-index.md#integration-tests) to find topic suites.

## Commands

Run the standard checks from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

The vendored pager is a separate package, outside the workspace. Its checks are:

```sh
cargo fmt --manifest-path vendor/minus/Cargo.toml --all -- --check
cargo clippy --manifest-path vendor/minus/Cargo.toml --locked --lib --features dynamic_output,search -- -D warnings
cargo test --manifest-path vendor/minus/Cargo.toml --locked --lib --features dynamic_output,search
```

For changes affecting pager features or examples, also run:

```sh
cargo clippy --manifest-path vendor/minus/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path vendor/minus/Cargo.toml --locked --all-features
```

Run release and packaging helper tests with Python 3.11 or newer:

```sh
python3 -m unittest discover -s .github/scripts -p 'test_*.py'
```

On Windows, use `python` and make Git Bash available as `bash` on `PATH` for the shell
release-note tests. The [CI workflow](../../.github/workflows/ci.yml) tests Rust code on
Linux, Windows, and macOS; [package installation checks](ci-release.md#rpm-and-arch-linux-packages)
run in Linux containers.

The optional [justfile](../../justfile) requires just 1.56.0 or newer. `just check` runs
the root Rust checks, `just check-pager` checks the pager, and `just check-all` adds
justfile and Debian-validator checks. `just fmt` formats both crates and the justfile.
Windows recipes require PowerShell 7.4 or newer (`pwsh.exe` on `PATH`).

`just check-package` runs `cargo publish --dry-run --locked`. It needs a clean checkout,
registry access, and a published pager version matching the root requirement; see
[release verification](ci-release.md#release-notes-and-verification).

## Test isolation

Use [tests/support.rs](../../tests/support.rs) to construct CLI processes. It sets an empty
configuration directory, removes ambient color and pager overrides, and supplies a stable
True Color terminal environment. Tests that need user configuration replace the directory
explicitly. Request `--color always` for ANSI/OSC assertions and `--color never` for plain
output; captured non-TTY stdout exercises the default color policy.

Unit tests that change environment variables use the shared mutex and restoring guards.
Tests of process-wide panic hooks run in child processes. Monitor tests wait for watcher
readiness before changing a file and clean up child processes on assertion failure.
Assertions compare document content, since native filesystem event counts can vary.

## Fixtures

Put reusable Markdown inputs in `tests/files/` with lowercase, hyphenated names. Create
one-off inputs inside the test. [docs/examples/config.yaml](../examples/config.yaml) is
the compatibility reference for settings; its bundled copy must stay synchronized.

## Minimum checks by change

| Change | Coverage |
|---|---|
| CLI or configuration | Parsing, precedence, validation, and an end-to-end command. |
| Markdown normalization | Transformed events, source positions, protected code/HTML, and rendered output. |
| Wrapping or layout | Unicode graphemes, visible widths, ANSI/OSC boundaries, and narrow terminals. |
| Links, tables, or footnotes | Reference ownership, numbering, nesting, and final placement. |
| Themes or styling | YAML inheritance, overrides, palette limits, and all color modes. |
| Pager or browser | State transitions, selection, refresh, and terminal handoff when affected. |
| Packaging | Helper tests and native installation checks for each affected format. |
| Documentation | Relative links, examples, and consistency with implementation and templates. |

Test observable behavior and failure cases. Check errors using otherwise valid input so an
unrelated failure cannot satisfy the assertion. Keep syntax parsing and precedence matrices
in unit tests; integration tests verify their effect at the CLI boundary.

Pager cache tests cover first use, width and numbering changes, refresh invalidation, and
reuse of active renders. Coordinate background work through channels. Test warmup deadlines
without waiting for the real five-second delay.

## Rendering performance

Compare reference and candidate binaries with the same input, width, configuration, terminal
environment, and color policy. Verify exit status and raw stdout before timing. Alternate
runs without concurrent builds, report medians, and distinguish whole-process time from
in-process measurements. Include larger inputs to detect growth with document size.
