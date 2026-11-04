import { expect, test } from 'bun:test';
import { documentationAnchor } from '../src/scripts/docs-section-navigation';

test('resolves Unicode and repeated documentation anchors on the current page', () => {
  const current = 'https://example.org/mdv/ru/docs/installation/';
  const destination = documentationAnchor('#установка-через-cargo', current);
  expect(destination.id).toBe('установка-через-cargo');
  expect(documentationAnchor(destination.url.href, destination.url.href).id).toBe(destination.id);
});

test('leaves other pages and links without fragments to normal navigation', () => {
  const current = 'https://example.org/mdv/docs/installation/';
  for (const href of ['https://other.example/#install', '../usage/#files', '?mode=other#install', './']) {
    expect(documentationAnchor(href, current)).toBeUndefined();
  }
});

