set minimum-version := '1.56.0'
set default-list

[unix]
set shell := ['sh', '-cu']

[windows]
set shell := ['pwsh.exe', '-NoLogo', '-NoProfile', '-CommandWithArgs']

[unix]
python := 'python3'

[windows]
python := 'python'

[doc('Check justfile, mdv, pager, and packaging tests.')]
check-all: check-just check check-pager test-packaging

[doc('Check mdv formatting, lints, tests, and the release build.')]
check: fmt-check lint test build

[doc('Check vendored pager formatting, lints, and library tests.')]
check-pager:
    cargo fmt --manifest-path vendor/minus/Cargo.toml --all -- --check
    cargo clippy --manifest-path vendor/minus/Cargo.toml --lib --features dynamic_output,search -- -D warnings
    cargo test --manifest-path vendor/minus/Cargo.toml --lib --features dynamic_output,search

[doc('Format the justfile, mdv, and the vendored pager.')]
fmt: fmt-just
    cargo fmt --all
    cargo fmt --manifest-path vendor/minus/Cargo.toml --all

[doc('Format the justfile.')]
fmt-just:
    just --fmt

[doc('Check justfile syntax and formatting (no edits).')]
check-just:
    just --fmt --check

[doc('Check mdv formatting without changing files.')]
fmt-check:
    cargo fmt --all -- --check

[doc('Run Clippy on all mdv targets; warnings are errors.')]
lint:
    cargo clippy --all-targets -- -D warnings

[doc('Run mdv tests with optional Cargo args and a filter.')]
[positional-arguments]
[unix]
test *args:
    cargo test "$@"

[doc('Run mdv tests with optional Cargo args and a filter.')]
[positional-arguments]
[windows]
test *args:
    @$ErrorActionPreference = 'Stop'; \
    cargo test @($args | Select-Object -Skip 1); \
    exit $LASTEXITCODE

[doc('Build the mdv release binary.')]
build:
    cargo build --release

[doc('Run mdv with optional application arguments.')]
[positional-arguments]
[unix]
run *args:
    cargo run -- "$@"

[doc('Run mdv with optional application arguments.')]
[positional-arguments]
[windows]
run *args:
    @$ErrorActionPreference = 'Stop'; \
    cargo run '--' @($args | Select-Object -Skip 1); \
    exit $LASTEXITCODE

[doc('Run mdv with the optimized, incremental dev-opt profile.')]
[positional-arguments]
[unix]
run-dev *args:
    cargo run --profile dev-opt -- "$@"

[doc('Run mdv with the optimized, incremental dev-opt profile.')]
[positional-arguments]
[windows]
run-dev *args:
    @$ErrorActionPreference = 'Stop'; \
    cargo run --profile dev-opt '--' @($args | Select-Object -Skip 1); \
    exit $LASTEXITCODE

[doc('Test DEB package validator; requires Python 3.11+.')]
test-packaging:
    {{ python }} .github/scripts/test_verify_deb.py

[doc('Verify crate without upload; requires a clean checkout.')]
check-package:
    cargo publish --dry-run --locked

[doc('Render docs/examples/sample.md with a preset.')]
demo preset='showcase': (run-dev 'docs/examples/sample.md' '--preset' preset)

[doc('Watch docs/examples/sample.md with a preset.')]
demo-monitor preset='showcase': (run-dev 'docs/examples/sample.md' '--preset' preset '--monitor')
