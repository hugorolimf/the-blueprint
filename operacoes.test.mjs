// As operações por item da página (web/js/operacoes.js, script clássico carregado num contexto isolado como no navegador).
//   node operacoes.test.mjs
// O risco maior é detectar o item movido: cada caso confere que aplicar as operações na base dá exatamente a tela (a mesma regra do servidor).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ctx = vm.createContext({ structuredClone });
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "operacoes.js"), "utf8"), ctx, { filename: "operacoes.js" });
const js = (x) => JSON.parse(JSON.stringify(x)); // objetos do contexto isolado têm outros prototypes
const ops = (b, a) => js(ctx.opsEntre(js(b), js(a)));
const aplicar = (arvore, lista) => { const t = js(arvore); for (const op of lista) ctx.aplicarOp(t, op); return js(t); };
/** as operações levam a base exatamente à tela */
const confere = (base, atual, msg) => { const l = ops(base, atual); assert.deepEqual(aplicar(base, l), js(atual), msg); return l; };

const plano = (id, extra = {}) => ({ id, texto: "T" + id, feito: false, ...extra });
const acao = (id, planos = [], extra = {}) => ({ id, nome: "A" + id, planos, checkpoints: [], prototipos: [], ...extra });
const raiz = (acoes, extra = {}) => ({ empresa: "E", blueprints: [{ id: "b1", nome: "B", acoes }], workspaces: [], ...extra });

// nada mudou: nada a mandar
{
  const b = raiz([acao("a1", [plano("p1")])]);
  assert.deepEqual(ops(b, b), []);
}
// alterar um campo traz o antes; remover um campo vira null
{
  const b = raiz([acao("a1", [plano("p1", { descricao: "d" })])]);
  const a = raiz([acao("a1", [{ id: "p1", texto: "novo", feito: false }])]);
  const l = confere(b, a);
  assert.deepEqual(l, [{ op: "alterar", tipo: "plano", id: "p1", campos: { texto: "novo", descricao: null }, antes: { texto: "Tp1", descricao: "d" } }]);
}
// a ordem das chaves não é mudança
{
  const b = raiz([acao("a1", [{ id: "p1", texto: "x", feito: false }])]);
  const a = raiz([acao("a1", [{ feito: false, texto: "x", id: "p1" }])]);
  assert.deepEqual(ops(b, a), []);
}
// campo do workspace raiz (perfil, chats) e do workspace filho
{
  const b = raiz([], { usuario: { nome: "A" }, workspaces: [{ id: "w2", nome: "W", blueprints: [] }] });
  const a = raiz([], { usuario: { nome: "Ana" }, workspaces: [{ id: "w2", nome: "W2", blueprints: [] }] });
  const l = confere(b, a);
  assert.deepEqual(l.map((o) => [o.op, o.tipo, o.id, Object.keys(o.campos)]), [["alterar", "workspace", "sm", ["usuario"]], ["alterar", "workspace", "w2", ["nome"]]]);
}
// criar: só o mais alto da subárvore, com os filhos novos dentro e a posição certa
{
  const b = raiz([acao("a1", [plano("p1"), plano("p2")])]);
  const a = raiz([acao("a1", [plano("p1"), plano("pn"), plano("p2")]), acao("an", [plano("q1"), plano("q2")])]);
  const l = confere(b, a);
  assert.deepEqual(l.map((o) => [o.op, o.tipo, o.id, o.pai, o.posicao]), [["criar", "plano", "pn", "a1", 1], ["criar", "acao", "an", "b1", 1]]);
  assert.deepEqual(l[1].valor.planos.map((p) => p.id), ["q1", "q2"]);
}
// apagar: só o mais alto
{
  const b = raiz([acao("a1", [plano("p1")]), acao("a2", [plano("p2"), plano("p3")])]);
  const l = confere(b, raiz([acao("a1", [])]));
  assert.deepEqual(l, [{ op: "apagar", tipo: "plano", id: "p1" }, { op: "apagar", tipo: "acao", id: "a2" }]);
}
// mover dentro da lista: só o item que saiu do lugar (não os vizinhos que "andaram" junto)
{
  const b = raiz([acao("a1", ["p1", "p2", "p3", "p4", "p5"].map((id) => plano(id)))]);
  const l = confere(b, raiz([acao("a1", ["p4", "p1", "p2", "p3", "p5"].map((id) => plano(id)))]));
  assert.deepEqual(l, [{ op: "mover", tipo: "plano", id: "p4", pai: "a1", posicao: 0 }]);
  const l2 = confere(b, raiz([acao("a1", ["p1", "p3", "p4", "p5", "p2"].map((id) => plano(id)))]));
  assert.deepEqual(l2.map((o) => [o.op, o.id, o.posicao]), [["mover", "p2", 4]]);
  confere(b, raiz([acao("a1", ["p5", "p4", "p3", "p2", "p1"].map((id) => plano(id)))]), "inverter a lista inteira");
}
// mover para outra ação, mover junto com apagar e criar na mesma lista
{
  const b = raiz([acao("a1", [plano("p1"), plano("p2"), plano("p3")]), acao("a2", [plano("q1")])]);
  const a = raiz([acao("a1", [plano("p3"), plano("novo")]), acao("a2", [plano("q1"), plano("p1")])]);
  const l = confere(b, a);
  assert.deepEqual(l.map((o) => [o.op, o.id]), [["criar", "novo"], ["mover", "p1"], ["apagar", "p2"]], "p3 ficou no lugar (os outros saíram de perto dele)");
  assert.equal(l.at(-1).op, "apagar", "apagar vem depois de mover (o que saiu de um item apagado não se perde)");
}
// o filho sai do pai apagado: mover antes do apagar
{
  const b = raiz([acao("a1", [plano("p1"), plano("p2")]), acao("a2")]);
  const a = raiz([acao("a2", [plano("p1")])]);
  const l = confere(b, a);
  assert.deepEqual(l.map((o) => [o.op, o.id]), [["mover", "p1"], ["apagar", "a1"]]);
}
// o filho existente entra num pai novo: o pai novo nasce sem ele e ele é movido
{
  const b = raiz([acao("a1", [plano("p1")])]);
  const a = raiz([acao("a1", []), acao("an", [plano("p1"), plano("q")])]);
  const l = confere(b, a);
  assert.deepEqual(l.map((o) => [o.op, o.id]), [["criar", "an"], ["mover", "p1"]]);
  assert.deepEqual(l[0].valor.planos.map((p) => p.id), ["q"]);
}
// mover o blueprint para outro workspace e reordenar os workspaces
{
  const b = raiz([acao("a1")], { workspaces: [{ id: "w2", nome: "2", blueprints: [] }, { id: "w3", nome: "3", blueprints: [] }] });
  const a = { ...js(b), blueprints: [], workspaces: [{ id: "w3", nome: "3", blueprints: [] }, { id: "w2", nome: "2", blueprints: js(b).blueprints }] };
  const l = confere(b, a);
  assert.ok(l.some((o) => o.op === "mover" && o.tipo === "workspace" && !("pai" in o)), JSON.stringify(l));
  assert.ok(l.some((o) => o.op === "mover" && o.tipo === "blueprint" && o.pai === "w2"));
}
// criar e apagar workspace
{
  const b = raiz([], { workspaces: [{ id: "w2", nome: "2", blueprints: [] }] });
  const l = confere(b, raiz([], { workspaces: [{ id: "w9", nome: "9", blueprints: [{ id: "b9", nome: "", acoes: [] }] }] }));
  assert.deepEqual(l.map((o) => [o.op, o.tipo, o.id]), [["criar", "workspace", "w9"], ["apagar", "workspace", "w2"]]);
}
// lista grande embaralhada: aplicar dá a tela
{
  const ids = Array.from({ length: 60 }, (_, i) => "p" + i);
  const b = raiz([acao("a1", ids.map((id) => plano(id)))]);
  let s = 7; const r = () => (s = (s * 9301 + 49297) % 233280) / 233280;
  const mexido = [...ids].sort(() => r() - 0.5).filter((_, i) => i % 7).concat(["x1", "x2"]);
  confere(b, raiz([acao("a1", mexido.map((id) => plano(id, id === "p3" ? { texto: "mudou" } : {})))]), "embaralhar, apagar e criar");
}

