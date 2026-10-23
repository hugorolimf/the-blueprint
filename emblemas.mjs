// Marca de cada folha: um objeto isométrico montado com as peças do kit do anatomy (lajes, cilindros, rodas, anéis, linhas, texto).
// A marca é só dados (uma lista de peças): quem a desenha é o kit, nunca um SVG escrito por fora. Serve ao navegador e ao Node.
// A ordem da lista é a ordem de construção: com o progresso do blueprint, as primeiras peças aparecem prontas (sólidas)
// e as seguintes ficam só no traço (como no blueprint, ainda por fazer).
import * as k from "./kit/iso-kit.mjs";
import * as G from "./kit/lathe.mjs";

// além dos tons de azul do blueprint, os do Claude (agentes-claude.mjs): corpo laranja, olho escuro, a cor de apoio de cada papel e aço/papel das ferramentas (web/css/claude.css)
const TONS = new Set(["hi", "mid", "lo", "faint", "lit", "claude", "olho", "exp", "con", "rev", "aco", "papel"]);
const TIPOS = new Set(["laje", "cilindro", "prisma", "anel", "serrilha", "linha", "risco", "pontos", "texto", "roda", "aro", "quadro"]);
// `libre` marca a ponta de uma linha que é feita para terminar no ar (ponteiro, traço de texto, régua); só para isso (o `lines.mjs` do anatomy confere o resto)
const livre = (v) => (v === true || v === "start" || v === "end" ? { libre: v } : {});
const num = (v, padrao, min, max) => { const x = Number(v); return Number.isFinite(x) ? Math.min(max, Math.max(min, x)) : padrao; };

// Aceita o que vier (inclusive de uma IA) e devolve só peças válidas, com números dentro de limites seguros.
// `limite` é o máximo de peças: 160 para o que vem de fora (IA, proposta); a figura do agente, que é nossa, pede mais (até LIMITE_MAXIMO).
export const LIMITE_PADRAO = 160, LIMITE_MAXIMO = 400;
export function sanitizaPecas(lista, limite = LIMITE_PADRAO) {
  if (!Array.isArray(lista)) return [];
  const pt = (q) => [num(q?.[0], 0, -320, 320), num(q?.[1], 0, -320, 320)];
  return lista.slice(0, Math.min(LIMITE_MAXIMO, Math.max(1, limite))).map((p) => {
    if (!p || !TIPOS.has(p.tipo)) return null;
    const o = { tipo: p.tipo, tom: TONS.has(p.tom) ? p.tom : "mid" };
    if (p.ordem !== undefined && p.ordem !== null) o.ordem = num(p.ordem, 0, 0, 500);
    const xy = (a, b) => ({ [a]: num(p[a], 0, -320, 320), [b]: num(p[b], 0, -320, 320) });
    switch (p.tipo) {
      case "laje": return { ...o, ...xy("x", "y"), w: num(p.w, 40, 1, 420), d: num(p.d, 40, 1, 420), z: num(p.z, 0, 0, 420), h: num(p.h, 10, 0.2, 320), r: num(p.r, 0, 0, 60), borda: num(p.borda, 0, 0, 3) };
      case "cilindro": return { ...o, ...xy("cx", "cy"), r: num(p.r, 20, 0.5, 260), z: num(p.z, 0, 0, 420), h: num(p.h, 10, 0.2, 320), borda: num(p.borda, 0, 0, 3) };
      case "prisma": { const pontos = (Array.isArray(p.pontos) ? p.pontos : []).slice(0, 12).map(pt); return pontos.length < 3 ? null : { ...o, pontos, z: num(p.z, 0, 0, 420), h: num(p.h, 4, 0.2, 320) }; }
      case "anel": return { ...o, ...xy("cx", "cy"), r: num(p.r, 20, 0.5, 260), z: num(p.z, 0, 0, 420) };
      case "serrilha": return { ...o, ...xy("cx", "cy"), r: num(p.r, 20, 0.5, 260), z: num(p.z, 0, 0, 420), h: num(p.h, 10, 0.2, 320), n: Math.round(num(p.n, 48, 6, 96)), ...(p.inset === undefined ? {} : { inset: num(p.inset, 0.4, 0, 2) }) };
      case "linha": return { ...o, ...xy("x", "y"), ...xy("x2", "y2"), z: num(p.z, 0, 0, 420), ...livre(p.libre) };
      case "risco": return { ...o, ...xy("x", "y"), ...xy("x2", "y2"), z: num(p.z, 0, 0, 420), z2: num(p.z2, 0, 0, 420), ...livre(p.libre) };
      case "quadro": return { ...o, plano: p.plano === "x" ? "x" : "y", ...xy("x", "y"), ...xy("x2", "y2"), z: num(p.z, 0, 0, 420), z2: num(p.z2, 0, 0, 420) }; // contorno fechado numa face em pé (placa, veneziana)
      case "pontos": { const pts = (Array.isArray(p.pts) ? p.pts : []).slice(0, 60).map((q) => [...pt(q), num(q?.[2], 0, 0, 420)]); return pts.length ? { ...o, pts, tam: num(p.tam, 0.5, 0.2, 2) } : null; }
      case "texto": { const texto = String(p.texto ?? "").slice(0, 12); return texto ? { ...o, ...xy("x", "y"), z: num(p.z, 0, 0, 420), texto, tam: num(p.tam, 10, 2, 140), ancora: p.ancora === "start" ? "start" : "middle" } : null; }
      case "roda": return { ...o, ...xy("cx", "cy"), z: num(p.z, 20, 0, 420), r: num(p.r, 20, 1, 120), w: num(p.w, 12, 1, 80), eixo: p.eixo === "x" ? "x" : "y" };
      case "aro": return { ...o, ...xy("cx", "cy"), z: num(p.z, 20, 0, 420), r: num(p.r, 8, 0.5, 120), eixo: p.eixo === "x" ? "x" : "y" };
    }
    return null;
  }).filter(Boolean);
}

