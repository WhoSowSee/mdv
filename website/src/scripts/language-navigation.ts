import { pendingSectionLayout, sectionNavigationLink, sectionScrollTop } from './section-scroll';

interface ReadingAnchor { id: string; top: number; }
interface ReadingPosition { anchor: string; next: string | null; progress: number; }

export const languagePositionKey = 'mdv-language-position';

export function languageReadingPosition(anchors: readonly ReadingAnchor[], scroll: number, maximum: number, translations: Readonly<Record<string, string>> = {}): ReadingPosition {
  const top = Math.max(0, Math.min(maximum, scroll));
  const points = [{ id: '', top: 0 }, ...anchors.filter((anchor) => anchor.top > 0 && anchor.top < maximum).sort((a, b) => a.top - b.top)];
  const current = points.findLast((anchor) => anchor.top <= top)!;
  const next = points.find((anchor) => anchor.top > top);
  const end = next?.top ?? maximum;
  const translate = (id: string) => Object.hasOwn(translations, id) ? translations[id] : id;
  return {
    anchor: translate(current.id),
    next: next ? translate(next.id) : null,
    progress: end > current.top ? (top - current.top) / (end - current.top) : 0,
  };
}

export function languageReadingScroll(position: ReadingPosition, anchors: readonly ReadingAnchor[], maximum: number) {
  if (!position || typeof position.anchor !== 'string' || (position.next !== null && typeof position.next !== 'string')
    || !Number.isFinite(position.progress) || position.progress < 0 || position.progress > 1) return;
  const start = position.anchor === '' ? 0 : anchors.find((anchor) => anchor.id === position.anchor)?.top;
  const end = position.next === null ? maximum : anchors.find((anchor) => anchor.id === position.next)?.top;
  if (start === undefined || end === undefined || end < start) return;
  return Math.max(0, Math.min(maximum, start + (end - start) * position.progress));
}

function readingLayout() {
  const scrolling = document.scrollingElement!;
  const maximum = scrolling.scrollHeight - scrolling.clientHeight;
  const padding = Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop);
  const anchors = [...document.querySelectorAll<HTMLElement>('main section[id], main :is(h1, h2, h3)[id]')]
    .filter((element) => element.getClientRects().length > 0)
    .map((element) => ({
      id: element.id,
      top: sectionScrollTop(element.getBoundingClientRect().top, window.scrollY,
        padding + Number.parseFloat(getComputedStyle(element).scrollMarginTop), maximum),
    }));
  return { anchors, maximum };
}

async function restoreReadingPosition() {
  const root = document.documentElement;
  try {
    const stored = sessionStorage.getItem(languagePositionKey);
    if (!stored) return;
    sessionStorage.removeItem(languagePositionKey);
    const saved: { url: string; position: ReadingPosition } = JSON.parse(stored);
    if (saved.url !== location.href) return;

    const pending = pendingSectionLayout();
    if (pending) await pending;
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    const { anchors, maximum } = readingLayout();
    const top = languageReadingScroll(saved.position, anchors, maximum);
    if (top === undefined) return;

    window.scrollTo({ top, behavior: 'instant' });
    root.dataset.languageRevealing = '';
  } catch {
    // Unavailable storage or invalid saved data must not block the page.
  } finally {
    delete root.dataset.languageRestoring;
  }
}

export function setupLanguageNavigation() {
  const language = document.querySelector<HTMLAnchorElement>('[data-language-toggle]')!;
  const translations: Record<string, string> = JSON.parse(language.dataset.languageAnchors!);
  const destination = new URL(language.href);

  function updateLink() {
    const { anchors, maximum } = readingLayout();
    const position = languageReadingPosition(anchors, window.scrollY, maximum, translations);
    destination.hash = position.anchor;
    language.href = destination.href;
    return position;
  }

  language.addEventListener('pointerdown', updateLink);
  language.addEventListener('focus', updateLink);
  language.addEventListener('click', (event) => {
    if (sectionNavigationLink(event) !== language) return;
    const position = updateLink();
    try {
      sessionStorage.setItem(languagePositionKey, JSON.stringify({ url: destination.href, position }));
      document.documentElement.dataset.languageDestination = destination.href;
    } catch {
      // The link still points to the translated section.
    }
  });
  void restoreReadingPosition();
}
