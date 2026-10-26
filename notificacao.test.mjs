// Regras das notificações do agente (web/js/notificacao-regras.js, script clássico: carregado num contexto isolado, como o navegador faz).
//   node notificacao.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const codigo = readFileSync(join(import.meta.dirname, "web", "js", "notificacao-regras.js"), "utf8");
const ctx = vm.createContext({ navigator: { languages: ["pt-BR"] }, location: { search: "" }, });
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), ctx, { filename: "i18n.js" });
vm.runInContext(codigo + "\nthis.api = { notificacaoDe, notificacaoEmpilha, NOTIF_FILA_MAX, NOTIF_IDADE_MAX };", ctx);
const { notificacaoDe, notificacaoEmpilha, NOTIF_FILA_MAX, NOTIF_IDADE_MAX } = ctx.api;

const agora = Date.parse("2026-10-23T12:00:00Z");
const item = (tipo, extra = {}) => ({ id: 1, em: "2026-10-23T11:59:30Z", tipo, texto: "oi", workspaceId: "sm", blueprintId: "p1", ...extra });

// o que avisa, com título, prioridade e nível
const casos = [["bloqueio", "O agente travou", 3, "erro"], ["nao-concluida", "Tarefa não concluída", 3, "erro"], ["checkpoint-nao-feito", "Checkpoint não atingido", 3, "erro"], ["proposta", "Nova proposta do agente", 2, ""], ["concluir", "Tarefa concluída", 2, "ok"], ["checkpoint-feito", "Checkpoint atingido", 2, "ok"], ["prototipo", "Protótipo anexado", 1, ""]];
for (const [tipo, titulo, prioridade, nivel] of casos) {
  const n = notificacaoDe(item(tipo), agora, "Portal › Base");
  assert.deepEqual([n.titulo, n.prioridade, n.nivel, n.onde], [titulo, prioridade, nivel, "Portal › Base"], tipo);
}
// o que NÃO avisa: progresso (fica no robô), tipo desconhecido e lixo
assert.equal(notificacaoDe(item("progresso"), agora), null);
assert.equal(notificacaoDe(item("inventado"), agora), null);
assert.equal(notificacaoDe(null, agora), null);
assert.equal(notificacaoDe({ tipo: "bloqueio" }, agora), null, "sem hora não dá para saber se é recente");
// fila de antes da página abrir: aplicada, sem avisar
assert.equal(notificacaoDe(item("bloqueio", { em: new Date(agora - NOTIF_IDADE_MAX - 1000).toISOString() }), agora), null);
// a mensagem do agente: a proposta traz a fala dele; o plano concluído, o plano e a evidência
assert.equal(notificacaoDe(item("proposta", { proposta: { mensagem: "Montei 3 ações." } }), agora).mensagem, "Montei 3 ações.");
assert.match(notificacaoDe(item("proposta", { proposta: {} }), agora).mensagem, /Revise a proposta/);
assert.equal(notificacaoDe(item("concluir", { plano: "Criar rota", texto: "cargo test ok" }), agora).mensagem, "Criar rota\n\ncargo test ok");
// texto malicioso continua texto: a regra nunca o interpreta (quem insere na tela usa textContent)
const html = '<img src=x onerror="alert(1)"><script>alert(2)</script>';
assert.equal(notificacaoDe(item("bloqueio", { texto: html }), agora).mensagem, html);
// mensagem enorme é cortada
const grande = notificacaoDe(item("bloqueio", { texto: "x".repeat(20000) }), agora).mensagem;
assert.ok(grande.length <= 4000 && grande.endsWith("…"));
// a chave muda quando o servidor reinicia e os números voltam a 1
assert.notEqual(notificacaoDe(item("bloqueio"), agora).chave, notificacaoDe(item("bloqueio", { em: "2026-10-23T11:59:40Z" }), agora).chave);

// fila: no máximo NOTIF_FILA_MAX; o que sai é o de menor prioridade mais antigo, e a contagem diz quantos
const fila = [];
let perdidas = 0;
for (let i = 0; i < NOTIF_FILA_MAX; i++) perdidas += notificacaoEmpilha(fila, { chave: "info" + i, prioridade: 1 });
assert.equal([fila.length, perdidas].join(), `${NOTIF_FILA_MAX},0`);
perdidas += notificacaoEmpilha(fila, { chave: "trava", prioridade: 3 });
assert.equal(perdidas, 1);
assert.deepEqual(fila.map((x) => x.chave), ["info1", "info2", "info3", "info4", "trava"], "saiu o informativo mais antigo");
for (let i = 0; i < 10; i++) notificacaoEmpilha(fila, { chave: "t" + i, prioridade: 3 });
assert.equal(fila.length, NOTIF_FILA_MAX);
assert.ok(fila.every((x) => x.prioridade === 3), "com a fila cheia de travas, nada mais importante é perdido antes");
console.log("ok: notificacao.test.mjs");
