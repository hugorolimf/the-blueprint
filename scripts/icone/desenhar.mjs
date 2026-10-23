// Desenha o ícone do app com o kit do anatomy: uma folha de blueprint (papel azul quadriculado, margem e carimbo),
// um esquadro 30/60 vazado com régua e um compasso de mola (agulha, grafite, fuso com roda) deitado ao lado.
// Os tamanhos pequenos (16 a 32 px) usam uma versão simples: sem grade fina, carimbo, régua, fuso nem serrilhas, e com traço grosso.
// Uso: node scripts/icone/desenhar.mjs [pasta]  → grava icone.svg, icone-pequeno.svg, icone.ico e previa.html (padrão: recursos/icone).
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import * as k from "../../kit/iso-kit.mjs";
import * as G from "../../kit/lathe.mjs";
import { empacotarIco, rasterizar } from "./ico.mjs";

const saida = resolve(process.argv[2] ?? "recursos/icone");
const GRANDES = [256, 128, 64, 48], PEQUENOS = [32, 24, 16];
let simples = false; // versão para 16 a 32 px
const grosso = (perfil) => perfil.map(([s, r, ...m]) => [s, r * (simples ? 2.2 : 1), ...m]); // na versão simples o compasso engrossa
const TAM = 512;

// ---- mundo (unidades ~ px do objeto; z para cima, papel em z = 0) ----
const PAPEL = { x: -100, y: -100, w: 200, d: 200, r: 2 };
const ESQ = { c: [-86, -86], a: 150, b: 88, h: 3.4, furo: 15 }; // ângulo reto em c; catetos ao longo de +x (a) e +y (b)
const COMPASSO = { cabeca: [42, -22], para: [0, 1], perna: 112, abertura: 30 }, EIXO_Z = 5.6; // pernas a EIXO_Z do papel (raio da roda)

const P = k.fitProjection(
  k.boxCorners(PAPEL, -4, 0),
  TAM, TAM, { pad: 6, azimuth: 90, elevation: 75 }, // de frente: a folha quase de cima, x para a direita e y para baixo na tela
);

const v3 = { sub: (a, b) => a.map((x, i) => x - b[i]), add: (a, b) => a.map((x, i) => x + b[i]), mul: (a, s) => a.map((x) => x * s) };
v3.len = (a) => Math.hypot(...a);
v3.unit = (a) => v3.mul(a, 1 / v3.len(a));
v3.cross = ([a, b, c], [d, e, f]) => [b * f - c * e, c * d - a * f, a * e - b * d];
const at = (p) => k.iso(p, P);
const fundo = (p) => k.depthOf(p, P);
const S = (paths, estilo) => k.solidSvg(paths, estilo);
const L = (d, estilo) => k.lineSvg(d, estilo);

// ---- a folha: duas folhas empilhadas, grade, margem, carimbo ----
const grade = () => {
  if (simples) return ""; // pequena demais para grade: ela só competiria com o compasso
  const fina = [], grossa = [];
  for (let x = PAPEL.x + 12; x < PAPEL.x + PAPEL.w - 11; x += 10) (Math.round(x - PAPEL.x - 12) % 50 === 0 ? grossa : fina).push(k.lineOnTop([x, PAPEL.y + 12], [x, PAPEL.y + PAPEL.d - 12], 0, P));
  for (let y = PAPEL.y + 12; y < PAPEL.y + PAPEL.d - 11; y += 10) (Math.round(y - PAPEL.y - 12) % 50 === 0 ? grossa : fina).push(k.lineOnTop([PAPEL.x + 12, y], [PAPEL.x + PAPEL.w - 12, y], 0, P));
  return L(fina.join(""), { tone: "faint" }) + L(grossa.join(""), { tone: "lo" });
};

