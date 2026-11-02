// Tela 5: o orquestrador, mestre do workspace, e os agentes. Cada plano com a caixinha marcada tem o seu: a figura mostra o que ele está fazendo, com a situação escrita, de quem depende, em
// que área mexe e em qual ramo trabalha. Em cima, os números, o limite e os avisos.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { txt, ret, fio, botao, rotulo, canto, quadro } from "./svg.mjs";
import { W, X0, X1, moldura } from "./folha.mjs";
import { SITUACOES, avisos, planosDe, diario } from "./agentes.mjs";
import { pisca } from "./animacao.mjs";
import { copia } from "./tela-folha.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const png = (pose) => `data:image/png;base64,${readFileSync(join(AQUI, "agentes", `${pose}.png`)).toString("base64")}`;
const FIG = Object.fromEntries(["trabalhando", "esperando", "bloqueado", "concluido", "parado"].map((n) => [n, png(n)]));

/** A figura do agente na pose da situação. */
export const figuraAgente = (x, y, estado, w = 44) => `<image x="${x}" y="${y}" width="${w}" height="${(w * 170) / 220}" href="${FIG[SITUACOES[estado][2]]}"/>`;

/** A figura do orquestrador: o agente principal do app, para não se confundir com os de cada plano. */
export const figuraOrquestrador = (x, y, w = 104) => `<image x="${x}" y="${y}" width="${w}" height="${(w * 170) / 220}" href="${png("orquestrador")}"/>`;

/** A faixa do orquestrador: quem ele é, o que está fazendo agora e o campo para falar com ele. Devolve [svg, altura]. */
function faixaOrquestrador(todos, exec, c, y) {
  const h = 138, rodando = todos.filter((q) => exec[q.id]?.estado === "trabalhando").length, linhasDiario = diario(todos, exec).slice(0, 5), xm = X0 + 372;
  const out = [ret(X0, y, X1 - X0, h, { fill: "#082b5e", stroke: "tinta" }), canto(X0, y, 16), figuraOrquestrador(X0 + 12, y + 14, 112)];
  out.push(txt(X0 + 136, y + 30, "Orquestrador", { tam: 16, peso: 700 }), txt(X0 + 136, y + 48, `Mestre do workspace ${c.workspace}`, { tam: 10.5, c: "medio", max: 220 }));
  out.push(pisca(`<circle cx="${X0 + 140}" cy="${y + 67}" r="3.5" fill="#bfe0ff"/>`, 1.4), txt(X0 + 150, y + 71, `Coordenando ${rodando} ${rodando === 1 ? "agente" : "agentes"}`, { tam: 11, c: "#bfe0ff" }));
  out.push(rotulo(xm, y + 22, "Agora", X1 - xm - 16));
  linhasDiario.forEach((t, i) => out.push(txt(xm, y + 42 + i * 17, t, { tam: 10.5, c: i ? "medio" : "tinta", max: X1 - xm - 16 })));
  out.push(ret(X0 + 12, y + 102, 300, 24, { fill: "#081d3c", stroke: "medio" }), txt(X0 + 22, y + 118, "Pedir ao orquestrador…", { tam: 10.5, c: "medio" }));
  return [out.join(""), h];
}

const COL = { plano: X0, situacao: 434, depende: 620, area: 700, ramo: 838 };

