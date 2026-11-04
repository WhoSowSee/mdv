import { setupCopyButtons } from './copy';
import { setupPixelScrollbar } from './pixel-scroll';
import { setupRenderPreviews } from './render-previews';
import { setupCodeFocus } from './code-focus';
import { setupAnimatedDetails } from './animated-details';

export function setupDocumentationBlocks(root: HTMLElement) {
  const template = root.querySelector<HTMLTemplateElement>('[data-code-copy-template]')!;

  for (const details of root.querySelectorAll<HTMLDetailsElement>('.mdv-preview-source, .mdv-reference-preview')) {
    const summary = details.querySelector<HTMLElement>(':scope > summary')!;
    const content = document.createElement('div');
    content.className = 'docs-disclosure-content';
    for (const child of [...details.childNodes]) {
      if (child !== summary) content.append(child);
    }
    details.append(content);
    setupAnimatedDetails(details, content);
  }

  const frames = Array.from(root.querySelectorAll<HTMLElement>('.code-frame'), (frame) => {
    const pre = frame.querySelector<HTMLPreElement>('pre')!;
    const code = pre.querySelector<HTMLElement>('code')!;
    const copy = template.content.firstElementChild!.cloneNode(true) as HTMLButtonElement;
    copy.dataset.copy = code.textContent ?? '';
    frame.dataset.singleLine = String(!copy.dataset.copy.replace(/\n$/, '').includes('\n'));
    frame.append(copy);
    setupPixelScrollbar(pre, frame);
    return { frame, code };
  });
  document.addEventListener('selectionchange', () => {
    const selection = document.getSelection();
    for (const { frame, code } of frames) {
      let selected = false;
      if (selection && !selection.isCollapsed) {
        for (let index = 0; index < selection.rangeCount; index++) {
          if (selection.getRangeAt(index).intersectsNode(code)) {
            selected = true;
            break;
          }
        }
      }
      frame.toggleAttribute('data-code-selected', selected);
    }
  });
  for (const table of root.querySelectorAll<HTMLElement>('.table-frame')) {
    const host = document.createElement('div');
    host.className = 'docs-table-scroll';
    table.before(host);
    host.append(table);
    setupPixelScrollbar(table, host);
  }
  for (const output of root.querySelectorAll<HTMLElement>('.mdv-preview-output')) {
    setupPixelScrollbar(output, output.parentElement!);
  }
  setupCopyButtons(root);
  setupRenderPreviews(root, setupCodeFocus(root));
}
