import { expect, test } from 'bun:test';
import { homeSectionDestination, homeSectionUrl } from '../src/scripts/home-section-navigation';

test('recognizes section requests from documentation in both languages without a hash jump', () => {
  for (const prefix of ['/mdv/', '/mdv/ru/']) {
    for (const id of ['features', 'install']) {
      const current = 'https://example.org' + prefix + 'docs/usage/';
      const target = homeSectionDestination(prefix + '?section=' + id, current, prefix);
      expect(target.id).toBe(id);
      expect(target.url.hash).toBe('');
      expect(target.url.pathname).toBe(prefix);
      expect(homeSectionUrl(target.url, id).href).toBe('https://example.org' + prefix + '#' + id);
    }
  }
});

test('keeps same-page anchors and gives an explicit fragment priority over a pending request', () => {
  const current = 'https://example.org/mdv/?section=features';
  expect(homeSectionDestination('#install', current, '/mdv/').id).toBe('install');
  expect(homeSectionDestination('#top', current, '/mdv/').id).toBe('top');
  expect(homeSectionDestination('/mdv/?section=features', current, '/mdv/').id).toBe('features');
  expect(homeSectionUrl(new URL(current + '&ref=docs'), 'features').searchParams.get('ref')).toBe('docs');
});

test('leaves documentation anchors, other locales, external URLs and unknown targets alone', () => {
  const current = 'https://example.org/mdv/docs/';
  for (const href of ['#features', '/mdv/ru/?section=features', '/mdv/?section=unknown', '/mdv/#main', 'https://elsewhere.org/mdv/?section=install']) {
    expect(homeSectionDestination(href, current, '/mdv/')).toBeUndefined();
  }
});
