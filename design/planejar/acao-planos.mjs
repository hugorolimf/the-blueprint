// Os planos de uma ação aberta. A caixinha chama o agente, e o próprio agente a marca ao concluir. Se a ação mudou, a lista vira um git diff:
// o que saiu em vermelho, o que entrou em verde, com a palavra exata destacada. Reprovar volta tudo ao estado original.
import { txt, ret, fio, botao, rotulo, caixinha, losango, largura } from "./svg.mjs";
import { SITUACOES } from "./agentes.mjs";
import { figuraAgente } from "./tela-agentes.mjs";
import { curta } from "./folha.mjs";
import { controleRevisao } from "./revisao.mjs";
import { pisca } from "./animacao.mjs";
import { TEMA } from "./estilo.mjs";

const VERDE = "#8be3b5", VERM = "#ff8f80", FUNDO = { mais: "#8be3b5", menos: "#ff8f80" };
const TAM = 11, ALT = 21;

/** A caixinha nos estados do plano: vazia chama o agente; trabalhando pisca; pronta, o agente marcou. */
export function caixaPlano(x, y, estado) {
  if (estado === "pronto") return caixinha(x, y, true);
  if (estado === "revisao") return ret(x, y, 13, 13, { stroke: "medio", dash: "2 2" });
  const base = caixinha(x, y);
  if (estado === "trabalhando") return base + pisca(ret(x + 3.5, y + 3.5, 6, 6, { fill: "#bfe0ff" }), 1.4);
  if (["fila", "dependencia", "area"].includes(estado)) return base + `<circle cx="${x + 6.5}" cy="${y + 6.5}" r="2" fill="${TEMA.ambar}"/>`;
  if (["bloqueado", "parado"].includes(estado)) return base + `<path d="M${x + 3.5} ${y + 3.5}l6 6m0 -6l-6 6" stroke="${VERM}" stroke-width="1.3" fill="none"/>`;
  return base;
}

/** A lista normal: caixinha, texto e, para quem já começou, a figura e a situação. */
export function listaPlanos(a, c, x, y, w) {
  const out = [], lista = a.planos ?? [];
  let yy = y;
  const feitos = lista.filter((q) => c.exec?.[q.id]?.estado === "pronto").length, livres = lista.filter((q) => (c.exec?.[q.id]?.estado ?? "nao") === "nao").length;
  out.push(rotulo(x, yy, `Planos · ${feitos}/${lista.length}`, w));
  yy += 22;
  for (const q of lista) {
    const e = c.exec?.[q.id] ?? { estado: "nao" }, [rot, cor] = SITUACOES[e.estado];
    out.push(caixaPlano(x, yy - 10, e.estado), txt(x + 21, yy + 1, q.texto, { tam: 11, max: w - 22 }));
    yy += 18;
    if (e.estado !== "nao") {
      out.push(figuraAgente(x + 17, yy - 8, e.estado, 20), txt(x + 40, yy + 3, e.estado === "pronto" ? "O agente concluiu e marcou" : `${rot}${e.por ? ` · ${e.por}` : ""}`, { tam: 9.5, c: cor, max: w - 42 }));
      yy += 16;
    }
    yy += 8;
  }
  if (livres > 1) { out.push(botao(x, yy - 8, `Iniciar os ${livres}`, { mini: true })[0]); yy += 26; }
  return [out.join(""), yy - y];
}

/** Uma linha de diff: o fundo, o sinal, o id e o texto com as palavras alteradas em destaque. */
function linhaDiff(x, y, w, sinal, id, pedacos, t) {
  const cor = sinal === "+" ? VERDE : sinal === "−" ? VERM : "medio";
  const out = [];
  if (sinal !== " ") out.push(ret(x, y, w, ALT, { fill: sinal === "+" ? FUNDO.mais : FUNDO.menos, op: 0.13 }), ret(x, y, 3, ALT, { fill: cor }));
  out.push(txt(x + 12, y + 15, sinal, { tam: TAM, peso: 700, c: cor }), txt(x + 30, y + 15, id, { tam: 10, c: "medio" }));
  let xx = x + 78;
  for (const p of pedacos) {
    // o SVG engole o espaço nas pontas do texto: a palavra é posta depois dele, e o destaque cobre só a palavra
    const antes = p.s.length - p.s.trimStart().length, palavra = p.s.trim(), xp = xx + largura(p.s.slice(0, antes), TAM);
    if (palavra) {
      if (p.forte) out.push(ret(xp - 1, y + 3, largura(palavra, TAM) + 2, ALT - 6, { fill: cor, op: 0.32 }));
      out.push(txt(xp, y + 15, palavra, { tam: TAM, c: sinal === " " ? t ?? "tinta" : "tinta" }));
    }
    xx += largura(p.s, TAM);
  }
  return out.join("");
}

