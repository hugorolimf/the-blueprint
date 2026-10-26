// Camada de idiomas (web/js/i18n.js, script clássico carregado num contexto isolado como no navegador): escolha do idioma, tradução, plural e fallback.
//   node i18n.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const carrega = () => {
  const ctx = vm.createContext({ URLSearchParams, Intl, navigator: { languages: ["pt-BR"] }, location: { search: "" }, localStorage: { getItem: () => null, setItem() {}, removeItem() {} } });
  vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), ctx, { filename: "i18n.js" });
  vm.runInContext("this.api = { I18N, i18nEscolhe, tr, trn, i18nNormaliza };", ctx);
  return ctx.api;
};
const { I18N, i18nEscolhe, tr, trn, i18nNormaliza } = carrega();

// escolha: URL > guardado > idioma do Windows > inglês
assert.equal(i18nEscolhe({ url: "", guardado: "", sistema: ["pt-BR"] }), "pt-BR", "Windows em português: português");
assert.equal(i18nEscolhe({ url: "", guardado: "", sistema: ["pt-PT"] }), "pt-BR", "qualquer português cai no pt-BR");
assert.equal(i18nEscolhe({ url: "", guardado: "", sistema: ["en-US"] }), "en");
assert.equal(i18nEscolhe({ url: "", guardado: "", sistema: ["fr-FR", "en-GB"] }), "en", "usa o primeiro idioma suportado da lista");
assert.equal(i18nEscolhe({ url: "", guardado: "", sistema: ["fr-FR"] }), "en", "idioma sem tradução: inglês");
assert.equal(i18nEscolhe({ url: "", guardado: "", sistema: [] }), "en", "sem nenhuma informação: inglês");
assert.equal(i18nEscolhe({ url: "", guardado: "en", sistema: ["pt-BR"] }), "en", "a escolha guardada vale mais que o Windows");
assert.equal(i18nEscolhe({ url: "", guardado: "auto", sistema: ["pt-BR"] }), "pt-BR", "auto segue o Windows");
assert.equal(i18nEscolhe({ url: "?idioma=en", guardado: "pt-BR", sistema: ["pt-BR"] }), "en", "a URL vale mais que tudo");
assert.equal(i18nNormaliza("EN-us"), "en");

// tradução com fallback para o texto-fonte e parâmetros
Object.assign(I18N.catalogos.en, { "Aplicar": "Apply", "{nome} concluiu": "{nome} finished", "{n} plano": ["{n} plan", "{n} plans"] });
I18N.ativo = "pt-BR";
assert.equal(tr("Aplicar"), "Aplicar", "o idioma-fonte devolve o próprio texto");
I18N.ativo = "en";
assert.equal(tr("Aplicar"), "Apply");
assert.equal(tr("Descartar"), "Descartar", "sem tradução: cai no texto-fonte, nunca quebra");
assert.equal(tr("{nome} concluiu", { nome: "Claude" }), "Claude finished");
assert.equal(tr("{x} sem valor", {}), "{x} sem valor", "parâmetro que falta fica como está");
assert.equal(tr("toString"), "toString", "chave herdada do Object não é tradução");

// plural pela regra do idioma ativo
assert.equal(trn(1, ["{n} plano", "{n} planos"]), "1 plan");
assert.equal(trn(0, ["{n} plano", "{n} planos"]), "0 plans", "inglês: zero é plural");
assert.equal(trn(3, ["{n} plano", "{n} planos"]), "3 plans");
Object.assign(I18N.catalogos.en, { "{n} blueprint": ["{n} blueprint", "{n} blueprints"] });
assert.equal(trn(1, ["{n} blueprint", "{n} blueprints"]), "1 blueprint", "forma em inglês igual à chave");
assert.equal(trn(2, ["{n} blueprint", "{n} blueprints"]), "2 blueprints");
I18N.ativo = "pt-BR";
assert.equal(trn(0, ["{n} plano", "{n} planos"]), "0 plano", "português: zero é singular");
assert.equal(trn(1, ["{n} plano", "{n} planos"]), "1 plano");
assert.equal(trn(2, ["{n} plano", "{n} planos"]), "2 planos");
console.log("i18n: ok (escolha do idioma, tradução, fallback, parâmetros e plural)");