// ---- aplicarRemota: a operação de outra pessoa entra na base e na tela, menos nos campos que esta tela mudou e ainda não mandou ----
{
  const base = raiz([acao("a1", [plano("p1", { descricao: "d" })])]);
  const tela = raiz([acao("a1", [plano("p1", { descricao: "minha" })])]);
  const [B, T] = [js(base), js(tela)];
  const r = js(ctx.aplicarRemota(B, T, { op: "alterar", tipo: "plano", id: "p1", campos: { texto: "dela", descricao: "dela" } }));
  assert.deepEqual(r.conflitos, ["descricao"], "o campo que eu estava editando");
  assert.equal(T.blueprints[0].acoes[0].planos[0].texto, "dela", "o que eu não mexi muda na tela");
  assert.equal(T.blueprints[0].acoes[0].planos[0].descricao, "minha", "o meu fica e vai no próximo lote");
  assert.equal(B.blueprints[0].acoes[0].planos[0].descricao, "dela", "a base é o servidor");
  assert.deepEqual(ops(B, T).map((o) => [o.op, Object.keys(o.campos || {})]), [["alterar", ["descricao"]]]);
}
{
  const base = raiz([acao("a1", [plano("p1")])]);
  const B = js(base), T = raiz([acao("a1", [plano("p1", { texto: "editei" })])]);
  const r = js(ctx.aplicarRemota(B, T, { op: "apagar", tipo: "acao", id: "a1" }));
  assert.equal(r.perdeu, true, "apagar vence editar e avisa");
  assert.deepEqual([B.blueprints[0].acoes, T.blueprints[0].acoes], [[], []]);
  const r2 = js(ctx.aplicarRemota(B, T, { op: "criar", tipo: "acao", id: "a5", pai: "b1", posicao: 0, valor: acao("a5") }));
  assert.equal(r2.perdeu, false);
  assert.deepEqual([B.blueprints[0].acoes[0].id, T.blueprints[0].acoes[0].id], ["a5", "a5"]);
}
// aplicarOp ignora o que não existe (item apagado, pai apagado, id repetido)
{
  const t = raiz([acao("a1")]);
  assert.equal(ctx.aplicarOp(t, { op: "alterar", tipo: "plano", id: "nao", campos: { texto: "x" } }), false);
  assert.equal(ctx.aplicarOp(t, { op: "criar", tipo: "plano", id: "p", pai: "nao", valor: {} }), false);
  assert.equal(ctx.aplicarOp(t, { op: "criar", tipo: "acao", id: "a1", pai: "b1", valor: {} }), false);
  assert.equal(ctx.aplicarOp(t, { op: "apagar", tipo: "workspace", id: "sm" }), false, "o raiz nunca sai");
}
console.log("operacoes.test.mjs: ok");
