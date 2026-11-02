// Tela 2: a ação aberta no lugar, num painel preso à folha. Tudo que se decide do plano se decide aqui: a mudança (como git diff), as
// decisões, as perguntas e os detalhes, cada um com o controle na própria linha. Depois vêm as tabelas e a prancha de detalhes.
import { txt, ret, fio, botao, rotulo, canto as cantoL, linhas, figuraEm, quadro } from "./svg.mjs";
import { W, X0, X1, LINHA, curta, escala, moldura, regua, cabecalhoProjeto, linhaAcao, separador } from "./folha.mjs";
import { acoesDe } from "./pacote.mjs";
import { chaveDecisao, chaveDetalhe, estadoDe, situacao, canto, notaRevisao, margemRevisao } from "./revisao.mjs";
import { copia } from "./tela-folha.mjs";
import { listaPlanos, diffPlanos, listaCheckpoints } from "./acao-planos.mjs";

const PAD = 20;

function decisoes(a, p, c, x, y, w) {
  const lista = (p.decisoes ?? []).filter((d) => d.acoes?.includes(a.id));
  if (!lista.length) return ["", 0];
  const out = [rotulo(x, y, `Decisões · ${lista.length}`, w)];
  let yy = y + 26;
  for (const d of lista) {
    const r = d.pede_aprovacao ? estadoDe(c, chaveDecisao(d)) : { estado: "aprovada" }, ini = yy - 12, fora = r.estado === "reprovada";
    out.push(txt(x + 10, yy, d.rotulo, { tam: 11.5, peso: 700, c: fora ? "medio" : "tinta", max: w - 270 }));
    if (d.pede_aprovacao) out.push(canto(x + w, yy, r, { ativo: r.compondo ? "revisar" : undefined }));
    const [t, h] = linhas(x + 10, yy + 18, d.texto, { tam: 11, max: w - 10, c: r.estado === "aprovada" || fora ? "medio" : "tinta", passo: 15, maxLinhas: 2 });
    out.push(t);
    if (fora) out.push(fio(x + 10, yy + 14, x + w, yy + 14, { c: "#ff8f80", op: 0.7 }));
    yy += h + 12;
    if (r.estado === "revisar" || r.compondo) { const [n, nh] = notaRevisao(x + 10, yy - 4, w - 10, r); out.push(n); yy += nh + 6; }
    if (d.pede_aprovacao) out.push(margemRevisao(x, ini, yy - ini - 8, r.compondo ? "revisar" : r.estado));
    yy += 14;
  }
  return [out.join(""), yy - y];
}

/** As perguntas do agente sobre esta ação: as opções lado a lado, a recomendada indicada. */
function perguntas(a, p, x, y, w) {
  const lista = (p.perguntas ?? []).filter((q) => q.acoes?.includes(a.id));
  if (!lista.length) return ["", 0];
  const out = [rotulo(x, y, `Perguntas · ${lista.length}`, w)];
  let yy = y + 24;
  for (const q of lista) {
    out.push(txt(x, yy, q.id, { tam: 10.5, peso: 700, c: "medio" }));
    const [t, h] = linhas(x + 30, yy, q.texto, { tam: 11.5, max: w - 30, passo: 16, maxLinhas: 2 });
    out.push(t);
    yy += h + 2;
    let bx = x + 30;
    for (const [i, o] of (q.opcoes ?? []).entries()) { const [b, l] = botao(bx, yy - 4, o, { mini: true }); out.push(b); if (i === q.recomendada) out.push(txt(bx + l / 2, yy + 31, "recomendado", { tam: 9, c: "medio", meio: true })); bx += l + 6; }
    if (q.opcoes?.length) out.push(botao(bx, yy - 4, "Outra…", { mini: true })[0]);
    yy += 50;
  }
  return [out.join(""), yy - y];
}

function tabelas(a, x, y, w) {
  const out = [];
  let yy = y;
  for (const t of a.tabelas ?? []) {
    out.push(rotulo(x, yy, t.titulo ?? "Tabela", w));
    yy += 20;
    const larg = t.larguras ?? t.colunas.map(() => 1 / t.colunas.length);
    const cols = larg.reduce((l, f, i) => [...l, (l[i] ?? 0) + f * w], [0]);
    t.colunas.forEach((col, i) => out.push(txt(x + cols[i], yy, col, { tam: 9.5, caixa: true, c: "medio", trilha: 0.1 })));
    yy += 7;
    for (const l of t.linhas) {
      out.push(fio(x, yy, x + w, yy, { c: "baixo", op: 0.5 }));
      l.forEach((v, i) => out.push(txt(x + cols[i], yy + 16, v, { tam: 11, max: larg[i] * w - 10 })));
      yy += 24;
    }
    yy += 24;
  }
  return [out.join(""), yy - y];
}

const proporcao = (d) => {
  const v = d.svg?.match(/viewBox="([^"]+)"/)?.[1]?.split(/\s+/).map(Number);
  return v ? v[2] / v[3] : 16 / 9;
};

/** Um ladrilho: a figura sobre o fundo escuro e, embaixo, o número, a situação e o título. A margem colorida diz o estado à distância. */
export function ladrilho(a, d, x, y, w, h, c) {
  const k = d.pede_aprovacao ? estadoDe(c, chaveDetalhe(a, d)).estado : null;
  return ret(x, y, w, h + 40, { fill: "#081d3c", stroke: "baixo" }) + figuraEm(d, x + 6, y + 6, w - 12, h - 10) +
    ret(x, y + h, w, 40, { fill: "#082b5e" }) + fio(x, y + h, x + w, y + h, { c: "baixo", op: 0.7 }) + (k ? margemRevisao(x, y, h + 40, k) : "") +
    txt(x + 9, y + h + 15, `${a.id}-${d.id}`, { tam: 9.5, c: "medio" }) + (k ? situacao(x + w - 8, y + h + 15, k, { fim: true, curta: true }) : "") +
    txt(x + 9, y + h + 31, d.titulo, { tam: 10.5, max: w - 18 });
}