// Desenha a marca: devolve { svg, largura, altura, feitas, total } com o miolo do SVG (câmera isométrica ajustada à caixa da peça).
// progresso (0 a 1): fração das peças, na ordem da lista, que aparecem prontas; as demais ficam só no traço.
export function desenhaEmblema(pecas, { largura = 420, altura = 320, pad = 10, azimute = 45, progresso = 1, sombra = true, limite = LIMITE_PADRAO, enquadra = [] } = {}) {
  const ps = sanitizaPecas(pecas, limite);
  const pts = enquadra.map(([x, y, z]) => [num(x, 0, -320, 320), num(y, 0, -320, 320), num(z, 0, 0, 420)]); // pontos que só entram na caixa do enquadramento (a câmera não muda entre poses)
  for (const p of ps) {
    if (p.tipo === "laje") pts.push(...k.boxCorners({ x: p.x, y: p.y, w: p.w, d: p.d }, p.z, p.z + p.h));
    else if (p.tipo === "cilindro" || p.tipo === "serrilha") pts.push(...k.boxCorners({ x: p.cx - p.r, y: p.cy - p.r, w: 2 * p.r, d: 2 * p.r }, p.z, p.z + p.h));
    else if (p.tipo === "prisma") p.pontos.forEach(([x, y]) => pts.push([x, y, p.z], [x, y, p.z + p.h]));
    else if (p.tipo === "anel") pts.push([p.cx - p.r, p.cy - p.r, p.z], [p.cx + p.r, p.cy + p.r, p.z]);
    else if (p.tipo === "linha") pts.push([p.x, p.y, p.z], [p.x2, p.y2, p.z]);
    else if (p.tipo === "risco") pts.push([p.x, p.y, p.z], [p.x2, p.y2, p.z2]);
    else if (p.tipo === "quadro") pts.push([p.x, p.y, p.z], [p.plano === "y" ? p.x2 : p.x, p.plano === "y" ? p.y : p.y2, p.z2]);
    else if (p.tipo === "pontos") p.pts.forEach((q) => pts.push(q));
    else if (p.tipo === "roda") pts.push(...(p.eixo === "y" ? [[p.cx - p.r, p.cy - p.w / 2, p.z - p.r], [p.cx + p.r, p.cy + p.w / 2, p.z + p.r]] : [[p.cx - p.w / 2, p.cy - p.r, p.z - p.r], [p.cx + p.w / 2, p.cy + p.r, p.z + p.r]]));
  }
  if (!pts.length) return null;
  const P = k.fitProjection(pts, largura, altura, { pad, azimuth: azimute });
  const S = (paths, tone) => k.solidSvg(paths, { tone, crease: tone === "hi" ? "lo" : "faint" });
  const L = (d, tone, free) => k.lineSvg(d, { tone, ...(free ? { free } : {}) });
  const prof = (x, y) => k.depthOf([x, y, 0], P);
  const eixoVetor = (e) => (e === "y" ? [0, 1, 0] : [1, 0, 0]);
  const itens = ps.map((p, i) => {
    let it = null;
    switch (p.tipo) {
      case "laje": it = { z0: p.z, p: prof(p.x + p.w / 2, p.y + p.d / 2), svg: S(k.slabOf({ x: p.x, y: p.y, w: p.w, d: p.d, r: p.r }, p.z, p.h, P, p.r > 0 ? 5 : 2, p.borda), p.tom) }; break;
      case "cilindro": it = { z0: p.z, p: prof(p.cx, p.cy), svg: S(k.cylinder(p.cx, p.cy, p.r, p.z, p.h, P, Math.max(18, Math.min(60, Math.round(p.r * 0.7))), p.borda), p.tom) }; break;
      case "prisma": it = { z0: p.z, p: prof(p.pontos.reduce((s, q) => s + q[0], 0) / p.pontos.length, p.pontos.reduce((s, q) => s + q[1], 0) / p.pontos.length), svg: S(k.extrude(p.pontos, p.z, p.h, P, { convex: true }), p.tom) }; break;
      case "anel": it = { z0: p.z + 0.01, p: prof(p.cx, p.cy), svg: L(k.ring(p.cx, p.cy, p.r, p.z, P, Math.max(24, Math.round(p.r * 0.8))), p.tom) }; break;
      case "serrilha": it = { z0: p.z + 0.001, p: prof(p.cx, p.cy), svg: L(k.knurl(p.cx, p.cy, p.r, p.z, p.z + p.h, p.n, P, p.inset), p.tom) }; break;
      case "linha": it = { z0: p.z + 0.01, p: prof((p.x + p.x2) / 2, (p.y + p.y2) / 2), svg: L(k.lineOnTop([p.x, p.y], [p.x2, p.y2], p.z, P), p.tom, p.libre) }; break;
      case "risco": it = { z0: Math.min(p.z, p.z2) + 0.01, p: prof((p.x + p.x2) / 2, (p.y + p.y2) / 2), svg: L(k.pathOf([k.iso([p.x, p.y, p.z], P), k.iso([p.x2, p.y2, p.z2], P)]), p.tom, p.libre) }; break;
      case "quadro": { const q = p.plano === "y" ? [[p.x, p.y, p.z], [p.x2, p.y, p.z], [p.x2, p.y, p.z2], [p.x, p.y, p.z2]] : [[p.x, p.y, p.z], [p.x, p.y2, p.z], [p.x, p.y2, p.z2], [p.x, p.y, p.z2]]; it = { z0: Math.min(p.z, p.z2) + 0.01, p: prof((p.x + p.x2) / 2, (p.y + p.y2) / 2), svg: L(k.pathOf(q.map((c) => k.iso(c, P)), true), p.tom) }; break; }
      case "pontos": it = { z0: p.pts[0][2] + 0.01, p: prof(p.pts[0][0], p.pts[0][1]), svg: k.dotsSvg(p.pts.map((q) => k.iso(q, P)), { size: p.tam, tone: p.tom }) }; break;
      case "texto": it = { z0: p.z + 0.02, p: prof(p.x, p.y), svg: k.faceTextSvg(k.topMatrix([p.x, p.y], p.z, P, "x"), p.texto, { size: p.tam, tone: p.tom, anchor: p.ancora }) }; break;
      case "roda": it = { z0: p.z - p.r, p: prof(p.cx, p.cy), svg: S(G.disc(-p.w / 2, p.w / 2, p.r, G.frameAlong([p.cx, p.cy, p.z], eixoVetor(p.eixo)), P), p.tom) }; break;
      case "aro": it = { z0: p.z - p.r + 0.02, p: prof(p.cx, p.cy), svg: L(k.sideRing([p.cx, p.cy, p.z], p.r, p.eixo === "y" ? "y" : "x", P, Math.max(16, Math.round(p.r))), p.tom) }; break;
    }
    return it && { ...it, z0: p.ordem ?? it.z0, i };
  }).filter(Boolean);
  // do chão para cima e, no mesmo nível, de trás para a frente: o que está apoiado em algo é desenhado depois dele
  itens.sort((a, b) => a.z0 - b.z0 || a.p - b.p || a.i - b.i);
  const prontas = Math.round(ps.length * Math.min(1, Math.max(0, progresso)));
  const xs = pts.map((q) => q[0]), ys = pts.map((q) => q[1]);
  const halo = `<path class="iso-halo" d="${k.haloOf({ x: Math.min(...xs), y: Math.min(...ys), w: Math.max(...xs) - Math.min(...xs), d: Math.max(...ys) - Math.min(...ys), r: 14 }, 0, 2, P)}"/>`;
  const miolo = itens.map((x) => (x.i < prontas ? x.svg : `<g class="fantasma-emb">${x.svg}</g>`)).join("");
  // `ordem`: os índices (da lista já sanitizada) na ordem em que foram pintados, de trás para a frente; a auditoria do anatomy confere esta ordem contra a profundidade de verdade
  return { svg: (sombra ? halo : "") + miolo, largura, altura, feitas: prontas, total: ps.length, ordem: itens.map((x) => x.i) };
}

