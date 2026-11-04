import { setupFocusScroll } from './focus-scroll';

export function setupKeyboard() {
  setupFocusScroll();
  document.addEventListener('pointerdown', () => {
    document.documentElement.dataset.inputMethod = 'pointer';
  }, { capture: true });

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Tab' && !event.isComposing) {
      document.documentElement.dataset.inputMethod = 'keyboard';
    }
    if (event.key !== 'Escape' || event.defaultPrevented || event.isComposing) return;
    const focused = document.activeElement;
    if (focused instanceof HTMLElement) focused.blur();
  });
}
