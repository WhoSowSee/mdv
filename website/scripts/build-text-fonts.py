from hashlib import sha256
from math import ceil, floor
from pathlib import Path
import unicodedata

from fontTools.feaLib.builder import addOpenTypeFeaturesFromString
from fontTools.misc.fixedTools import floatToFixed
from fontTools.pens.basePen import BasePen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont


FONT_DIR = Path(__file__).resolve().parents[1] / "src/assets/fonts"
FAMILY = "MDV Pixel Text"
COMMA_DROP = 120
PIPE_EXTENSION = 120
SOURCE_HASHES = {
    "regular": "f6e886d8380bd9e22adbbe1b5ecc1b09427cfe160edf41c1af79b285daf41a8b",
    "bold": "aeb0586c1c750f8813631d4e2ca3d17dafcfea362977c4dd3a5a3ca52c883b12",
}


class OutlinePen(BasePen):
    def __init__(self, glyph_set):
        super().__init__(glyph_set)
        self.segments = []

    def _moveTo(self, point):
        self.start = self.point = point

    def _lineTo(self, point):
        self.segments.append((self.point, point))
        self.point = point

    def _qCurveToOne(self, control, end):
        start = self.point
        for step in range(1, 17):
            t = step / 16
            self._lineTo(tuple(
                (1 - t) ** 2 * start[axis]
                + 2 * (1 - t) * t * control[axis] + t ** 2 * end[axis]
                for axis in (0, 1)
            ))

    def _curveToOne(self, first, second, end):
        start = self.point
        for step in range(1, 17):
            t = step / 16
            self._lineTo(tuple(
                (1 - t) ** 3 * start[axis]
                + 3 * (1 - t) ** 2 * t * first[axis]
                + 3 * (1 - t) * t ** 2 * second[axis] + t ** 3 * end[axis]
                for axis in (0, 1)
            ))

    def _closePath(self):
        self._lineTo(self.start)


def profiles(font, names):
    glyph_set = font.getGlyphSet()
    result = {}
    for name in names:
        pen = OutlinePen(glyph_set)
        glyph_set[name].draw(pen)
        rows = {}
        for row in range(-360, 1440, 30):
            y = row + 15
            intersections = []
            for (x1, y1), (x2, y2) in pen.segments:
                if min(y1, y2) <= y < max(y1, y2):
                    intersections.append(x1 + (y - y1) * (x2 - x1) / (y2 - y1))
            if intersections:
                rows[row] = (min(intersections), max(intersections))
        result[name] = rows
    return result


def add_kerning(font, gap):
    names = sorted({
        name for codepoint, name in font.getBestCmap().items()
        if (unicodedata.name(chr(codepoint), "").startswith(("LATIN ", "CYRILLIC "))
            or 33 <= codepoint <= 126)
        and font["hmtx"][name][0] > 0
    })
    outlines = profiles(font, names)
    rules = []
    for left in names:
        advance = font["hmtx"][left][0]
        for right in names:
            distances = [
                advance + outlines[right][row][0] - bounds[1]
                for row, bounds in outlines[left].items()
                if row in outlines[right]
            ]
            if not distances:
                continue
            extra = min(distances) - gap
            adjustment = min(120, floor(max(0, extra) / 2 / 60) * 60)
            if adjustment:
                rules.append(f"pos {left} {right} -{adjustment};")
    feature = "\n".join([
        "languagesystem DFLT dflt;",
        "languagesystem latn dflt;",
        "languagesystem cyrl dflt;",
        "feature kern {", *rules, "} kern;",
    ])
    addOpenTypeFeaturesFromString(font, feature, tables=["GPOS"])
    return len(rules)


def lowercase_i(style):
    expansion = 24 if style == "bold" else 0
    pen = TTGlyphPen(None)
    left, right = 180 - expansion, 300 + expansion
    contours = (
        [(left, -expansion), (left, 480 - expansion),
         (60 - expansion, 480 - expansion), (60 - expansion, 600 + expansion),
         (right, 600 + expansion), (right, -expansion)],
        [(left, 720 - expansion), (left, 840 + expansion),
         (right, 840 + expansion), (right, 720 - expansion)],
    )
    for contour in contours:
        pen.moveTo(contour[0])
        for point in contour[1:]:
            pen.lineTo(point)
        pen.closePath()
    return pen.glyph()


def extend_pipe(glyph, glyf):
    glyph.recalcBounds(glyf)
    bottom, top = glyph.yMin, glyph.yMax
    for index, (x, y) in enumerate(glyph.coordinates):
        if y == bottom:
            glyph.coordinates[index] = (x, y - PIPE_EXTENSION)
        elif y == top:
            glyph.coordinates[index] = (x, y + PIPE_EXTENSION)
    glyph.recalcBounds(glyf)