// ---- marcas prontas -------------------------------------------------------------------------------
const roda = (n, f) => Array.from({ length: n }, (_, i) => f(i));
const T = (cx, cy, r, z, h, tom, extra = {}) => ({ tipo: "cilindro", cx, cy, r, z, h, tom, ...extra });
const moeda = (cx, cy, r, z, h, tom, n, extra = {}) => [T(cx, cy, r, z, h, tom, extra), { tipo: "serrilha", cx, cy, r, z, h, n, tom: "lo" }];

// moedas empilhadas com um cifrão grande na de cima: o financeiro (SM Cash)
export const MOEDAS = [
  ...moeda(14, 10, 92, 0, 14, "mid", 72), ...moeda(-6, -6, 92, 14, 14, "mid", 72), ...moeda(8, 4, 92, 28, 14, "mid", 72), ...moeda(0, 0, 92, 42, 14, "hi", 72, { borda: 1.2 }),
  { tipo: "anel", cx: 0, cy: 0, r: 80, z: 56, tom: "mid" }, { tipo: "anel", cx: 0, cy: 0, r: 74, z: 56, tom: "lo" },
  { tipo: "texto", x: 0, y: 44, z: 56, texto: "$", tam: 118, tom: "hi", ancora: "middle" },
  ...moeda(150, 80, 40, 0, 8, "lo", 40, { borda: 0.6 }), { tipo: "texto", x: 150, y: 94, z: 8, texto: "$", tam: 46, tom: "mid" },
  ...moeda(-130, 70, 34, 0, 7, "lo", 36, { borda: 0.6 }), { tipo: "texto", x: -130, y: 82, z: 7, texto: "$", tam: 38, tom: "mid" },
];

