import { setupDocumentationBlocks } from './docs-blocks';
import { setupDocumentationPanelScrolling } from './docs-panel-scroll';
import { setupDocumentationPanels } from './docs-panels';
import { setupDocumentationSearch } from './docs-search';
import { setupDocumentationSectionNavigation } from './docs-section-navigation';

export function setupDocumentation(root: HTMLElement) {
  setupDocumentationPanels(root);
  setupDocumentationBlocks(root);
  setupDocumentationPanelScrolling(root);
  setupDocumentationSearch(root);
  setupDocumentationSectionNavigation(root);
}
