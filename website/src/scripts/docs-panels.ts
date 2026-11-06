import { selectionPath } from './selection-shape';
import { createHoverDelay, panelContainsPointer, PANEL_CLOSE_DELAY, PANEL_OPEN_DELAY } from './docs-panel-hover';

interface DocumentationPanel {
  element: HTMLElement;
  trigger: HTMLButtonElement;
  outline: SVGSVGElement;
  media: MediaQueryList;
  floating: boolean;
}

export function setupDocumentationPanels(root: HTMLElement) {
  const hover = matchMedia('(any-hover: hover)');
  const header = document.querySelector<HTMLElement>('.site-header')!;
  const panels = Array.from(root.querySelectorAll<HTMLElement>('[data-docs-panel]')).map((element): DocumentationPanel => ({
    element,
    trigger: root.querySelector<HTMLButtonElement>(`[data-docs-panel-trigger="${element.id}"]`)!,
    outline: element.querySelector<SVGSVGElement>('[data-docs-panel-outline]')!,
    media: matchMedia(element.dataset.docsPanelMedia!),
    floating: false,
  }));
  let active: DocumentationPanel | undefined;
  let pointer: { x: number; y: number } | undefined;
  const opening = createHoverDelay<DocumentationPanel>(PANEL_OPEN_DELAY, (panel) => {
    if (hover.matches && !document.hidden && panel.floating && panel.trigger.matches(':hover')
      && !document.documentElement.hasAttribute('data-page-scrollbar-dragging')) open(panel);
  });
  const closing = createHoverDelay<DocumentationPanel>(PANEL_CLOSE_DELAY, (panel) => {
    const focused = panel.element.contains(document.activeElement);
    const dragging = panel.element.querySelector('[data-dragging]');
    if (active === panel && !hovered(panel) && !focused && !dragging) close();
  });

  function render(panel: DocumentationPanel) {
    const open = panel.floating && active === panel;
    const changed = panel.element.dataset.panelFloating !== String(panel.floating)
      || panel.element.dataset.panelOpen !== String(open);
    panel.element.dataset.panelFloating = String(panel.floating);
    panel.element.dataset.panelOpen = String(open);
    panel.element.inert = panel.floating && !open;
    panel.trigger.hidden = !panel.floating;
    panel.trigger.setAttribute('aria-expanded', String(open));
    if (changed) document.dispatchEvent(new Event('mdv:disclosure-change'));
  }

  function close(restoreFocus = false) {
    closing.cancel();
    const panel = active;
    if (!panel) return;
    active = undefined;
    if (restoreFocus) panel.trigger.focus({ preventScroll: true });
    render(panel);
  }

  function open(panel: DocumentationPanel, keyboard = false) {
    if (!panel.floating) return;
    opening.cancel();
    closing.cancel();
    if (active !== panel) close();
    active = panel;
    render(panel);
    if (keyboard) panel.element.querySelector<HTMLElement>('input, a[href]')!.focus({ preventScroll: true });
  }

  function hovered(panel: DocumentationPanel) {
    if (!pointer) return false;
    return panelContainsPointer(panel.element.getBoundingClientRect(), panel.trigger.getBoundingClientRect(),
      pointer, panel.trigger.classList.contains('docs-panel-trigger-left') ? 'left' : 'right',
      { width: window.innerWidth, height: window.innerHeight });
  }

  function scheduleClosing(panel: DocumentationPanel) {
    if (active !== panel) return;
    if (hovered(panel)) closing.cancel();
    else closing.schedule(panel);
  }

  function updatePointer(event: PointerEvent) {
    if (event.pointerType !== 'mouse') return;
    pointer = { x: event.clientX, y: event.clientY };
    if (active) scheduleClosing(active);
  }

  function synchronize() {
    opening.cancel();
    const focused = document.activeElement;
    for (const panel of panels) {
      const wasFloating = panel.floating;
      panel.floating = panel.media.matches;
      if (!panel.floating && active === panel) close();
      if (panel.floating && !wasFloating && panel.element.contains(focused)) {
        if (active !== panel) close();
        active = panel;
      }
      render(panel);
    }
  }

  function position() {
    const top = `${Math.ceil(header.getBoundingClientRect().bottom) + 12}px`;
    if (root.style.getPropertyValue('--docs-panel-top') !== top) root.style.setProperty('--docs-panel-top', top);
  }

  const outlines = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const panel = panels.find((item) => item.element === entry.target)!;
      if (!panel.floating) continue;
      const width = Math.round(entry.target.getBoundingClientRect().width);
      const height = Math.round(entry.target.getBoundingClientRect().height);
      panel.outline.setAttribute('viewBox', `0 0 ${width} ${height}`);
      const outer = selectionPath({ left: 0, top: 0, right: width, bottom: height });
      const inner = selectionPath({ left: 1, top: 1, right: width - 1, bottom: height - 1 });
      panel.outline.querySelector('path')!.setAttribute('d', outer + inner);
    }
  });

  for (const panel of panels) {
    outlines.observe(panel.element);
    panel.media.addEventListener('change', synchronize);
    panel.trigger.addEventListener('pointerenter', (event) => {
      updatePointer(event);
      if (event.pointerType === 'mouse' && hover.matches && active !== panel
        && !document.documentElement.hasAttribute('data-page-scrollbar-dragging')) opening.schedule(panel);
    });
    panel.trigger.addEventListener('pointerleave', (event) => {
      if (event.pointerType !== 'mouse') return;
      opening.cancel(panel);
      updatePointer(event);
    });
    panel.trigger.addEventListener('click', (event) => {
      opening.cancel();
      const keyboard = event.detail === 0;
      if (active === panel && (keyboard || event.pointerType === 'touch' || !hover.matches)) close(true);
      else open(panel, keyboard);
    });
    panel.trigger.addEventListener('focus', () => {
      if (active === panel && document.documentElement.dataset.inputMethod === 'keyboard') close();
    });
    panel.element.addEventListener('pointerenter', updatePointer);
    panel.element.addEventListener('pointerleave', updatePointer);
    panel.element.addEventListener('focusout', () => queueMicrotask(() => scheduleClosing(panel)));
    panel.element.addEventListener('lostpointercapture', () => scheduleClosing(panel));
  }

  document.addEventListener('pointermove', updatePointer, { passive: true });
  document.addEventListener('pointerout', (event) => {
    if (event.pointerType !== 'mouse' || event.relatedTarget !== null) return;
    pointer = undefined;
    opening.cancel();
    if (active) scheduleClosing(active);
  });
  document.addEventListener('pointerdown', (event) => {
    opening.cancel();
    updatePointer(event);
    if (!active || !(event.target instanceof Node)) return;
    if (!active.element.contains(event.target) && !active.trigger.contains(event.target) && !hovered(active)) close();
  }, { capture: true });
  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    opening.cancel();
    if (!active) return;
    event.preventDefault();
    close(true);
  }, { capture: true });
  document.addEventListener('focusin', () => {
    if (active && !active.element.contains(document.activeElement)
      && document.activeElement !== active.trigger && !hovered(active)) close();
  });
  root.addEventListener('click', (event) => {
    if (active && event.target instanceof Element && active.element.contains(event.target)
      && event.target.closest('a[href]')) close();
  });
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) opening.cancel();
  });
  window.addEventListener('resize', () => {
    position();
    synchronize();
  });
  new ResizeObserver(position).observe(header);
  position();
  synchronize();
}
