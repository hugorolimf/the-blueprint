import assert from "node:assert/strict";
import { carregar, exemplo } from "./teste-comum.mjs";

const { L, comFuncao } = carregar();
const { conteudo, figuras } = exemplo();
assert.equal(L.auditarCliente(conteudo).length, 0, "o exemplo é limpo para o público cliente");
assert.equal(figuras[0].impressao, L.impressao(conteudo), "figuras.json está em dia com conteudo.json (rode exemplo/gerar.mjs)");

const [fig] = comFuncao(figuras);
const r = fig.codigo(L, conteudo);
const solidos = (r.svg.match(/class="iso-solid"/g) || []).length;
assert.ok(solidos >= 50 && solidos <= 80, `a figura tem ${solidos} sólidos (esperado de 50 a 80)`);
assert.equal((r.svg.match(/data-i="/g) || []).length, 14, "quatorze fichas");
assert.equal((r.svg.match(/data-peca="leitor"/g) || []).length, 1);
assert.equal((r.svg.match(/data-tone="lit"/g) || []).length, 0, "a peça lit é ligada por data-lit, não fixa no desenho");
assert.equal(typeof r.vivo, "function");
assert.ok(r.svg.length < 200000, "figura dentro do limite de 200 KB");
assert.ok(figuras[0].codigo.length < 200000);
// a página do exemplo sai com a figura na capa
const { html, vivos } = L.casca.html(conteudo, [fig]);
assert.ok(html.includes("Fig 1") && html.includes("Fichário de cadastro") && html.includes('data-figura="capa"'));
assert.equal(vivos.length, 1);
console.log("exemplo.test.mjs: ok");
