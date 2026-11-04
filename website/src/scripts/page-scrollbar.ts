import { scrollbarGeometry, scrollPositionForThumb } from './scrollbar-geometry';

export function setupPageScrollbar() {
  const root = document.documentElement;
  const rail = document.querySelector<HTMLElement>('[data-page-scrollbar]')!;
  const thumb = rail.querySelector<HTMLElement>('[data-page-scrollbar-thumb]')!;
  const rightPanel = document.getElementById('docs-toc-panel');
  let pointer: { x: number; y: number } | undefined;
  let drag: { id: number; startY: number; startScroll: number } | undefined;
  let frame = 0;

  const blocked = () => rightPanel?.dataset.panelFloating === 'true' && rightPanel.dataset.panelOpen === 'true';
  const extent = () => Math.max(root.scrollHeight, document.body.scrollHeight);
  const geometry = () => scrollbarGeometry(window.innerHeight, extent(), window.scrollY, rail.clientHeight);

  function contains(x: number, y: number) {
    const bounds = rail.getBoundingClientRect();
    return !rail.hidden && x >= bounds.left && x <= bounds.right && y >= bounds.top && y <= bounds.bottom;
  }

  function updateHover() {
    rail.toggleAttribute('data-hovered', !!pointer && !blocked() && contains(pointer.x, pointer.y));
  }

  function finishDrag() {
    if (!drag) return;
    const id = drag.id;
    drag = undefined;
    delete rail.dataset.dragging;
    delete root.dataset.pageScrollbarDragging;
    if (thumb.hasPointerCapture(id)) thumb.releasePointerCapture(id);
    updateHover();
  }

  function update() {
    frame = 0;
    rail.hidden = extent() <= window.innerHeight + 1;
    if (rail.hidden) {
      finishDrag();
      rail.removeAttribute('data-hovered');
      return;
    }
    const { length, offset } = geometry();
    rail.style.setProperty('--page-scrollbar-thumb-height', `${length}px`);
    thumb.style.transform = `translateY(${offset}px)`;
    updateHover();
  }

  function scheduleUpdate() {
    if (!frame) frame = requestAnimationFrame(update);
  }

  function updatePointer(event: PointerEvent) {
    if (event.pointerType !== 'mouse' && event.pointerType !== 'pen') return;
    pointer = { x: event.clientX, y: event.clientY };
    updateHover();
  }

  function startDrag(event: PointerEvent, jump: boolean) {
    event.preventDefault();
    if (jump) {
      const { length, travel, maximum } = geometry();
      const offset = event.clientY - rail.getBoundingClientRect().top - length / 2;
      window.scrollTo({ top: scrollPositionForThumb(offset, travel, maximum), behavior: 'instant' });
    }
    drag = { id: event.pointerId, startY: event.clientY, startScroll: window.scrollY };
    rail.dataset.dragging = 'true';
    root.dataset.pageScrollbarDragging = 'true';
    thumb.setPointerCapture(event.pointerId);
    update();
  }

  document.addEventListener('pointermove', updatePointer, { passive: true });
  document.addEventListener('pointerout', (event) => {
    if (event.relatedTarget !== null || (event.pointerType !== 'mouse' && event.pointerType !== 'pen')) return;
    pointer = undefined;
    updateHover();
  });
  document.addEventListener('pointerdown', (event) => {
    if (event.button !== 0 || event.pointerType === 'touch' || blocked() || rail.hidden) return;
    updatePointer(event);
    if (!contains(event.clientX, event.clientY) || !(event.target instanceof Element)) return;
    if (thumb.contains(event.target)) startDrag(event, false);
    else if (!event.target.closest('a, button, input, textarea, select, summary, [data-docs-panel]')) startDrag(event, true);
  }, { capture: true });
  thumb.addEventListener('pointermove', (event) => {
    if (!drag || event.pointerId !== drag.id) return;
    const { maximum, travel } = geometry();
    const top = scrollPositionForThumb(event.clientY - drag.startY, travel, maximum, drag.startScroll);
    window.scrollTo({ top, behavior: 'instant' });
    scheduleUpdate();
  });
  thumb.addEventListener('pointerup', finishDrag);
  thumb.addEventListener('pointercancel', finishDrag);
  thumb.addEventListener('lostpointercapture', finishDrag);
  window.addEventListener('blur', () => {
    pointer = undefined;
    finishDrag();
    updateHover();
  });
  window.addEventListener('scroll', scheduleUpdate, { passive: true });
  window.addEventListener('resize', scheduleUpdate);
  const observer = new ResizeObserver(scheduleUpdate);
  observer.observe(document.body);
  observer.observe(rail);
  if (rightPanel) {
    new MutationObserver(() => {
      if (blocked()) finishDrag();
      updateHover();
    }).observe(rightPanel, { attributes: true, attributeFilter: ['data-panel-floating', 'data-panel-open'] });
  }
  document.fonts.addEventListener('loadingdone', scheduleUpdate);
  void document.fonts.ready.then(scheduleUpdate);
  update();
}
