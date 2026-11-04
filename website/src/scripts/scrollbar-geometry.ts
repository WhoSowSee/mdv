const clamp = (value: number, limit: number) => Math.max(0, Math.min(value, limit));

export function scrollbarGeometry(viewport: number, extent: number, position: number, track: number, minimumThumb = 36) {
  const maximum = Math.max(0, extent - viewport);
  const length = Math.min(track, Math.max(minimumThumb, track * viewport / Math.max(viewport, extent)));
  const travel = track - length;
  const offset = maximum > 0 ? travel * clamp(position, maximum) / maximum : 0;
  return { maximum, length, travel, offset };
}

export function scrollPositionForThumb(offset: number, travel: number, maximum: number, start = 0) {
  return travel > 0 ? clamp(start + offset / travel * maximum, maximum) : clamp(start, maximum);
}
