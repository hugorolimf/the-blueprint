import assert from "node:assert/strict";
import { carregar } from "./teste-comum.mjs";

const { L } = carregar();
const pt = Object.keys(L.TEXTOS["pt-BR"]).sort(), en = Object.keys(L.TEXTOS.en).sort();
assert.equal(pt.join(), en.join(), "pt-BR e en têm as mesmas chaves");
assert.equal(L.t("ir_fig", "pt-BR", { n: 3 }), "Ir para a figura 3");
assert.equal(L.t("ir_fig", "en", { n: 3 }), "Go to figure 3");
assert.equal(L.t("ir_fig", "fr", { n: 1 }), "Go to figure 1", "idioma desconhecido cai no inglês");
assert.equal(L.t("ir_fig", "pt", { n: 1 }), "Ir para a figura 1", "pt cai no pt-BR");
assert.equal(L.t("chave-que-nao-existe", "en"), "chave-que-nao-existe");
console.log("textos.test.mjs: ok");
