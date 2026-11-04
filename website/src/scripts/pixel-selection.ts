import { intersectRect, subtractRect, selectionLines, selectionPath, type SelectionRect } from './selection-shape';
import { cloneSelectionText } from './selection-clone';

const namespace = 'http://www.w3.org/2000/svg';
const blockDisplays = new Set(['block', 'list-item', 'table-cell', 'flex', 'inline-flex', 'grid', 'inline-block']);

function textBlock(node: Text) {
  for (let element = node.parentElement; element && element !== document.body; element = element.parentElement) {
    if (blockDisplays.has(getComputedStyle(element).display)) return element;
  }
}

function visibleArea(block: HTMLElement) {
  let clip: SelectionRect = { left: 0, top: 0, right: document.documentElement.clientWidth, bottom: document.documentElement.clientHeight };
  for (let element: HTMLElement | null = block; element && element !== document.body; element = element.parentElement) {
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    const bounds = { left: rect.left + element.clientLeft, top: rect.top + element.clientTop, right: rect.left + element.clientLeft + element.clientWidth, bottom: rect.top + element.clientTop + element.clientHeight };
    const x = /auto|scroll|hidden|clip/.test(style.overflowX);
    const y = /auto|scroll|hidden|clip/.test(style.overflowY);
    if (x || y) clip = { left: x ? Math.max(clip.left, bounds.left) : clip.left, top: y ? Math.max(clip.top, bounds.top) : clip.top, right: x ? Math.min(clip.right, bounds.right) : clip.right, bottom: y ? Math.min(clip.bottom, bounds.bottom) : clip.bottom };
  }
  const header = document.querySelector<HTMLElement>('.site-header');
  if (header && !header.contains(block)) clip.top = Math.max(clip.top, header.getBoundingClientRect().bottom);
  return clip;
}

export function setupPixelSelection() {
  const layer = document.createElementNS(namespace, 'svg');
  layer.classList.add('pixel-selection-layer');
  layer.setAttribute('aria-hidden', 'true');
  document.body.append(layer);
  let pending = false;

  function clear() {
    layer.replaceChildren();
    delete document.documentElement.dataset.pixelSelection;
  }

  function render() {
    pending = false;
    const selection = document.getSelection();
    const active = document.activeElement;
    if (!selection || selection.isCollapsed || !selection.rangeCount || active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement || (active instanceof HTMLElement && active.isContentEditable) || matchMedia('(forced-colors: active)').matches) {
      clear();
      return;
    }
    const panels = Array.from(document.querySelectorAll<HTMLElement>('[data-docs-panel][data-panel-floating="true"]'))
      .filter((panel) => panel.checkVisibility({ visibilityProperty: true }) && Number.parseFloat(getComputedStyle(panel).opacity) > 0)
      .map((element) => ({ element, bounds: element.getBoundingClientRect() }));
    const groups = new Map<HTMLElement, SelectionRect[]>();
    for (let index = 0; index < selection.rangeCount; index++) {
      const selected = selection.getRangeAt(index);
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
        acceptNode(node) {
          if (node === layer || !selected.intersectsNode(node)) return NodeFilter.FILTER_REJECT;
          if (node instanceof Element && node.matches('script, style, template, input, textarea, [aria-hidden="true"], [inert]')) return NodeFilter.FILTER_REJECT;
          return node instanceof Text && node.parentElement?.checkVisibility({ visibilityProperty: true })
            ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP;
        },
      });
      while (walker.nextNode()) {
        const text = walker.currentNode as Text;
        const block = textBlock(text);
        if (!block) continue;
        const range = document.createRange();
        range.selectNodeContents(text);
        if (range.compareBoundaryPoints(Range.START_TO_START, selected) < 0) range.setStart(selected.startContainer, selected.startOffset);
        if (range.compareBoundaryPoints(Range.END_TO_END, selected) > 0) range.setEnd(selected.endContainer, selected.endOffset);
        const clip = visibleArea(block);
        let rectangles = Array.from(range.getClientRects()).map((rect) => intersectRect(rect, clip)).filter((rect): rect is SelectionRect => !!rect);
        for (const panel of panels) {
          if (!panel.element.contains(block)) rectangles = rectangles.flatMap((rect) => subtractRect(rect, panel.bounds));
        }
        if (rectangles.length) groups.set(block, [...groups.get(block) ?? [], ...rectangles]);
      }
    }
    const fragment = document.createDocumentFragment();
    const definitions = document.createElementNS(namespace, 'defs');
    fragment.append(definitions);
    let index = 0;
    for (const [block, rectangles] of groups) {
      const lines = selectionLines(rectangles);
      if (!lines.length) continue;
      const path = lines.map(selectionPath).join('');
      const clip = document.createElementNS(namespace, 'clipPath');
      clip.id = `pixel-selected-${index++}`;
      const shape = document.createElementNS(namespace, 'path');
      shape.setAttribute('d', path);
      for (const line of lines) {
        const rect = document.createElementNS(namespace, 'rect');
        rect.setAttribute('x', String(line.left));
        rect.setAttribute('y', String(line.top));
        rect.setAttribute('width', String(line.right - line.left));
        rect.setAttribute('height', String(line.bottom - line.top));
        clip.append(rect);
      }
      definitions.append(clip);
      const background = shape.cloneNode() as SVGPathElement;
      background.classList.add('pixel-selection-background');
      fragment.append(background);
      const rect = block.getBoundingClientRect();
      const group = document.createElementNS(namespace, 'g');
      group.setAttribute('clip-path', `url(#${clip.id})`);
      const content = document.createElementNS(namespace, 'foreignObject');
      content.setAttribute('x', String(rect.left - block.scrollLeft));
      content.setAttribute('y', String(rect.top - block.scrollTop));
      content.setAttribute('width', String(rect.width + block.scrollLeft));
      content.setAttribute('height', String(rect.height + block.scrollTop));
      content.append(cloneSelectionText(block, rect.width));
      group.append(content);
      fragment.append(group);
    }
    if (!index) { clear(); return; }
    layer.replaceChildren(fragment);
    const viewport = layer.getBoundingClientRect();
    layer.setAttribute('viewBox', `0 0 ${viewport.width} ${viewport.height}`);
    document.documentElement.dataset.pixelSelection = 'true';
  }

  function schedule() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(render);
  }

  document.addEventListener('selectionchange', schedule);
  document.addEventListener('scroll', schedule, { capture: true, passive: true });
  document.addEventListener('focusin', schedule);
  document.addEventListener('click', schedule);
  document.addEventListener('toggle', schedule, { capture: true });
  document.addEventListener('mdv:disclosure-change', schedule);
  window.addEventListener('resize', schedule);
  document.fonts.addEventListener('loadingdone', schedule);
}