// livros, um tablet e o botão de play: a plataforma de treinamento
export const LIVROS = [
  { tipo: "laje", x: -105, y: -58, w: 200, d: 136, z: 0, h: 18, r: 4, tom: "lo", borda: 0.6 },
  { tipo: "linha", x: -105, y: 78, x2: 95, y2: 78, z: 9, tom: "faint" },
  { tipo: "laje", x: -92, y: -50, w: 176, d: 124, z: 18, h: 16, r: 4, tom: "mid", borda: 0.6 },
  { tipo: "laje", x: -108, y: -54, w: 192, d: 128, z: 34, h: 14, r: 4, tom: "lo", borda: 0.6 },
  { tipo: "linha", x: -108, y: 74, x2: 84, y2: 74, z: 41, tom: "faint" },
  { tipo: "laje", x: -78, y: -44, w: 156, d: 108, z: 48, h: 8, r: 9, tom: "hi", borda: 1 },
  { tipo: "laje", x: -68, y: -34, w: 136, d: 88, z: 56, h: 0.8, r: 4, tom: "lo" },
  { tipo: "prisma", pontos: [[-16, 0], [-16, 44], [26, 22]], z: 56.8, h: 7, tom: "hi" },
  { tipo: "pontos", pts: [[-72, -38, 56], [72, -38, 56], [-72, 58, 56], [72, 58, 56]], tam: 0.7, tom: "mid" },
];

