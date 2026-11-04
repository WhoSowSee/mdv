import { scrollbarGeometry } from './scrollbar-geometry';

export function setupCommandScrollbars(root: HTMLElement) {
  for (const wrapper of root.querySelectorAll<HTMLElement>('[data-command-scroll]')) {
    const content = wrapper.querySelector<HTMLPreElement>('pre')!;
    const code = content.querySelector<HTMLElement>('code')!;
    const scrollbar = wrapper.querySelector<HTMLElement>('.command-scrollbar')!;
    const range = scrollbar.querySelector<HTMLInputElement>('input')!;

    function update() {
      scrollbar.hidden = content.scrollWidth <= content.clientWidth;
      if (scrollbar.hidden) return;
      const { maximum, length } = scrollbarGeometry(content.clientWidth, content.scrollWidth, content.scrollLeft, content.clientWidth, 24);
      range.max = String(maximum);
      range.value = String(content.scrollLeft);
      range.style.setProperty('--thumb-width', `${length}px`);
    }

    content.addEventListener('scroll', () => { range.value = String(content.scrollLeft); });
    range.addEventListener('input', () => { content.scrollLeft = range.valueAsNumber; });
    const observer = new ResizeObserver(update);
    observer.observe(content);
    observer.observe(code);
    update();
  }
}
