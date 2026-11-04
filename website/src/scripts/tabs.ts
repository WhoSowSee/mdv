export function setupTabs(
  root: ParentNode,
  select: (tab: HTMLButtonElement, index: number) => void,
  focusOptions?: FocusOptions,
) {
  const tabs = Array.from(root.querySelectorAll<HTMLButtonElement>('[role="tab"]'));

  for (const [index, tab] of tabs.entries()) {
    const activate = () => {
      for (const item of tabs) item.setAttribute('aria-selected', String(item === tab));
      select(tab, index);
    };
    tab.addEventListener('focus', activate);
    tab.addEventListener('click', activate);
    tab.addEventListener('keydown', (event) => {
      let next: number;
      switch (event.key) {
        case 'ArrowRight': next = (index + 1) % tabs.length; break;
        case 'ArrowLeft': next = (index + tabs.length - 1) % tabs.length; break;
        case 'Home': next = 0; break;
        case 'End': next = tabs.length - 1; break;
        default: return;
      }
      event.preventDefault();
      tabs[next].focus(focusOptions);
    });
  }
}
