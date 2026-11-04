import { setupActiveDocumentationSection } from './docs-active-section';
import { documentationSearchDestination } from './docs-search-model';
import { documentationSearchRange, revealDocumentationSearchTarget, revealSearchRangeHorizontally } from './docs-search-position';
import { createSectionScroller, focusSectionTarget, isSectionScrollInterrupt, pendingSectionLayout, sectionNavigationLink, sectionScrollTarget } from './section-scroll';

export function documentationAnchor(href: string, current: string) {
  const destination = new URL(href, current);
  const page = new URL(current);
  if (destination.origin !== page.origin || destination.pathname !== page.pathname || destination.search !== page.search || !destination.hash) return;
  return { url: destination, id: decodeURIComponent(destination.hash.slice(1)) };
}

export function setupDocumentationSectionNavigation(root: HTMLElement) {
  const outline = root.querySelector<HTMLElement>('.docs-toc')!;
  const prose = root.querySelector<HTMLElement>('.docs-prose')!;
  const results = root.querySelector<HTMLElement>('.docs-search-results')!;
  const setDestination = setupActiveDocumentationSection(prose, outline);
  const scroller = createSectionScroller();
  let request = 0;
  let source: 'outline' | 'search' | undefined;

  function cancel() {
    request++;
    scroller.cancel();
    source = undefined;
    setDestination();
  }

  function scroll(top: number, current: number) {
    scroller.scroll(top, () => {
      requestAnimationFrame(() => {
        if (current !== request) return;
        source = undefined;
        setDestination();
      });
    });
  }

  window.addEventListener('wheel', cancel, { passive: true });
  window.addEventListener('resize', cancel);
  window.addEventListener('popstate', cancel);
  window.addEventListener('touchstart', () => {
    if (source === 'search') cancel();
  }, { passive: true });
  document.addEventListener('pointerdown', (event) => {
    const link = event.target instanceof Element ? event.target.closest('a[href]') : null;
    if (source === 'search' || !link || !outline.contains(link)) cancel();
  }, { capture: true });
  document.addEventListener('keydown', (event) => {
    if (isSectionScrollInterrupt(event.key)) cancel();
  });
  outline.addEventListener('pointerdown', (event) => {
    if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
    const link = event.target instanceof Element ? event.target.closest<HTMLAnchorElement>('a[href]') : null;
    link?.focus({ preventScroll: true });
  });
  outline.addEventListener('click', async (event) => {
    const link = sectionNavigationLink(event);
    if (!link) return;
    const destination = documentationAnchor(link.href, location.href);
    const target = destination && document.getElementById(destination.id);
    if (!destination || !target) return;
    event.preventDefault();
    scroller.cancel();
    const current = ++request;
    source = 'outline';
    setDestination(target);
    const pending = pendingSectionLayout();
    if (pending) await pending;
    if (current !== request) return;
    const padding = Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop) || 0;
    const margin = Number.parseFloat(getComputedStyle(target).scrollMarginTop) || 0;
    const top = sectionScrollTarget(target.getBoundingClientRect().top, padding + margin);
    const previous = location.href;
    destination.url.searchParams.delete('match');
    if (destination.url.href !== previous) {
      history.pushState(history.state, '', destination.url);
      window.dispatchEvent(new HashChangeEvent('hashchange', { oldURL: previous, newURL: destination.url.href }));
    }
    focusSectionTarget(target);
    scroll(top, current);
  });

  async function navigateToMatch(destination: NonNullable<ReturnType<typeof documentationSearchDestination>>) {
    const target = root.querySelector<HTMLElement>(`[data-doc-search-id="${destination.id}"]`);
    if (!target) return;
    cancel();
    const current = request;
    source = 'search';
    revealDocumentationSearchTarget(target);
    const pending = pendingSectionLayout();
    if (pending) await pending;
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    if (current !== request) return;
    const selected = documentationSearchDestination(location.href, location.href);
    const range = selected?.id === destination.id && selected.query === destination.query
      ? documentationSearchRange(target, destination.query)
      : undefined;
    if (!range) {
      source = undefined;
      return;
    }
    setDestination(target);
    const offset = Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop) + 12;
    const top = sectionScrollTarget(range.getBoundingClientRect().top, offset);
    const focusTarget = target.tagName === 'PRE' ? target.parentElement! : target;
    focusTarget.dataset.docSearchFocus = '';
    focusSectionTarget(focusTarget);
    revealSearchRangeHorizontally(target, range);
    scroll(top, current);
  }

  results.addEventListener('click', (event) => {
    const link = sectionNavigationLink(event);
    if (!link) return;
    const destination = documentationSearchDestination(link.href, location.href);
    if (!destination || destination.url.pathname !== location.pathname) return;
    event.preventDefault();
    if (destination.url.href !== location.href) history.pushState(history.state, '', destination.url);
    void navigateToMatch(destination);
  });
  const destination = documentationSearchDestination(location.href, location.href);
  if (destination) void navigateToMatch(destination);
}

