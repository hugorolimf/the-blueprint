// Sólidos prontos: sela, prisma, estádio e esfera.
import { iso, pathOf } from "../iso-kit.mjs";
import { TAU, BANDS, groupsOf, oriented } from "./interno.mjs";
import { dot3, add3, sub3, mul3, unit3 } from "./vetor.mjs";
import { viewOf, lightOf, bandOf } from "./luz.mjs";
import { frameAlong, pointOf, radialOf } from "./quadro.mjs";
import { hullOf, runsOf, stepsFor } from "./planar.mjs";

export const saddleStart = (R, a, angle) => Math.sqrt(Math.max(0, R * R - a * a * Math.cos(angle) ** 2));

export function saddleOf(F, a, start, s1, P, { steps = stepsFor(a, 24) } = {}) {
  const V = viewOf(P);
  const L = lightOf(P);
  const n = steps;
  const angle = (j) => (j / n) * TAU;
  const from = typeof start === "function" ? start : (t) => saddleStart(start, a, t);
  const low = Array.from({ length: n }, (_, j) => iso(pointOf(F, from(angle(j)), a, angle(j)), P));
  const high = Array.from({ length: n }, (_, j) => iso(pointOf(F, s1, a, angle(j)), P));
  const seen = [];
  const tones = [];
  for (let j = 0; j < n; j++) {
    const normal = radialOf(F, angle(j + 0.5));
    seen.push(dot3(normal, V) > 1e-6);
    tones.push(bandOf(dot3(normal, L)));
  }
  const buckets = [[], [], [], [], []];
  const fills = [];
  const keys = seen.map((visible, j) => (visible ? tones[j] : null));
  for (const { key, run } of groupsOf(keys)) {
    const d = pathOf(oriented([...run.map((j) => low[j]), ...run.map((j) => high[j]).reverse()]), true);
    buckets[key].push(d);
    fills.push(d);
  }
  const capSeen = dot3(F.a, V) > 1e-6;
  if (capSeen) {
    const d = pathOf(oriented(high), true);
    buckets[bandOf(dot3(F.a, L))].push(d);
    fills.push(d);
  }
  const front = runsOf(seen);
  const back = runsOf(seen.map((visible) => !visible));
  const edges = [];
  for (const run of front) {
    edges.push(pathOf(run.map((j) => low[j])));
    edges.push(pathOf([low[run[0]], high[run[0]]]), pathOf([low[run[run.length - 1]], high[run[run.length - 1]]]));
  }
  for (const run of capSeen ? back : front) edges.push(pathOf(run.map((j) => high[j])));
  return {
    fill: fills.join(""),
    outline: edges.join(""),
    crease: capSeen ? front.map((run) => pathOf(run.map((j) => high[j]))).join("") : "",
    top: buckets[4].join(""),
    shades: buckets.slice(0, 4).map((list) => list.join("")),
    seamless: true,
  };
}

export function prismOf(F, polygon, s0, s1, P) {
  const V = viewOf(P);
  const L = lightOf(P);
  const ring = oriented(polygon.map(([x, y]) => [x, y]));
  const n = ring.length;
  const at = (s, [x, y]) => [F.o[0] + F.a[0] * s + F.u[0] * x + F.v[0] * y, F.o[1] + F.a[1] * s + F.u[1] * x + F.v[1] * y, F.o[2] + F.a[2] * s + F.u[2] * x + F.v[2] * y];
  const low = ring.map((point) => iso(at(s0, point), P));
  const high = ring.map((point) => iso(at(s1, point), P));
  const buckets = [[], [], [], [], []];
  const seen = [];
  for (let j = 0; j < n; j++) {
    const [x0, y0] = ring[j];
    const [x1, y1] = ring[(j + 1) % n];
    const normal = unit3(add3(mul3(F.u, y1 - y0), mul3(F.v, -(x1 - x0))));
    const visible = dot3(normal, V) > 1e-6;
    seen.push(visible);
    if (visible) buckets[bandOf(dot3(normal, L))].push(pathOf([low[j], low[(j + 1) % n], high[(j + 1) % n], high[j]], true));
  }
  const ends = [[s0, low, mul3(F.a, -1)], [s1, high, F.a]];
  const capSeen = ends.map(([, , normal]) => dot3(normal, V) > 1e-6);
  ends.forEach(([, points, normal], index) => capSeen[index] && buckets[bandOf(dot3(normal, L))].push(pathOf(points, true)));
  const hull = hullOf([...low, ...high]);
  const creases = [];
  for (let j = 0; j < n; j++) {
    const prev = (j - 1 + n) % n;
    const a = [ring[j][0] - ring[prev][0], ring[j][1] - ring[prev][1]];
    const b = [ring[(j + 1) % n][0] - ring[j][0], ring[(j + 1) % n][1] - ring[j][1]];
    const sharp = (a[0] * b[0] + a[1] * b[1]) / (Math.hypot(a[0], a[1]) * Math.hypot(b[0], b[1]) || 1) < Math.cos(0.35);
    if (sharp && seen[j] && seen[prev]) creases.push(pathOf([low[j], high[j]]));
    ends.forEach(([, points], index) => capSeen[index] && seen[j] && creases.push(pathOf([points[j], points[(j + 1) % n]])));
  }
  return { fill: pathOf(hull, true), outline: pathOf(hull, true), crease: creases.join(""), top: buckets[4].join(""), shades: buckets.slice(0, 4).map((list) => list.join("")) };
}

export function stadium(r0, r1, offset, steps = 32) {
  const circle = (cx, r) => Array.from({ length: steps }, (_, index) => [cx + r * Math.cos((index / steps) * TAU), r * Math.sin((index / steps) * TAU)]);
  return hullOf([...circle(0, r0), ...circle(offset, r1)]);
}

export function sphereOf(centre, r, P, { steps = 48 } = {}) {
  const V = viewOf(P);
  const L = lightOf(P);
  const F = frameAlong(centre, V);
  const flat = (p) => iso(p, P);
  const rim = Array.from({ length: steps }, (_, index) => pointOf(F, 0, r, (index / steps) * TAU));
  const outline = pathOf(rim.map(flat), true);
  const capOf = (threshold) => {
    const points = rim.filter((p) => dot3(sub3(p, centre), L) / r >= threshold).map(flat);
    const c = dot3(L, V);
    const LF = frameAlong(add3(centre, mul3(L, r * threshold)), L);
    const radius = r * Math.sqrt(Math.max(0, 1 - threshold * threshold));
    for (let index = 0; index < steps; index++) {
      const p = pointOf(LF, 0, radius, (index / steps) * TAU);
      if (dot3(sub3(p, centre), V) >= -1e-9) points.push(flat(p));
    }
    if (c > threshold) points.push(flat(add3(centre, mul3(V, r))));
    return points.length > 2 ? pathOf(hullOf(points), true) : "";
  };
  const shades = [outline, capOf(BANDS[1]), capOf(BANDS[2]), capOf(BANDS[3])];
  return { fill: outline, outline, crease: "", top: "", shades, seamless: true };
}