function folha() {
  const baixo = { ...PAPEL, x: PAPEL.x + 5, y: PAPEL.y + 4 };
  const out = [`<path class="iso-halo" d="${k.haloOf(k.insetPlan(PAPEL, -4), -4, 0.1, P)}"/>`];
  out.push(S(k.slabOf(baixo, -3.6, 1.6, P, 2), { tone: "lo", crease: "none" }));
  out.push(S(k.slabOf(PAPEL, -1.8, 1.8, P, 2), { tone: "mid", crease: "none" }));
  out.push(grade());
  const margem = k.insetPlan(PAPEL, 8);
  out.push(L(k.planOutline({ ...margem, r: 0.5 }, 0, P, 1), { tone: "mid" }));
  if (simples) return out.join("");
  // carimbo no canto da frente: moldura, divisória e três linhas de texto
  const cb = { x: margem.x, y: margem.y + margem.d - 26, w: 70, d: 26, r: 0 };
  out.push(`<path class="iso-fill" d="${k.onTop(k.ringOf(cb, 1), 0, P, true)}" style="fill:var(--anatomy-top)"/>`);
  out.push(L(k.planOutline(cb, 0, P, 1), { tone: "mid" }));
  out.push(L(k.lineOnTop([cb.x + 46, cb.y], [cb.x + 46, cb.y + cb.d], 0, P), { tone: "mid" }));
  out.push(L(k.lineOnTop([cb.x, cb.y + 13], [cb.x + 46, cb.y + 13], 0, P), { tone: "lo" }));
  out.push(L([[5, 7, 36], [5, 19, 28]].map(([x, y, w]) => k.lineOnTop([cb.x + x, cb.y + y], [cb.x + x + w, cb.y + y], 0, P)).join(""), { tone: "hi", free: true }));
  out.push(L(k.ring(cb.x + 58, cb.y + 13, 6, 0, P, 32), { tone: "mid" }));
  return out.join("");
}

// ---- o esquadro: prisma triangular vazado, régua nos catetos ----
function recuo(poli, t) { // desloca cada lado para dentro por t e cruza os vizinhos (polígono anti-horário)
  const n = poli.length, lados = poli.map((p, i) => { const q = poli[(i + 1) % n], d = [q[0] - p[0], q[1] - p[1]], l = Math.hypot(...d), nn = [-d[1] / l, d[0] / l]; return [[p[0] + nn[0] * t, p[1] + nn[1] * t], d]; });
  return lados.map((_, i) => {
    const [[ax, ay], [adx, ady]] = lados[(i + n - 1) % n], [[bx, by], [bdx, bdy]] = lados[i];
    const s = ((bx - ax) * bdy - (by - ay) * bdx) / (adx * bdy - ady * bdx);
    return [ax + adx * s, ay + ady * s];
  });
}

function esquadro() {
  const [cx, cy] = ESQ.c, h = ESQ.h;
  const tri = [[cx, cy], [cx + ESQ.a, cy], [cx, cy + ESQ.b]];
  const furo = recuo(tri, ESQ.furo);
  const out = [`<path class="iso-halo" d="${k.extrude(tri, 0, 0.1, P).fill}" style="opacity:.35"/>`];
  out.push(S(k.extrude(tri, 0, h, P, { bevel: 0.8 }), { tone: "hi", className: "esquadro" }));
  // o vão: o papel (com a grade) visto por dentro, as paredes do fundo e a borda de cima
  const id = "vao-esquadro";
  const vao = k.onTop(furo, h, P, true);
  const [vx, vy] = k.towardViewer(P);
  const paredes = furo.map((p, i) => {
    const q = furo[(i + 1) % 3], d = [q[0] - p[0], q[1] - p[1]], para = [-d[1], d[0]]; // para dentro do vão (furo anti-horário)
    if (para[0] * vx + para[1] * vy >= 0) return "";
    const tom = (-para[1] * Math.sin(P.azimuth * k.RADIANS) + para[0] * Math.cos(P.azimuth * k.RADIANS)) > 0 ? 3 : 1;
    return `<path class="iso-shade" data-shade="${tom}" d="${k.pathOf([at([...p, h]), at([...q, h]), at([...q, 0]), at([...p, 0])], true)}"/>`;
  }).join("");
  out.push(`<clipPath id="${id}"><path d="${vao}"/></clipPath><g clip-path="url(#${id})"><path class="iso-top" d="${k.onTop(furo, 0, P, true)}" style="fill:var(--papel-top)"/>${grade()}<g class="iso-solid">${paredes}</g></g>`);
  out.push(L(vao, { tone: "mid" }));
  if (simples) return { svg: out.join(""), fundo: fundo([cx + ESQ.a / 3, cy + ESQ.b / 3, 0]) };
  // régua: traços a cada 4, maior a cada 5º, nos dois catetos
  const rx = k.topTicks(cx + 6, cx + ESQ.a - 34, 4, 5, cy + 0.9, h, [2.4, 4.4], P, "x");
  const ry = k.topTicks(cy + 6, cy + ESQ.b - 26, 4, 5, cx + 0.9, h, [2.4, 4.4], P, "y");
  out.push(L(rx.minor + ry.minor, { tone: "lo" }), L(rx.major + ry.major, { tone: "mid" }));
  out.push(k.dotsSvg([at([cx + 8, cy + 8, h])], { size: 0.9, tone: "hi" }));
  return { svg: out.join(""), fundo: fundo([cx + ESQ.a / 3, cy + ESQ.b / 3, 0]) };
}

