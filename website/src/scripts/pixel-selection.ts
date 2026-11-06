import { selectionLines, selectionPath, type SelectionRect } from './selection-shape';
import { cloneSelectionText } from './selection-clone';

const namespace = 'http://www.w3.org/2000/svg';
const blockDisplays = new Set(['block', 'list-item', 'table-cell', 'flex', 'inline-flex', 'grid', 'inline-block']);

function textBlock(node: Text) {
  for (let element = node.parentElement; element && element !== document.body; element = element.parentElement) {
    if (blockDisplays.has(getComputedStyle(element).display)) return element;
  }
}

function createLayer(block: HTMLElement, lines: SelectionRect[]) {
  const left = Math.min(...lines.map((line) => line.left));
  const top = Math.min(...lines.map((line) => line.top));
  const width = Math.max(...lines.map((line) => line.right)) - left;
  const height = Math.max(...lines.map((line) => line.bottom)) - top;
  const bounds = block.getBoundingClientRect();
  const host = document.createElement('span');
  host.className = 'pixel-selection-layer';
  host.inert = true;
  host.style.left = `${left - bounds.left - block.clientLeft + block.scrollLeft}px`;
  host.style.top = `${top - bounds.top - block.clientTop + block.scrollTop}px`;
  // Keep copied glyphs out of the document's selection and text content.
  const root = host.attachShadow({ mode: 'open' });
  const svg = document.createElementNS(namespace, 'svg');
  svg.style.display = 'block';
  svg.setAttribute('width', String(width));
  svg.setAttribute('height', String(height));
  const clip = document.createElementNS(namespace, 'clipPath');
  clip.id = 'pixel-selected';
  const background = document.createElementNS(namespace, 'path');
  background.setAttribute('fill', 'var(--color-accent)');
  background.setAttribute('shape-rendering', 'crispEdges');
  let path = '';
  for (const line of lines) {
    const local = { left: line.left - left, top: line.top - top, right: line.right - left, bottom: line.bottom - top };
    path += selectionPath(local);
    const rect = document.createElementNS(namespace, 'rect');
    rect.setAttribute('x', String(local.left));
    rect.setAttribute('y', String(local.top));
    rect.setAttribute('width', String(local.right - local.left));
    rect.setAttribute('height', String(local.bottom - local.top));
    clip.append(rect);
  }
  background.setAttribute('d', path);
  const content = document.createElementNS(namespace, 'foreignObject');
  content.setAttribute('clip-path', 'url(#pixel-selected)');
  content.setAttribute('x', String(bounds.left - left - block.scrollLeft));
  content.setAttribute('y', String(bounds.top - top - block.scrollTop));
  content.setAttribute('width', String(bounds.width));
  content.setAttribute('height', String(bounds.height));
  content.setAttribute('overflow', 'visible');
  content.append(cloneSelectionText(block, bounds.width));
  svg.append(clip, background, content);
  root.append(svg);
  return host;
}

export function setupPixelSelection() {
  const layers = new Map<HTMLElement, HTMLElement>();
  const forcedColors = matchMedia('(forced-colors: active)');
  let pending = false;

  function render() {
    pending = false;
    for (const [block, layer] of layers) {
      layer.remove();
      delete block.dataset.pixelSelection;
    }
    layers.clear();
    const selection = document.getSelection();
    const active = document.activeElement;
    if (!selection || selection.isCollapsed || !selection.rangeCount || active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement || (active instanceof HTMLElement && active.isContentEditable) || forcedColors.matches) return;

    const groups = new Map<HTMLElement, SelectionRect[]>();
    for (let index = 0; index < selection.rangeCount; index++) {
      const selected = selection.getRangeAt(index);
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
        acceptNode(node) {
          if (!selected.intersectsNode(node)) return NodeFilter.FILTER_REJECT;
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
        const rectangles = groups.get(block) ?? [];
        for (const rect of range.getClientRects()) {
          if (rect.width > 0 && rect.height > 0) rectangles.push(rect);
        }
        if (rectangles.length) groups.set(block, rectangles);
      }
    }
    for (const [block, rectangles] of groups) {
      layers.set(block, createLayer(block, selectionLines(rectangles)));
    }
    for (const [block, layer] of layers) {
      block.dataset.pixelSelection = getComputedStyle(block).position === 'static' ? 'anchor' : '';
      block.append(layer);
    }
  }

  function schedule() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(render);
  }

  document.addEventListener('selectionchange', schedule);
  document.addEventListener('focusin', schedule);
  document.addEventListener('click', schedule);
  document.addEventListener('toggle', schedule, { capture: true });
  document.addEventListener('mdv:disclosure-change', schedule);
  window.addEventListener('resize', schedule);
  forcedColors.addEventListener('change', schedule);
  document.fonts.addEventListener('loadingdone', schedule);
}
