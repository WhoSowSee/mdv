import { createSectionScroller, focusSectionTarget, isSectionScrollInterrupt, pendingSectionLayout, sectionNavigationLink, sectionScrollTarget } from './section-scroll';

const sections = new Set(['features', 'install', 'usage', 'top']);

export function homeSectionDestination(href: string, current: string, home: string) {
  const page = new URL(current);
  const url = new URL(href, current);
  const homeUrl = new URL(home, current);
  if (url.origin !== page.origin || url.pathname !== homeUrl.pathname) return;
  const fragment = url.hash.slice(1);
  const id = fragment ? fragment : url.searchParams.get('section');
  if (!id || !sections.has(id)) return;
  return { url, id };
}

export function homeSectionUrl(destination: URL, id: string) {
  const url = new URL(destination);
  url.searchParams.delete('section');
  url.hash = id;
  return url;
}

export function setupHomeSectionNavigation() {
  const home = document.querySelector<HTMLElement>('.site-nav')!.dataset.homeUrl!;
  const scroller = createSectionScroller();
  let request = 0;

  function cancel() {
    request++;
    scroller.cancel();
  }

  function updateUrl(url: URL, replace: boolean) {
    const previous = location.href;
    if (url.href === previous) return;
    if (replace) history.replaceState(history.state, '', url);
    else history.pushState(history.state, '', url);
    if (url.hash !== new URL(previous).hash) {
      window.dispatchEvent(new HashChangeEvent('hashchange', { oldURL: previous, newURL: url.href }));
    }
  }

  async function navigate(destination: NonNullable<ReturnType<typeof homeSectionDestination>>, fromTop: boolean) {
    const target = document.getElementById(destination.id);
    if (!target) return;
    cancel();
    const current = request;
    if (fromTop) window.scrollTo({ top: 0, behavior: 'instant' });
    const pending = pendingSectionLayout();
    if (pending) await pending;
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    if (current !== request) return;
    if (fromTop) window.scrollTo({ top: 0, behavior: 'instant' });
    const offset = Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop)
      + Number.parseFloat(getComputedStyle(target).scrollMarginTop);
    const top = sectionScrollTarget(target.getBoundingClientRect().top, offset);
    const heading = destination.id === 'top'
      ? document.getElementById('hero-heading')!
      : target.querySelector<HTMLElement>('h2')!;
    heading.dataset.homeSectionFocus = '';
    focusSectionTarget(heading);
    scroller.scroll(top, () => {
      if (current === request) updateUrl(homeSectionUrl(destination.url, destination.id), true);
    });
  }

  document.addEventListener('pointerdown', (event) => {
    cancel();
    if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
    const link = event.target instanceof Element ? event.target.closest<HTMLAnchorElement>('a[href]') : null;
    const destination = link && homeSectionDestination(link.href, location.href, home);
    if (destination) link!.focus({ preventScroll: true });
  }, { capture: true });
  document.addEventListener('click', (event) => {
    const link = sectionNavigationLink(event);
    if (!link) return;
    const destination = homeSectionDestination(link.href, location.href, home);
    if (!destination || destination.url.pathname !== location.pathname) return;
    event.preventDefault();
    updateUrl(homeSectionUrl(destination.url, destination.id), false);
    void navigate(destination, false);
  });
  window.addEventListener('wheel', cancel, { passive: true });
  window.addEventListener('touchstart', cancel, { passive: true });
  window.addEventListener('resize', cancel);
  window.addEventListener('popstate', cancel);
  document.addEventListener('keydown', (event) => {
    if (isSectionScrollInterrupt(event.key)) cancel();
  });

  const initial = homeSectionDestination(location.href, location.href, home);
  if (initial && !initial.url.hash && initial.url.pathname === location.pathname) void navigate(initial, true);
}
