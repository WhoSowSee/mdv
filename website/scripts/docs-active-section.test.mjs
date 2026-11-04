import { expect, test } from 'bun:test';
import { activeSectionIndex } from '../src/scripts/docs-active-section';

test('tracks sections passing the sticky-header boundary in either direction', () => {
  expect(activeSectionIndex([200, 500, 900], 66, false)).toBe(0);
  expect(activeSectionIndex([-200, 65.5, 500], 66, false)).toBe(1);
  expect(activeSectionIndex([-600, -200, 65.5], 66, false)).toBe(2);
  expect(activeSectionIndex([65.5, 350, 800], 66, false)).toBe(0);
});

test('marks the final section at the document bottom even when it cannot reach the header', () => {
  expect(activeSectionIndex([-500, 100, 300], 66, true)).toBe(2);
  expect(activeSectionIndex([-500, 100, 300], 66, false)).toBe(0);
  expect(activeSectionIndex([], 66, false)).toBe(-1);
});

test('keeps the bottom-selected section while its content grows or scrolls further down', () => {
  const tops = [-2100, -1240, -600, 143];
  expect(activeSectionIndex(tops, 66, true)).toBe(3);
  expect(activeSectionIndex(tops, 66, false, 143)).toBe(3);
  expect(activeSectionIndex([-2120, -1260, -620, 123], 66, false, 143)).toBe(3);
});

test('releases the bottom selection when the final heading moves back below its entry position', () => {
  expect(activeSectionIndex([-2080, -1220, -580, 163], 66, false, 143)).toBe(2);
  expect(activeSectionIndex([-2080, -1220, -580, 163], 66, true, 143)).toBe(3);
});