// ---- o compasso de mola, deitado no papel: pernas no plano horizontal, apoiado na roda e na articulação ----
function compasso() {
  const cabeca = [...COMPASSO.cabeca, EIXO_Z], para = v3.unit([...COMPASSO.para, 0]);
  const gira = (ang) => [para[0] * Math.cos(ang) - para[1] * Math.sin(ang), para[0] * Math.sin(ang) + para[1] * Math.cos(ang), 0];
  const pecas = [];
  const perna = (dir, grafite) => {
    const Lp = COMPASSO.perna, F = G.frameAlong(cabeca, dir), ponta = v3.add(cabeca, v3.mul(dir, Lp));
    const corpo = grafite
      ? [[5, 3.1], [Lp - 30, 2.3], [Lp - 30, 3.4], [Lp - 16, 3.4], [Lp - 16, 1.2], [Lp - 4, 1.2], [Lp, 0.15]]
      : [[5, 3.1], [Lp - 20, 2.3], [Lp - 20, 3.0], [Lp - 13, 3.0], [Lp - 13, 0.7], [Lp, 0.1]];
    const svg = [S(G.solidOf(grosso(corpo), F, P), { tone: "hi", className: "metal" })];
    const ponteira = grafite ? [[Lp - 16, 1.2], [Lp - 4, 1.2], [Lp, 0.15]] : [[Lp - 13, 0.7], [Lp, 0.1]];
    svg.push(S(G.solidOf(grosso(ponteira), F, P), { tone: "lit", className: grafite ? "grafite" : "agulha" }));
    if (grafite && !simples) {
      svg.push(k.fadedSvg(G.ribsOf(F, Lp - 30, Lp - 16, 3.4, 18, P, { fade: [0.04, 0.4] }), { tone: "lo" }));
      // parafuso de aperto do porta-grafite, para cima
      const Fp = G.frameAlong(v3.add(G.pointOf(F, Lp - 23, 0, 0), [0, 0, 3.2]), [0, 0, 1]);
      svg.push(S(G.solidOf([[0, 1.7], [1.6, 1.7], [1.6, 2.8], [4.6, 2.8]], Fp, P), { tone: "hi", className: "metal" }));
    }
    return { svg: svg.join(""), fundo: fundo(v3.mul(v3.add(cabeca, ponta), 0.5)), em: (t) => v3.add(cabeca, v3.mul(dir, Lp * t)) };
  };
  const meia = (COMPASSO.abertura * Math.PI) / 360;
  const pA = perna(gira(-meia), false), pB = perna(gira(meia), true);
  pecas.push(pA, pB);
  if (!simples) pecas.push(fuso(pA, pB));
  pecas.sort((p, q) => p.fundo - q.fundo);
  // articulação: disco de eixo vertical apoiado no papel, com o pino em cima; o cabo serrilhado sai do lado oposto às pernas
  const Fe = G.frameAlong([cabeca[0], cabeca[1], 0], [0, 0, 1]);
  const cab = [S(G.disc(0, EIXO_Z * 2, simples ? 14 : 8.5, Fe, P), { tone: "hi", className: "metal" })];
  if (!simples) cab.push(L(k.ring(cabeca[0], cabeca[1], 3, EIXO_Z * 2, P, 24), { tone: "mid" }), k.dotsSvg([at([cabeca[0], cabeca[1], EIXO_Z * 2])], { size: 0.9, tone: "hi" }));
  const Fc = G.frameAlong(v3.add(cabeca, v3.mul(para, -8.2)), v3.mul(para, -1));
  cab.push(S(G.solidOf(grosso([[0, 2], [4, 2], [4, 3.2], [22, 3.2], [23.4, 2.3]]), Fc, P), { tone: "hi", className: "metal" }));
  if (!simples) cab.push(k.fadedSvg(G.ribsOf(Fc, 7, 20, 3.2, 26, P, { fade: [0.04, 0.42], seams: true }), { tone: "lo" }));
  // sombra no papel: uma faixa estreita sob cada perna e o círculo da articulação
  const faixa = (p) => { const d = v3.unit(v3.sub(p, cabeca)), n = [-d[1] * 3, d[0] * 3]; return [[cabeca[0] + n[0], cabeca[1] + n[1]], [p[0] + n[0] * 0.3, p[1] + n[1] * 0.3], [p[0] - n[0] * 0.3, p[1] - n[1] * 0.3], [cabeca[0] - n[0], cabeca[1] - n[1]]]; };
  const sombra = [faixa(pA.em(1)), faixa(pB.em(1)), k.circleRing(cabeca[0], cabeca[1], 9, 24)].map((anel) => k.extrude(anel, 0, 0.1, P).fill).join("");
  const ordem = [`<path class="iso-halo" d="${sombra}" style="opacity:.5;filter:blur(4px)"/>`, ...pecas.map((p) => p.svg), cab.join("")];
  return { svg: ordem.join(""), fundo: fundo(v3.add(cabeca, v3.mul(para, COMPASSO.perna / 2))) };
}

