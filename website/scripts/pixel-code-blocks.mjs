export default {
  name: 'pixel-code-blocks',
  element: {
    filter: ['pre'],
    visit(node, context) {
      const parent = context.parent(node);
      if (parent?.type === 'element' && parent.tagName === 'div' && parent.properties.className?.includes('code-frame')) return;
      context.wrapNode(node, {
        type: 'element', tagName: 'div', properties: { className: ['code-frame'] }, children: [],
      });
    },
  },
};
