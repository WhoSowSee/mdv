from hashlib import sha256
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont


def read_source(path, expected):
    if sha256(path.read_bytes()).hexdigest() != expected['sha256']:
        raise ValueError(f"Unexpected font source: {path.name}")
    return TTFont(path, recalcTimestamp=False)


def outline(font, name):
    glyphs = font.getGlyphSet()
    bounds = BoundsPen(glyphs)
    glyphs[name].draw(bounds)
    recorded = DecomposingRecordingPen(glyphs)
    glyphs[name].draw(recorded)
    return recorded, bounds.bounds


def glyph_metrics(font, codepoints):
    result = {}
    for codepoint in codepoints:
        name = font.getBestCmap()[codepoint]
        _, bounds = outline(font, name)
        result[str(codepoint)] = {'bounds': list(bounds), 'advance': font['hmtx'][name][0]}
    return result


def validate_geometry(font, codepoints, wide_codepoints):
    metrics = glyph_metrics(font, codepoints)
    for large, small in zip([0xf444, 0xf445, 0xf4c3, 0xf51d], [0xf09de, 0xf0a13, 0xf14dc, 0xf0a14]):
        if str(large) not in metrics or str(small) not in metrics:
            continue
        large_bounds, small_bounds = metrics[str(large)]['bounds'], metrics[str(small)]['bounds']
        if large_bounds[3] - large_bounds[1] <= small_bounds[3] - small_bounds[1]:
            raise ValueError('Nerd Font large markers must be larger than small markers.')
    for codepoint, metric in metrics.items():
        if int(codepoint) < 0xe000:
            continue
        left, _, right, _ = metric['bounds']
        padding = metric['advance'] if int(codepoint) in wide_codepoints else 0
        if left < -padding or right > metric['advance'] + padding:
            raise ValueError(f'Preview icon exceeds its cell: U+{int(codepoint):X}')
    return metrics


def add_unicode_glyphs(target, unicode_font, codepoints):
    target_units = target['head'].unitsPerEm
    scale = target_units / unicode_font['head'].unitsPerEm
    for codepoint in codepoints:
        name = f'unicode{codepoint:X}'
        source_name = unicode_font.getBestCmap()[codepoint]
        recorded, bounds = outline(unicode_font, source_name)
        scale_to_cell = min(scale, 600 / (bounds[2] - bounds[0]))
        offset = 300 - (bounds[0] + bounds[2]) / 2 * scale_to_cell
        pen = TTGlyphPen(None)
        recorded.replay(TransformPen(pen, (scale_to_cell, 0, 0, scale_to_cell, offset, 0)))
        target['glyf'][name] = pen.glyph()
        target['hmtx'][name] = (600, round(bounds[0] * scale_to_cell + offset))
        for table in target['cmap'].tables:
            if table.isUnicode() and (codepoint <= 0xffff or table.format == 12):
                table.cmap[codepoint] = name


def fit_terminal_cells(font, wide_codepoints):
    names = set(font.getBestCmap().values())
    line_names = {name for codepoint, name in font.getBestCmap().items() if 0x2500 <= codepoint <= 0x257f}
    vertical_names = {name for codepoint, name in font.getBestCmap().items() if codepoint in {0x2502, 0x2503, 0x2506}}
    wide_names = {font.getBestCmap()[codepoint] for codepoint in wide_codepoints}
    outlines = {}
    for name in names:
        recorded, bounds = outline(font, name)
        if name in vertical_names:
            transform = (1, 0, 0, 1, (2000 / 3 - font['hmtx'][name][0]) / 2, 0)
        elif name in line_names:
            transform = (10 / 9, 0, 0, 1, 0, 0)
        else:
            scale = 1 if name in wide_names else min(1, 600 / (bounds[2] - bounds[0]))
            transform = (scale, 0, 0, scale, 1000 / 3 - (bounds[0] + bounds[2]) / 2 * scale, 0)
        pen = TTGlyphPen(None)
        recorded.replay(TransformPen(pen, transform))
        glyph = pen.glyph()
        glyph.recalcBounds(font['glyf'])
        if name in vertical_names and glyph.xMax - glyph.xMin != bounds[2] - bounds[0]:
            raise ValueError('Vertical border stroke width changed.')
        outlines[name] = glyph
    for name, glyph in outlines.items():
        font['glyf'][name] = glyph
        font['hmtx'][name] = (667, glyph.xMin)
