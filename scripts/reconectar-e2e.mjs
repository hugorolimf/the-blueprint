// A página não fica com a tela velha depois de uma queda (docs/colaboracao.md, seção 6, "Reconexão pela versão" e "Sem rede").
// Um proxy TCP entre o Edge e o servidor derruba a conexão de propósito:
//   1) queda curta: enquanto a página está fora, um script grava duas mudanças; ao voltar, o navegador reconecta sozinho com Last-Event-ID e
//      recebe só os lotes que perdeu (sem GET inteiro);
//   2) o servidor reinicia (a contagem de versões recomeça) com uma edição da página ainda não gravada: a página recebe `recarregar`, faz o
//      GET, junta com o que tinha na tela e grava o que faltava.
//   node scripts/reconectar-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; ~20 s)
import assert from "node:assert/strict";
import net from "node:net";
import { rmSync } from "node:fs";
import { espera, janela } from "./lib-e2e-edge.mjs";
import { dormir, exePadrao, http, ligarServidor, pastaDeDados, portaLivre, postJson, raiz } from "./lib-e2e-rust.mjs";

const pasta = await pastaDeDados("reconectar", { BLUEPRINT_ASSETS_DIR: raiz });
let servidor = await ligarServidor(exePadrao(), pasta);
const filhos = [];

// o proxy: conta os GET /api/dados e, fechado, recusa conexões (como uma rede caída)
let aberto = true, getsDados = 0;
const conexoes = new Set();
const proxy = net.createServer((c) => {
  if (!aberto) return c.destroy();
  const s = net.connect(pasta.porta, "127.0.0.1");
  const fim = () => { c.destroy(); s.destroy(); conexoes.delete(c); };
  conexoes.add(c);
  c.on("data", (d) => { if (d.toString("latin1").startsWith("GET /api/dados")) getsDados++; });
  c.pipe(s); s.pipe(c);
  for (const x of [c, s]) { x.on("error", fim); x.on("close", fim); }
});
const portaProxy = await portaLivre();
await new Promise((r) => proxy.listen(portaProxy, "127.0.0.1", r));
const cair = () => { aberto = false; for (const c of conexoes) c.destroy(); };
const voltar = () => { aberto = true; };

const plano = async () => (await http(pasta.url + "/api/dados")).json.blueprints[0].acoes[0].planos[0];
const mudarPeloScript = (campos) => postJson(pasta.url + "/api/mudancas", { cliente: "script", ops: [{ op: "alterar", tipo: "plano", id: "t1", campos }] });
const naTela = (campo) => `opsAchar(arquivo, "plano", "t1").no.${campo}`;
try {
  const A = await janela({ dir: pasta.dir, url: `http://127.0.0.1:${portaProxy}` }, "a", "/#/p/p1", filhos);
  await espera(async () => (await A.js(`typeof aoVivoAberto === "function" && aoVivoAberto() && !!aoVivoBase && aoVivoES.readyState === 1`)) === true, "a página abriu o SSE");

  // 1) queda curta: perde dois lotes e os recebe pela versão ao voltar
  const getsAntes = getsDados;
  await A.js(`window.recargas = 0; { const o = aoVivoRecarregar; aoVivoRecarregar = () => { window.recargas++; return o(); }; }`);
  cair();
  await dormir(300);
  assert.equal((await mudarPeloScript({ texto: "durante a queda 1" })).status, 200);
  assert.equal((await mudarPeloScript({ descricao: "durante a queda 2" })).status, 200);
  await dormir(500);
  assert.equal(await A.js(naTela("texto")), "Plano um", "fora da rede, a tela ainda não sabe");
  voltar();
  const levou = await espera(async () => (await A.js(`${naTela("texto")} + "|" + ${naTela("descricao")}`)) === "durante a queda 1|durante a queda 2", "ao voltar, recebe os dois lotes perdidos", 8000);
  assert.equal(await A.js(`window.recargas`), 0, "retomou pelo Last-Event-ID, sem recarregar (o GET que houver é o do SSE do MCP, que confere ao reconectar)");
  console.log(`  1) queda curta: ok (recuperou em ${levou} ms depois de a rede voltar, só os lotes perdidos)`);

  // 2) o servidor reinicia com uma edição da página ainda não gravada
  cair();
  servidor.kill();
  await dormir(500);
  await A.js(`${naTela("texto")} = "editado sem rede"; mudou()`);
  await espera(async () => (await A.js(`document.getElementById("estado").textContent`)).includes("erro"), "sem servidor, o salvar avisa o erro");
  servidor = await ligarServidor(exePadrao(), pasta);
  assert.equal((await mudarPeloScript({ descricao: "outro mexeu enquanto a página estava fora" })).status, 200);
  voltar();
  await espera(async () => (await plano()).texto === "editado sem rede", "depois de recarregar e juntar, a edição da página foi gravada", 10000);
  const p = await plano();
  assert.equal(p.descricao, "outro mexeu enquanto a página estava fora", "e não apagou o que o outro gravou");
  await espera(async () => (await A.js(naTela("descricao"))) === p.descricao, "a tela mostra a mudança do outro");
  assert.ok(getsDados > getsAntes && (await A.js(`window.recargas`)) >= 1, "o recarregar fez o GET inteiro");
  assert.deepEqual(A.erros, [], "sem erro no console");
  console.log("  2) servidor reiniciado: ok (recarregar, junção e o que faltava gravado)");
  console.log("reconectar e2e: ok");
} finally {
  for (const f of [...filhos, servidor]) { try { f.kill(); } catch {} }
  proxy.close();
  setTimeout(() => { try { rmSync(pasta.dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800);
}
