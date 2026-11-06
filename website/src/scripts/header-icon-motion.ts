export function setupHeaderIconMotion() {
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  const controls = document.querySelectorAll<HTMLElement>('[data-header-icon]');
  const activeAttribute = 'data-icon-activated';

  function clearFeedback() {
    for (const control of controls) control.removeAttribute(activeAttribute);
  }

  for (const control of controls) {
    control.addEventListener('click', () => {
      const replaying = control.hasAttribute(activeAttribute);
      control.removeAttribute(activeAttribute);
      if (reducedMotion.matches) return;
      if (replaying) void control.offsetWidth;
      control.setAttribute(activeAttribute, '');
    });
    control.addEventListener('animationend', (event) => {
      if (event.animationName.startsWith('header-press-')) control.removeAttribute(activeAttribute);
    });
  }

  reducedMotion.addEventListener('change', clearFeedback);
  window.addEventListener('pageshow', (event) => {
    if (event.persisted) clearFeedback();
  });
}
