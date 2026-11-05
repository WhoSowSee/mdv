export const motionPauseKey = 'mdv-motion-paused';

export function setupMotionToggle(button: HTMLButtonElement) {
  const root = document.documentElement;

  function setPaused(paused: boolean) {
    root.classList.toggle('motion-paused', paused);
    button.setAttribute('aria-pressed', String(paused));
    const label = paused ? button.dataset.resumeLabel! : button.dataset.pauseLabel!;
    button.setAttribute('aria-label', label);
    button.title = label;
  }

  setPaused(root.classList.contains('motion-paused'));

  button.addEventListener('click', () => {
    const paused = !root.classList.contains('motion-paused');
    setPaused(paused);
    try {
      sessionStorage.setItem(motionPauseKey, String(paused));
    } catch {
      // The toggle still controls the current document when storage is blocked.
    }
  });

  window.addEventListener('pageshow', (event) => {
    if (!event.persisted) return;
    try {
      setPaused(sessionStorage.getItem(motionPauseKey) === 'true');
    } catch {
      // Preserve the current state if storage access changed while cached.
    }
  });
}
