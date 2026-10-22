// Constantes e auxiliares internos compartilhados entre as partes (não reexportado pelo índice).

export const TAU = Math.PI * 2;
export const BANDS = [0, 0.2, 0.42, 0.62];

export function hullTagged(points) {
  const sorted = [...points].sort((a, b) => a.p[0] - b.p[0] || a.p[1] - b.p[1]);
  if (sorted.length < 3) return sorted;
  const cross = (o, a, b) => (a.p[0] - o.p[0]) * (b.p[1] - o.p[1]) - (a.p[1] - o.p[1]) * (b.p[0] - o.p[0]);
  const lower = [];
  for (const point of sorted) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], point) <= 1e-9) lower.pop();
    lower.push(point);
  }
  const upper = [];
  for (let index = sorted.length - 1; index >= 0; index--) {
    const point = sorted[index];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], point) <= 1e-9) upper.pop();
    upper.push(point);
  }
  return [...lower.slice(0, -1), ...upper.slice(0, -1)];
}

export function groupsOf(keys) {
  const count = keys.length;
  const out = [];
  const start = keys.findIndex((key, index) => key !== keys[(index + count - 1) % count]);
  if (start < 0) {
    if (keys[0] !== null) out.push({ key: keys[0], run: Array.from({ length: count + 1 }, (_, index) => index % count) });
    return out;
  }
  let current = null;
  for (let step = 0; step < count; step++) {
    const index = (start + step) % count;
    const key = keys[index];
    if (current && current.key === key) {
      current.run.push((index + 1) % count);
      continue;
    }
    if (current && current.key !== null) out.push(current);
    current = { key, run: [index, (index + 1) % count] };
  }
  if (current && current.key !== null) out.push(current);
  return out;
}

export const signedArea = (points) => points.reduce((sum, [x, y], index) => {
  const [nx, ny] = points[(index + 1) % points.length];
  return sum + x * ny - nx * y;
}, 0);

export const oriented = (points) => (signedArea(points) < 0 ? [...points].reverse() : points);
