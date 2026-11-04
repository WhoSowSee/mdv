export function activeSectionIndex(tops: readonly number[], boundary: number, atBottom: boolean, bottomSelectionTop?: number) {
  if (!tops.length) return -1;
  if (atBottom || (bottomSelectionTop !== undefined && tops[tops.length - 1] <= bottomSelectionTop + 1)) return tops.length - 1;
  let active = 0;
  for (const [index, top] of tops.entries()) {
    if (top <= boundary) active = index;
  }
  return active;
}

export function setupActiveDocumentationSection(prose: HTMLElement, outline: HTMLElement) {
  const sections = Array.from(outline.querySelectorAll<HTMLAnchorElement>('a[href]')).flatMap((link) => {
    const heading = document.getElementById(decodeURIComponent(new URL(link.href).hash.slice(1)));
    return heading ? [{ link, heading }] : [];
  });
  let active = -1;
  let destination = -1;
  let scheduled = false;
  let bottomSelectionTop: number | undefined;

  function update() {
    scheduled = false;
    const scrolling = document.scrollingElement!;
    const maximum = scrolling.scrollHeight - scrolling.clientHeight;
    const boundary = (Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop) || 0) + 1;
    const tops = sections.map(({ heading }) => heading.getBoundingClientRect().top);
    const atBottom = maximum > 1 && window.scrollY >= maximum - 1;
    if (atBottom) bottomSelectionTop = tops[tops.length - 1];
    const observed = activeSectionIndex(tops, boundary, atBottom, bottomSelectionTop);
    if (observed !== sections.length - 1) bottomSelectionTop = undefined;
    const next = destination >= 0 ? destination : observed;
    if (next === active) return;
    active = next;
    for (const [index, { link }] of sections.entries()) {
      if (index === active) link.setAttribute('aria-current', 'location');
      else link.removeAttribute('aria-current');
    }
  }

  function schedule() {
    if (scheduled) return;
    scheduled = true;
    requestAnimationFrame(update);
  }

  if (sections.length) {
    window.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', () => {
      bottomSelectionTop = undefined;
      schedule();
    });
    window.addEventListener('hashchange', schedule);
    document.fonts.addEventListener('loadingdone', schedule);
    void document.fonts.ready.then(schedule);
    new ResizeObserver(schedule).observe(prose);
    schedule();
  }

  return function setDestination(target?: HTMLElement) {
    const index = target ? sections.findLastIndex(({ heading }) =>
      heading === target || Boolean(heading.compareDocumentPosition(target) & Node.DOCUMENT_POSITION_FOLLOWING),
    ) : -1;
    if (destination === index) return;
    destination = index;
    update();
  };
}
