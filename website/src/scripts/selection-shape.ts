export interface SelectionRect { left: number; top: number; right: number; bottom: number; }

export function intersectRect(rect: SelectionRect, clip: SelectionRect): SelectionRect | undefined {
  const result = { left: Math.max(rect.left, clip.left), top: Math.max(rect.top, clip.top), right: Math.min(rect.right, clip.right), bottom: Math.min(rect.bottom, clip.bottom) };
  return result.right > result.left && result.bottom > result.top ? result : undefined;
}

export function subtractRect(rect: SelectionRect, cover: SelectionRect): SelectionRect[] {
  const overlap = intersectRect(rect, cover);
  if (!overlap) return [rect];
  return [
    { ...rect, bottom: overlap.top },
    { ...rect, top: overlap.bottom },
    { left: rect.left, top: overlap.top, right: overlap.left, bottom: overlap.bottom },
    { left: overlap.right, top: overlap.top, right: rect.right, bottom: overlap.bottom },
  ].filter((piece) => piece.right > piece.left && piece.bottom > piece.top);
}

export function selectionLines(rectangles: SelectionRect[]) {
  const lines: SelectionRect[] = [];
  for (const rect of [...rectangles].sort((a, b) => a.top - b.top || a.left - b.left)) {
    const line = lines.find((line) => Math.abs(line.top - rect.top) < 2 && Math.abs(line.bottom - rect.bottom) < 2 && rect.left <= line.right + 1.5 && rect.right >= line.left - 1.5);
    if (line) {
      line.left = Math.min(line.left, rect.left);
      line.right = Math.max(line.right, rect.right);
      line.top = Math.min(line.top, rect.top);
      line.bottom = Math.max(line.bottom, rect.bottom);
    } else lines.push({ ...rect });
  }
  return lines;
}

export function selectionPath(rect: SelectionRect) {
  const { left: x, top: y, right: r, bottom: b } = rect;
  const step = Math.min(2, Math.max(1, Math.floor((b - y) / 12)), (r - x) / 4, (b - y) / 4);
  const cut = step * 2;
  return `M${x + cut},${y}H${r - cut}V${y + step}H${r - step}V${y + cut}H${r}V${b - cut}H${r - step}V${b - step}H${r - cut}V${b}H${x + cut}V${b - step}H${x + step}V${b - cut}H${x}V${y + cut}H${x + step}V${y + step}H${x + cut}Z`;
}
