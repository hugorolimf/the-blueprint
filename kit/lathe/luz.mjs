// Câmera, luz e tons: direção de vista, direção da luz, escala e faixas de tom.
import { cameraOf } from "../iso-kit.mjs";
import { BANDS } from "./interno.mjs";
import { dot3, unit3 } from "./vetor.mjs";

const LIGHT_UP = 1.2;
export function viewOf(P) {
  const { sinA, cosA, sinE, cosE } = cameraOf(P);
  return [cosA * cosE, sinA * cosE, sinE];
}

export function lightOf(P) {
  const { sinA, cosA } = cameraOf(P);
  return unit3([-sinA, cosA, LIGHT_UP]);
}

export const scaleOf = (P) => cameraOf(P).k;

export function bandOf(score) {
  let tone = 0;
  while (tone < BANDS.length && score >= BANDS[tone]) tone++;
  return tone;
}

export const toneOf = (normal, P) => bandOf(dot3(unit3(normal), lightOf(P)));
