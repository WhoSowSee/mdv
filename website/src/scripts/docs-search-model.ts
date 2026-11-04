export interface SearchBlock { id: string; section: string; text: string; kind: string; }
export interface SearchPage { title: string; url: string; blocks: SearchBlock[]; }
interface SearchResult { title: string; url: string; block: SearchBlock; snippet: string; score: number; }
interface TextMatch { start: number; end: number; }

const normalize = (text: string) => text.replace(/\s+/g, ' ').trim();
const literal = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const searchWords = (query: string) => [...new Set(normalize(query).toLocaleLowerCase().split(' ').filter(Boolean))];

export function searchTextMatches(text: string, query: string): TextMatch[] {
  const words = searchWords(query).sort((a, b) => b.length - a.length);
  if (!words.length) return [];
  return [...text.matchAll(new RegExp(words.map(literal).join('|'), 'giu'))]
    .map((match) => ({ start: match.index!, end: match.index! + match[0].length }));
}

export function primarySearchMatch(text: string, query: string): TextMatch | undefined {
  const phrase = normalize(query);
  if (!phrase) return;
  const exact = new RegExp(literal(phrase), 'iu').exec(text);
  if (exact) return { start: exact.index, end: exact.index + exact[0].length };
  const candidates = searchWords(query).map((word) => ({ word, matches: searchTextMatches(text, word) }))
    .filter(({ matches }) => matches.length)
    .sort((a, b) => a.matches.length - b.matches.length || b.word.length - a.word.length);
  return candidates[0]?.matches[0];
}

const snippetLength = 180;

export function searchSnippet(content: string, query: string) {
  const text = normalize(content);
  if (text.length <= snippetLength) return text;
  const match = primarySearchMatch(text, query)!;
  let start = Math.max(0, match.start - 55);
  if (start) {
    const boundary = text.indexOf(' ', start);
    if (boundary >= start && boundary < match.start) start = boundary + 1;
  }
  if (/[\uDC00-\uDFFF]/.test(text[start])) start--;
  let end = Math.min(text.length, Math.max(start + snippetLength, match.end));
  if (end < text.length) {
    const boundary = text.lastIndexOf(' ', end);
    if (boundary >= match.end) end = boundary;
  }
  if (/[\uD800-\uDBFF]/.test(text[end - 1])) end--;
  return `${start ? '… ' : ''}${text.slice(start, end)}${end < text.length ? ' …' : ''}`;
}

export function searchDocumentation(pages: SearchPage[], query: string): SearchResult[] {
  const phrase = normalize(query).toLocaleLowerCase();
  const words = searchWords(query);
  if (!words.length) return [];
  const terms = words.map((word) => new RegExp(literal(word), 'iu'));
  const matches: SearchResult[] = [];
  for (const page of pages) {
    let best: SearchResult | undefined;
    for (const block of page.blocks) {
      const text = block.text.toLocaleLowerCase();
      const context = `${page.title} ${block.section}`.toLocaleLowerCase();
      const count = terms.filter((term) => term.test(block.text)).length;
      if (!count || !terms.every((term) => term.test(block.text) || term.test(context))) continue;
      const score = count * 10 + (text.includes(phrase) ? 50 : 0) + (count === words.length ? 20 : 0)
        + (block.text.length < snippetLength ? 5 : 0) - (block.kind === 'page' ? 20 : block.kind === 'heading' ? 10 : 0);
      if (!best || score > best.score) best = { title: page.title, url: page.url, block, snippet: searchSnippet(block.text, query), score };
    }
    if (best) {
      best.score += page.title.toLocaleLowerCase().includes(phrase) ? 100 : 0;
      matches.push(best);
    }
  }
  return matches.sort((a, b) => b.score - a.score);
}

export function searchResultUrl(page: string, id: string, query: string, current: string) {
  const url = new URL(page, current);
  url.searchParams.set('q', normalize(query));
  url.searchParams.set('match', id);
  url.hash = '';
  return url;
}

export function documentationSearchDestination(href: string, current: string) {
  const url = new URL(href, current);
  const query = url.searchParams.get('q')?.trim();
  const id = url.searchParams.get('match');
  if (url.origin !== new URL(current).origin || url.hash || !query || !id
    || !/^doc-search-(?:[a-f0-9]{16}-\d+|title|description)$/.test(id)) return;
  return { url, query, id };
}
