# Documentation previews

[Русский](README-RU.md)

Recorded mdv output is stored in `src/data/render-previews.json` and
`src/data/interactive-previews.json`. Regenerate it after changing rendering or interactive behavior.

## Generate and check

Requires Bun 1.4.4 or newer and Python 3.11 or newer. Start from the repository root.

```sh
cargo build --release
cd website
python -m pip install -r scripts/preview-requirements.txt
bun run update-previews
bun run update-previews --check
```

`update-previews` accepts an optional path to the mdv binary, whose version must
match `Cargo.toml`. `--check` compares both snapshots without rewriting them.
Capture ignores user configuration and environment color overrides.

## Fonts

Regenerate preview fonts after changing text fonts or adding characters to captures.
Source URLs and checksums are in [symbol-sources.json](symbol-sources.json).
Run from `website`, passing paths to those source fonts.

```sh
python -m pip install -r scripts/font-requirements.txt
python scripts/build-preview-symbols.py JetBrainsMonoNerdFont-Regular.ttf NotoSansSymbols2-Regular.ttf
```
