// A folha desenhada a partir de um plano: a moldura de desenho técnico com o selo e a legenda, e a tabela com a régua (a ação e os seus
// números à esquerda, como uma planilha; a barra na data à direita). Nada aqui conhece um projeto específico.
import { txt, ret, fio, botao, circulo, losango, largura, COR } from "./svg.mjs";
import { acoesDe } from "./pacote.mjs";
import { pendentesDaAcao } from "./revisao.mjs";
import { TEMA } from "./estilo.mjs";

export const W = 1100, X0 = 52, X1 = W - 52, XR = 480;
export const LINHA = 38;
const dia = (s) => Date.parse(`${s}T12:00:00Z`) / 864e5;
const MES = ["JAN", "FEV", "MAR", "ABR", "MAI", "JUN", "JUL", "AGO", "SET", "OUT", "NOV", "DEZ"];
export const curta = (s) => (s ? `${s.slice(8, 10)}/${s.slice(5, 7)}` : "");
const COLUNAS = [["Planos", 348], ["Det.", 384], ["Dec.", 416], ["Aprovar", 468]];

/** A escala da régua: uma semana de passado e o hoje perto do começo, para o plano usar a largura toda. */
export function escala(p, hoje, x0 = XR, x1 = X1) {
  const datas = acoesDe(p).flatMap((a) => (a.ate ? [dia(a.ate)] : []));
  const h = dia(hoje), fim = Math.max(h + 30, ...datas) + 2, ini = h - (fim - h) * 0.08;
  const x = (s) => x0 + ((dia(s) - ini) / (fim - ini)) * (x1 - x0);
  return { x, x0, x1, ini, fim, hoje };
}

/** O selo no canto, em células, como numa prancha. */
function selo(h, celulas) {
  const w = celulas.reduce((s, [, , l]) => s + l, 0), y = h - 90;
  const out = [ret(X1 - w, y, w, 60, { fill: "#082b5e", stroke: "tinta" })];
  let x = X1 - w;
  celulas.forEach(([rot, val, l], i) => {
    if (i) out.push(fio(x, y, x, y + 60, { c: "medio" }));
    out.push(txt(x + 10, y + 19, rot, { tam: 9, caixa: true, c: "medio", trilha: 0.12 }), txt(x + 10, y + 42, val, { tam: 13, peso: 700, max: l - 18 }));
    x += l;
  });
  return out.join("");
}

/** A legenda dos símbolos, à esquerda do selo. */
function legenda(h) {
  const y = h - 54;
  return ret(X0, y - 8, 22, 12, { fill: "#082b5e", stroke: "medio" }) + txt(X0 + 28, y + 2, "ação", { tam: 10, c: "medio" }) +
    losango(X0 + 78, y - 2, 5) + txt(X0 + 88, y + 2, "checkpoint", { tam: 10, c: "medio" }) +
    ret(X0 + 170, y - 8, 22, 12, { stroke: "tinta", dash: "4 2" }) + txt(X0 + 198, y + 2, "proposta", { tam: 10, c: "medio" });
}

