import json
from pathlib import Path
import re
import sys

from fontTools import subset
from fontTools.ttLib import TTFont
from previews.symbol_font import add_unicode_glyphs, fit_terminal_cells, read_source, validate_geometry
from previews.text_font import build_preview_text


website = Path(__file__).resolve().parents[1]
manifest = json.loads((website / 'scripts/previews/symbol-sources.json').read_text(encoding='utf-8'))
source = read_source(Path(sys.argv[1]), manifest['nerd'])
unicode_source = read_source(Path(sys.argv[2]), manifest['unicode'])
text_font = TTFont(website / 'src/assets/fonts/mdv-pixel-text-regular.woff2')
characters = set(range(32, 127))
for filename in ['render-previews.json', 'interactive-previews.json']:
    snapshot = json.loads((website / 'src/data' / filename).read_text(encoding='utf-8'))
    for preview in snapshot['previews'].values():
        for locale in ['en', 'ru']:
            for variant in preview[locale]['variants']:
                plain = re.sub(r'\x1b\].*?(?:\x07|\x1b\\)|\x1b\[[0-9;:]*m', '', variant['ansi'])
                characters.update(ord(c) for c in plain if ord(c) >= 32)
codepoints = sorted(characters - text_font.getBestCmap().keys())
add_unicode_glyphs(source, unicode_source, set(codepoints) & set(manifest['unicode']['codepoints']))
missing = set(codepoints) - source.getBestCmap().keys()
if missing:
    raise ValueError(f'Missing preview symbols: {missing}')
options = subset.Options()
options.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14, 16, 17]
options.drop_tables += ['GSUB', 'GPOS']
subsetter = subset.Subsetter(options=options)
subsetter.populate(unicodes=codepoints)
subsetter.subset(source)
wide_codepoints = set(codepoints) & set(manifest['nerd']['wideCodepoints'])
fit_terminal_cells(source, wide_codepoints)
metrics = validate_geometry(source, codepoints, wide_codepoints)
metadata = {'sources': manifest, 'codepoints': codepoints, 'textCodepoints': sorted(characters), 'metrics': metrics}
(website / 'src/data/preview-symbols.json').write_text(json.dumps(metadata, indent=2) + '\n', encoding='utf-8')
license_text = (website / 'public/third-party/monocraft-license.txt').read_text(encoding='utf-8')
license_text = license_text[license_text.index('-----------------------------------------------------------'):]
notice = 'MDV Preview Symbols, subsets of JetBrains Mono Nerd Font and Noto Sans Symbols 2.\n'
notice += '\n'.join(entry['url'] for entry in manifest.values())
notice += '\n\nCopyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono)\n'
notice += 'Copyright 2022 The Noto Project Authors (https://github.com/notofonts/symbols)\n'
notice += 'Nerd Fonts modifications: https://github.com/ryanoasis/nerd-fonts\n\n'
(website / 'public/third-party/preview-symbols-license.txt').write_text(notice + license_text, encoding='utf-8')
print(f'{len(codepoints)} supplementary symbols')
build_preview_text(website, source, characters)

