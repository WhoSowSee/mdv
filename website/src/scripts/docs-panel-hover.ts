export const PANEL_OPEN_DELAY = 500;
export const PANEL_CLOSE_DELAY = 300;

interface Bounds { left: number; top: number; right: number; bottom: number; }
interface Point { x: number; y: number; }
interface Clock {
  setTimeout(action: () => void, delay: number): number;
  clearTimeout(timer: number): void;
}

export function panelContainsPointer(
  panel: Bounds, trigger: Bounds, point: Point, side: 'left' | 'right', viewport: { width: number; height: number },
) {
  const bounds = {
    left: side === 'left' ? 0 : panel.left - 2,
    right: side === 'right' ? viewport.width : panel.right + 2,
    top: 0,
    bottom: viewport.height,
  };
  const contains = (area: Bounds) => point.x >= area.left && point.x <= area.right
    && point.y >= area.top && point.y <= area.bottom;
  return contains(bounds) || contains(trigger);
}

export function createHoverDelay<T>(delay: number, action: (value: T) => void, clock: Clock = window) {
  let pending: { value: T; timer: number } | undefined;

  function cancel(value?: T) {
    if (!pending || (value !== undefined && pending.value !== value)) return;
    clock.clearTimeout(pending.timer);
    pending = undefined;
  }

  function schedule(value: T) {
    if (pending?.value === value) return;
    cancel();
    const timer = clock.setTimeout(() => {
      pending = undefined;
      action(value);
    }, delay);
    pending = { value, timer };
  }

  return { schedule, cancel };
}
