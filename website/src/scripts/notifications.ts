let timeout: ReturnType<typeof setTimeout>;

export function notify(message: string) {
  const status = document.querySelector<HTMLElement>('#site-status');
  if (!status) return;
  clearTimeout(timeout);
  status.textContent = message;
  status.classList.add('visible');
  timeout = setTimeout(() => {
    status.classList.remove('visible');
    void Promise.allSettled(status.getAnimations().map((animation) => animation.finished)).then(() => {
      if (!status.classList.contains('visible')) status.textContent = '';
    });
  }, 3500);
}