// fuso com rosca entre as pernas e a roda serrilhada no meio, que encosta no papel
function fuso(pA, pB) {
  const fa = pA.em(0.55), fb = pB.em(0.55), dir = v3.unit(v3.sub(fb, fa)), Lf = v3.len(v3.sub(fb, fa));
  const Ff = G.frameAlong(fa, dir);
  const out = [S(G.solidOf([[0, 1.1], [Lf, 1.1]], Ff, P), { tone: "mid", className: "metal" })];
  out.push(k.fadedSvg(G.ribsOf(Ff, 3, Lf - 3, 1.1, 10, P, { twist: 14, fade: [0.04, 0.4] }), { tone: "lo" }));
  const Fr = G.frameAlong(v3.add(fa, v3.mul(dir, Lf / 2 - 2)), dir);
  out.push(S(G.disc(0, 4, EIXO_Z, Fr, P), { tone: "hi", className: "metal" }));
  out.push(k.fadedSvg(G.ribsOf(Fr, 0.4, 3.6, EIXO_Z, 40, P, { fade: [0.04, 0.45] }), { tone: "lo" }));
  return { svg: out.join(""), fundo: fundo(v3.add(fa, v3.mul(dir, Lf / 2))) };
}

// ---- montagem ----
const CSS = `${k.ISO_CSS}
.iso{--papel-top:#2b7de0;--anatomy-paper:#1f6ccc;--anatomy-face:#2a78d8;--anatomy-top:#2b7de0;--anatomy-shade-0:#0f4a9c;--anatomy-shade-1:#1455ad;--anatomy-shade-2:#1a60bb;--anatomy-shade-3:#1f6ac7;--anatomy-hi:#ffffff;--anatomy-mid:#cfe4ff;--anatomy-lo:#8fbff5;--anatomy-faint:#5c9ceb;--anatomy-lit:#ffffff;--anatomy-halo:#03183a;--anatomy-dot:#ffffff;--anatomy-weight:2.2px}
.iso-svg path,.iso-svg circle{vector-effect:none}
.iso-halo{fill:var(--anatomy-halo);filter:blur(8px);opacity:.45}
.esquadro>.iso-fill{fill:#bfe0ff;fill-opacity:.35}.esquadro>.iso-top{fill:#d6ebff;fill-opacity:.42}.esquadro>.iso-shade{fill:#9cc9f7;fill-opacity:.6}
.metal>.iso-fill{fill:#c9d9ec}.metal>.iso-shade[data-shade="0"]{fill:#7f9bbd}.metal>.iso-shade[data-shade="1"]{fill:#9db5d2}.metal>.iso-shade[data-shade="2"]{fill:#bccfe5}.metal>.iso-shade[data-shade="3"]{fill:#dce8f5}.metal>.iso-top{fill:#eef4fb}
.metal>.iso-edge,.metal>.iso-crease,.agulha>.iso-edge,.grafite>.iso-edge{stroke:#0b3470}.metal .iso-line[data-tone=lo]{stroke:#5a7ea8}
.agulha>.iso-fill,.agulha>.iso-top,.agulha>.iso-shade{fill:#ffffff}.grafite>.iso-fill,.grafite>.iso-top,.grafite>.iso-shade{fill:#1d2f4a}`;
const GROSSO = ".iso{--anatomy-weight:10px}.iso-line[data-tone=lo],.iso-line[data-tone=faint]{stroke:var(--anatomy-mid)}.metal>.iso-fill,.metal>.iso-top,.metal>.iso-shade{fill:#fff}.metal>.iso-edge,.agulha>.iso-edge,.grafite>.iso-edge{stroke-width:6px}"; // ~0,6 px a 32 px