// chip de circuito com "IA" no centro: a integração de IA
export const CHIP = [
  { tipo: "laje", x: -140, y: -140, w: 280, d: 280, z: 0, h: 6, r: 12, tom: "lo", borda: 0.6 },
  { tipo: "laje", x: -92, y: -92, w: 184, d: 184, z: 6, h: 16, r: 8, tom: "mid", borda: 1 },
  { tipo: "laje", x: -56, y: -56, w: 112, d: 112, z: 22, h: 8, r: 4, tom: "hi", borda: 0.8 },
  ...roda(10, (i) => ({ tipo: "laje", x: -76 + i * 17, y: -118, w: 9, d: 30, z: 6.5, h: 4, tom: "mid" })),
  ...roda(10, (i) => ({ tipo: "laje", x: -118, y: -76 + i * 17, w: 30, d: 9, z: 6.5, h: 4, tom: "mid" })),
  ...roda(10, (i) => ({ tipo: "laje", x: -76 + i * 17, y: 88, w: 9, d: 30, z: 6.5, h: 4, tom: "mid" })),
  ...roda(10, (i) => ({ tipo: "laje", x: 88, y: -76 + i * 17, w: 30, d: 9, z: 6.5, h: 4, tom: "mid" })),
  ...[-36, -12, 12, 36].flatMap((v) => [
    { tipo: "linha", x: -56, y: v, x2: -92, y2: v, z: 22, tom: "lo" }, { tipo: "linha", x: 56, y: v, x2: 92, y2: v, z: 22, tom: "lo" },
    { tipo: "linha", x: v, y: -56, x2: v, y2: -92, z: 22, tom: "lo" }, { tipo: "linha", x: v, y: 56, x2: v, y2: 92, z: 22, tom: "lo" },
  ]),
  { tipo: "pontos", pts: [[-84, -84, 22], [84, -84, 22], [-84, 84, 22], [84, 84, 22]], tam: 0.8, tom: "mid" },
  { tipo: "texto", x: 0, y: 17, z: 30, texto: "IA", tam: 50, tom: "lo", ancora: "middle" },
];

