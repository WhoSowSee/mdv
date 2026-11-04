export function hastText(node) {
  return node.type === 'text' ? node.value : (node.children ?? []).map(hastText).join('');
}