/** A moldura: colunas em cima, letras dos lados, título, botões, legenda e selo. */
export function moldura(h, { workspace, blueprint, folha = "Folha 1 de 1", botoes = ["Planejar", "Compartilhar", "Apresentar"] }) {
  const out = [ret(7, 7, W - 14, h - 14, { stroke: "baixo" }), ret(26, 26, W - 52, h - 52, { stroke: "tinta", sw: 1.6 })];
  [W * 0.25, W * 0.75].forEach((x, i) => out.push(txt(x, 21, i + 1, { tam: 9, c: "medio", meio: true })));
  out.push(fio(W / 2, 7, W / 2, 26, { c: "baixo" }));
  const passo = 150, nl = Math.round((h - 52) / passo);
  for (let i = 0; i < nl; i++) {
    const y = 26 + ((i + 0.5) * (h - 52)) / nl, L = String.fromCharCode(65 + (i % 26));
    out.push(txt(16.5, y + 3, L, { tam: 9, c: "medio", meio: true }), txt(W - 16.5, y + 3, L, { tam: 9, c: "medio", meio: true }));
    if (i) { const yl = 26 + (i * (h - 52)) / nl; out.push(fio(7, yl, 26, yl, { c: "baixo" }), fio(W - 26, yl, W - 7, yl, { c: "baixo" })); }
  }
  out.push(txt(X0, 62, blueprint, { tam: 18, caixa: true, peso: 700, trilha: 0.06, max: 400 }), txt(X0, 82, `${workspace} · ${folha}`, { tam: 10.5, caixa: true, c: "medio", trilha: 0.1, max: 400 }));
  let bx = X1;
  for (const t of [...botoes].reverse()) { const w = botao(0, 0, t, { mini: true })[1]; bx -= w; out.push(botao(bx, 50, t, { mini: true })[0]); bx -= 6; }
  out.push(legenda(h), selo(h, [["Workspace", workspace, 140], ["Blueprint", blueprint, 210], [folha.startsWith("Detalhe") ? "Detalhe" : "Folha", folha.split(" · ")[0].replace(/^(Folha|Detalhe) /, ""), 110]]));
  return out.join("");
}

/** A régua: meses, marcas semanais e a etiqueta do hoje, com a linha até `yFim`. Com `colunas`, escreve os títulos da tabela. */
export function regua(e, y, yFim, { colunas = true } = {}) {
  const out = [fio(e.x0, y, e.x1, y, { c: "tinta" })];
  const ate = new Date(e.fim * 864e5);
  for (let d = new Date(Math.ceil(e.ini) * 864e5); d <= ate; d = new Date(d.getTime() + 864e5)) {
    const s = d.toISOString().slice(0, 10), x = e.x(s);
    if (d.getUTCDate() === 1) out.push(fio(x, y - 9, x, y, { c: "tinta" }), (x + 30 < e.x1 ? txt(x + 4, y - 11, MES[d.getUTCMonth()], { tam: 10, trilha: 0.1 }) : ""));
    else if (d.getUTCDay() === 1) out.push(fio(x, y - 4, x, y, { c: "medio" }), txt(x, y + 12, String(d.getUTCDate()).padStart(2, "0"), { tam: 8.5, c: "medio", meio: true }));
  }
  const xh = e.x(e.hoje), t = `Hoje ${curta(e.hoje)}`, w = largura(t.toUpperCase(), 9.5, 0.1) + 12;
  out.push(fio(xh, y, xh, yFim, { c: "tinta", dash: "12 3 2 3", op: 0.8 }), ret(xh - w / 2, y - 24, w, 15, { fill: "tinta" }), txt(xh, y - 13, t, { tam: 9.5, caixa: true, peso: 700, c: "papel", meio: true }));
  if (colunas) {
    out.push(txt(X0, y + 1, "Ação", { tam: 9.5, caixa: true, c: "medio", trilha: 0.12 }));
    for (const [n, x] of COLUNAS) out.push(txt(x, y + 1, n, { tam: 9.5, caixa: true, c: "medio", trilha: 0.06, fim: true }));
  }
  return out.join("");
}

const numero = (x, y, n, op = {}) => txt(x, y, n || "—", { tam: 11, fim: true, c: n ? "tinta" : "baixo", ...op });

/** Os números de uma ação (ou a soma do projeto) nas colunas da tabela. */
const numeros = (y, [pl, de, dc, ap], op) => numero(348, y, pl, op) + numero(384, y, de, op) + numero(416, y, dc, op) + (ap ? txt(468, y, ap, { tam: 11, peso: 700, c: TEMA.ambar, fim: true }) : numero(468, y, 0));
const dec = (a, p) => (p.decisoes ?? []).filter((d) => d.acoes?.includes(a.id)).length;
export const numerosDaAcao = (a, p, c) => [a.planos?.length ?? 0, a.detalhes?.length ?? 0, dec(a, p), pendentesDaAcao(a, p, c)];

