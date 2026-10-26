// Entrada pelo convite e sessão (web/js/convite.js e web/js/sessao.js, scripts clássicos carregados num contexto isolado, como o navegador faz):
// o segredo sai da barra antes de qualquer pedido e só viaja no corpo; destino só da própria página; papel por escopo; botões por papel; erros por código.
//   node convite.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const SEGREDO = "Ab3_dE-fGhIjKlMnOpQrStUvWxYz0123456789abcd"; // 43 caracteres base64url, como o servidor gera
function contexto({ hash = "", respostas = {}, idioma = "pt-BR" } = {}) {
  const els = {}, el = (id) => (els[id] ||= { id, textContent: "", value: "", hidden: true, firstChild: null, innerHTML: "", disabled: false, focus() {}, querySelector: () => el(id + ":botao"), set onclick(f) {}, set onsubmit(f) { this.submit = f; } });
  const eventos = [], ouvintes = {};
  const ctx = vm.createContext({
    addEventListener: (tipo, f) => { ouvintes[tipo] = f; },
    navigator: { languages: [idioma] }, URLSearchParams, Intl, localStorage: { getItem: () => null, setItem() {}, removeItem() {} },
    location: { search: "", pathname: "/convite", hash, replace: (u) => eventos.push(["replace", u]) },
    history: { replaceState: (_a, _b, u) => { eventos.push(["replaceState", u]); ctx.location.hash = ""; } },
    document: { body: { classList: { add() {}, remove() {}, toggle() {} } } },
    fetch: async (url, op = {}) => { eventos.push(["fetch", url, op.body || ""]); const [status, corpo] = respostas[url] || [404, { codigo: "so-local" }]; return { ok: status < 300, status, json: async () => corpo }; },
    emblemasProntos: Promise.resolve(), EMB: null, previaMarca: () => "",
  });
  ctx.$ = el;
  const carrega = (pasta, a) => vm.runInContext(readFileSync(join(import.meta.dirname, "web", pasta, a), "utf8"), ctx, { filename: a });
  carrega("js", "i18n.js");
  if (idioma === "en") ["en-convite.js", "en-base.js"].forEach((c) => carrega("i18n", c));
  carrega("js", "sessao.js"); carrega("js", "convite.js");
  vm.runInContext("this.api = { segredoDoFragmento, destinoSeguro, problemaDoNome, papelNoEscopo, poderesDaSessao, textoDoErro, entradaPeloConvite, entrarPeloConvite, nomeDoPapel };", ctx);
  return { api: ctx.api, els, eventos, ouvintes, ctx };
}
const espera = () => new Promise((r) => setTimeout(r, 0));

// funções puras
const { api } = contexto();
assert.equal(api.segredoDoFragmento("#" + SEGREDO), SEGREDO);
assert.equal(api.segredoDoFragmento("#curto"), "", "segredo curto demais não é segredo");
assert.equal(api.segredoDoFragmento("#" + SEGREDO + "<script>"), "", "só base64url");
assert.equal(api.destinoSeguro("/#/c/sm"), "/#/c/sm");
assert.equal(api.destinoSeguro("/#/p/bp1a2b3"), "/#/p/bp1a2b3");
for (const ruim of ["https://outro.site/", "//outro.site/#/c/x", "/#/c/x/../../y", "javascript:alert(1)", "", null]) assert.equal(api.destinoSeguro(ruim), "/", `destino recusado: ${ruim}`);
assert.equal(api.problemaDoNome("  "), "falta-nome");
assert.equal(api.problemaDoNome("x".repeat(61)), "nome-invalido");
assert.equal(api.problemaDoNome("Ana\u0007"), "nome-invalido", "sem caractere de controle");
assert.equal(api.problemaDoNome("Ana Lúcia"), "");

// papel por escopo: '*' > workspace > blueprint; o maior que cobre
const acessos = [{ workspaceId: "w1", blueprintId: "", papel: "supervisor" }, { workspaceId: "w1", blueprintId: "b1", papel: "colaborador" }, { workspaceId: "w2", blueprintId: "b9", papel: "dono" }];
assert.equal(api.papelNoEscopo(acessos, "w1"), "supervisor", "no workspace vale o acesso do workspace");
assert.equal(api.papelNoEscopo(acessos, "w1", "b1"), "colaborador", "no blueprint, o maior que cobre");
assert.equal(api.papelNoEscopo(acessos, "w2"), "", "acesso a um blueprint não cobre o workspace inteiro");
assert.equal(api.papelNoEscopo(acessos, "w3"), "");
assert.equal(api.papelNoEscopo([{ workspaceId: "*", blueprintId: "", papel: "dono" }], "qualquer", "x"), "dono", "'*' cobre tudo");

