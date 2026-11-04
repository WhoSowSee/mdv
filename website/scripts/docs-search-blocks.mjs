import { createHash } from 'node:crypto';
import { hastText } from './hast-text.mjs';

const headings = new Set(['h1', 'h2', 'h3', 'h4', 'h5', 'h6']);
const nestedBlocks = new Set(['ul', 'ol', 'p', 'pre', 'table', 'blockquote']);

export default {
  name: 'documentation-search-blocks',
  after(root, context) {
    const blocks = [];
    const occurrences = new Map();
    let section = '';

    function visit(node) {
      if (node.type !== 'element') {
        for (const child of node.children ?? []) visit(child);
        return;
      }
      if (['script', 'style', 'template'].includes(node.tagName)) return;
      const heading = headings.has(node.tagName);
      if (heading) section = hastText(node).replace(/\s+/g, ' ').trim();
      let content = '';
      let kind;
      if (heading) { content = hastText(node); kind = 'heading'; }
      else if (node.tagName === 'pre') { content = hastText(node); kind = 'code'; }
      else if (node.tagName === 'tr') { content = node.children.map(hastText).join(' '); kind = 'table'; }
      else if (node.tagName === 'p' || node.tagName === 'summary') { content = hastText(node); kind = 'paragraph'; }
      else if (node.tagName === 'li') {
        content = node.children.filter((child) => !nestedBlocks.has(child.tagName)).map(hastText).join('');
        kind = 'list';
      }
      content = content.replace(/\s+/g, ' ').trim();
      if (content) {
        const hash = createHash('sha256').update(`${kind}\n${section}\n${content}`).digest('hex').slice(0, 16);
        const occurrence = (occurrences.get(hash) ?? 0) + 1;
        occurrences.set(hash, occurrence);
        const id = `doc-search-${hash}-${occurrence}`;
        context.setProperty(node, 'data-doc-search-id', id);
        if (!heading && !node.properties.id) context.setProperty(node, 'id', id);
        blocks.push({ id, section, text: content, kind });
      }
      if (node.tagName === 'pre' || node.tagName === 'tr' || node.tagName === 'p' || heading) return;
      for (const child of node.children ?? []) visit(child);
    }

    visit(root);
    context.data.astro.frontmatter.searchBlocks = blocks;
  },
};