export function telaAgentes(p, c) {
  const todos = planosDe(p), exec = c.exec, lista = p.projetos.filter((pr) => pr.acoes.some((a) => (a.planos ?? []).some((q) => exec[q.id]?.estado !== "nao")));
  const conta = (...e) => todos.filter((q) => e.includes(exec[q.id]?.estado)).length;
  const av = avisos(todos, exec), out = [];
  let y = 118;
  const [fo, fh] = faixaOrquestrador(todos, exec, c, y);
  out.push(fo);
  y += fh + 18;
  // os números e o limite
  const nums = [[conta("trabalhando"), "trabalhando", "#bfe0ff"], [conta("fila"), "na fila", "#9cc4ee"], [conta("dependencia", "area", "revisao"), "esperando", "#ffcf6b"], [conta("parado", "bloqueado"), "parados", "#ff8f80"], [conta("pronto"), "prontos", "#8be3b5"]];
  nums.forEach(([n, r, cor], i) => out.push(txt(X0 + i * 112, y + 16, n, { tam: 26, peso: 700, c: cor }), txt(X0 + i * 112, y + 34, r, { tam: 10.5, c: "medio" })));
  out.push(txt(X1, y + 4, "Agentes ao mesmo tempo", { tam: 10, caixa: true, c: "medio", trilha: 0.1, fim: true }));
  const [bm, lm] = botao(X1 - 92, y + 12, "−", { mini: true }), [bp, lp] = botao(X1 - lp0(), y + 12, "+", { mini: true });
  out.push(bm, bp, txt(X1 - 92 + lm + 16, y + 29, c.limite, { tam: 14, peso: 700, meio: true }));
  y += 56;
  // os avisos
  if (av.length) {
    const vis = av.slice(0, 6), h = 34 + Math.ceil(vis.length / 2) * 19;
    out.push(ret(X0, y, X1 - X0, h, { fill: "#082b5e", stroke: "#ffcf6b" }), canto(X0, y, 16), txt(X0 + 14, y + 20, `Avisos · ${av.length}`, { tam: 10, caixa: true, c: "#ffcf6b", trilha: 0.12 }));
    vis.forEach((t, i) => out.push(txt(X0 + 14 + (i % 2) * 492, y + 40 + Math.floor(i / 2) * 19, t, { tam: 10.5, max: 476 })));
    y += h + 22;
  }
  // a tabela: um plano por linha
  for (const [n, x] of [["Plano", COL.plano], ["Situação", COL.situacao], ["Depende de", COL.depende], ["Área", COL.area], ["Ramo do agente", COL.ramo]]) out.push(txt(x, y, n, { tam: 9.5, caixa: true, c: "medio", trilha: 0.1 }));
  y += 8;
  out.push(fio(X0, y, X1, y, { c: "tinta" }));
  for (const pr of lista) for (const a of pr.acoes) {
    if (!a.planos?.length) continue;
    y += 22;
    out.push(txt(COL.plano, y, `${a.id} ${a.nome}`, { tam: 11.5, peso: 700, max: 340 }));
    y += 8;
    out.push(fio(X0, y, X1, y, { c: "baixo", op: 0.5 }));
    for (const q of a.planos) {
      const e = exec[q.id] ?? { estado: "nao" }, [rot, cor] = SITUACOES[e.estado], ativo = e.estado === "trabalhando", yy = y + 22;
      if (ativo) out.push(ret(X0, y + 1, X1 - X0, 33, { fill: "#14509b", op: 0.75 }));
      out.push(txt(COL.plano + 8, yy, q.id, { tam: 10, c: "medio" }), txt(COL.plano + 54, yy, q.texto, { tam: 11, max: COL.situacao - COL.plano - 64 }));
      out.push(figuraAgente(COL.situacao - 4, y + 2, e.estado, 38), txt(COL.situacao + 40, yy, rot, { tam: 10.5, c: cor, max: 142 }));
      const dep = e.por && (e.estado === "dependencia" || e.estado === "parado") ? e.por : (q.depende_de ?? []).join(", ");
      out.push(txt(COL.depende, yy, dep || "—", { tam: 10.5, c: e.estado === "dependencia" || e.estado === "parado" ? cor : "medio", max: 74 }));
      out.push(txt(COL.area, yy, q.area ?? "—", { tam: 10.5, c: e.estado === "area" ? cor : "medio", max: 130 }));
      out.push(txt(COL.ramo, yy, ["trabalhando", "pronto"].includes(e.estado) ? `agente/${q.id}` : "—", { tam: 10.5, c: ativo ? "tinta" : "medio", max: 190 }));
      y += 34;
      out.push(fio(X0, y, X1, y, { c: "baixo", op: 0.35 }));
    }
  }
  const H = y + 120;
  return quadro(`g${c.id}`, W, H, moldura(H, { ...copia(p, c), botoes: ["Folha", "Planejar", "Compartilhar"] }) + out.join(""), "Os agentes: um por plano iniciado, com situação, dependência e área");
  function lp0() { return botao(0, 0, "+", { mini: true })[1]; }
}