// botões por papel (o servidor confere de novo; isto só decide o que aparece)
const pode = (s) => JSON.stringify(api.poderesDaSessao(s));
assert.equal(pode({ modo: "local" }), JSON.stringify({ pessoas: false, chave: false, acompanhar: false, soLeitura: false }), "modo local: nada muda");
assert.equal(pode({ modo: "servidor", membro: { id: "m" }, visitante: false, acessos: [{ papel: "dono" }] }), JSON.stringify({ pessoas: true, chave: true, acompanhar: true, soLeitura: false }));
assert.equal(pode({ modo: "servidor", membro: { id: "m" }, visitante: false, acessos: [{ papel: "colaborador" }] }), JSON.stringify({ pessoas: false, chave: true, acompanhar: false, soLeitura: false }));
assert.equal(pode({ modo: "servidor", membro: { id: "m" }, visitante: false, acessos: [{ papel: "supervisor" }] }), JSON.stringify({ pessoas: true, chave: true, acompanhar: true, soLeitura: true }));
assert.equal(pode({ modo: "servidor", membro: null, visitante: true, acessos: [{ papel: "visitante" }] }), JSON.stringify({ pessoas: false, chave: false, acompanhar: false, soLeitura: true }));

// erros pelo código estável; sem código conhecido, o texto do servidor
assert.match(api.textoDoErro("muitas-tentativas", { segundos: 42 }), /42 s/);
assert.match(api.textoDoErro("nome-invalido", { max: 60 }), /60/);
assert.equal(api.textoDoErro("desconhecido", {}, "texto do servidor"), "texto do servidor");
const en = contexto({ idioma: "en" }).api;
assert.equal(en.textoDoErro("ultimo-dono"), "The workspace needs at least one owner.", "inglês pelo catálogo");
assert.equal(en.nomeDoPapel("supervisor"), "Supervisor");

// fluxo: membro. O segredo sai da barra ANTES do primeiro pedido e só vai no corpo; depois de entrar, location.replace(destino)
{
  const c = contexto({ hash: "#" + SEGREDO, respostas: {
    "/api/convite/conferir": [200, { papel: "colaborador", workspace: { id: "w1", nome: "Equipe" }, blueprint: null, abre: "folha", pedeNome: true }],
    "/api/convite/entrar": [200, { ok: true, destino: "/#/c/w1" }] } });
  await c.api.entradaPeloConvite();
  assert.deepEqual(c.eventos[0], ["replaceState", "/convite"], "replaceState vem antes de qualquer fetch");
  assert.deepEqual(c.eventos[1], ["fetch", "/api/convite/conferir", JSON.stringify({ segredo: SEGREDO })], "o segredo vai no corpo, não na URL");
  assert.equal(c.els["convite-campos"].hidden, false, "pede o nome");
  assert.match(c.els["convite-papel"].textContent, /^Colaborador:/, "explica o papel");
  assert.match(c.els["convite-titulo"].textContent, /Equipe/);
  await c.api.entrarPeloConvite("Bruno");
  assert.deepEqual(c.eventos[2], ["fetch", "/api/convite/entrar", JSON.stringify({ segredo: SEGREDO, nome: "Bruno" })]);
  assert.deepEqual(c.eventos[3], ["replace", "/#/c/w1"], "location.replace: o convite não fica no histórico");
  assert.equal(c.eventos.some((e) => e[0] === "fetch" && e[1].includes(SEGREDO)), false, "nenhuma URL leva o segredo");
}
// fluxo: visitante segue direto, sem nome; destino estranho cai em "/"
{
  const c = contexto({ hash: "#" + SEGREDO, respostas: {
    "/api/convite/conferir": [200, { papel: "visitante", workspace: { id: "sm", nome: "X" }, blueprint: { id: "b1", nome: "Portal" }, abre: "apresentacao", pedeNome: false }],
    "/api/convite/entrar": [200, { ok: true, destino: "https://outro.site/" }] } });
  await c.api.entradaPeloConvite(); await espera();
  assert.deepEqual(c.eventos.filter((e) => e[0] === "fetch").map((e) => e[2]), [JSON.stringify({ segredo: SEGREDO }), JSON.stringify({ segredo: SEGREDO })], "visitante entra sem nome");
  assert.deepEqual(c.eventos.at(-1), ["replace", "/"], "destino de outro site é recusado");
}
// fluxo: convite inválido mostra o aviso e nem tenta entrar; sem fragmento, nenhum pedido
{
  const c = contexto({ hash: "#" + SEGREDO, respostas: { "/api/convite/conferir": [404, { erro: "x", codigo: "convite-invalido" }] } });
  await c.api.entradaPeloConvite();
  assert.equal(c.els["convite-titulo"].textContent, "Convite indisponível");
  assert.equal(c.eventos.filter((e) => e[0] === "fetch").length, 1);
  const v = contexto({ hash: "" });
  await v.api.entradaPeloConvite();
  assert.equal(v.eventos.filter((e) => e[0] === "fetch").length, 0, "sem segredo: nenhum pedido");
  assert.equal(v.els["convite-titulo"].textContent, "Abra o link completo");
  // outro link colado na mesma aba só muda o fragmento: a página confere o novo (e o tira da barra de novo)
  v.ctx.location.hash = "#" + SEGREDO;
  v.ouvintes.hashchange(); await espera();
  assert.deepEqual(v.eventos.slice(-2), [["replaceState", "/convite"], ["fetch", "/api/convite/conferir", JSON.stringify({ segredo: SEGREDO })]]);
}
console.log("convite: ok (segredo fora da URL, replaceState antes do pedido, destino seguro, papel por escopo, botões por papel, erros por código, inglês)");
