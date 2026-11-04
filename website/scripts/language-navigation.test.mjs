import { expect, test } from 'bun:test';
import { languageReadingPosition, languageReadingScroll } from '../src/scripts/language-navigation';

const anchors = [
  { id: 'features', top: 300 },
  { id: 'install', top: 900 },
  { id: 'usage', top: 1500 },
];

test('captures the current reading position after scrolling down or back up', () => {
  expect(languageReadingPosition(anchors, 1200, 2100)).toEqual({ anchor: 'install', next: 'usage', progress: .5 });
  expect(languageReadingPosition(anchors, 450, 2100)).toEqual({ anchor: 'features', next: 'install', progress: .25 });
});

test('retains progress inside the current section when the translated layout changes height', () => {
  const position = languageReadingPosition(anchors, 600, 2100);
  const translated = [{ id: 'features', top: 420 }, { id: 'install', top: 1300 }, { id: 'usage', top: 1900 }];
  expect(languageReadingScroll(position, translated, 2600)).toBe(860);
  const returned = languageReadingPosition(translated, 860, 2600);
  expect(languageReadingScroll(returned, anchors, 2100)).toBe(600);
});

test('maps the actual documentation section to its translated anchor', () => {
  const position = languageReadingPosition([
    { id: 'установка', top: 200 }, { id: 'использование', top: 700 },
  ], 400, 1200, { установка: 'installation', использование: 'usage' });
  expect(position).toEqual({ anchor: 'installation', next: 'usage', progress: .4 });
  expect(languageReadingScroll(position, [{ id: 'installation', top: 180 }, { id: 'usage', top: 580 }], 1000)).toBe(340);
});

test('preserves the top, bottom and pages that fit without scrolling', () => {
  expect(languageReadingScroll(languageReadingPosition(anchors, 0, 2100), anchors, 2100)).toBe(0);
  expect(languageReadingScroll(languageReadingPosition(anchors, 2100, 2100), anchors, 2700)).toBe(2700);
  expect(languageReadingPosition([], 0, 0)).toEqual({ anchor: '', next: null, progress: 0 });
  expect(languageReadingScroll(languageReadingPosition([], 0, 0), [], 0)).toBe(0);
  expect(languageReadingPosition([{ id: 'hidden', top: 0 }, ...anchors, { id: 'footer', top: 2100 }], -50, 2100).progress).toBe(0);
});

test('rejects stale or invalid positions instead of scrolling to an unrelated section', () => {
  const position = { anchor: 'features', next: 'install', progress: .5 };
  expect(languageReadingScroll({ ...position, anchor: 'removed' }, anchors, 2100)).toBeUndefined();
  expect(languageReadingScroll({ ...position, next: 'removed' }, anchors, 2100)).toBeUndefined();
  for (const progress of [-1, 2, NaN, Infinity]) {
    expect(languageReadingScroll({ ...position, progress }, anchors, 2100)).toBeUndefined();
  }
  expect(languageReadingScroll({ ...position, anchor: 'usage' }, anchors, 2100)).toBeUndefined();
});
