export function setupAnimatedDetails(details: HTMLDetailsElement, content: HTMLElement) {
  const summary = details.querySelector<HTMLElement>(':scope > summary')!;
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  let expanded = details.open;
  let animation: Animation | undefined;
  let frame = 0;
  details.dataset.expanded = String(expanded);
  summary.setAttribute('aria-expanded', String(expanded));

  function updateSelection() {
    document.dispatchEvent(new Event('mdv:disclosure-change'));
    if (animation) frame = requestAnimationFrame(updateSelection);
  }

  function finish() {
    animation?.cancel();
    animation = undefined;
    cancelAnimationFrame(frame);
    details.open = expanded;
    content.inert = false;
    document.dispatchEvent(new Event('mdv:disclosure-change'));
  }

  details.addEventListener('toggle', () => {
    if (animation) return;
    expanded = details.open;
    details.dataset.expanded = String(expanded);
    summary.setAttribute('aria-expanded', String(expanded));
  });

  details.addEventListener('mdv:details-open', () => {
    expanded = true;
    details.dataset.expanded = 'true';
    summary.setAttribute('aria-expanded', 'true');
    finish();
  });

  summary.addEventListener('click', (event) => {
    event.preventDefault();
    const height = details.open ? content.getBoundingClientRect().height : 0;
    const opacity = details.open ? getComputedStyle(content).opacity : '0';
    animation?.cancel();
    cancelAnimationFrame(frame);
    expanded = !expanded;
    details.dataset.expanded = String(expanded);
    summary.setAttribute('aria-expanded', String(expanded));
    content.inert = !expanded;
    if (reducedMotion.matches) {
      finish();
      return;
    }
    details.open = true;
    const targetHeight = expanded ? content.getBoundingClientRect().height : 0;
    const current = content.animate([
      { height: `${height}px`, opacity },
      { height: `${targetHeight}px`, opacity: expanded ? 1 : 0 },
    ], { duration: 220, easing: 'ease-in-out', fill: 'both' });
    animation = current;
    current.onfinish = () => { if (animation === current) finish(); };
    updateSelection();
  });
  reducedMotion.addEventListener('change', () => {
    if (reducedMotion.matches) finish();
  });
}
