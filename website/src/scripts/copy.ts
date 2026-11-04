import { notify } from './notifications';

export function setupCopyButtons(root: ParentNode) {
  for (const button of root.querySelectorAll<HTMLButtonElement>('[data-copy]')) {
    const status = button.querySelector<HTMLElement>('[data-copy-status]')!;
    let resetTimer: ReturnType<typeof setTimeout> | undefined;

    function resetCopied() {
      delete button.dataset.copied;
      status.textContent = '';
    }

    button.addEventListener('click', async () => {
      if (button.disabled) return;
      button.disabled = true;
      clearTimeout(resetTimer);
      try {
        await navigator.clipboard.writeText(button.dataset.copy!);
        button.dataset.copied = 'true';
        status.textContent = button.dataset.copySuccess!;
        resetTimer = setTimeout(resetCopied, 2000);
      } catch {
        resetCopied();
        notify(button.dataset.copyFailed!);
      } finally {
        button.disabled = false;
      }
    });
  }
}