function desenho(pequeno) {
  simples = pequeno;
  const objetos = [esquadro(), compasso()].sort((a, b) => a.fundo - b.fundo).map((o) => o.svg).join("");
  return `<svg xmlns="http://www.w3.org/2000/svg" class="iso-svg iso" viewBox="0 0 ${TAM} ${TAM}" role="img" aria-label="The Blueprint"><style>${CSS}${pequeno ? GROSSO : ""}</style>${folha()}${objetos}</svg>`;
}

mkdirSync(saida, { recursive: true });
const svgs = { "icone.svg": desenho(false), "icone-pequeno.svg": desenho(true) };
for (const [nome, svg] of Object.entries(svgs)) writeFileSync(join(saida, nome), svg);
const pngs = await rasterizar([...GRANDES.map((t) => [svgs["icone.svg"], t]), ...PEQUENOS.map((t) => [svgs["icone-pequeno.svg"], t])]);
writeFileSync(join(saida, "icone.ico"), empacotarIco(pngs));
const arquivo = (t) => (t > 32 ? "icone.svg" : "icone-pequeno.svg");
const linha = (fundoCor, nome) => `<section style="background:${fundoCor}"><h2>${nome}</h2><div>${[...GRANDES, ...PEQUENOS].map((t) => `<figure><img src="${arquivo(t)}" width="${t}" height="${t}"><figcaption>${t}</figcaption></figure>`).join("")}</div></section>`;
writeFileSync(join(saida, "previa.html"), `<!doctype html><meta charset="utf-8"><title>Ícone do Blueprint</title><style>body{margin:0;font:13px system-ui;color:#888}section{padding:16px 24px}h2{font-size:12px;margin:0 0 8px;font-weight:500}div{display:flex;align-items:flex-end;gap:24px}figure{margin:0;text-align:center}figcaption{margin-top:4px}.grande{display:flex;justify-content:center;background:#202020;padding:24px}</style>
<div class="grande"><img src="icone.svg" width="512" height="512"></div>${linha("#202020", "Barra de tarefas escura")}${linha("#f3f3f3", "Explorador claro")}${linha("#0c3a78", "Sobre o papel do app")}`);
console.log("gravado em", saida, "(icone.svg, icone-pequeno.svg, icone.ico com", pngs.length, "tamanhos, previa.html)");