// caminhão de logística: marca opcional de um workspace (`icone: "caminhao"`), na folha da lista de blueprints.
// Aponta para +x; as rodas da frente (lado de y positivo) vão por cima da carroceria (ordem 40).
export const CAMINHAO = [
  { tipo: "laje", x: -150, y: -40, w: 290, d: 80, z: 14, h: 8, tom: "lo" },
  ...[-105, -55, 92].map((cx) => ({ tipo: "roda", cx, cy: -58, z: 21, r: 21, w: 14, eixo: "y", tom: "lo" })),
  ...[-105, -55, 92].map((cx) => ({ tipo: "roda", cx, cy: 58, z: 21, r: 21, w: 14, eixo: "y", tom: "lo", ordem: 40 })),
  ...[-105, -55, 92].flatMap((cx) => [{ tipo: "aro", cx, cy: 65.2, z: 21, r: 12, eixo: "y", tom: "mid", ordem: 40.5 }, { tipo: "aro", cx, cy: 65.2, z: 21, r: 5, eixo: "y", tom: "faint", ordem: 40.5 }]),
  { tipo: "laje", x: -150, y: -52, w: 196, d: 104, z: 26, h: 92, r: 3, tom: "mid", borda: 0.8 },
  ...[-110, -70, -30, 10].map((x) => ({ tipo: "risco", x, y: 52, z: 26, x2: x, y2: 52, z2: 118, tom: "faint" })),
  { tipo: "risco", x: -150, y: 52, z: 58, x2: 46, y2: 52, z2: 58, tom: "faint" }, { tipo: "risco", x: -150, y: 52, z: 88, x2: 46, y2: 52, z2: 88, tom: "faint" },
  ...[-110, -70, -30, 10].map((x) => ({ tipo: "linha", x, y: -52, x2: x, y2: 52, z: 118, tom: "faint" })),
  { tipo: "laje", x: 50, y: -48, w: 78, d: 96, z: 26, h: 66, r: 6, tom: "hi", borda: 1 },
  { tipo: "laje", x: 52, y: -46, w: 58, d: 92, z: 92, h: 12, r: 4, tom: "mid" },
  { tipo: "laje", x: 127, y: -40, w: 2.5, d: 80, z: 58, h: 30, tom: "lo" },
  { tipo: "laje", x: 64, y: 47.5, w: 52, d: 2.5, z: 56, h: 28, tom: "lo" },
  { tipo: "risco", x: 90, y: 48, z: 28, x2: 90, y2: 48, z2: 54, tom: "faint" }, { tipo: "risco", x: 100, y: 48, z: 46, x2: 108, y2: 48, z2: 46, tom: "lo" },
  { tipo: "laje", x: 126, y: -50, w: 10, d: 100, z: 16, h: 10, tom: "mid" },
  { tipo: "laje", x: 134, y: -42, w: 3, d: 14, z: 32, h: 8, tom: "hi" }, { tipo: "laje", x: 134, y: 28, w: 3, d: 14, z: 32, h: 8, tom: "hi" },
  ...[-26, -10, 6, 22].map((y) => ({ tipo: "risco", x: 128, y, z: 36, x2: 128, y2: y, z2: 54, tom: "faint" })),
  { tipo: "texto", x: -52, y: 16, z: 118, texto: "SM", tam: 46, tom: "hi", ancora: "middle" },
];

