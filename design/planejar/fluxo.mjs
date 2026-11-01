// Como o plano anda: o agente lê a memória, escreve a proposta como página, você responde e aplica, a folha recebe as ações e o
// agente conta o andamento por ela (registrar_progresso e marcar_concluido, que já existem).
import { txt, ret, rotulo, canto, quadro, COR } from "./svg.mjs";

const W = 1120, H = 380;

function caixa(x, y, w, h, rot, linhas, cheia = false) {
  const out = [ret(x, y, w, h, { fill: cheia ? "#1a467d" : "#0c264b", stroke: cheia ? "tinta" : "baixo" }), canto(x, y), rotulo(x + 16, y + 26, rot)];
  linhas.forEach((l, i) => out.push(txt(x + 16, y + 50 + i * 18, l, { tam: 11.5, c: i ? "medio" : "tinta" })));
  return out.join("");
}

const seta = (d, rot, lx, ly, { dash, fim = false } = {}) =>
  `<path d="${d}" fill="none" stroke="${COR.tinta}" stroke-width="1.2"${dash ? ` stroke-dasharray="${dash}"` : ""} marker-end="url(#fx-seta)"/>` + txt(lx, ly, rot, { tam: 10, c: "medio", fim });

export function fluxo() {
  const out = [`<defs><marker id="fx-seta" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10Z" fill="${COR.tinta}"/></marker></defs>`];
  out.push(caixa(30, 130, 240, 96, "Agente", ["Claude Code, Codex…", "trabalha na pasta do código"]));
  out.push(caixa(440, 24, 240, 86, "Memória", ["fatos do blueprint", "o que é, por quê, como aplicar"]));
  out.push(caixa(440, 214, 240, 96, "Planejar · página", ["a proposta em blocos", "perguntas P1, P2…"], true));
  out.push(caixa(850, 24, 240, 86, "Você", ["lê como uma nota", "responde, ajusta, aplica"]));
  out.push(caixa(850, 214, 240, 96, "Folha do blueprint", ["ações, planos, checkpoints", "a régua e o hoje"]));
  out.push(seta("M270 150 C340 120 380 80 438 70", "ler_memoria · lembrar", 300, 104));
  out.push(seta("M270 200 C340 230 380 250 438 258", "escrever_proposta", 330, 212));
  out.push(seta("M440 286 C360 300 320 260 272 218", "as respostas", 296, 304, { dash: "4 3" }));
  out.push(seta("M850 84 C780 120 740 190 682 236", "responde e aplica", 842, 150, { fim: true }));
  out.push(seta("M682 262 L848 262", "Aplicar na folha", 694, 252));
  out.push(seta("M970 312 C900 380 260 380 120 230", "registrar_progresso · marcar_concluido (o andamento)", 560, 372, { dash: "4 3" }));
  return quadro("fx", W, H, out.join(""), "O caminho do plano: o agente lê a memória e escreve a proposta como página; você responde e aplica; a folha recebe as ações e o agente conta o andamento");
}
