import { expect, test } from 'bun:test';
import { intersectRect, subtractRect, selectionLines, selectionPath } from '../src/scripts/selection-shape';

const rect = (left, top, right, bottom) => ({ left, top, right, bottom });

test('joins adjacent styled text and whitespace without joining separate lines', () => {
  const fragments = [rect(0, 20, 30, 34), rect(31, 20, 40, 34), rect(40, 20, 70, 34), rect(0, 42, 60, 56)];
  expect(selectionLines(fragments)).toEqual([rect(0, 20, 70, 34), rect(0, 42, 60, 56)]);
  expect(fragments[0]).toEqual(rect(0, 20, 30, 34));
});

test('retains gaps between columns and combines duplicate range rectangles', () => {
  expect(selectionLines([rect(0, 0, 10, 14), rect(0, 0, 10, 14), rect(50, 0, 70, 14)]))
    .toEqual([rect(0, 0, 10, 14), rect(50, 0, 70, 14)]);
});

test('clips scrolled text to the visible content area and ignores empty intersections', () => {
  expect(intersectRect(rect(-80, 10, 200, 24), rect(20, 0, 100, 50))).toEqual(rect(20, 10, 100, 24));
  expect(intersectRect(rect(0, -30, 20, -10), rect(0, 0, 100, 100))).toBeUndefined();
  expect(intersectRect(rect(20, 10, 20, 24), rect(0, 0, 100, 100))).toBeUndefined();
});

test('draws two square steps at each corner, including narrow selections', () => {
  expect(selectionPath(rect(0, 0, 60, 14))).toBe('M2,0H58V1H59V2H60V12H59V13H58V14H2V13H1V12H0V2H1V1H2Z');
  const narrow = selectionPath(rect(0, 0, 2, 14));
  expect(narrow).toStartWith('M1,0H1V0.5');
  expect(narrow).not.toMatch(/NaN|Infinity|[CQAS]/);
});

test('clips selection behind a left or right overlay without changing unobscured text', () => {
  const line = rect(0, 0, 100, 14);
  expect(subtractRect(line, rect(-10, -10, 35, 100))).toEqual([rect(35, 0, 100, 14)]);
  expect(subtractRect(line, rect(60, -10, 140, 50))).toEqual([rect(0, 0, 60, 14)]);
  expect(subtractRect(line, rect(200, 0, 240, 50))).toEqual([line]);
  expect(subtractRect(line, rect(-10, -10, 140, 50))).toEqual([]);
  expect(line).toEqual(rect(0, 0, 100, 14));
});

test('preserves selection above and below a partially covering overlay', () => {
  const cover = rect(30, 3, 70, 11);
  const pieces = subtractRect(rect(0, 0, 100, 14), cover);
  expect(pieces).toHaveLength(4);
  expect(pieces.reduce((area, piece) => area + (piece.right - piece.left) * (piece.bottom - piece.top), 0)).toBe(1080);
  for (const piece of pieces) expect(intersectRect(piece, cover)).toBeUndefined();
});
