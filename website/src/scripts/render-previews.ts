import type { PreviewVariant } from '../data/previews';
import { renderTerminalOutput, plainTerminalOutput } from './terminal-preview';
import { setupTabs } from './tabs';

function setupPreviewInstructions(preview: HTMLElement, reducedMotion: MediaQueryList) {
  const state = preview.querySelector<HTMLElement>('[data-preview-state]');
  if (!state) return;
  const frame = state.parentElement!;
  const keys = state.querySelector<HTMLElement>('[data-preview-keys]')!;
  const instruction = state.querySelector<HTMLElement>('[data-preview-instruction]')!;
  let animation: Animation | undefined;

  reducedMotion.addEventListener('change', () => {
    if (reducedMotion.matches) animation?.cancel();
  });

  return (variant: PreviewVariant, tabChanged: boolean) => {
    const animate = tabChanged && !reducedMotion.matches;
    const previousHeight = animate ? frame.getBoundingClientRect().height : 0;
    state.hidden = !variant.description;
    instruction.textContent = variant.description ?? '';
    keys.replaceChildren(...(variant.keys ?? []).map((key) => {
      const element = document.createElement('kbd');
      element.textContent = key;
      return element;
    }));
    if (!animate) return;

    animation?.cancel();
    const height = frame.getBoundingClientRect().height;
    animation = height === previousHeight ? undefined : frame.animate(
      [{ height: `${previousHeight}px` }, { height: `${height}px` }],
      { duration: 180, easing: 'ease' },
    );
  };
}

export function setupRenderPreviews(root: ParentNode, updateCodeFocus: () => void) {
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  for (const preview of root.querySelectorAll<HTMLElement>('[data-render-preview]')) {
    const variants: PreviewVariant[] = JSON.parse(preview.querySelector('[data-preview-variants]')!.textContent!);
    const panel = preview.querySelector<HTMLElement>('[role="tabpanel"]')!;
    const command = preview.querySelector<HTMLElement>('[data-preview-command]')!;
    const output = preview.querySelector<HTMLElement>('[data-preview-output]')!;
    const updateInstructions = setupPreviewInstructions(preview, reducedMotion);

    setupTabs(preview, (tab, index) => {
      const variant = variants[index];
      const tabChanged = panel.getAttribute('aria-labelledby') !== tab.id;
      panel.setAttribute('aria-labelledby', tab.id);
      updateInstructions?.(variant, tabChanged);
      command.textContent = variant.command;
      const copy = command.closest('.code-frame')!.querySelector<HTMLButtonElement>('[data-copy]');
      if (copy) {
        copy.dataset.copy = variant.command;
        delete copy.dataset.copied;
      }
      output.innerHTML = renderTerminalOutput(variant.ansi);
      output.setAttribute('aria-label', plainTerminalOutput(variant.ansi));
      output.parentElement!.style.setProperty('--preview-rows', String(variant.rows));
      output.parentElement!.style.setProperty('--preview-columns', String(variant.columns));
      command.parentElement!.scrollLeft = 0;
      output.parentElement!.scrollLeft = 0;
      updateCodeFocus();
    }, { preventScroll: true });
  }
}
