import assert from "node:assert/strict";
import { carregar } from "./teste-comum.mjs";

const { L } = carregar();
const pt = Object.keys(L.TEXTOS["pt-BR"]).sort(), en = Object.keys(L.TEXTOS.en).sort();
assert.equal(pt.join(), en.join(), "pt-BR e en têm as mesmas chaves");
assert.equal(L.t("dia_de", "pt-BR", { n: 3, total: 20 }), "Dia 3 de 20");
assert.equal(L.t("dia_de", "en", { n: 3, total: 20 }), "Day 3 of 20");
assert.equal(L.t("dia_de", "fr", { n: 1, total: 2 }), "Day 1 of 2", "idioma desconhecido cai no inglês");
assert.equal(L.t("dia_de", "pt", { n: 1, total: 2 }), "Dia 1 de 2", "pt cai no pt-BR");
assert.equal(L.t("chave-que-nao-existe", "en"), "chave-que-nao-existe");
assert.equal(L.dataCurta("2026-10-26", "pt-BR"), "26 de out");
assert.equal(L.dataCurta("2026-10-26", "en"), "Oct 26");
assert.equal(L.dataCurta("lixo", "en"), "");
console.log("textos.test.mjs: ok");