/** O git diff dos planos: devolve [svg, altura]. */
export function diffPlanos(a, comp, c, x, y, w) {
  const out = [], r = comp.resumo;
  const partes = [r.mais && `+${r.mais}`, r.menos && `−${r.menos}`, r.alterados && `${r.alterados} ${r.alterados > 1 ? "alterados" : "alterado"}`, r.detalhes && `+${r.detalhes} ${r.detalhes > 1 ? "detalhes" : "detalhe"}`].filter(Boolean).join(" · ");
  let yy = y;
  out.push(txt(x, yy + 1, "Plano alterado", { tam: 12, peso: 700 }), txt(x + 116, yy + 1, `${c.agente ?? "agente"} · ${partes}`, { tam: 10, c: "medio", max: w - 400 }));
  out.push(controleRevisao(x + w, yy - 15, { meio: "Editar" })[0]);
  yy += 14;
  const yCaixa = yy;
  yy += 1;
  const corpo = [];
  for (const l of comp.linhas) {
    if (l.tipo === "prazo") {
      corpo.push(linhaDiff(x, yy, w, "−", "prazo", [{ s: `${curta(l.antes.de)} → ` }, { s: curta(l.antes.ate), forte: true }]));
      yy += ALT;
      corpo.push(linhaDiff(x, yy, w, "+", "prazo", [{ s: `${curta(l.depois.de)} → ` }, { s: curta(l.depois.ate), forte: true }]));
      yy += ALT;
    } else if (l.tipo === "igual") {
      const e = c.exec?.[l.plano.id]?.estado ?? "nao";
      corpo.push(linhaDiff(x, yy, w, " ", l.plano.id, [{ s: l.plano.texto }]), caixaPlano(x + w - 22, yy + 4, e));
      yy += ALT;
    } else if (l.tipo === "novo") {
      corpo.push(linhaDiff(x, yy, w, "+", l.plano.id, [{ s: l.plano.texto }]));
      yy += ALT;
    } else if (l.tipo === "removido") {
      corpo.push(linhaDiff(x, yy, w, "−", l.plano.id, [{ s: l.plano.texto }]));
      yy += ALT;
    } else {
      corpo.push(linhaDiff(x, yy, w, "−", l.plano.id, l.palavras.filter((p) => p.t !== "mais").map((p) => ({ s: p.s, forte: p.t === "menos" }))));
      yy += ALT;
      corpo.push(linhaDiff(x, yy, w, "+", l.plano.id, l.palavras.filter((p) => p.t !== "menos").map((p) => ({ s: p.s, forte: p.t === "mais" }))));
      yy += ALT;
    }
  }
  for (const d of comp.detalhes) { corpo.push(linhaDiff(x, yy, w, "+", `${a.id}-${d.id}`, [{ s: `Detalhe: ${d.titulo}` }])); yy += ALT; }
  out.push(ret(x, yCaixa, w, yy - yCaixa + 1, { fill: "#081d3c", stroke: "baixo" }), corpo.join(""));
  yy += 17;
  out.push(txt(x, yy, "Aprovar vale a versão nova; reprovar volta ao estado original. O que mudou só pode ser iniciado depois disso.", { tam: 10, c: "medio", max: w }));
  return [out.join(""), yy - y + 12];
}

export function listaCheckpoints(a, x, y, w) {
  if (!a.checkpoints?.length) return ["", 0];
  const out = [rotulo(x, y, `Checkpoints · 0/${a.checkpoints.length}`, w)];
  let yy = y + 22;
  for (const k of a.checkpoints) { out.push(losango(x + 6, yy - 4, 6), txt(x + 21, yy + 1, k.texto, { tam: 11.5, max: w - 70 }), txt(x + w, yy + 1, curta(k.data), { tam: 10, c: "medio", fim: true })); yy += 23; }
  return [out.join(""), yy - y];
}
