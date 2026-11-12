import { setupCommandScrollbars } from './command-scroll';
import { setupTabs } from './tabs';

function animateHeight(element: HTMLElement, from: number, to: number) {
  return element.animate([
    { height: `${from}px`, overflow: 'clip' },
    { height: `${to}px`, overflow: 'clip' },
  ], { duration: 180, easing: 'ease' });
}

export function setupInstall() {
  const root = document.querySelector<HTMLElement>('[data-install]');
  if (!root) return;
  setupCommandScrollbars(root);
  const panels = Array.from(root.querySelectorAll<HTMLElement>(':scope > [role="tabpanel"]'), (panel) => ({
    panel,
    command: panel.querySelector<HTMLElement>('.install-command')!,
  }));
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  let active = panels.find(({ panel }) => !panel.hidden)!;
  let animations: Animation[] = [];

  function cancelAnimations() {
    for (const animation of animations) animation.cancel();
    animations = [];
  }

  reducedMotion.addEventListener('change', () => {
    if (reducedMotion.matches) cancelAnimations();
  });

  setupTabs(root, (tab) => {
    const panelId = tab.getAttribute('aria-controls');
    const next = panels.find(({ panel }) => panel.id === panelId)!;
    if (next === active) return;

    const animate = !reducedMotion.matches && !document.documentElement.classList.contains('motion-paused');
    const previousHeight = animate ? active.panel.getBoundingClientRect().height : 0;
    const previousCommandHeight = animate ? active.command.getBoundingClientRect().height : 0;
    cancelAnimations();
    active.panel.hidden = true;
    next.panel.hidden = false;
    active = next;
    if (!animate) return;

    const height = next.panel.getBoundingClientRect().height;
    const commandHeight = next.command.getBoundingClientRect().height;
    const commandResizes = commandHeight !== previousCommandHeight;
    if (height !== previousHeight || commandResizes) {
      animations.push(animateHeight(next.panel, previousHeight, height));
    }
    if (commandResizes) {
      animations.push(animateHeight(next.command, previousCommandHeight, commandHeight));
    }
  });
}
