import { documentationSearchIndex } from '../../data/docs';

export function GET() {
  return Response.json(documentationSearchIndex('en'));
}
