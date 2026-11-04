import { expect, test } from 'bun:test';
import { createHoverDelay, panelContainsPointer, PANEL_OPEN_DELAY, PANEL_CLOSE_DELAY } from '../src/scripts/docs-panel-hover';

const viewport = { width: 800, height: 800 };

function fakeClock() {
  let time = 0;
  let sequence = 0;
  const tasks = new Map();
  return {
    setTimeout(action, delay) {
      const id = ++sequence;
      tasks.set(id, { action, time: time + delay });
      return id;
    },
    clearTimeout(id) { tasks.delete(id); },
    advance(duration) {
      const end = time + duration;
      while (true) {
        const next = [...tasks].filter(([, task]) => task.time <= end).sort((a, b) => a[1].time - b[1].time)[0];
        if (!next) break;
        time = next[1].time;
        tasks.delete(next[0]);
        next[1].action();
      }
      time = end;
    },
  };
}

test('opens after 500 ms of continuous hover and restarts after leaving', () => {
  const clock = fakeClock(), opened = [];
  const delay = createHoverDelay(PANEL_OPEN_DELAY, (panel) => opened.push(panel), clock);
  delay.schedule('left');
  clock.advance(499);
  expect(opened).toEqual([]);
  delay.cancel('left');
  clock.advance(100);
  delay.schedule('left');
  clock.advance(499);
  expect(opened).toEqual([]);
  clock.advance(1);
  expect(opened).toEqual(['left']);
});

test('closes after 300 ms outside without extending the delay on pointer movement', () => {
  const clock = fakeClock(), closed = [];
  const delay = createHoverDelay(PANEL_CLOSE_DELAY, (panel) => closed.push(panel), clock);
  delay.schedule('right');
  clock.advance(100);
  delay.schedule('right');
  clock.advance(199);
  expect(closed).toEqual([]);
  clock.advance(1);
  expect(closed).toEqual(['right']);
});

test('returning to the safe region cancels closing and the next exit starts a fresh delay', () => {
  const clock = fakeClock(), closed = [];
  const delay = createHoverDelay(PANEL_CLOSE_DELAY, (panel) => closed.push(panel), clock);
  delay.schedule('left');
  clock.advance(299);
  delay.cancel();
  clock.advance(1000);
  expect(closed).toEqual([]);
  delay.schedule('left');
  clock.advance(300);
  expect(closed).toEqual(['left']);
});

test('switching trigger candidates discards the old opening timer', () => {
  const clock = fakeClock(), opened = [];
  const delay = createHoverDelay(PANEL_OPEN_DELAY, (panel) => opened.push(panel), clock);
  delay.schedule('left');
  clock.advance(300);
  delay.schedule('right');
  delay.cancel('left');
  clock.advance(499);
  expect(opened).toEqual([]);
  clock.advance(1);
  expect(opened).toEqual(['right']);
});

test('keeps both outer edge gaps and a two-pixel content-side margin inside the hover region', () => {
  const left = { left: 4, right: 324, top: 80, bottom: 700 };
  const right = { left: 508, right: 796, top: 80, bottom: 700 };
  const leftTrigger = { left: 0, right: 20, top: 80, bottom: 700 };
  const rightTrigger = { left: 780, right: 800, top: 80, bottom: 700 };
  for (const [panel, trigger, side, points] of [
    [left, leftTrigger, 'left', [{ x: 0, y: 400 }, { x: 3, y: 400 }, { x: 326, y: 400 }, { x: 200, y: 78 }, { x: 200, y: 702 }]],
    [right, rightTrigger, 'right', [{ x: 800, y: 400 }, { x: 798, y: 400 }, { x: 506, y: 400 }, { x: 600, y: 78 }, { x: 600, y: 702 }]],
  ]) {
    for (const point of points) expect(panelContainsPointer(panel, trigger, point, side, viewport)).toBe(true);
  }
  expect(panelContainsPointer(left, leftTrigger, { x: 327, y: 400 }, 'left', viewport)).toBe(false);
  expect(panelContainsPointer(right, rightTrigger, { x: 505, y: 400 }, 'right', viewport)).toBe(false);
});

test('retains panels across the full top and bottom gaps within their horizontal span', () => {
  for (const [panel, trigger, side, x, outside] of [
    [{ left: 4, right: 324, top: 80, bottom: 700 }, { left: 0, right: 20, top: 80, bottom: 700 }, 'left', 200, 327],
    [{ left: 508, right: 796, top: 80, bottom: 700 }, { left: 780, right: 800, top: 80, bottom: 700 }, 'right', 600, 505],
  ]) {
    for (const y of [0, 30, 79, 701, 750, 799, 800]) {
      expect(panelContainsPointer(panel, trigger, { x, y }, side, viewport)).toBe(true);
      expect(panelContainsPointer(panel, trigger, { x: outside, y }, side, viewport)).toBe(false);
    }
    expect(panelContainsPointer(panel, trigger, { x, y: -1 }, side, viewport)).toBe(false);
    expect(panelContainsPointer(panel, trigger, { x, y: 801 }, side, viewport)).toBe(false);
  }
});