// Prancheta de planejamento: o roadmap é impresso no papel, com ferramentas apoiadas na base.
export const BLUEPRINT = [
  ...[[-132, -92], [112, -92], [-132, 98], [112, 98]].map(([x, y]) => ({ tipo: "laje", x, y, w: 20, d: 16, z: 0, h: 5, r: 3, tom: "lo" })),
  { tipo: "laje", x: -148, y: -108, w: 296, d: 232, z: 5, h: 9, r: 7, borda: 1, tom: "mid" },
  { tipo: "laje", x: -143, y: -103, w: 286, d: 222, z: 14, h: 1, r: 4, tom: "lo" },
  { tipo: "laje", x: -125, y: -85, w: 250, d: 165, z: 15, h: 1, r: 1, tom: "hi" },
  // A grade termina nas bordas da folha, sem traços soltos fora do papel.
  ...roda(11, (i) => ({ tipo: "linha", x: -125 + i * 25, y: -85, x2: -125 + i * 25, y2: 80, z: 16, tom: "faint" })),
  ...roda(6, (i) => ({ tipo: "linha", x: -125, y: -85 + i * 33, x2: 125, y2: -85 + i * 33, z: 16, tom: "faint" })),
  // Presilha: base, duas dobradiças, ponte, mandíbula e parafusos.
  { tipo: "laje", x: -38, y: -101, w: 76, d: 16, z: 15, h: 3, r: 3, tom: "lo" },
  ...[-29, 22].map((x) => ({ tipo: "laje", x, y: -98, w: 7, d: 12, z: 18, h: 8, r: 2, tom: "mid" })),
  { tipo: "laje", x: -31, y: -97, w: 62, d: 6, z: 26, h: 3, r: 2, tom: "hi" },
  { tipo: "laje", x: -32, y: -82, w: 64, d: 6, z: 16, h: 3, r: 2, tom: "mid" },
  { tipo: "prisma", pontos: [[-30, -91], [30, -91], [30, -82], [-30, -82]], z: 19, h: 7, tom: "mid" },
  { tipo: "pontos", pts: [[-33, -91, 18], [33, -91, 18]], tam: 1, tom: "hi" },
  // Três faixas de planejamento, com uma etapa ativa e as próximas no papel.
  ...[-42, -8, 26].flatMap((y, i) => [
    { tipo: "laje", x: -104, y, w: 18, d: 10, z: 16, h: 0.5, r: 1, tom: "lo" },
    { tipo: "laje", x: -72 + i * 34, y, w: 65, d: 10, z: 16, h: 1, r: 1, tom: i === 0 ? "hi" : "mid" },
    { tipo: "linha", x: -7 + i * 34, y: y + 5, x2: 106, y2: y + 5, z: 16, tom: "lo" },
    { tipo: "prisma", pontos: [[100, y + 5], [106, y - 1], [112, y + 5], [106, y + 11]], z: 16, h: 1, tom: i === 0 ? "hi" : "mid" },
  ]),
  // Carimbo impresso no rodapé da folha.
  { tipo: "laje", x: 20, y: 55, w: 92, d: 18, z: 16, h: 0.3, tom: "lo" },
  { tipo: "linha", x: 50, y: 55, x2: 50, y2: 73, z: 16.3, tom: "mid" },
  { tipo: "linha", x: 20, y: 64, x2: 112, y2: 64, z: 16.3, tom: "mid" },
  { tipo: "texto", x: 80, y: 67, z: 16.3, texto: "BLUEPRINT", tam: 7, tom: "hi" },
  { tipo: "laje", x: -122, y: 86, w: 244, d: 12, z: 15, h: 1.5, r: 1, tom: "mid" },
  ...roda(23, (i) => ({ tipo: "linha", x: -110 + i * 10, y: 86, x2: -110 + i * 10, y2: i % 5 === 0 ? 95 : 91, z: 16.5, tom: i % 5 === 0 ? "hi" : "lo" })),
  // Lápis apoiado na canaleta inferior, com a ponta voltada para a direita.
  { tipo: "laje", x: -115, y: 104, w: 230, d: 9, z: 15, h: 0.5, r: 3, tom: "lo" },
  { tipo: "laje", x: -96, y: 105, w: 172, d: 7, z: 15.5, h: 5, r: 1, tom: "mid" },
  { tipo: "linha", x: -96, y: 108.5, x2: 76, y2: 108.5, z: 20.5, tom: "hi" },
  { tipo: "laje", x: -105, y: 105, w: 9, d: 7, z: 15.5, h: 5, r: 1, tom: "lo" },
  { tipo: "prisma", pontos: [[76, 105], [92, 108.5], [76, 112]], z: 15.5, h: 5, tom: "hi" },
  { tipo: "prisma", pontos: [[87, 107.4], [94, 108.5], [87, 109.6]], z: 15.5, h: 5, tom: "lo" },
];

export const MARCAS = { moedas: MOEDAS, livros: LIVROS, chip: CHIP, caminhao: CAMINHAO };

