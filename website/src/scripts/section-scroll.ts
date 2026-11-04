export function sectionScrollDuration(distance: number) {
  return Math.min(560, 240 + Math.sqrt(Math.abs(distance)) * 5);
}

export function sectionScrollPosition(from: number, to: number, elapsed: number, duration: number) {
  const progress = Math.max(0, Math.min(1, elapsed / duration));
  const eased = (1 - Math.cos(Math.PI * progress)) / 2;
  return from + (to - from) * eased;
}

export function sectionScrollTop(targetTop: number, currentScroll: number, offset: number, maximum: number) {
  return Math.max(0, Math.min(maximum, targetTop + currentScroll - offset));
}

export function sectionScrollTarget(targetTop: number, offset: number) {
  const scrolling = document.scrollingElement!;
  return sectionScrollTop(targetTop, window.scrollY, offset, scrolling.scrollHeight - scrolling.clientHeight);
}

let documentLoaded: Promise<void> | undefined;
const pendingFonts = () => document.fonts.status === 'loading' ? document.fonts.ready : undefined;

export function pendingSectionLayout() {
  if (document.readyState === 'complete') return pendingFonts();
  documentLoaded ??= new Promise<void>((resolve) => {
    window.addEventListener('load', () => resolve(), { once: true });
  });
  return documentLoaded.then(pendingFonts);
}

const interruptKeys = new Set(['Tab', 'Escape', 'ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' ']);

export function isSectionScrollInterrupt(key: string) {
  return interruptKeys.has(key);
}

export function focusSectionTarget(target: HTMLElement) {
  if (!target.hasAttribute('tabindex')) {
    target.tabIndex = -1;
    target.addEventListener('blur', () => target.removeAttribute('tabindex'), { once: true });
  }
  target.focus({ preventScroll: true });
}

export function sectionNavigationLink(event: MouseEvent) {
  if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
  const link = event.target instanceof Element ? event.target.closest<HTMLAnchorElement>('a[href]') : null;
  if (link && !link.target && !link.hasAttribute('download')) return link;
}

export function createSectionScroller() {
  let frame = 0;

  function cancel() {
    cancelAnimationFrame(frame);
    frame = 0;
  }

  function scroll(top: number, complete: () => void) {
    cancel();
    const from = window.scrollY;
    if (Math.abs(top - from) < 1 || matchMedia('(prefers-reduced-motion: reduce)').matches) {
      window.scrollTo({ top, behavior: 'instant' });
      complete();
      return;
    }
    const duration = sectionScrollDuration(top - from);
    window.scrollTo({ top: from, behavior: 'instant' });
    const start = performance.now();
    function update(now: number) {
      const elapsed = now - start;
      window.scrollTo({ top: sectionScrollPosition(from, top, elapsed, duration), behavior: 'instant' });
      if (elapsed < duration) frame = requestAnimationFrame(update);
      else { frame = 0; complete(); }
    }
    frame = requestAnimationFrame(update);
  }

  return { scroll, cancel };
}
