// A lógica pura do modo foco (web/js/foco-logica.js, script clássico carregado num contexto isolado depois de claude-logica.js, como o navegador faz):
// os cartões (quem aparece e a ordem: vários agentes, nenhum, repouso, plano concluído), o agente em foco, seguir, rolagem, o Esc, desde quando no plano e os eventos de cada agente.
// O DOM fica em foco.js, foco-vista.js e foco-cartao.js.
//   node foco-logica.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ctx = vm.createContext({ navigator: { languages: ["pt-BR"] }, location: { search: "" } });
const le = (f) => readFileSync(join(import.meta.dirname, "web", "js", f), "utf8");
vm.runInContext(le("i18n.js"), ctx, { filename: "i18n.js" });
vm.runInContext(le("claude-logica.js"), ctx, { filename: "claude-logica.js" });
vm.runInContext(le("foco-logica.js") + "\nthis.api = { focoCartoes, focoEmFoco, focoLugar, focoProximo, focoDeveRolar, focoEsc, focoFixoSumiu, focoDesde, focoMinutos, focoEventosDoAgente, FOCO_EVENTOS, FOCO_HISTORICO };", ctx);
const A = ctx.api;
const J = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes
const nomes = (c) => c.itens.map((i) => i.agente);

const bps = [{ id: "b1", acoes: [
  { id: "f1", nome: "Ação um", planos: [{ id: "t1", texto: "Plano um", feito: false, agente: { inicioEm: "2026-10-26T10:00:00Z" } }, { id: "t2", texto: "Plano dois", feito: false }, { id: "t3", texto: "Plano três", feito: true }] },
  { id: "f2", nome: "Ação dois", planos: [{ id: "t4", texto: "Plano quatro", feito: false }] },
] }];
const ag = (agente, extra = {}) => ({ agente, blueprintId: "b1", acaoId: "f1", planoId: "t1", estado: "escrevendo", em: "2026-10-26T10:10:00Z", mensagem: "fazendo", onde: ["a.js"], ...extra });

// ---- nenhum agente: estado vazio (a tela mostra a dica)
const nada = J(A.focoCartoes([], bps));
assert.equal(nada.vazio, true);
assert.deepEqual(nada.itens, []);
assert.equal(nada.principal, null);
assert.equal(J(A.focoCartoes(undefined, undefined)).vazio, true, "entradas ausentes não quebram");

// ---- um agente: o cartão traz o lugar resolvido nos dados e o que ele disse
const um = A.focoCartoes([ag("claude", { percentual: 40 })], bps);
const c1 = J(um.itens[0]);
assert.equal(um.vazio, false);
assert.deepEqual([c1.acaoNome, c1.planoTexto, c1.planoId, c1.acaoId, c1.estado, c1.mensagem, c1.onde, c1.percentual], ["Ação um", "Plano um", "t1", "f1", "escrevendo", "fazendo", ["a.js"], 40]);
assert.equal(c1.inicioEm, Date.parse("2026-10-26T10:00:00Z"), "o início medido do plano");
assert.equal(um.principal.agente, "claude");

// ---- vários agentes: o principal (sem papel) primeiro; depois por urgência (bloqueado, trabalhando, aguardando, concluiu, repouso); empate: o que avisou há menos tempo, depois o nome
const varios = A.focoCartoes([
  ag("zeta", { papel: "revisor", estado: "ocioso", planoId: "", acaoId: "" }),
  ag("rev", { papel: "revisor", estado: "aguardando" }),
  ag("exp", { papel: "explorador", estado: "lendo", em: "2026-10-26T10:05:00Z" }),
  ag("claude"),
  ag("trava", { papel: "construtor", estado: "bloqueado" }),
  ag("fim", { papel: "construtor", estado: "concluido", planoId: "t3" }),
  ag("cons", { papel: "construtor", estado: "escrevendo", planoId: "t2", em: "2026-10-26T10:20:00Z" }),
], bps);
assert.deepEqual(nomes(varios), ["claude", "trava", "cons", "exp", "rev", "fim", "zeta"]);
assert.equal(varios.principal.agente, "claude");

// ---- agente em repouso: cartão sem lugar (não aponta para plano nenhum)
const rep = J(A.focoCartoes([ag("c", { repouso: true })], bps).itens[0]);
assert.deepEqual([rep.estado, rep.repouso, rep.acaoId, rep.planoId], ["ocioso", true, "", ""]);
assert.equal(J(A.focoCartoes([ag("c", { estado: "ocioso" })], bps).itens[0]).acaoId, "", "ocioso também");

// ---- o que sumiu dos dados não derruba o cartão
const sumiu = J(A.focoCartoes([ag("c", { blueprintId: "zz" })], bps).itens[0]);
assert.deepEqual([sumiu.acaoId, sumiu.planoId, sumiu.blueprintId], ["", "", "zz"]);
const semPlano = J(A.focoCartoes([ag("c", { planoId: "zz" })], bps).itens[0]);
assert.deepEqual([semPlano.acaoId, semPlano.planoId], ["f1", ""], "plano que sumiu: fica a ação");

// ---- plano concluído: o agente que concluiu continua (até o usuário fechar a mensagem), com o plano feito
const fim = J(A.focoCartoes([ag("c", { estado: "concluido", planoId: "t3" })], bps).itens[0]);
assert.deepEqual([fim.estado, fim.feito, fim.planoId], ["concluido", true, "t3"]);