/** O cabeçalho de um projeto: o número, o nome, a soma das colunas e a linha que abraça as datas dele na régua. */
export function cabecalhoProjeto(n, pr, nomePadrao, p, c, e, y, { colunas = true, xFim = X1 } = {}) {
  const out = [circulo(X0 + 11, y - 4, 11, { fill: "papel", stroke: "tinta" }), txt(X0 + 11, y, String(n).padStart(2, "0"), { tam: 9.5, peso: 700, meio: true })];
  out.push(txt(X0 + 30, y, pr.nome ?? nomePadrao, { tam: 12, peso: 700, max: colunas ? 240 : e.x0 - X0 - 36 }));
  if (colunas) {
    const s = pr.acoes.map((a) => numerosDaAcao(a, p, c)).reduce((t, l) => t.map((v, i) => v + l[i]));
    out.push(numeros(y, s, { peso: 700 }));
  }
  const datas = pr.acoes.filter((a) => a.de);
  if (datas.length) {
    const x0 = Math.min(...datas.map((a) => e.x(a.de))), x1 = Math.max(...datas.map((a) => e.x(a.ate)));
    out.push(fio(x0, y - 4, x1, y - 4, { c: "tinta", sw: 2 }), fio(x0, y - 8, x0, y, { c: "tinta", sw: 2 }), fio(x1, y - 8, x1, y, { c: "tinta", sw: 2 }));
  }
  out.push(fio(X0, y + 11, xFim, y + 11, { c: "tinta", op: 0.7 }));
  return out.join("");
}

/** A barra da ação na data, com o checkpoint e as datas ao lado (ou antes, quando não cabe). */
export function barraAcao(e, a, y, { nova = false, alta = 16, exec } = {}) {
  if (!a.de) return txt(e.x(e.hoje) + 8, y + 4, "sem data", { tam: 10, c: "medio" });
  const x0 = e.x(a.de), w = e.x(a.ate) - x0, rot = `${curta(a.de)} → ${curta(a.ate)}`, cabe = x0 + w + 8 + largura(rot, 10) < e.x1;
  const out = [ret(x0, y - alta / 2, w, alta, { fill: nova ? "none" : "#082b5e", stroke: nova ? "tinta" : "medio", dash: nova ? "4 3" : undefined, sw: 1.2 })];
  const planos = a.planos ?? [], prontos = planos.filter((q) => exec?.[q.id]?.estado === "pronto").length;
  if (prontos) out.push(ret(x0, y - alta / 2, (w * prontos) / planos.length, alta, { fill: "tinta", op: 0.75 }));
  if (a.checkpoints?.length) out.push(losango(x0 + w, y, 5));
  out.push(cabe ? txt(x0 + w + 10, y + 4, rot, { tam: 10, c: "medio" }) : txt(x0 - 8, y + 4, rot, { tam: 10, c: "medio", fim: true }));
  return out.join("");
}

/** A linha fechada de uma ação: o id, o nome, os números e a barra. */
export function linhaAcao(e, a, p, c, y, { colunas = true, xNome = X0 + 30, marca } = {}) {
  const xn = xNome + 40, maxN = colunas ? 318 - xn : e.x0 - xn - 12;
  const out = [txt(xNome, y + 4, a.id, { tam: 11, c: "medio" }), txt(xn, y + 4, a.nome, { tam: 12, max: maxN })];
  if (colunas) out.push(numeros(y + 4, numerosDaAcao(a, p, c)));
  out.push(barraAcao(e, a, y, { nova: marca === "nova", exec: c.exec }));
  const rodando = (a.planos ?? []).filter((q) => c.exec?.[q.id]?.estado === "trabalhando").length;
  if (colunas && c.comp?.alteradas?.[a.id]) out.push(txt(318, y + 4, "alterado", { tam: 9, c: "#ffcf6b", fim: true }));
  if (rodando && a.de && c.figura) out.push(c.figura(e.x(a.de) - 25, y - 13, "trabalhando", 24), rodando > 1 ? txt(e.x(a.de) - 25, y + 13, `×${rodando}`, { tam: 9, peso: 700 }) : "");
  return out.join("");
}

export const separador = (y, xFim = X1) => fio(X0, y, xFim, y, { c: "baixo", op: 0.4 });
export { COR };
