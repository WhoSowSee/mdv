import { primarySearchMatch, searchHighlightMatches } from './docs-search-model';

export function documentationSearchRanges(target: HTMLElement, query: string) {
  const walker = document.createTreeWalker(target, NodeFilter.SHOW_TEXT);
  const positions: { node: Text; offset: number }[] = [];
  let text = '';
  let node: Node | null;
  while ((node = walker.nextNode())) {
    const parent = node.parentElement!;
    if (parent.closest('script, style, [aria-hidden="true"]') || parent.closest('[data-doc-search-id]') !== target) continue;
    const value = node.textContent!;
    for (let offset = 0; offset < value.length; offset++) {
      const space = /\s/.test(value[offset]);
      if (space && text.endsWith(' ')) continue;
      text += space ? ' ' : value[offset];
      positions.push({ node: node as Text, offset });
    }
  }
  const primary = primarySearchMatch(text, query);
  if (!primary) return;

  function toRange({ start, end }: { start: number; end: number }) {
    const first = positions[start], last = positions[end - 1];
    const range = document.createRange();
    range.setStart(first.node, first.offset);
    range.setEnd(last.node, last.offset + 1);
    return range;
  }

  const primaryRange = toRange(primary);
  const matches = searchHighlightMatches(text, query).map((match) =>
    match.start === primary.start && match.end === primary.end ? primaryRange : toRange(match));
  return { primary: primaryRange, matches };
}

const searchHighlight = 'docs-search-match';
let highlightAnimation: Animation | undefined;

export function setDocumentationSearchHighlight(target?: HTMLElement, ranges?: Range[]) {
  const registry = CSS.highlights;
  if (!registry) return;

  highlightAnimation?.cancel();
  highlightAnimation = undefined;
  if (!target || !ranges?.length) {
    registry.delete(searchHighlight);
    return;
  }

  const fade = !matchMedia('(prefers-reduced-motion: reduce), (forced-colors: active)').matches
    && !document.documentElement.classList.contains('motion-paused');
  const hidden = { '--docs-search-match-opacity': '0%' };
  const visible = { '--docs-search-match-opacity': '100%' };
  const easing = 'cubic-bezier(.25, 0, .75, 1)';
  const animation = target.animate(fade ? [
    { ...hidden, easing },
    { ...visible, offset: .25 },
    { ...visible, offset: .5, easing },
    hidden,
  ] : [visible, visible], { duration: fade ? 1650 : 1750, fill: 'backwards' });

  highlightAnimation = animation;
  animation.onfinish = () => {
    if (highlightAnimation !== animation) return;
    registry.delete(searchHighlight);
    highlightAnimation = undefined;
  };
  registry.set(searchHighlight, new Highlight(...ranges));
}

export function revealDocumentationSearchTarget(target: HTMLElement) {
  const ancestors: HTMLDetailsElement[] = [];
  for (let parent = target.parentElement; parent; parent = parent.parentElement) {
    if (parent instanceof HTMLDetailsElement) ancestors.unshift(parent);
  }
  for (const details of ancestors) {
    details.dispatchEvent(new Event('mdv:details-open'));
    details.open = true;
  }
}

export function revealSearchRangeHorizontally(target: HTMLElement, range: Range) {
  for (let element: HTMLElement | null = target; element; element = element.parentElement) {
    if (element.scrollWidth <= element.clientWidth + 1 || !/auto|scroll/.test(getComputedStyle(element).overflowX)) continue;
    const bounds = element.getBoundingClientRect(), match = range.getBoundingClientRect();
    if (match.left < bounds.left + 32 || match.right > bounds.right - 32) {
      const offset = match.width > bounds.width - 64
        ? match.left - bounds.left - 32
        : (match.left + match.right - bounds.left - bounds.right) / 2;
      element.scrollTo({ left: element.scrollLeft + offset, behavior: 'instant' });
    }
  }
}
