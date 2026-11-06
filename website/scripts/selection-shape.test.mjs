import { expect, test } from 'bun:test';
import { selectionLines, selectionPath } from '../src/scripts/selection-shape';

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

test('reads range rectangles with coordinates inherited like DOMRect', () => {
  const first = Object.create(rect(5, 10, 20, 24));
  const second = Object.create(rect(20, 10, 35, 24));
  expect(selectionLines([first, second])).toEqual([rect(5, 10, 35, 24)]);
});

test('draws two square steps at each corner, including narrow selections', () => {
  expect(selectionPath(rect(0, 0, 60, 14))).toBe('M2,0H58V1H59V2H60V12H59V13H58V14H2V13H1V12H0V2H1V1H2Z');
  const narrow = selectionPath(rect(0, 0, 2, 14));
  expect(narrow).toStartWith('M1,0H1V0.5');
  expect(narrow).not.toMatch(/NaN|Infinity|[CQAS]/);
});
