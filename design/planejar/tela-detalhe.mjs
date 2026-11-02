// Tela 3: a folha de detalhe. A figura manda: ocupa a largura da folha, com a barra de aprovação logo embaixo dela; depois a legenda, as
// decisões da ação e a fita com os detalhes vizinhos. Figura baixa (uma barra, um recorte) fica centrada num campo de altura mínima.
import { txt, ret, fio, botao, rotulo, linhas, figuraEm, quadro } from "./svg.mjs";
import { W, X0, X1, moldura } from "./folha.mjs";
import { acoesDe } from "./pacote.mjs";
import { chaveDetalhe, estadoDe, situacao, canto, SITUACAO, notaRevisao } from "./revisao.mjs";
import { copia } from "./tela-folha.mjs";

export function telaDetalhe(p, id, idDet, c) {
  const a = acoesDe(p).find((x) => x.id === id), i = a.detalhes.findIndex((d) => d.id === idDet), d = a.detalhes[i], fw = X1 - X0, out = [];
  const [, , vw, vh] = (d.svg?.match(/viewBox="([^"]+)"/)?.[1] ?? "0 0 16 9").split(/\s+/).map(Number);
  const fig = Math.min(Math.round((fw * vh) / vw), 620), campo = Math.max(fig, 240);
  let y = 112;
  // a linha de navegação
  const [volta, lv] = botao(X0, y, `← ${a.id} ${a.nome}`, { mini: true });
  out.push(volta, txt(X0 + lv + 14, y + 17, `${a.id}-${d.id}`, { tam: 11, c: "medio" }), txt(X0 + lv + 62, y + 17, d.titulo, { tam: 14, peso: 700, max: 560 }));
  const prox = botao(0, 0, "Próximo ›", { mini: true })[1], ant = botao(0, 0, "‹ Anterior", { mini: true })[1];
  out.push(txt(X1 - prox - ant - 16, y + 16, `${i + 1} de ${a.detalhes.length}`, { tam: 10.5, c: "medio", fim: true }));
  if (a.detalhes[i - 1]) out.push(botao(X1 - prox - ant - 8, y, "‹ Anterior", { mini: true })[0]);
  if (a.detalhes[i + 1]) out.push(botao(X1 - prox, y, "Próximo ›", { mini: true })[0]);
  y += 36;
  // a figura e a barra de aprovação coladas nela
  out.push(ret(X0, y, fw, campo, { fill: "#081d3c", stroke: "tinta" }), figuraEm(d, X0 + 1, y + (campo - fig) / 2 + 1, fw - 2, fig - 2));
  y += campo;
  if (d.pede_aprovacao) {
    const r = estadoDe(c, chaveDetalhe(a, d));
    out.push(ret(X0, y, fw, 42, { fill: "#082b5e", stroke: "tinta" }), fio(X0, y, X0, y + 42, { c: SITUACAO[r.estado][1], sw: 3 }));
    out.push(situacao(X0 + 14, y + 25, r.estado), txt(X0 + 160, y + 25, r.estado === "pendente" ? "O agente pediu a sua revisão desta figura" : "", { tam: 10.5, c: "medio" }));
    out.push(canto(X1 - 10, y + 26, r));
    y += 42;
    if (r.estado === "revisar") { const [n, h] = notaRevisao(X0, y + 6, fw, r); out.push(n); y += h + 6; }
  }
  y += 26;
  // a legenda e as decisões
  const wl = 560, xd = X0 + wl + 36, wd = X1 - xd, ini = y;
  out.push(rotulo(X0, y, "Legenda", wl));
  const [l, lh] = linhas(X0, y + 24, d.legenda || d.titulo, { tam: 12, max: wl, passo: 18, maxLinhas: 8 });
  out.push(l);
  const rel = (p.decisoes ?? []).filter((x) => x.acoes?.includes(a.id));
  if (rel.length) {
    out.push(rotulo(xd, y, `Decisões de ${a.id} · ${rel.length}`, wd));
    rel.slice(0, 5).forEach((r, n) => {
      const yy = y + 24 + n * 21;
      out.push(txt(xd, yy, r.rotulo, { tam: 11.5, max: wd - 120 }));
      if (r.pede_aprovacao) out.push(situacao(X1, yy, estadoDe(c, `d:${r.id}`).estado, { fim: true }));
    });
  }
  y = ini + Math.max(lh + 24, Math.min(rel.length, 5) * 21 + 24) + 24;
  // a fita com os vizinhos
  const viz = a.detalhes.slice(Math.max(0, i - 2), i + 5);
  if (viz.length > 1) {
    const n = viz.length, g = 10, w = (fw - (n - 1) * g) / n;
    out.push(rotulo(X0, y, `Detalhes de ${a.id}`, fw));
    y += 14;
    viz.forEach((v, k) => {
      const x = X0 + k * (w + g), atual = v === d;
      out.push(ret(x, y, w, 76, { fill: "#081d3c", stroke: atual ? "tinta" : "baixo", sw: atual ? 2 : 1 }), figuraEm(v, x + 4, y + 4, w - 8, 68), txt(x, y + 90, `${a.id}-${v.id}`, { tam: 9.5, c: atual ? "tinta" : "medio", peso: atual ? 700 : 400 }));
    });
    y += 96;
  }
  const H = y + 110;
  const mold = moldura(H, { ...copia(p, c), folha: `Detalhe ${i + 1} de ${a.detalhes.length} · ${a.nome}` });
  return quadro(`d${c.id}`, W, H, mold + out.join(""), `Folha de detalhe ${a.id}-${d.id}`);
}
