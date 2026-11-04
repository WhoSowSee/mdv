export function setupCodeFocus(root: ParentNode) {
  const blocks = root.querySelectorAll<HTMLElement>('pre[tabindex], .final-command code, .table-frame');

  function update() {
    for (const block of blocks) {
      const scrollable = block.scrollWidth > block.clientWidth + 1 || block.scrollHeight > block.clientHeight + 1;
      block.tabIndex = scrollable ? 0 : -1;
    }
  }

  const observer = new ResizeObserver(update);
  document.fonts.addEventListener('loadingdone', update);
  void document.fonts.ready.then(update);
  for (const block of blocks) {
    observer.observe(block);
    if (block.firstElementChild) observer.observe(block.firstElementChild);
  }
  update();
  return update;
}
