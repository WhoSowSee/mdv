import { hastText } from './hast-text.mjs';

export default {
  name: 'scroll-tables',
  element: {
    filter: ['table'],
    visit(node, context) {
      const parent = context.parent(node);
      if (parent?.type === 'element' && parent.properties.className?.includes('table-frame')) return;
      const header = node.children.find((child) => child.tagName === 'thead');
      const label = header?.children.flatMap((row) => row.children ?? []).filter((cell) => cell.tagName === 'th').map(hastText).join(', ');
      context.wrapNode(node, {
        type: 'element', tagName: 'div',
        properties: { className: ['table-frame'], tabIndex: 0, role: 'region', ariaLabel: label }, children: [],
      });
    },
  },
};
