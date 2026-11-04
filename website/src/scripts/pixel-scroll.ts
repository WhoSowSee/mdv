import { scrollbarGeometry, scrollPositionForThumb } from './scrollbar-geometry';

export function setupPixelScrollbar(content: HTMLElement, host: HTMLElement, vertical = false) {
  const rail = document.createElement('div');
  rail.className = vertical ? 'pixel-scroll-rail vertical' : 'pixel-scroll-rail';
  rail.setAttribute('aria-hidden', 'true');
  const thumb = document.createElement('div');
  thumb.className = 'pixel-scroll-thumb';
  rail.append(thumb);
  host.classList.add('pixel-scroll-host');
  host.append(rail);
  content.classList.add('pixel-scroll-content');
  let dragging = false;
  let pointerStart = 0;
  let scrollStart = 0;

  const extent = () => vertical ? content.scrollHeight : content.scrollWidth;
  const viewport = () => vertical ? content.clientHeight : content.clientWidth;
  const position = () => vertical ? content.scrollTop : content.scrollLeft;
  const trackSize = () => vertical ? rail.clientHeight : rail.clientWidth;
  const geometry = () => scrollbarGeometry(viewport(), extent(), position(), trackSize(), 24);
  const setPosition = (value: number) => {
    if (vertical) content.scrollTop = value;
    else content.scrollLeft = value;
  };

  function update() {
    if (vertical) {
      host.toggleAttribute('data-scroll-before', position() > 1);
      host.toggleAttribute('data-scroll-after', position() + viewport() < extent() - 1);
    }
    rail.hidden = extent() <= viewport() + 1;
    if (rail.hidden) return;
    const { length, offset } = geometry();
    thumb.style.setProperty('--thumb-length', `${length}px`);
    thumb.style.transform = vertical ? `translateY(${offset}px)` : `translateX(${offset}px)`;
  }

  rail.addEventListener('pointerdown', (event) => {
    if (event.button !== 0) return;
    event.preventDefault();
    if (event.target !== thumb) {
      const rect = rail.getBoundingClientRect();
      const coordinate = vertical ? event.clientY - rect.top : event.clientX - rect.left;
      const { length, travel, maximum } = geometry();
      setPosition(scrollPositionForThumb(coordinate - length / 2, travel, maximum));
    }
    pointerStart = vertical ? event.clientY : event.clientX;
    scrollStart = position();
    dragging = true;
    rail.dataset.dragging = 'true';
    rail.setPointerCapture(event.pointerId);
    update();
  });
  rail.addEventListener('pointermove', (event) => {
    if (!dragging) return;
    const coordinate = vertical ? event.clientY : event.clientX;
    const { travel, maximum } = geometry();
    setPosition(scrollPositionForThumb(coordinate - pointerStart, travel, maximum, scrollStart));
  });
  rail.addEventListener('lostpointercapture', () => {
    dragging = false;
    delete rail.dataset.dragging;
  });
  rail.addEventListener('wheel', (event) => {
    event.preventDefault();
    setPosition(position() + (vertical ? event.deltaY : event.deltaX || event.deltaY));
  }, { passive: false });
  content.addEventListener('scroll', update, { passive: true });
  const observer = new ResizeObserver(update);
  observer.observe(content);
  for (const child of content.children) observer.observe(child);
  new MutationObserver(update).observe(content, { childList: true, subtree: true, attributes: true, attributeFilter: ['hidden'] });
  document.fonts.addEventListener('loadingdone', update);
  void document.fonts.ready.then(update);
  update();
}