// Esquadro e compasso sobre uma mesa de desenho: a marca acompanha o aviso de recurso em construção.
const A = [-114, -52], B = [-2, -52], C = [-58, 39], Ai = [-105, -44], Bi = [-11, -44], Ci = [-58, 28];
const linha = (a, b, z, tom = "lo") => ({ tipo: "linha", x: a[0], y: a[1], x2: b[0], y2: b[1], z, tom });
const interpola = (a, b, t) => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
export const CONSTRUCAO = [
  ...[[-126, -84], [126, -84], [-126, 84], [126, 84]].map(([cx, cy]) => ({ tipo: "cilindro", cx, cy, r: 10, z: 0, h: 6, tom: "lo" })),
  { tipo: "laje", x: -140, y: -100, w: 280, d: 200, z: 6, h: 8, r: 7, borda: 1, tom: "mid" },
  { tipo: "laje", x: -136, y: -96, w: 272, d: 192, z: 14, h: 0.6, r: 5, tom: "lo" },
  { tipo: "laje", x: -132, y: -92, w: 264, d: 184, z: 14.6, h: 0.4, r: 3, tom: "hi" },
  ...roda(5, (i) => linha([-120 + i * 48, -80], [-120 + i * 48, 80], 15.1, "faint")),
  ...roda(4, (i) => linha([-120, -80 + i * 40], [120, -80 + i * 40], 15.1, "faint")),
  { tipo: "laje", x: -113, y: 82, w: 226, d: 10, z: 14.6, h: 1, r: 1, tom: "mid" },
  ...roda(8, (i) => linha([-104 + i * 29.7, 83], [-104 + i * 29.7, 87 + (i % 2 ? 0 : 3)], 15.6, i % 2 ? "lo" : "hi")),
  { tipo: "prisma", pontos: [A, B, Bi, Ai], z: 15.2, h: 5, tom: "mid" },
  { tipo: "prisma", pontos: [B, C, Ci, Bi], z: 15.2, h: 5, tom: "hi" },
  { tipo: "prisma", pontos: [C, A, Ai, Ci], z: 15.2, h: 5, tom: "mid" },
  ...[[A, B], [B, C], [C, A]].flatMap(([a, b]) => [linha(a, b, 20.3, "hi"), linha(a === A ? Ai : a === B ? Bi : Ci, b === A ? Ai : b === B ? Bi : Ci, 20.3, "lo")]),
  ...[[A, B, Ai, Bi], [B, C, Bi, Ci], [C, A, Ci, Ai]].flatMap(([a, b, ai, bi]) => roda(6, (i) => linha(interpola(a, b, (i + 1) / 7), interpola(ai, bi, (i + 1) / 7), 20.3, i % 2 ? "lo" : "faint"))),
  { tipo: "cilindro", cx: 57, cy: -3, r: 8, z: 55, h: 8, tom: "lo" },
  { tipo: "anel", cx: 57, cy: -3, r: 6, z: 63.1, tom: "hi" },
  { tipo: "laje", x: 49, y: -11, w: 16, d: 16, z: 63, h: 3.5, r: 2, tom: "mid" },
  { tipo: "cilindro", cx: 57, cy: -3, r: 4.5, z: 66.5, h: 2.5, tom: "hi" },
  { tipo: "serrilha", cx: 57, cy: -3, r: 4.5, z: 66.5, h: 2.5, n: 16, tom: "lo" },
  { tipo: "anel", cx: 57, cy: -3, r: 2, z: 69.1, tom: "mid" },
  ...[[[52, -3, 61], [19, 53, 17]], [[55, -3, 61], [22, 53, 17]], [[59, -3, 61], [88, 45, 17]], [[62, -3, 61], [91, 45, 17]]].map(([[x, y, z], [x2, y2, z2]], i) => ({ tipo: "risco", x, y, z, x2, y2, z2, tom: i < 2 ? "hi" : "mid" })),
  { tipo: "cilindro", cx: 20.5, cy: 53, r: 3, z: 15, h: 2, tom: "mid" },
  { tipo: "cilindro", cx: 89.5, cy: 45, r: 3, z: 15, h: 2, tom: "lo" },
  { tipo: "risco", x: 20.5, y: 53, z: 17, x2: 20.5, y2: 53, z2: 15, tom: "hi" },
  { tipo: "risco", x: 89.5, y: 45, z: 17, x2: 89.5, y2: 45, z2: 15, tom: "lo" },
  { tipo: "pontos", pts: [[-132, -92, 15], [132, -92, 15], [-132, 92, 15], [132, 92, 15]], tam: 0.8, tom: "mid" },
];
