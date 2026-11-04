const ansiColors = ['#000000', '#800000', '#008000', '#808000', '#000080', '#800080', '#008080', '#c0c0c0', '#808080', '#ff0000', '#00ff00', '#ffff00', '#0000ff', '#ff00ff', '#00ffff', '#ffffff'];
const ansiSequence = /\x1b\][\s\S]*?(?:\x07|\x1b\\)|\x1b\[[0-?]*[ -/]*[@-~]/g;
const escape = (value: string) => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;').replaceAll("'", '&#39;');

function indexedColor(index: number) {
  if (index < 16) return ansiColors[index];
  if (index >= 232) return `rgb(${Array(3).fill(8 + (index - 232) * 10).join(',')})`;
  const value = index - 16;
  const levels = [0, 95, 135, 175, 215, 255];
  return `rgb(${[Math.floor(value / 36), Math.floor(value / 6) % 6, value % 6].map((n) => levels[n]).join(',')})`;
}

export function plainTerminalOutput(value: string) {
  return value.replace(ansiSequence, '');
}

export function renderTerminalOutput(value: string) {
  let position = 0;
  let link = '';
  let styles: Record<string, string> = {};
  const chunks: string[] = [];
  function decoration(value: string, enabled: boolean) {
    const values = new Set(styles['text-decoration']?.split(' ') ?? []);
    if (enabled) values.add(value);
    else values.delete(value);
    if (values.size) styles['text-decoration'] = [...values].join(' ');
    else delete styles['text-decoration'];
  }
  function text(content: string) {
    if (!content) return;
    let html = escape(content);
    const properties = { ...styles };
    if (styles['background-color']) properties['--terminal-background'] = styles['background-color'];
    const css = Object.entries(properties).map(([key, val]) => `${key}:${val}`).join(';');
    if (css) html = `<span style="${css}">${html}</span>`;
    if (link) html = `<a href="${escape(link)}" tabindex="-1" target="_blank" rel="noopener noreferrer">${html}</a>`;
    chunks.push(html);
  }
  for (const match of value.matchAll(ansiSequence)) {
    text(value.slice(position, match.index));
    const token = match[0];
    position = match.index + token.length;
    if (token.startsWith('\x1b]')) {
      if (token.startsWith('\x1b]8;;')) {
        const url = token.slice(5).replace(/(?:\x07|\x1b\\)$/, '');
        link = /^(https?:|mailto:)/i.test(url) ? url : '';
      }
      continue;
    }
    if (!token.endsWith('m')) continue;
    const codes = token.slice(2, -1).split(';').map(Number);
    for (let i = 0; i < codes.length; i++) {
      const code = codes[i];
      if (code === 0) styles = {};
      else if (code === 1) styles['font-weight'] = '700';
      else if (code === 3) styles['font-style'] = 'italic';
      else if (code === 4) decoration('underline', true);
      else if (code === 9) decoration('line-through', true);
      else if (code === 22) delete styles['font-weight'];
      else if (code === 23) delete styles['font-style'];
      else if (code === 24) decoration('underline', false);
      else if (code === 29) decoration('line-through', false);
      else if (code === 39) delete styles.color;
      else if (code === 49) delete styles['background-color'];
      else if ((code >= 30 && code <= 37) || (code >= 90 && code <= 97)) styles.color = ansiColors[code >= 90 ? code - 90 + 8 : code - 30];
      else if ((code >= 40 && code <= 47) || (code >= 100 && code <= 107)) styles['background-color'] = ansiColors[code >= 100 ? code - 100 + 8 : code - 40];
      else if (code === 38 || code === 48) {
        const property = code === 38 ? 'color' : 'background-color';
        if (codes[i + 1] === 2 && i + 4 < codes.length && codes.slice(i + 2, i + 5).every((v) => Number.isInteger(v) && v >= 0 && v <= 255)) {
          styles[property] = `rgb(${codes.slice(i + 2, i + 5).join(',')})`;
          i += 4;
        } else if (codes[i + 1] === 5 && codes[i + 2] >= 0 && codes[i + 2] <= 255) {
          styles[property] = indexedColor(codes[i + 2]);
          i += 2;
        }
      }
    }
  }
  text(value.slice(position));
  return chunks.join('');
}
