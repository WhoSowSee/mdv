import { primarySearchMatch } from './docs-search-model';

export function documentationSearchRange(target: HTMLElement, query: string) {
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
  const match = primarySearchMatch(text, query);
  if (!match) return;
  const first = positions[match.start], last = positions[match.end - 1];
  const range = document.createRange();
  range.setStart(first.node, first.offset);
  range.setEnd(last.node, last.offset + 1);
  return range;
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
