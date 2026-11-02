// Peças de desenho do protótipo do Planejar, no visual do app: papel azul com a grade de 12 e 60 px, traço claro, Cascadia Mono,
// rótulos em caixa alta com espaçamento, botões contornados e um só botão cheio por tela. Tudo em SVG, porque a nota do Obsidian
// tira <style>: cada figura leva as cores nos próprios atributos.

export const COR = { papel: "#0c3a78", fundo: "#082b5e", claro: "#1250a0", tinta: "#eaf4ff", medio: "#9cc4ee", baixo: "#5b93d0", tenue: "#2f66aa", erro: "#ffb4a8", barra: "#082b5e" };
export const FONTE = "'Cascadia Mono','Cascadia Code',ui-monospace,Consolas,monospace";
/** A Cascadia avança 1200/2048 do tamanho por letra. */
const AVANCO = 0.5859375;

export const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const cor = (c) => COR[c] ?? c;

/** A largura do texto como a página mede: avanço por letra mais o espaçamento (em em) depois de cada uma. */
export const largura = (t, tam = 12, trilha = 0) => [...String(t)].length * (AVANCO + trilha) * tam;

/** Texto com a linha de base em `y`. `caixa`: caixa alta com espaçamento .1em (os rótulos e botões do app). */
export function txt(x, y, t, { tam = 12, c = "tinta", peso = 400, caixa = false, trilha, fim = false, meio = false, max, op } = {}) {
  let s = caixa ? String(t).toUpperCase() : String(t);
  const tr = trilha ?? (caixa ? 0.1 : 0);
  if (max && largura(s, tam, tr) > max) {
    const n = Math.max(1, Math.floor(max / ((AVANCO + tr) * tam)) - 1);
    s = [...s].slice(0, n).join("") + "…";
  }
  const ancora = fim ? ' text-anchor="end"' : meio ? ' text-anchor="middle"' : "";
  return `<text x="${x}" y="${y}" font-size="${tam}" font-weight="${peso}" fill="${cor(c)}"${tr ? ` letter-spacing="${(tr * tam).toFixed(2)}"` : ""}${ancora}${op != null ? ` opacity="${op}"` : ""}>${esc(s)}</text>`;
}

export function ret(x, y, w, h, { fill = "none", stroke, sw = 1, dash, op, rx } = {}) {
  const tr = stroke ? ` stroke="${cor(stroke)}" stroke-width="${sw}"${dash ? ` stroke-dasharray="${dash}"` : ""}` : "";
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${cor(fill)}"${tr}${rx ? ` rx="${rx}"` : ""}${op != null ? ` opacity="${op}"` : ""}/>`;
}

export const fio = (x1, y1, x2, y2, { c = "baixo", sw = 1, dash, op } = {}) =>
  `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${cor(c)}" stroke-width="${sw}"${dash ? ` stroke-dasharray="${dash}"` : ""}${op != null ? ` opacity="${op}"` : ""}/>`;

export const circulo = (cx, cy, r, { fill = "none", stroke = "tinta", sw = 1, op } = {}) =>
  `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${cor(fill)}" stroke="${cor(stroke)}" stroke-width="${sw}"${op != null ? ` opacity="${op}"` : ""}/>`;

/** O losango de checkpoint (cheio quando feito). */
export const losango = (cx, cy, r, feito = false) =>
  `<path d="M${cx} ${cy - r}L${cx + r} ${cy}L${cx} ${cy + r}L${cx - r} ${cy}Z" fill="${feito ? COR.tinta : COR.papel}" stroke="${COR.tinta}" stroke-width="1.4"/>`;

/** A caixa de marcar dos planos. */
export const caixinha = (x, y, feita = false) =>
  ret(x, y, 13, 13, { fill: "papel", stroke: "tinta" }) + (feita ? `<path d="M${x + 3} ${y + 7}l3 3l5 -6" fill="none" stroke="${COR.tinta}" stroke-width="1.4"/>` : "");

/** Botão do app: caixa alta, 11 px, contorno médio; `cheio` é o primário (tinta com letra de papel). Devolve [svg, largura]. */
export function botao(x, y, t, { cheio = false, mini = false, ativo } = {}) {
  const tam = mini ? 10 : 11, px = mini ? 9 : 14, h = mini ? 24 : 32;
  const w = Math.round(largura(t.toUpperCase(), tam, 0.1) + 2 * px);
  const marcado = cheio || ativo;
  const s = ret(x, y, w, h, { fill: marcado ? "tinta" : "#0c3a78cc", stroke: marcado ? "tinta" : "medio" }) +
    txt(x + px, y + h / 2 + tam * 0.36, t, { tam, caixa: true, c: marcado ? "papel" : "tinta", peso: cheio ? 700 : 400 });
  return [s, w];
}

