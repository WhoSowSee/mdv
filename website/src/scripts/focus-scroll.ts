interface ScrollPosition {
  element: HTMLElement;
  top: number;
  left: number;
  behavior: string;
  priority: string;
}

export function setupFocusScroll() {
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  let positions: ScrollPosition[] = [];
  let focused: Element | null = null;
  let frame = 0;

  function finish(animate: boolean) {
    cancelAnimationFrame(frame);
    frame = 0;
    const movedFocus = focused !== document.activeElement;
    const movements = positions.map((position) => ({
      ...position, targetTop: position.element.scrollTop, targetLeft: position.element.scrollLeft,
    }));
    positions = [];
    for (const position of movements) {
      if (position.behavior) position.element.style.setProperty('scroll-behavior', position.behavior, position.priority);
      else position.element.style.removeProperty('scroll-behavior');
    }
    if (!animate || !movedFocus || reducedMotion.matches) return;
    for (const { element, top, left, targetTop, targetLeft } of movements) {
      if (top === targetTop && left === targetLeft) continue;
      // Replace the browser's automatic focus scroll before its next paint.
      element.scrollTo({ top, left, behavior: 'instant' });
      element.scrollTo({ top: targetTop, left: targetLeft, behavior: 'smooth' });
    }
  }

  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Tab' || event.defaultPrevented || event.isComposing || event.ctrlKey || event.metaKey || event.altKey) return;
    for (const { element, top, left } of positions) element.scrollTo({ top, left, behavior: 'instant' });
    finish(false);
    if (reducedMotion.matches) return;
    focused = document.activeElement;
    const containers = new Set([
      document.scrollingElement as HTMLElement,
      ...document.querySelectorAll<HTMLElement>('[data-docs-vertical-scroll], .pixel-scroll-content, pre[tabindex], .table-frame, .final-command code'),
    ]);
    positions = [...containers].filter((element) => element.checkVisibility({ visibilityProperty: true })
      && (element.scrollHeight > element.clientHeight || element.scrollWidth > element.clientWidth))
      .map((element) => ({
        element, top: element.scrollTop, left: element.scrollLeft,
        behavior: element.style.getPropertyValue('scroll-behavior'),
        priority: element.style.getPropertyPriority('scroll-behavior'),
      }));
    for (const { element } of positions) element.style.setProperty('scroll-behavior', 'auto', 'important');
    frame = requestAnimationFrame(() => finish(true));
  }, { capture: true });

  document.addEventListener('pointerdown', () => finish(false), { capture: true });
  window.addEventListener('wheel', () => finish(false), { passive: true });
}
