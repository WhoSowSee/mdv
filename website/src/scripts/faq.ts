import { setupAnimatedDetails } from './animated-details';

export function setupFaq() {
  for (const details of document.querySelectorAll<HTMLDetailsElement>('.faq details')) {
    setupAnimatedDetails(details, details.querySelector<HTMLElement>('.faq-answer')!);
  }
}
