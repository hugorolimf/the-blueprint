import assert from "node:assert/strict";
import { carregar, exemplo } from "./teste-comum.mjs";

const { L, comFuncao } = carregar();
const { conteudo, figuras } = exemplo();
assert.equal(L.auditarCliente(conteudo).length, 0, "o exemplo é limpo para o público cliente");
assert.equal(figuras.find((f) => f.id === "visao").impressao, L.impressao({ titulo: conteudo.titulo, resumo: conteudo.resumo }), "figuras.json está em dia com conteudo.json (rode exemplo/gerar.mjs)");
assert.equal(figuras.find((f) => f.id === "fichario").impressao, L.impressao(conteudo));

const [visao, fig] = comFuncao(figuras);
const r = fig.codigo(L, conteudo);
const solidos = (r.svg.match(/class="iso-solid"/g) || []).length;
assert.ok(solidos >= 50 && solidos <= 80, `a figura tem ${solidos} sólidos (esperado de 50 a 80)`);
assert.equal((r.svg.match(/data-i="/g) || []).length, 14, "quatorze fichas");
assert.equal((r.svg.match(/data-peca="leitor"/g) || []).length, 1);
assert.equal((r.svg.match(/data-tone="lit"/g) || []).length, 0, "a peça lit é ligada por data-lit, não fixa no desenho");
assert.equal(typeof r.vivo, "function");
assert.ok(r.svg.length < 200000, "figura dentro do limite de 200 KB");
assert.ok(figuras.every((f) => f.codigo.length < 200000));
// a página do exemplo põe cada figura no painel da sua seção: o carrinho na Visão geral, o fichário em "o que você recebe"
const { html, vivos } = L.casca.html(conteudo, [visao, fig]);
const painel = (id) => { const i = html.indexOf(`id="s-${id}"`); return html.slice(i, html.indexOf("</section>", i)); };
assert.ok(painel("visao").includes('data-figura="visao"') && !painel("visao").includes("fichario"));
assert.ok(painel("entregas").includes('data-figura="fichario"') && painel("entregas").includes("Fichário de cadastro") && painel("entregas").includes("ap-com-fig"));
assert.equal(vivos.length, 2);
console.log("exemplo.test.mjs: ok");
