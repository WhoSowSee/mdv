import renderSnapshot from './render-previews.json';
import interactiveSnapshot from './interactive-previews.json';
import type { Locale } from '../i18n';

export interface PreviewVariant {
  label: string;
  command: string;
  ansi: string;
  rows: number;
  columns: number;
  keys?: string[];
  description?: string;
}

interface Preview {
  title: Record<Locale, string>;
  description?: Record<Locale, string>;
  sourceLabel?: Record<Locale, string>;
  en: { source: string; variants: PreviewVariant[] };
  ru: { source: string; variants: PreviewVariant[] };
}

const renderPreviews: Record<string, Preview> = renderSnapshot.previews;
const interactivePreviews: Record<string, Preview> = interactiveSnapshot.previews;

export function getDocumentationPreview(id: string) {
  if (Object.hasOwn(interactivePreviews, id)) {
    return { preview: interactivePreviews[id], interactive: true, version: interactiveSnapshot.version };
  }
  if (Object.hasOwn(renderPreviews, id)) {
    return { preview: renderPreviews[id], interactive: false, version: renderSnapshot.version };
  }
  throw new Error(`Unknown documentation preview: ${id}`);
}
