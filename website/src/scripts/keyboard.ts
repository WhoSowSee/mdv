import { setupFocusScroll } from './focus-scroll';

const focusKeys = new Set(['Tab', 'Enter', ' ', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End']);

export function setupKeyboard() {
  setupFocusScroll();
  document.addEventListener('pointerdown', () => {
    document.documentElement.dataset.inputMethod = 'pointer';
  }, { capture: true });

  document.addEventListener('keydown', (event) => {
    if (focusKeys.has(event.key) && !event.isComposing && !event.ctrlKey && !event.metaKey && !event.altKey) {
      document.documentElement.dataset.inputMethod = 'keyboard';
    }
  }, { capture: true });

  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape' || event.defaultPrevented || event.isComposing) return;
    const focused = document.activeElement;
    if (focused instanceof HTMLElement) focused.blur();
  });
}
