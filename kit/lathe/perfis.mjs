// Sólidos e faixas gerados a partir de um perfil: disco, anel, haste e cortes por faixa.
import { iso, pathOf } from "../iso-kit.mjs";
import { TAU } from "./interno.mjs";
import { scaleOf } from "./luz.mjs";
import { pointOf } from "./quadro.mjs";
import { hullOf, stepsFor } from "./planar.mjs";
import { lathe } from "./torno.mjs";
import { silhouettePoints } from "./silhueta.mjs";
import { profileCurve } from "./curva.mjs";
import { arcOf } from "./arcos.mjs";

export const disc = (s0, s1, r, F, P, options = {}) =>
  lathe(
    [
      [s0, 0],
      [s0, r],
      [s1, r],
      [s1, 0],
    ],
    F,
    P,
    { steps: stepsFor(r, 32, 160, scaleOf(P)), ...options },
  );

export const ringBand = (s0, s1, rIn, rOut, F, P, options = {}) =>
  lathe(
    [
      [s0, rIn],
      [s0, rOut],
      [s1, rOut],
      [s1, rIn],
    ],
    F,
    P,
    { steps: stepsFor(rOut, 32, 160, scaleOf(P)), hole: null, ...options },
  );

export function solidOf(profile, F, P, options = {}) {
  const first = profile[0];
  const last = profile[profile.length - 1];
  const meridian = [[first[0], 0], ...profile, [last[0], 0]];
  const most = Math.max(...profile.map(([, r]) => r));
  return lathe(meridian, F, P, { steps: stepsFor(most, 32, 160, scaleOf(P)), ...options });
}

export function bandOfProfile(profile, F, P, options = {}) {
  const most = Math.max(...profile.map(([, r]) => r));
  return lathe(profile, F, P, { steps: stepsFor(most, 32, 160, scaleOf(P)), ...options });
}

export function bandsOf(profile, cuts, F, P, { slope = null, radius = null, steps, rows = 6, rims = "none", fill = "auto", samples = 72 } = {}) {
  const curve = profileCurve(profile, { slope, radius });
  const { s0, s1 } = curve;
  const stops = [s0, ...[...new Set(cuts)].filter((s) => s > s0 + 1e-6 && s < s1 - 1e-6).sort((a, b) => a - b), s1];
  const most = Math.max(...profile.map(([, r]) => r), ...stops.map((s) => curve.radius(s)));
  const n = steps ?? stepsFor(most, 32, 160, scaleOf(P));
  const knot = (s, soft) => (soft ? [s, curve.radius(s), 1] : [s, curve.radius(s)]);
  const meridianOf = (a, b) => [knot(a), ...profile.filter(([s]) => s > a + 1e-9 && s < b - 1e-9).map(([s, , soft]) => knot(s, soft)), knot(b)];
  const shading = { slope: curve.slope, radius: curve.radius, rows };
  const bands = stops.slice(0, -1).map((a, index) => {
    const b = stops[index + 1];
    const meridian = meridianOf(a, b);
    const paths = lathe(meridian, F, P, { steps: n, rims, fill, smooth: shading });
    const seam = index < stops.length - 2 ? arcOf(F, b, curve.radius(b), P, { slope: curve.slope(b) }) : "";
    return { s0: a, s1: b, profile: meridian, paths: { ...paths, outline: "" }, outline: paths.outline, seam };
  });
  const sidesOf = (from = s0, to = s1, { ends = false, count = samples } = {}) => {
    const span = [Math.max(s0, from), Math.min(s1, to)];
    const dense = Array.from({ length: count + 1 }, (_, index) => knot(span[0] + ((span[1] - span[0]) * index) / count, true));
    const points = silhouettePoints(dense, F, P, { samples: count, slope: curve.slope, radius: curve.radius });
    const rim = (s) => arcOf(F, s, curve.radius(s), P, { slope: curve.slope(s) });
    const rimsAt = ends === true ? span : ends === "first" ? [span[0]] : ends === "last" ? [span[1]] : [];
    return { d: points.map((list) => pathOf(list.map((point) => iso(point, P)))).join("") + rimsAt.map(rim).join(""), points };
  };
  return { bands, sidesOf, radius: curve.radius, slope: curve.slope, steps: n, s0, s1 };
}

export function rodOut(F, from, to, r, P, { stroke = 0, ...options } = {}) {
  const pad = 0.35 / scaleOf(P);
  const ring = (s) => Array.from({ length: 48 }, (_, index) => iso(pointOf(F, s, r + pad, (index / 48) * TAU), P));
  return {
    paths: disc(from - stroke, to, r, F, P, options),
    clip: pathOf(hullOf([...ring(from), ...ring(to + stroke)]), true),
    from,
    to,
    stroke,
    reach: to + stroke,
  };
}
