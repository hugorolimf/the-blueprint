// O estilo das figuras de um pacote, pronto para quem não entende var() nem color-mix() (o resvg do app): troca as variáveis pelas
// cores do tema, calcula as misturas e tira o que não é desenho. Genérico: serve para o CSS de qualquer pacote.

export const TEMA = {
  papel: "#0c3a78", cartao: "#082b5e", campo: "#081d3c", botao: "#16407a", escolhido: "#1f5299", texto: "#c9dcf3", forte: "#eaf4ff", medio: "#9cc4ee", linha: "#5b93d0", tenue: "#2f66aa", borda: "#3d72b3",
  vermelho: "#ff8f80", ambar: "#ffcf6b", verde: "#8be3b5", azul: "#bfe0ff", destaque: "#d9b8ff", mapa: "#0b3268", quadra: "#14509b", topo: "#1250a0", face: "#123866", sombra: "#0c2a55",
};

const hex = (c) => {
  const h = c.trim().slice(1);
  const v = h.length === 3 ? [...h].map((x) => x + x) : [h.slice(0, 2), h.slice(2, 4), h.slice(4, 6)];
  return v.map((x) => parseInt(x, 16));
};
const mistura = (a, p, b) => {
  const [x, y] = [a, b === "transparent" ? a : b].map(hex);
  return "#" + x.map((v, i) => Math.round((v * p + y[i] * (100 - p)) / 100).toString(16).padStart(2, "0")).join("");
};

/** Tira blocos @… {…} com chaves aninhadas. */
function semArroba(css) {
  let out = "", i = 0;
  while (i < css.length) {
    const a = css.indexOf("@", i);
    if (a < 0) return out + css.slice(i);
    out += css.slice(i, a);
    let j = css.indexOf("{", a), d = 1;
    for (j++; d && j < css.length; j++) d += css[j] === "{" ? 1 : css[j] === "}" ? -1 : 0;
    i = j;
  }
  return out;
}

/** `css` do pacote + `tema` do pacote (variável → nome de cor do app; sem nome, a cor original da figura) → { css só com cores prontas, vars }. */
export function compilarEstilo(css, tema = {}, cores = TEMA) {
  let s = semArroba(css.replace(/\/\*[\s\S]*?\*\//g, ""));
  const vars = Object.fromEntries(Object.entries(cores).map(([k, v]) => [`--${k}`, v]));
  for (const m of s.matchAll(/^\s*\.[\w-]+\s*\{([^}]*--[^}]*)\}/gm)) for (const d of m[1].matchAll(/(--[\w-]+)\s*:\s*([^;]+)/g)) vars[d[1]] ??= d[2].trim();
  for (const [v, nome] of Object.entries(tema)) if (cores[nome]) vars[v] = cores[nome];
  s = s.replace(/[^{}]*\{[^{}]*--[\w-]+\s*:[^{}]*\}/g, "");
  return { css: resolver(s, vars), vars };
}

/** Troca var() e color-mix() por cores prontas, no CSS ou nos atributos de uma figura. */
export function resolver(s, vars) {
  for (let n = 0; n < 4; n++) s = s.replace(/var\((--[\w-]+)(?:,\s*([^)]+))?\)/g, (_, v, alt) => vars[v] ?? alt ?? "currentColor");
  s = s.replace(/color-mix\(in srgb,\s*currentColor\s+\d+%,\s*transparent\s*\)/gi, "currentColor");
  return s.replace(/color-mix\(in srgb,\s*(#[0-9a-f]+)\s+(\d+)%,\s*(#[0-9a-f]+|transparent)\s*\)/gi, (_, a, p, b) => mistura(a, +p, b.toLowerCase()));
}

/** Prende cada regra do CSS num escopo, para o estilo de um pacote não vazar para o resto da página. */
export const comEscopo = (css, escopo) =>
  css.replace(/([^{}]+)\{/g, (_, sel) => sel.split(",").map((p) => `${escopo} ${p.trim()}`).join(", ") + " {");