/** A prancha: quatro colunas; figura muito larga (uma barra, um recorte) ocupa duas e fica baixa. */
function prancha(a, itens, c, x, y, w) {
  const u = (w - 36) / 4, out = [];
  let col = 0, yy = y, alt = 0, antes = null;
  const fecha = () => { yy += alt + 52; col = 0; alt = 0; };
  // as figuras normais primeiro e as largas depois, para cada linha da prancha ter uma altura só
  for (const d of [...itens.filter((i) => proporcao(i) <= 5), ...itens.filter((i) => proporcao(i) > 5)]) {
    const larga = proporcao(d) > 5, span = larga ? 2 : 1, h = larga ? 62 : 118;
    if ((antes !== null && antes !== larga && col) || col + span > 4) fecha();
    antes = larga;
    out.push(ladrilho(a, d, x + col * (u + 12), yy, span * u + (span - 1) * 12, h, c));
    col += span;
    alt = Math.max(alt, h);
  }
  if (col) fecha();
  return [out.join(""), yy - y];
}

/** O painel da ação aberta; devolve [svg, altura]. */
function painelAcao(a, p, c, y) {
  const x = X0 + PAD, w = X1 - X0 - 2 * PAD, xd = x + 330, wd = w - 330, out = [], comp = c.comp?.alteradas?.[a.id];
  let yy = y + 32;
  out.push(txt(x, yy, a.id, { tam: 14, c: "medio" }), txt(x + 42, yy, a.nome, { tam: 16, peso: 700 }));
  out.push(botao(X1 - PAD - 86, yy - 17, "− Recolher", { mini: true })[0]);
  if (a.de) out.push(txt(X1 - PAD - 98, yy, `${curta(a.de)} → ${curta(a.ate)}`, { tam: 10.5, c: "medio", fim: true }));
  yy += 8;
  if (a.texto) { const [s, h] = linhas(x, yy + 14, a.texto, { tam: 11.5, max: w - 40, c: "medio", passo: 16, maxLinhas: 2 }); out.push(s); yy += h + 2; }
  yy += 28;
  let esq = 0;
  if (comp) {
    const [s, h] = diffPlanos(a, comp, c, x, yy, w);
    out.push(s);
    yy += h + 20;
  }
  const [ps, ph] = comp ? ["", 0] : listaPlanos(a, c, x, yy, 296);
  const yk = yy + ph + (ph ? 6 : 0), [ks, kh] = listaCheckpoints(a, x, yk, 296);
  esq = ph + (ph ? 6 : 0) + kh;
  const [ds, dh] = decisoes(a, p, c, xd, yy, wd), [qs, qh] = perguntas(a, p, xd, yy + dh, wd);
  out.push(ps, ks, ds, qs);
  yy += Math.max(esq, dh + qh) + 8;
  const [ts, th] = tabelas(a, x, yy, w);
  out.push(ts);
  yy += th;
  if (a.detalhes?.length) {
    const espera = a.detalhes.filter((d) => d.pede_aprovacao && estadoDe(c, chaveDetalhe(a, d)).estado === "pendente").length;
    out.push(rotulo(x, yy, `Detalhes · ${a.detalhes.length}${espera ? ` · ${espera} aguardando você` : ""}`, w));
    yy += 20;
    const grupos = [];
    for (const d of a.detalhes) { const g = grupos.find((n) => n.nome === (d.grupo ?? "")); if (g) g.itens.push(d); else grupos.push({ nome: d.grupo ?? "", itens: [d] }); }
    for (const g of grupos) {
      if (g.nome) { out.push(txt(x, yy + 3, g.nome, { tam: 11, peso: 700 }), txt(x + w, yy + 3, String(g.itens.length), { tam: 10, c: "medio", fim: true })); yy += 14; }
      const [s, h] = prancha(a, g.itens, c, x, yy, w);
      out.push(s);
      yy += h + 2;
    }
  }
  const alt = yy - y + 2;
  return [ret(X0, y, X1 - X0, alt, { fill: "#082b5e", op: 0.86, stroke: "tinta" }) + cantoL(X0, y) + out.join(""), alt];
}

export function telaAcao(p, id, c) {
  const a = acoesDe(p).find((x) => x.id === id), e = escala(p, c.hoje), pr = p.projetos.find((x) => x.acoes.includes(a)), out = [];
  const i = pr.acoes.indexOf(a);
  let y = 182;
  out.push(cabecalhoProjeto(p.projetos.indexOf(pr) + 1, pr, p.blueprint, p, c, e, y));
  y += 32;
  for (const o of pr.acoes.slice(0, i)) { out.push(linhaAcao(e, o, p, c, y)); y += LINHA; out.push(separador(y - LINHA / 2)); }
  const yHoje = y - 18;
  const [ps, ph] = painelAcao(a, p, c, y - 8);
  out.push(ps);
  y += ph + 20;
  for (const o of pr.acoes.slice(i + 1)) { out.push(linhaAcao(e, o, p, c, y)); y += LINHA; out.push(separador(y - LINHA / 2)); }
  const H = y + 100;
  return quadro(`a${c.id}`, W, H, moldura(H, copia(p, c)) + regua(e, 138, yHoje) + out.join(""), `A ação ${a.id} aberta na folha`);
}
