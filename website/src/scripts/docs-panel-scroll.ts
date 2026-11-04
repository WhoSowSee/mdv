import { setupPixelScrollbar } from './pixel-scroll';

export function setupDocumentationPanelScrolling(root: HTMLElement) {
  const sidebar = root.querySelector<HTMLElement>('.docs-sidebar-content')!;
  let scheduled = false;

  function fit() {
    scheduled = false;
    const atEnd = sidebar.scrollTop > 0 && sidebar.scrollTop + sidebar.clientHeight >= sidebar.scrollHeight - 1;
    sidebar.style.removeProperty('max-height');
    const limit = Number.parseFloat(getComputedStyle(sidebar).maxHeight);
    if (!Number.isFinite(limit) || sidebar.scrollHeight <= limit + 1) return;
    const top = sidebar.getBoundingClientRect().top;
    const links = Array.from(sidebar.querySelectorAll<HTMLElement>('.docs-nav-group a, .docs-search-results a'));
    let height = limit;
    for (const link of links) {
      const rect = link.getBoundingClientRect();
      if (!rect.height) continue;
      const peek = rect.top - top + sidebar.scrollTop + Math.min(30, rect.height * .75);
      if (peek <= limit) height = peek;
    }
    sidebar.style.maxHeight = `${Math.floor(height)}px`;
    if (atEnd) sidebar.scrollTop = sidebar.scrollHeight;
  }

  function schedule() {
    if (scheduled) return;
    scheduled = true;
    requestAnimationFrame(fit);
  }

  const panels = root.querySelectorAll<HTMLElement>('[data-docs-vertical-scroll]');
  for (const content of panels) setupPixelScrollbar(content, content.parentElement!, true);
  for (const content of panels) {
    content.addEventListener('focusin', (event) => {
      const target = event.target;
      if (!(target instanceof HTMLElement)) return;
      const frame = content.getBoundingClientRect();
      const item = target.getBoundingClientRect();
      if (item.top < frame.top + 24) content.scrollTop -= frame.top + 24 - item.top;
      else if (item.bottom > frame.bottom - 24) content.scrollTop += item.bottom - frame.bottom + 24;
    });
  }
  window.addEventListener('resize', schedule);
  document.fonts.addEventListener('loadingdone', schedule);
  void document.fonts.ready.then(schedule);
  const observer = new ResizeObserver(schedule);
  for (const child of sidebar.children) observer.observe(child);
  schedule();
}
