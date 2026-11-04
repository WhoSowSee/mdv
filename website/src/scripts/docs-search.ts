import { searchDocumentation, searchResultUrl, searchTextMatches, type SearchPage } from './docs-search-model';
interface SearchMessages { searching: string; matches: string; none: string; unavailable: string; }

export function setupDocumentationSearch(documentation: HTMLElement) {
  const root = documentation.querySelector<HTMLElement>('[data-docs-search]');
  const input = root?.querySelector<HTMLInputElement>('input');
  const results = root?.querySelector<HTMLUListElement>('.docs-search-results');
  const status = root?.querySelector<HTMLElement>('.docs-search-status');
  const groups = documentation.querySelector<HTMLElement>('[data-docs-groups]');
  if (!root || !input || !results || !status || !groups) return;
  const messages: SearchMessages = JSON.parse(root.dataset.messages!);
  let index: Promise<SearchPage[]> | undefined;
  let request = 0;

  async function search() {
    const current = ++request;
    const query = input!.value.trim();
    if (!query) {
      const url = new URL(location.href);
      if (url.searchParams.has('q') || url.searchParams.has('match')) {
        url.searchParams.delete('q');
        url.searchParams.delete('match');
        history.replaceState(history.state, '', url);
      }
      results!.hidden = true;
      status!.hidden = true;
      groups!.hidden = false;
      return;
    }
    groups!.hidden = true;
    results!.hidden = true;
    status!.hidden = false;
    status!.textContent = messages.searching;
    index ??= fetch(root!.dataset.index!, { cache: 'no-cache' }).then(async (response) => {
      if (!response.ok) throw new Error('Search index request failed');
      return await response.json() as SearchPage[];
    });
    try {
      const pages = await index;
      if (current !== request || input!.value.trim() !== query) return;
      const matches = searchDocumentation(pages, query);
      results!.replaceChildren(...matches.map((match) => {
        const item = document.createElement('li');
        const link = document.createElement('a');
        const title = document.createElement('span');
        const snippet = document.createElement('span');
        link.href = searchResultUrl(match.url, match.block.id, query, location.href).href;
        title.className = 'docs-search-page';
        title.textContent = match.title;
        link.append(title);
        if (match.block.section && match.block.section !== match.title) {
          const section = document.createElement('span');
          section.className = 'docs-search-section';
          section.textContent = match.block.section;
          link.append(section);
        }
        snippet.className = 'docs-search-snippet';
        let position = 0;
        for (const part of searchTextMatches(match.snippet, query)) {
          snippet.append(document.createTextNode(match.snippet.slice(position, part.start)));
          const mark = document.createElement('mark');
          mark.textContent = match.snippet.slice(part.start, part.end);
          snippet.append(mark);
          position = part.end;
        }
        snippet.append(document.createTextNode(match.snippet.slice(position)));
        link.append(snippet);
        item.append(link);
        return item;
      }));
      status!.textContent = matches.length ? `${messages.matches} ${matches.length}` : messages.none;
      results!.hidden = matches.length === 0;
    } catch {
      index = undefined;
      if (current === request) status!.textContent = messages.unavailable;
    }
  }

  input.addEventListener('input', search);
  input.addEventListener('blur', () => { delete input.dataset.editing; });
  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' && !event.isComposing && !event.defaultPrevented) {
      input.dataset.editing = 'true';
    }
    if (event.key === 'Escape') {
      input.value = '';
      void search();
    }
  });
  const query = new URL(location.href).searchParams.get('q');
  if (query) input.value = query;
  if (input.value) void search();
  window.addEventListener('pageshow', () => { void search(); });
}
