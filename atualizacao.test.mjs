// A interface da atualização (web/js/atualizacao.js, script clássico): carregada num contexto isolado com um DOM de mentira, como o navegador a carregaria.
//   node atualizacao.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const html = []; // tudo o que foi escrito como HTML: o texto vindo do GitHub nunca pode estar aqui
class El {
  constructor() { this.hidden = false; this.textContent = ""; this.style = {}; this.disabled = false; this.classes = new Set(); this.filhos = new Map(); this.classList = { toggle: (c, on) => this.classes[on ? "add" : "delete"](c) }; }
  querySelector(s) { if (!this.filhos.has(s)) this.filhos.set(s, new El()); return this.filhos.get(s); }
  setAttribute() {}
  set innerHTML(v) { html.push(v); }
}
const caixa = {};
const chamadas = [];
let respostaInstalar = { ok: true, status: 200, json: async () => ({}) };
const armazem = new Map();
const ctx = vm.createContext({
  document: { createElement: () => new El(), body: { appendChild: (el) => { caixa.el = el; } } },
  localStorage: { getItem: (k) => armazem.get(k) ?? null, setItem: (k, v) => armazem.set(k, v) },
  fetch: async (url, opcoes) => { chamadas.push([url, opcoes?.method || "GET", opcoes?.body]); return url.endsWith("/instalar") ? respostaInstalar : { ok: true, json: async () => ctx.estadoServidor }; },
  Date, JSON,
});
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "atualizacao.js"), "utf8") + "\nthis.api = { atualizacaoRecebe, atualizacaoBusca, atualizacaoInstala };", ctx);
const { atualizacaoRecebe, atualizacaoBusca, atualizacaoInstala } = ctx.api;
const q = (s) => caixa.el.querySelector(s);

// em dia, parada e desligada: nada na tela
for (const fase of ["parada", "em-dia", "desligada", "verificando"]) { atualizacaoRecebe({ fase, atual: "0.1.0" }); assert(!caixa.el || caixa.el.hidden, fase); }

// baixando: indicador discreto com o percentual e a barra, sem botões
atualizacaoRecebe({ fase: "baixando", atual: "0.1.0", versao: "0.2.0", percentual: 40 });
assert.equal(caixa.el.hidden, false);
assert.match(q(".at-t").textContent, /Baixando a versão 0\.2\.0… 40%/);
assert.equal(q(".at-barra").hidden, false);
assert.equal(q(".at-barra i").style.transform, "scaleX(0.4)");
assert.equal(q(".at-acoes").hidden, true, "baixando não tem botão: nada a decidir ainda");

// pronta: aviso com a primeira linha das notas e os botões; as notas entram como texto, nunca como HTML
atualizacaoRecebe({ fase: "pronta", atual: "0.1.0", versao: "0.2.0", notas: "\n<img src=x onerror=alert(1)> melhorias\noutra linha", motivo: null });
assert.match(q(".at-t").textContent, /A versão 0\.2\.0 está pronta \(você tem a 0\.1\.0\)/);
assert.equal(q(".at-notas").textContent, "<img src=x onerror=alert(1)> melhorias");
assert.equal(q(".at-acoes").hidden, false);
assert.equal(q(".at-barra").hidden, true);
assert(!html.some((h) => h.includes("onerror")), "o texto do GitHub não entra por innerHTML");

// o clique em Atualizar chama o POST de instalar (e só ele); com a fila cheia o servidor recusa e o motivo aparece, continuando pronta
await atualizacaoInstala();
assert.deepEqual(chamadas.at(-1), ["/api/atualizacao/instalar", "POST", "{}"]);
respostaInstalar = { ok: false, status: 409, json: async () => ({ erro: "Há 2 aviso(s) do agente que a tela ainda não aplicou.", codigo: "agora-nao" }) };
await atualizacaoInstala();
assert.equal(q(".at-motivo").textContent, "Há 2 aviso(s) do agente que a tela ainda não aplicou.");
assert.equal(q(".at-motivo").hidden, false);
assert.equal(caixa.el.classes.has("erro"), true);
assert.equal(q(".at-ok").disabled, false, "o botão volta para tentar de novo");

// o motivo que o servidor guarda (recusa pela bandeja) também aparece
atualizacaoRecebe({ fase: "pronta", atual: "0.1.0", versao: "0.2.0", notas: "", motivo: "Fila cheia." });
assert.equal(q(".at-motivo").textContent, "Fila cheia.");

// instalando
atualizacaoRecebe({ fase: "instalando", atual: "0.1.0", versao: "0.2.0" });
assert.match(q(".at-t").textContent, /fecha e abre de novo/);
assert.equal(q(".at-acoes").hidden, true);

// "Depois": some por 24 h para aquela versão e volta para uma versão mais nova
atualizacaoRecebe({ fase: "pronta", atual: "0.1.0", versao: "0.2.0", notas: "" });
q(".at-depois").onclick();
assert.equal(caixa.el.hidden, true);
atualizacaoRecebe({ fase: "pronta", atual: "0.1.0", versao: "0.2.0", notas: "" });
assert.equal(caixa.el.hidden, true, "a mesma versão continua adiada");
atualizacaoRecebe({ fase: "pronta", atual: "0.1.0", versao: "0.3.0", notas: "" });
assert.equal(caixa.el.hidden, false, "uma versão mais nova avisa de novo");

// lixo do servidor não derruba a página
atualizacaoRecebe(null); atualizacaoRecebe("x");
ctx.estadoServidor = { fase: "em-dia", atual: "0.1.0" };
await atualizacaoBusca();
assert.equal(chamadas.at(-1)[0], "/api/atualizacao");
// o instalador baixado mudou depois do download: o servidor vai para "erro" e o usuário que estava no aviso vê o porquê (e some quando o estado normaliza)
atualizacaoRecebe({ fase: "pronta", atual: "0.1.0", versao: "0.4.0", notas: "" });
atualizacaoRecebe({ fase: "erro", atual: "0.1.0", erro: "O instalador baixado mudou depois do download e foi descartado." });
assert.equal(caixa.el.hidden, false);
assert.equal(q(".at-motivo").textContent, "O instalador baixado mudou depois do download e foi descartado.");
atualizacaoRecebe({ fase: "em-dia", atual: "0.1.0" });
assert.equal(caixa.el.hidden, true);
// erro em segundo plano (sem rede, por exemplo) não incomoda: nada na tela
atualizacaoRecebe({ fase: "verificando", atual: "0.1.0" });
atualizacaoRecebe({ fase: "erro", atual: "0.1.0", erro: "não consegui falar com o GitHub" });
assert.equal(caixa.el.hidden, true);
console.log("ok: atualizacao.test.mjs");
