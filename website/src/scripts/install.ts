import { setupCommandScrollbars } from './command-scroll';
import { setupTabs } from './tabs';

export function setupInstall() {
  const root = document.querySelector<HTMLElement>('[data-install]');
  if (!root) return;
  setupCommandScrollbars(root);
  const panels = root.querySelectorAll<HTMLElement>('[role="tabpanel"]');

  setupTabs(root, (tab) => {
    const panelId = tab.getAttribute('aria-controls');
    for (const panel of panels) {
      panel.hidden = panel.id !== panelId;
    }
  });
}