/** A cantoneira dos cartões do app (o «L» de tinta no canto de cima). */
export const canto = (x, y, l = 22) => `<path d="M${x} ${y + l}V${y}H${x + l}" fill="none" stroke="${COR.tinta}" stroke-width="2"/>`;

/** Um cartão: fundo um pouco mais escuro, contorno baixo (tracejado se `pontilhado`) e a cantoneira. */
export const cartao = (x, y, w, h, { pontilhado = false, marcado = false } = {}) =>
  ret(x, y, w, h, { fill: marcado ? "#14509b" : "#082b5e", stroke: marcado ? "tinta" : "baixo", dash: pontilhado ? "4 3" : undefined, op: 1 }) + canto(x, y);

/** Rótulo de seção: 10 px, caixa alta, médio, e a linha baixa embaixo até `x + w`. */
export const rotulo = (x, y, t, w) => txt(x, y, t, { tam: 10, caixa: true, c: "medio", trilha: 0.14 }) + (w ? fio(x, y + 7, x + w, y + 7, { c: "baixo", op: 0.8 }) : "");

/** Etiqueta pequena invertida (NOVA, ALTERADA, Pn…). */
export function etiqueta(x, y, t, { cheia = true, c = "tinta" } = {}) {
  const w = Math.round(largura(t.toUpperCase(), 9, 0.1) + 12);
  return [ret(x, y - 10, w, 14, { fill: cheia ? c : "none", stroke: c }) + txt(x + 6, y, t, { tam: 9, caixa: true, peso: 700, c: cheia ? "papel" : c }), w];
}

/** O papel do app com a grade e o brilho no meio; `id` único por figura (várias moram na mesma nota). */
export function papel(id, w, h) {
  return `<defs><radialGradient id="${id}-luz" cx="50%" cy="38%" r="75%"><stop offset="0" stop-color="${COR.claro}"/><stop offset=".62" stop-color="${COR.papel}"/><stop offset="1" stop-color="${COR.fundo}"/></radialGradient>` +
    `<pattern id="${id}-g" width="12" height="12" patternUnits="userSpaceOnUse"><path d="M12 0H0V12" fill="none" stroke="#fff" stroke-opacity=".07"/></pattern>` +
    `<pattern id="${id}-G" width="60" height="60" patternUnits="userSpaceOnUse"><path d="M60 0H0V60" fill="none" stroke="#fff" stroke-opacity=".11"/></pattern></defs>` +
    `<rect width="${w}" height="${h}" fill="url(#${id}-luz)"/><rect width="${w}" height="${h}" fill="url(#${id}-g)"/><rect width="${w}" height="${h}" fill="url(#${id}-G)"/>`;
}

/** A figura inteira: `corpo` desenhado sobre o papel, com a fonte do app. */
export const quadro = (id, w, h, corpo, legenda) =>
  `<svg viewBox="0 0 ${w} ${h}" width="100%" role="img" aria-label="${esc(legenda)}" style="display:block;font-family:${FONTE}" font-family="${FONTE}">${papel(id, w, h)}${corpo}</svg>`;

/** Texto em várias linhas: quebra por palavra até `max` de largura; devolve [svg, altura usada]. */
export function linhas(x, y, t, { tam = 11, passo, max, maxLinhas = 99, ...op } = {}) {
  const dy = passo ?? Math.round(tam * 1.5), saida = [];
  let atual = "";
  for (const p of String(t).split(/\s+/)) {
    const nova = atual ? `${atual} ${p}` : p;
    if (largura(nova, tam) > max && atual) { saida.push(atual); atual = p; } else atual = nova;
  }
  if (atual) saida.push(atual);
  const corte = saida.length > maxLinhas;
  const vis = saida.slice(0, maxLinhas);
  if (corte) vis[maxLinhas - 1] = vis[maxLinhas - 1].replace(/\s*\S*$/, "") + "…";
  return [vis.map((l, i) => txt(x, y + i * dy, l, { tam, ...op })).join(""), vis.length * dy];
}

/** Uma figura do plano dentro de um quadro (x, y, w, h), inteira e centrada; sem SVG, o nome do arquivo. */
export function figuraEm(d, x, y, w, h) {
  if (!d.svg) return ret(x, y, w, h, { fill: "#082b5e", stroke: "baixo", dash: "4 3" }) + txt(x + w / 2, y + h / 2 + 4, d.figura ?? d.titulo ?? "", { tam: 10, c: "medio", meio: true, max: w - 16 });
  return d.svg.replace(/<\?xml[^>]*>/, "").replace(/<svg\b([^>]*)>/, (_, a) => `<svg${a.replace(/\s(xmlns|width|height|x|y|style|role)="[^"]*"/g, "")} x="${x}" y="${y}" width="${Math.max(1, w)}" height="${Math.max(1, h)}">`);
}
