const properties = ['display', 'font-family', 'font-size', 'font-weight', 'font-style', 'font-stretch', 'font-kerning', 'font-feature-settings', 'font-variation-settings', 'font-variant-ligatures', 'line-height', 'letter-spacing', 'word-spacing', 'white-space', 'tab-size', 'text-indent', 'text-transform', 'text-align', 'text-decoration', 'vertical-align', 'word-break', 'overflow-wrap', 'padding', 'margin', 'border-width', 'border-style', 'box-sizing', 'list-style', 'width', 'height', 'min-width', 'max-width', 'min-height', 'max-height', 'gap', 'align-items', 'justify-content', 'flex-direction', 'flex-wrap', 'flex-grow', 'flex-shrink', 'flex-basis', 'order', 'grid-template-columns', 'grid-template-rows', 'grid-auto-flow', 'grid-column', 'grid-row'];

export function cloneSelectionText(source: HTMLElement, width: number) {
  const clone = source.cloneNode(true) as HTMLElement;
  const originals = [source, ...source.querySelectorAll<HTMLElement>('*')];
  const copies = [clone, ...clone.querySelectorAll<HTMLElement>('*')];
  for (const [index, original] of originals.entries()) {
    const copy = copies[index];
    if (!original.checkVisibility({ visibilityProperty: true })) {
      copy.remove();
      continue;
    }
    for (const attribute of [...copy.attributes]) {
      if (/^(on|data-|id$|class$|role$|name$|href$|src$|srcset$|tabindex$|contenteditable$)/.test(attribute.name)) copy.removeAttribute(attribute.name);
    }
    const style = getComputedStyle(original);
    for (const property of properties) copy.style.setProperty(property, style.getPropertyValue(property), 'important');
    copy.style.setProperty('color', 'var(--color-background)', 'important');
    copy.style.setProperty('-webkit-text-fill-color', 'var(--color-background)', 'important');
    copy.style.setProperty('background', 'transparent', 'important');
    copy.style.setProperty('text-shadow', 'none', 'important');
    copy.style.setProperty('text-decoration-color', 'currentColor', 'important');
    copy.style.setProperty('visibility', 'visible', 'important');
  }
  clone.querySelectorAll('script, style, template, iframe, object, img, video, audio, input, textarea').forEach((element) => element.remove());
  for (const [property, value] of Object.entries({ position: 'static', margin: '0', width: `${width}px`, 'min-width': '0', 'max-width': 'none', height: 'auto', overflow: 'visible', transform: 'none' })) {
    clone.style.setProperty(property, value, 'important');
  }
  return clone;
}
