// Diálogo Compartilhar do app local (web/js/compartilhar.js, módulo ES): estados do túnel, textos de cada estado e o corpo do ligar.
// A tela inteira (botão, ligar, endereço, convite, pessoas, desligar) é coberta por scripts/compartilhar-e2e.mjs; aqui ficam as regras puras.
//   node compartilhar.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const avisar = process.emitWarning;
process.emitWarning = (w, ...r) => (/MODULE_TYPELESS/.test(JSON.stringify(r)) ? undefined : avisar.call(process, w, ...r));
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), { filename: "i18n.js" });
vm.runInThisContext(`I18N.ativo = "pt-BR"`);
const { ESTADOS, normalizar, corpoDoLigar, vistaDoEstado } = await import("./web/js/compartilhar.js");

// o servidor pode mandar só o estado; estado desconhecido ou resposta vazia vira erro (nunca um botão errado)
assert.deepEqual(ESTADOS, ["desligado", "baixando", "ligando", "ligado", "erro"]);
assert.deepEqual(normalizar({ estado: "ligado", url: "https://a.trycloudflare.com", workspaces: ["w1", "w2"], workspaceId: "w1", porta: 4100 }), { estado: "ligado", url: "https://a.trycloudflare.com", workspaces: ["w1", "w2"], erro: "" });
assert.deepEqual(normalizar({ estado: "ligado", workspaceId: "w1" }).workspaces, ["w1"], "servidor antigo, com um workspace só");
assert.equal(normalizar({ estado: "quebrado" }).estado, "erro");
assert.equal(normalizar(null).estado, "erro");
assert.equal(normalizar({ estado: "erro", erro: "sem rede" }).erro, "sem rede");

// corpo de POST /api/compartilhar/ligar: o workspace aberto
assert.deepEqual(corpoDoLigar("w1"), { workspaceId: "w1" });
assert.deepEqual(corpoDoLigar(""), { erro: "falta-workspace" });

// cada estado: botão, barra de progresso e endereço
const nomes = { w1: "Produto", w2: "Pessoal" }, nomeDe = (id) => nomes[id] || id;
const v = (s, ws = "w1") => vistaDoEstado(normalizar(s), ws, nomeDe);
const desligado = v({ estado: "desligado" });
assert.equal(desligado.botao, "ligar");
assert.equal(desligado.ocupado, false);
assert.match(desligado.texto, /«Produto»/, "fala do workspace aberto");
for (const e of ["baixando", "ligando"]) {
  const x = v({ estado: e, workspaceId: "w1" });
  assert.equal(x.ocupado, true, `${e}: mostra o progresso`);
  assert.equal(x.botao, "cancelar", `${e}: dá para cancelar`);
  assert.equal(x.url, "", `${e}: ainda sem endereço`);
}
assert.match(v({ estado: "baixando" }).texto, /primeira vez/);
const ligado = v({ estado: "ligado", url: "https://a.trycloudflare.com", workspaceId: "w1" });
assert.deepEqual([ligado.botao, ligado.ocupado, ligado.url], ["desligar", false, "https://a.trycloudflare.com"]);
assert.equal(ligado.titulo, "Compartilhando «Produto»");
assert.match(ligado.texto, /só vê este workspace/);
// túnel ligado para outro workspace: o diálogo continua falando do workspace aberto e oferece incluí-lo no mesmo endereço
const outro = v({ estado: "ligado", url: "https://b.trycloudflare.com", workspaces: ["w2"] }, "w1");
assert.equal(outro.titulo, "«Produto» não está compartilhado");
assert.match(outro.texto, /incluir «Produto» no mesmo endereço/);
assert.deepEqual([outro.botao, outro.url], ["ligar", ""], "sem endereço nem pessoas de outro workspace");
const dois = v({ estado: "ligado", url: "https://b.trycloudflare.com", workspaces: ["w2", "w1"] }, "w1");
assert.deepEqual([dois.titulo, dois.botao, dois.url], ["Compartilhando «Produto»", "desligar", "https://b.trycloudflare.com"]);
const erro = v({ estado: "erro", erro: "A Cloudflare não respondeu." });
assert.deepEqual([erro.botao, erro.texto], ["ligar", "A Cloudflare não respondeu."], "erro: mostra o texto do servidor e deixa tentar de novo");
assert.equal(v({ estado: "erro" }).texto, "O túnel não ligou. Tente de novo.");

// em inglês, os textos vêm do catálogo (web/i18n/en-compartilhar.js)
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "i18n", "en-compartilhar.js"), "utf8"));
vm.runInThisContext(`I18N.ativo = "en"`);
assert.equal(v({ estado: "ligado", url: "https://a.trycloudflare.com", workspaceId: "w1" }).titulo, "Sharing “Produto”");
assert.equal(v({ estado: "desligado" }).titulo, "Sharing is off");
console.log("compartilhar.test: ok");
