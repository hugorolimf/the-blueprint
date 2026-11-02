// A animação da escrita em tempo real, em CSS: o Obsidian tira o <animate> do SVG e mantém as classes. Cada momento vira uma classe com o
// seu @keyframes, e cssAnimacao() devolve as que foram usadas. Sem CSS, a tela fica no estado final, sem quebrar. Na interface nativa, isto
// é o próprio plano chegando pelo MCP, peça por peça.
export const CICLO = 22;
const pct = (t) => ((t / CICLO) * 100).toFixed(2);
const mais = (t) => (Number(pct(t)) + 0.01).toFixed(2);
const usadas = new Map();

function classe(nome, quadros, dur = CICLO) {
  usadas.set(nome, `@keyframes ${nome}{${quadros}}.${nome}{animation:${nome} ${dur}s infinite}`);
  return nome;
}
const marca = (t) => pct(t).replace(".", "_");

/** Aparece no instante `t` do ciclo. */
export const aparece = (svg, t) => `<g class="${classe(`bp-an-a${marca(t)}`, `0%,${pct(t)}%{opacity:0}${mais(t)}%,100%{opacity:1}`)}">${svg}</g>`;
/** Visível só entre `t0` e `t1`; parado, fica escondido. */
export const entre = (svg, t0, t1) => `<g opacity="0" class="${classe(`bp-an-e${marca(t0)}-${marca(t1)}`, `0%,${pct(t0)}%{opacity:0}${mais(t0)}%,${pct(t1)}%{opacity:1}${mais(t1)}%,100%{opacity:0}`)}">${svg}</g>`;
/** Pisca sem parar (o cursor, o ponto de quem está trabalhando). */
export const pisca = (svg, d = 1) => `<g class="${classe(`bp-an-p${String(d).replace(".", "_")}`, "0%,49%{opacity:1}50%,100%{opacity:0}", d)}">${svg}</g>`;

let n = 0;
/** Escreve da esquerda para a direita entre `t0` e `t1`, como quem digita. */
export function digita(svg, x, y, w, h, t0, t1) {
  const id = `dg${++n}`, larg = Math.ceil(w);
  const c = classe(`bp-an-d${marca(t0)}-${marca(t1)}-${larg}`, `0%,${pct(t0)}%{width:0}${pct(t1)}%,100%{width:${larg}px}`);
  return `<clipPath id="${id}"><rect class="${c}" x="${x}" y="${y}" height="${h}" width="${larg}"/></clipPath><g clip-path="url(#${id})">${svg}</g>`;
}

/** O CSS das animações usadas até aqui. */
export const cssAnimacao = () => [...usadas.values()].join("\n");
