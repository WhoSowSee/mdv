import type { PreviewVariant } from '../data/previews';
import { renderTerminalOutput, plainTerminalOutput } from './terminal-preview';
import { setupTabs } from './tabs';

export function setupRenderPreviews(root: ParentNode, updateCodeFocus: () => void) {
  for (const preview of root.querySelectorAll<HTMLElement>('[data-render-preview]')) {
    const variants: PreviewVariant[] = JSON.parse(preview.querySelector('[data-preview-variants]')!.textContent!);
    const panel = preview.querySelector<HTMLElement>('[role="tabpanel"]')!;
    const command = preview.querySelector<HTMLElement>('[data-preview-command]')!;
    const output = preview.querySelector<HTMLElement>('[data-preview-output]')!;
    const state = preview.querySelector<HTMLElement>('[data-preview-state]')!;
    const keys = preview.querySelector<HTMLElement>('[data-preview-keys]')!;
    const instruction = preview.querySelector<HTMLElement>('[data-preview-instruction]')!;

    setupTabs(preview, (tab, index) => {
      const variant = variants[index];
      panel.setAttribute('aria-labelledby', tab.id);
      state.hidden = !variant.description;
      instruction.textContent = variant.description ?? '';
      keys.replaceChildren(...(variant.keys ?? []).map((key) => {
        const element = document.createElement('kbd');
        element.textContent = key;
        return element;
      }));
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
