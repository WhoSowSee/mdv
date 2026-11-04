import { expect, test } from 'bun:test';
import { sectionScrollDuration, sectionScrollPosition, sectionScrollTop } from '../src/scripts/section-scroll';

test('aligns the heading below the sticky header from any current scroll position', () => {
  expect(sectionScrollTop(1800, 0, 65, 4000)).toBe(1735);
  expect(sectionScrollTop(-1200, 3000, 65, 4000)).toBe(1735);
  expect(sectionScrollTop(1800, 0, 77, 4000)).toBe(1723);
  expect(sectionScrollTop(20, 0, 65, 4000)).toBe(0);
  expect(sectionScrollTop(4200, 0, 65, 4000)).toBe(4000);
});

test('keeps section transitions short for nearby and distant targets', () => {
  expect(sectionScrollDuration(100)).toBeGreaterThanOrEqual(240);
  expect(sectionScrollDuration(100)).toBeLessThan(sectionScrollDuration(2000));
  expect(sectionScrollDuration(20000)).toBe(560);
  expect(sectionScrollDuration(-2000)).toBe(sectionScrollDuration(2000));
});

test('accelerates and stops smoothly without overshooting in either direction', () => {
  for (const [from, to] of [[0, 2000], [2400, 100]]) {
    expect(sectionScrollPosition(from, to, -10, 400)).toBe(from);
    expect(sectionScrollPosition(from, to, 200, 400)).toBeCloseTo((from + to) / 2);
    expect(sectionScrollPosition(from, to, 600, 400)).toBe(to);
    const positions = Array.from({ length: 26 }, (_, index) => sectionScrollPosition(from, to, index * 16, 400));
    for (let index = 1; index < positions.length; index++) {
      expect((positions[index] - positions[index - 1]) * Math.sign(to - from)).toBeGreaterThanOrEqual(0);
    }
    const first = Math.abs(positions[1] - positions[0]);
    const middle = Math.abs(positions[13] - positions[12]);
    const last = Math.abs(positions[25] - positions[24]);
    expect(first).toBeLessThan(middle);
    expect(last).toBeLessThan(middle);
  }
});