// ---- o agente em foco: o fixado (se existe) ou o principal
assert.equal(A.focoEmFoco(varios, "exp").agente, "exp");
assert.equal(A.focoEmFoco(varios, "").agente, "claude");
assert.equal(A.focoEmFoco(varios, "sumiu").agente, "claude", "fixado que não existe mais: volta ao principal");
assert.equal(A.focoEmFoco(A.focoCartoes([], bps), "x"), null);

// ---- seguir: só anda quando o lugar muda
assert.equal(A.focoLugar(null), "");
assert.equal(A.focoLugar(um.itens[0]), "b1|f1|t1");
assert.equal(A.focoLugar(rep), "", "em repouso não há lugar");
assert.deepEqual(J(A.focoProximo("", um.itens[0])), { chave: "b1|f1|t1", mudou: true });
assert.deepEqual(J(A.focoProximo("b1|f1|t1", um.itens[0])), { chave: "b1|f1|t1", mudou: false }, "mesmo plano: não anda");
const outro = A.focoCartoes([ag("claude", { planoId: "t2" })], bps).itens[0];
assert.deepEqual(J(A.focoProximo("b1|f1|t1", outro)), { chave: "b1|f1|t2", mudou: true }, "outro plano: anda");
assert.deepEqual(J(A.focoProximo("b1|f1|t1", null)), { chave: "", mudou: false }, "sem agente: fica onde está");

// ---- rolar só se o alvo saiu da faixa confortável da janela
assert.equal(A.focoDeveRolar({ top: 400, bottom: 422 }, 1000), false);
assert.equal(A.focoDeveRolar({ top: 50, bottom: 72 }, 1000), true, "colado no topo");
assert.equal(A.focoDeveRolar({ top: 950, bottom: 972 }, 1000), true, "colado no fim");
assert.equal(A.focoDeveRolar(null, 1000), true, "alvo não desenhado: tenta rolar");

// ---- Esc, um degrau por vez; o fixado que some volta à lista
assert.equal(A.focoEsc("claude"), "lista");
assert.equal(A.focoEsc(""), "fechar");
assert.equal(A.focoFixoSumiu(varios, "exp"), false);
assert.equal(A.focoFixoSumiu(varios, "x"), true);
assert.equal(A.focoFixoSumiu(varios, ""), false);

// ---- desde quando no plano: o início medido, ou o começo da sequência de eventos do agente neste plano (do mais novo ao mais antigo)
const evs = [
  { agente: "claude", alvo: "t1", em: "2026-10-26T10:30:00Z", tipo: "progresso" },
  { agente: "claude", alvo: "t1", em: "2026-10-26T10:20:00Z", tipo: "progresso" },
  { agente: "claude", alvo: "t1", em: "2026-10-26T10:10:00Z", tipo: "progresso" },
  { agente: "claude", alvo: "t0", em: "2026-10-26T10:00:00Z", tipo: "progresso" },
];
assert.equal(A.focoDesde(123, evs, "claude", "t1"), 123, "o início medido vence");
assert.equal(A.focoDesde(0, evs, "claude", "t1"), Date.parse("2026-10-26T10:10:00Z"), "começo da sequência neste plano");
assert.equal(A.focoDesde(0, evs, "outro", "t1"), 0, "outro agente: não sabe");
assert.equal(A.focoDesde(0, [], "claude", "t1"), 0);
assert.equal(A.focoDesde(0, [{ alvo: "t2", agente: "claude", em: "2026-10-26T09:00:00Z" }, ...evs], "claude", "t1"), Date.parse("2026-10-26T10:10:00Z"), "evento mais novo de outro plano não atrapalha");
assert.equal(A.focoMinutos(0, 5), 0);
assert.equal(A.focoMinutos(1000, 1000 + 12 * 60000), 12);
assert.equal(A.focoMinutos(1000, 1000 + 5000), 1, "menos de um minuto conta como 1");
assert.equal(A.focoMinutos(5000, 1000), 0, "fim antes do início: 0");

// ---- eventos do agente: os dele neste plano (e os gerais), até n, na ordem recebida
const todos = [{ id: 1, agente: "claude", alvo: "t1" }, { id: 2, agente: "outro", alvo: "t1" }, { id: 3, agente: "claude", alvo: "t2" }, { id: 4 }, { id: 5, agente: "claude", alvo: "t1" }, { id: 6, agente: "claude", alvo: "t1" }, { id: 7, agente: "claude", alvo: "t1" }];
const eu = { agente: "claude", planoId: "t1" };
assert.deepEqual(J(A.focoEventosDoAgente(todos, eu)).map((e) => e.id), [1, 4, 5], "no máximo FOCO_EVENTOS (3), sem os de outro agente nem de outro plano");
assert.deepEqual(J(A.focoEventosDoAgente(todos, eu, A.FOCO_HISTORICO)).map((e) => e.id), [1, 4, 5, 6, 7]);
assert.deepEqual(J(A.focoEventosDoAgente(todos, { agente: "claude", planoId: "" })).map((e) => e.id), [1, 3, 4], "sem plano: todos os dele na ação");
assert.deepEqual(J(A.focoEventosDoAgente(undefined, eu)), []);
assert.equal(A.FOCO_EVENTOS, 3);

console.log("foco-logica: cartões (ordem, repouso, concluído), agente em foco, seguir, rolagem, Esc, desde quando no plano e eventos do agente conferidos");
