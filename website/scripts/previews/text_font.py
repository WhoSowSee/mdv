from copy import deepcopy

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont


def build_preview_text(website, symbols, codepoints):
    for style, weight in [('regular', 400), ('bold', 700)]:
        text = TTFont(website / f'src/assets/fonts/mdv-pixel-text-{style}.woff2')
        units = text['head'].unitsPerEm
        advance = units * 2 // 3
        glyphs, metrics, cmap = {}, {}, {}
        for codepoint in sorted(codepoints | {32}):
            name = f'u{codepoint:06X}'
            source = text if codepoint in text.getBestCmap() else symbols
            original_name = source.getBestCmap()[codepoint]
            source_advance = source['hmtx'][original_name][0]
            scale = units / source['head'].unitsPerEm
            shift = round((advance - source_advance * scale) / 2)
            if source is text:
                glyph = deepcopy(source['glyf'][original_name])
                if glyph.isComposite():
                    raise ValueError('Expected independent MDV Pixel Text outlines.')
                if glyph.numberOfContours:
                    glyph.coordinates.translate((shift, 0))
            else:
                recorded = DecomposingRecordingPen(source.getGlyphSet())
                source.getGlyphSet()[original_name].draw(recorded)
                pen = TTGlyphPen(None)
                recorded.replay(TransformPen(pen, (scale, 0, 0, scale, shift, 0)))
                glyph = pen.glyph()
            glyphs[name] = glyph
            glyph.recalcBounds(glyphs)
            metrics[name] = (advance, glyph.xMin if glyph.numberOfContours else 0)
            cmap[codepoint] = name
        empty = TTGlyphPen(None).glyph()
        glyphs['.notdef'] = empty
        metrics['.notdef'] = (advance, 0)
        builder = FontBuilder(units, isTTF=True)
        builder.setupGlyphOrder(['.notdef', *cmap.values()])
        builder.setupCharacterMap(cmap)
        builder.setupGlyf(glyphs)
        builder.setupHorizontalMetrics(metrics)
        builder.setupHorizontalHeader(ascent=1224, descent=-324)
        builder.setupOS2(sTypoAscender=1224, sTypoDescender=-324, usWinAscent=1224, usWinDescent=324, usWeightClass=weight, fsSelection=0x20 if weight == 700 else 0x40)
        builder.font['head'].macStyle = 1 if weight == 700 else 0
        builder.font['head'].created = text['head'].created
        builder.font['head'].modified = text['head'].modified
        builder.font.recalcTimestamp = False
        builder.setupNameTable({
            'familyName': 'MDV Preview Text', 'styleName': style.title(),
            'uniqueFontIdentifier': f'MDVPreviewText-{style}-1.3',
            'fullName': f'MDV Preview Text {style.title()}',
            'psName': f'MDVPreviewText-{style.title()}', 'version': 'Version 1.3',
            'copyright': text['name'].getDebugName(0) + '\n' + symbols['name'].getDebugName(0) + '\nCopyright 2022 The Noto Project Authors',
            'licenseDescription': 'SIL Open Font License, Version 1.1',
            'licenseInfoURL': 'https://openfontlicense.org/',
        })
        builder.setupPost(isFixedPitch=1)
        builder.setupMaxp()
        builder.font.flavor = 'woff2'
        destination = website / f'src/assets/fonts/mdv-preview-text-{style}.woff2'
        builder.font.save(destination)
        verify_preview_text(TTFont(destination), text, codepoints)
        print(f'{destination.name}: {destination.stat().st_size} bytes, preserved custom text outlines')


def verify_preview_text(preview, text, codepoints):
    advance = preview['head'].unitsPerEm * 2 // 3
    if set(preview.getBestCmap()) != codepoints | {32}:
        raise ValueError('Incomplete preview font coverage.')
    if any(width != advance for width, _ in preview['hmtx'].metrics.values()):
        raise ValueError('Preview characters must share one terminal cell width.')
    for codepoint in codepoints & text.getBestCmap().keys():
        original_name = text.getBestCmap()[codepoint]
        old = text['glyf'][original_name]
        new = preview['glyf'][preview.getBestCmap()[codepoint]]
        if not old.numberOfContours:
            continue
        shift = round((advance - text['hmtx'][original_name][0]) / 2)
        expected = [(x + shift, y) for x, y in old.coordinates]
        if expected != [tuple(point) for point in new.coordinates] or list(old.flags) != list(new.flags):
            raise ValueError(f'Custom letter shape changed: U+{codepoint:X}')