def rename(font, style):
    label = style.title()
    values = {
        1: FAMILY,
        2: label,
        3: f"MDVPixelText-1.004-{label}",
        4: f"{FAMILY} {label}",
        5: "Version 1.004; proportional derivative of Monocraft 4.2",
        6: f"MDVPixelText-{label}",
        10: "Proportional derivative of Monocraft 4.2. Lowercase i redrawn, comma lowered, vertical bar extended; spacing and kerning modified.",
        13: "Licensed under the SIL Open Font License, Version 1.1. See monocraft-license.txt.",
        14: "https://openfontlicense.org/",
        16: FAMILY,
        17: label,
    }
    font["name"].names = [
        record for record in font["name"].names if record.nameID not in values
    ]
    for name_id, value in values.items():
        for platform, encoding, language in ((3, 1, 0x409), (1, 0, 0)):
            font["name"].setName(value, name_id, platform, encoding, language)
    font["post"].isFixedPitch = 0
    font["head"].fontRevision = 1.004
    # Derive the WOFF2 container version from the renamed font, not its source.
    font.flavorData.majorVersion = None
    font.flavorData.minorVersion = None
    font["OS/2"].panose.bProportion = 3
    font["OS/2"].recalcAvgCharWidth(font)


def verify(source, destination, style):
    modified = TTFont(destination, checkChecksums=2)
    if modified.getBestCmap() != source.getBestCmap():
        raise ValueError("Character coverage changed")
    for tag in ("GSUB", "GDEF"):
        if source[tag].compile(source) != modified[tag].compile(modified):
            raise ValueError(f"Character substitution behavior changed: {tag}")
    i_name = source.getBestCmap()[ord("i")]
    comma_name = source.getBestCmap()[ord(",")]
    pipe_name = source.getBestCmap()[ord("|")]
    for name in source.getGlyphOrder():
        expected = lowercase_i(style) if name == i_name else source["glyf"][name]
        old, old_ends, old_flags = expected.getCoordinates(source["glyf"])
        new, new_ends, new_flags = modified["glyf"][name].getCoordinates(modified["glyf"])
        if old_ends != new_ends or old_flags != new_flags or len(old) != len(new):
            raise ValueError(f"Glyph contours changed: {name}")
        if old:
            delta = new[0][0] - old[0][0]
            vertical_shift = -COMMA_DROP if name == comma_name else 0
            for (x, y), point in zip(old, new):
                expected_y = y + vertical_shift
                if name == pipe_name:
                    if y == expected.yMin:
                        expected_y -= PIPE_EXTENSION
                    elif y == expected.yMax:
                        expected_y += PIPE_EXTENSION
                if (x + delta, expected_y) != tuple(point):
                    raise ValueError(f"Glyph shape changed: {name}")
    for tag, attributes in {
        "head": ("unitsPerEm",),
        "hhea": ("ascent", "descent", "lineGap"),
        "OS/2": ("sTypoAscender", "sTypoDescender", "sTypoLineGap", "usWinAscent", "usWinDescent"),
    }.items():
        if any(getattr(source[tag], attr) != getattr(modified[tag], attr) for attr in attributes):
            raise ValueError(f"Vertical metrics changed: {tag}")
    if modified["post"].isFixedPitch or modified["name"].getDebugName(1) != FAMILY:
        raise ValueError("Derivative font metadata is incorrect")
    container_revision = modified.flavorData.majorVersion * 65536 + modified.flavorData.minorVersion
    if container_revision != floatToFixed(modified["head"].fontRevision, 16):
        raise ValueError("WOFF2 container version differs from the font revision")
    if not any(record.FeatureTag == "kern" for record in modified["GPOS"].table.FeatureList.FeatureRecord):
        raise ValueError("Kerning feature is missing")


def build(style, expected_hash):
    source_path = FONT_DIR / f"monocraft-{style}.woff2"
    if sha256(source_path.read_bytes()).hexdigest() != expected_hash:
        raise ValueError(f"Unexpected source font: {source_path.name}")
    source = TTFont(source_path)
    font = TTFont(source_path, recalcTimestamp=False)
    glyf = font["glyf"]
    glyf[font.getBestCmap()[ord("i")]] = lowercase_i(style)
    bearing = 60 if style == "regular" else 36
    gap = bearing * 2
    whitespace = {}
    cmap = font.getBestCmap()
    for codepoint in (0x20, 0xA0):
        whitespace[cmap[codepoint]] = 360
    for name in font.getGlyphOrder():
        glyph = glyf[name]
        advance, _ = font["hmtx"][name]
        if glyph.isComposite():
            raise ValueError("Expected the pinned Monocraft outlines without composites")
        if name in whitespace:
            font["hmtx"][name] = (whitespace[name], 0)
        elif advance and glyph.numberOfContours:
            glyph.recalcBounds(glyf)
            width = glyph.xMax - glyph.xMin
            glyph.coordinates.translate((bearing - glyph.xMin, 0))
            glyph.recalcBounds(glyf)
            font["hmtx"][name] = (ceil(width + gap), bearing)
        glyph.removeHinting()
    for tag in ("fpgm", "prep", "cvt "):
        if tag in font:
            del font[tag]
    rename(font, style)
    pairs = add_kerning(font, gap)
    # Keep pair spacing independent of the comma and pipe's vertical edits.
    comma = glyf[cmap[ord(",")]]
    comma.coordinates.translate((0, -COMMA_DROP))
    comma.recalcBounds(glyf)
    extend_pipe(glyf[cmap[ord("|")]], glyf)
    destination = FONT_DIR / f"mdv-pixel-text-{style}.woff2"
    font.save(destination)
    verify(source, destination, style)
    print(f"{destination.name}: lowercase i redrawn, comma lowered, pipe extended, {len(font.getGlyphOrder()) - 2} preserved shapes, {pairs} kerning pairs")


if __name__ == "__main__":
    for style, expected_hash in SOURCE_HASHES.items():
        build(style, expected_hash)
