// Tela 6: o plano sendo escrito em tempo real. As ações já estão na folha, «Aguardando plano»; o agente escreve uma de cada vez e tudo
// aparece enquanto ele escreve: os planos, o checkpoint, a decisão e as figuras. Na interface nativa, isto chega pelo MCP, peça por peça.
import { txt, ret, rotulo, caixinha, losango, canto, largura, linhas, figuraEm, quadro } from "./svg.mjs";
import { W, X0, X1, LINHA, escala, moldura, regua, cabecalhoProjeto, barraAcao, separador } from "./folha.mjs";
import { figuraOrquestrador } from "./tela-agentes.mjs";
import { aparece, entre, digita } from "./animacao.mjs";
import { copia } from "./tela-folha.mjs";

const XN = X0 + 30;

/** O estado da ação que ainda não tem plano: tracejado, à espera, com os três pontinhos. */
function aguardando(e, y) {
  const x = e.x(e.hoje) + 12;
  return ret(x, y - 8, 96, 16, { stroke: "medio", dash: "3 3" }) + txt(x + 106, y + 4, "Aguardando plano", { tam: 10, c: "medio" });
}

const nomeLinha = (a, y) => txt(XN, y + 4, a.id, { tam: 11, c: "medio" }) + txt(XN + 40, y + 4, a.nome, { tam: 12, max: 230 });

export function telaEscrita(p, c) {
  const pr = p.projetos.find((x) => x.acoes.filter((a) => a.planos?.length).length > 1) ?? p.projetos[0], lista = pr.acoes;
  const i = Math.min(1, lista.length - 1), a = lista[i], e = escala(p, c.hoje), out = [], x = X0 + 20, w = X1 - X0 - 40;
  let y = 182;
  out.push(cabecalhoProjeto(1, { ...pr, acoes: [] , nome: pr.nome ?? p.blueprint }, p.blueprint, p, c, e, y, { colunas: false }));
  y += 32;
  for (const o of lista.slice(0, i)) {
    out.push(nomeLinha(o, y), barraAcao(e, o, y), txt(320, y + 4, "Plano pronto", { tam: 10, c: "#8be3b5" }));
    y += LINHA;
    out.push(separador(y - LINHA / 2));
  }
  // a ação sendo escrita
  const y0 = y - 8, detal = (a.detalhes ?? []).slice(0, 4), pl = (a.planos ?? []).slice(0, 5);
  const corpo = [];
  let yy = y0 + 30;
  corpo.push(txt(x, yy, a.id, { tam: 14, c: "medio" }), txt(x + 42, yy, a.nome, { tam: 16, peso: 700 }));
  corpo.push(figuraOrquestrador(X1 - 20 - 268, yy - 24, 40), entre(txt(X1 - 20, yy - 2, "O orquestrador está escrevendo…", { tam: 10.5, c: "#bfe0ff", fim: true }), 0, 17), aparece(txt(X1 - 20, yy - 2, "Plano pronto", { tam: 10.5, c: "#8be3b5", fim: true }), 17));
  yy += 28;
  if (a.texto) { const [s, h] = linhas(x, yy, a.texto, { tam: 11.5, max: w - 40, c: "medio", passo: 16, maxLinhas: 2 }); const altT = h + 2; corpo.push(digita(s, x, yy - 14, w, altT + 6, 0.5, 2.2)); yy += altT + 12; }
  const yP = yy;
  corpo.push(aparece(rotulo(x, yy, `Planos · 0/${pl.length}`, 296), 2.2));
  yy += 22;
  let t = 2.4;
  for (const q of pl) {
    const larg = Math.min(largura(q.texto, 11), 276) + 2, dur = Math.max(0.5, larg / 180);
    corpo.push(aparece(caixinha(x, yy - 10), t), digita(txt(x + 21, yy + 1, q.texto, { tam: 11, max: 274 }), x + 21, yy - 12, larg, 18, t, t + dur));
    corpo.push(entre(ret(x + 21, yy - 9, 7, 13, { fill: "#bfe0ff" }), t, t + 0.12));
    t += dur + 0.25;
    yy += 25;
  }
  yy += 12;
  if (a.checkpoints?.length) {
    const k = a.checkpoints[0], tk = t;
    corpo.push(aparece(rotulo(x, yy, "Checkpoints · 0/1", 296) + losango(x + 6, yy + 18, 6) + txt(x + 21, yy + 23, k.texto, { tam: 11, max: 230 }), tk));
    t += 0.7;
    yy += 44;
  }
  // a direita: decisões e, embaixo, as figuras chegando
  const xd = x + 330, wd = w - 330, dec = (p.decisoes ?? []).filter((d) => d.acoes?.includes(a.id)).slice(0, 2);
  let yd = yP;
  if (dec.length) {
    corpo.push(aparece(rotulo(xd, yd, "Decisões", wd), t));
    yd += 24;
    for (const d of dec) {
      const [s, h] = linhas(xd, yd + 16, d.texto, { tam: 10.5, max: wd, c: "medio", passo: 15, maxLinhas: 2 });
      corpo.push(aparece(txt(xd, yd, d.rotulo, { tam: 11, peso: 700 }) + s, t + 0.2));
      t += 0.8;
      yd += h + 26;
    }
  }
  yy = Math.max(yy, yd) + 6;
  if (detal.length) {
    corpo.push(aparece(rotulo(x, yy, `Detalhes · ${detal.length}`, w), t));
    yy += 16;
    const cw = (w - 36) / 4;
    detal.forEach((d, n) => {
      const xc = x + n * (cw + 12), t0 = t + 0.3 + n * 1.0;
      corpo.push(aparece(ret(xc, yy, cw, 118, { fill: "#081d3c", stroke: "baixo", dash: "3 3" }), t0));
      corpo.push(entre(txt(xc + cw / 2, yy + 62, "desenhando…", { tam: 10, c: "medio", meio: true }), t0, t0 + 0.9));
      corpo.push(aparece(ret(xc, yy, cw, 118, { fill: "#081d3c", stroke: "baixo" }) + figuraEm(d, xc + 5, yy + 5, cw - 10, 92) + txt(xc + 8, yy + 111, d.titulo, { tam: 9.5, max: cw - 16 }), t0 + 0.9));
    });
    yy += 132;
  }
  const alt = yy - y0;
  out.push(ret(X0, y0, X1 - X0, alt, { fill: "#082b5e", op: 0.86, stroke: "tinta" }) + canto(X0, y0), corpo.join(""));
  y = y0 + alt + 28;
  for (const o of lista.slice(i + 1)) {
    out.push(nomeLinha(o, y), aguardando(e, y));
    y += LINHA;
    out.push(separador(y - LINHA / 2));
  }
  for (const [n, o] of p.projetos.filter((x) => x !== pr).entries()) {
    y += 16;
    out.push(cabecalhoProjeto(n + 2, { ...o, acoes: [] }, p.blueprint, p, c, e, y, { colunas: false }));
    y += 32;
    for (const q of o.acoes.slice(0, 3)) { out.push(nomeLinha(q, y), aguardando(e, y)); y += LINHA; out.push(separador(y - LINHA / 2)); }
    if (o.acoes.length > 3) { out.push(txt(XN + 40, y + 2, `+ ${o.acoes.length - 3} ações aguardando plano`, { tam: 10.5, c: "medio" })); y += 28; }
  }
  const H = y + 100;
  return quadro(`w${c.id}`, W, H, moldura(H, copia(p, c)) + regua(e, 138, y0 - 10, { colunas: false }) + out.join(""), "O plano sendo escrito em tempo real");
}
