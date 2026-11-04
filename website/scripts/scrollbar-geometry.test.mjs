import { expect, test } from 'bun:test';
import { scrollbarGeometry, scrollPositionForThumb } from '../src/scripts/scrollbar-geometry';

test('maps the beginning, middle and end of a document to the scrollbar track', () => {
  expect(scrollbarGeometry(800, 3200, 0, 784)).toEqual({ maximum: 2400, length: 196, travel: 588, offset: 0 });
  expect(scrollbarGeometry(800, 3200, 1200, 784).offset).toBe(294);
  expect(scrollbarGeometry(800, 3200, 2400, 784).offset).toBe(588);
  expect(scrollPositionForThumb(294, 588, 2400)).toBe(1200);
});

test('keeps the thumb usable on very long pages and clamps overscroll at both ends', () => {
  const first = scrollbarGeometry(800, 100000, -100, 784);
  expect(first.length).toBe(36);
  expect(first.offset).toBe(0);
  expect(scrollbarGeometry(800, 100000, 200000, 784).offset).toBe(748);
  expect(scrollPositionForThumb(-20, 748, 99200)).toBe(0);
  expect(scrollPositionForThumb(800, 748, 99200)).toBe(99200);
});

test('handles short documents and tracks smaller than the minimum thumb without division by zero', () => {
  expect(scrollbarGeometry(800, 400, 0, 784)).toEqual({ maximum: 0, length: 784, travel: 0, offset: 0 });
  expect(scrollbarGeometry(800, 3200, 100, 20)).toEqual({ maximum: 2400, length: 20, travel: 0, offset: 0 });
  expect(scrollPositionForThumb(10, 0, 2400)).toBe(0);
  expect(scrollPositionForThumb(10, 0, 2400, 300)).toBe(300);
});

test('uses the smaller element thumb and maps drags from their initial scroll position', () => {
  expect(scrollbarGeometry(200, 4000, 1000, 180, 24)).toEqual({ maximum: 3800, length: 24, travel: 156, offset: 156 * 1000 / 3800 });
  expect(scrollPositionForThumb(25, 100, 400, 200)).toBe(300);
  expect(scrollPositionForThumb(-25, 100, 400, 200)).toBe(100);
  expect(scrollPositionForThumb(-100, 100, 400, 200)).toBe(0);
  expect(scrollPositionForThumb(100, 100, 400, 200)).toBe(400);
});
