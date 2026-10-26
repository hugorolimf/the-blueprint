// O endereço fixo do compartilhamento de ponta a ponta (ação "Endereço fixo para o compartilhamento", docs/compartilhar.md), no binário real com um `tailscale` falso
// (scripts/tailscale-falso.mjs, que sobe o túnel falso com o endereço FIXO https://maq.tail1234.ts.net): o dono liga escolhendo "fixo", a convidada entra pelo convite,
// o dono desliga e liga de novo SEM dizer o endereço (a escolha ficou guardada) e a convidada continua entrando no mesmo endereço com a mesma sessão, sem link novo; o
// diálogo mostra a escolha (e a trava enquanto liga). APPDATA temporário: a escolha não vai para a pasta do app de verdade. Nada de rede nem de Tailscale de verdade.
//   node scripts/endereco-fixo-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; precisa do Microsoft Edge; ~15 s; Windows)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { request } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { espera, janela } from "./lib-e2e-edge.mjs";
import { dormir, exePadrao, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = exePadrao();
assert.ok(existsSync(exe), `não achei ${exe}`);
const WS = "lab0001", FIXO = "https://maq.tail1234.ts.net";
const tmp = mkdtempSync(join(tmpdir(), "endereco-fixo-e2e-"));
const porta = await portaLivre(), url = `http://127.0.0.1:${porta}`, info = join(tmp, "tunel.json");
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
mkdirSync(join(tmp, "appdata"));
writeFileSync(join(tmp, "tailscale-falso.cmd"), `@node "${join(raiz, "scripts", "tailscale-falso.mjs")}" %*\r\n`);
writeFileSync(join(tmp, "tunel-falso.cmd"), `@node "${join(raiz, "scripts", "tunel-falso.mjs")}" %*\r\n`);
const servidor = spawn(exe, ["serve"], { env: { ...process.env, APPDATA: join(tmp, "appdata"), HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: raiz, BLUEPRINT_SEM_ATUALIZACAO: "1", BLUEPRINT_TAILSCALE_COMANDO: join(tmp, "tailscale-falso.cmd"), BLUEPRINT_TUNEL_COMANDO: join(tmp, "tunel-falso.cmd"), TUNEL_FALSO_INFO: info }, stdio: "ignore", windowsHide: true });
const filhos = [servidor];

const pedir = (endereco, { metodo = "GET", cabecalhos = {}, corpo } = {}) => new Promise((ok, falha) => {
  const u = new URL(endereco), texto = corpo === undefined ? undefined : JSON.stringify(corpo);
  const req = request({ hostname: u.hostname, port: u.port, path: u.pathname + u.search, method: metodo, headers: { ...(texto ? { "content-type": "application/json" } : {}), ...cabecalhos } }, (res) => {
    let t = ""; res.on("data", (d) => (t += d)); res.on("end", () => { let json = null; try { json = JSON.parse(t); } catch {} ok({ status: res.statusCode, texto: t, json, cabecalhos: res.headers }); });
  });
  req.on("error", falha);
  if (texto) req.write(texto);
  req.end();
});
const local = { origin: url };
const estado = async () => (await pedir(`${url}/api/compartilhar`)).json;
const esperaEstado = async (quer) => { let e; for (let i = 0; i < 300; i++) { e = await estado(); if (e.estado === quer) return e; await dormir(50); } assert.fail(`não chegou a ${quer}: ${JSON.stringify(e)}`); };

try {
  for (let i = 0; ; i++) { try { await fetch(url + "/api/dados"); break; } catch { if (i > 100) throw new Error("o servidor não respondeu"); await dormir(50); } }
  assert.deepEqual(await estado(), { estado: "desligado", provedor: "temporario" }, "o padrão de sempre");
  assert.equal((await pedir(`${url}/api/compartilhar/ligar`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS, provedor: "zerotier" } })).json.codigo, "provedor-invalido");

  // 1. liga escolhendo o endereço fixo: o endereço é o do Tailscale, não o da Cloudflare
  assert.equal((await pedir(`${url}/api/compartilhar/ligar`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS, provedor: "fixo" } })).status, 202);
  const ligado = await esperaEstado("ligado");
  assert.deepEqual([ligado.url, ligado.provedor], [FIXO, "fixo"]);
  const t1 = JSON.parse(readFileSync(info, "utf8"));
  assert.equal(t1.publica, FIXO);

  // 2. a convidada entra pelo convite e a sessão dela vale pelo endereço fixo
  const convite = await pedir(`${url}/api/convites`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS, papel: "colaborador" } });
  assert.equal(convite.json.link.startsWith(FIXO + "/convite#"), true, "o link de convite já sai no endereço fixo: " + convite.json.link);
  const entrou = await pedir(`${t1.local}/api/convite/entrar`, { metodo: "POST", cabecalhos: { origin: t1.local }, corpo: { segredo: convite.json.link.split("#")[1], nome: "Bia" } });
  assert.equal(entrou.status, 200, entrou.texto);
  const cookie = String(entrou.cabecalhos["set-cookie"]).split(";")[0];
  const sessao = (t) => pedir(`${t.local}/api/sessao`, { cabecalhos: { origin: t.local, cookie } });
  assert.equal((await sessao(t1)).json.membro.nome, "Bia");

  // 3. o dono desliga (a convidada perde o acesso na hora) e liga de novo SEM dizer o endereço: a escolha ficou guardada e o endereço é o mesmo
  await pedir(`${url}/api/compartilhar/desligar`, { metodo: "POST", cabecalhos: local, corpo: {} });
  assert.deepEqual(await estado(), { estado: "desligado", provedor: "fixo" }, "desligado, a escolha continua à vista");
  assert.equal(JSON.parse(readFileSync(join(tmp, "appdata", "Blueprint", "compartilhar.json"), "utf8")).provedor, "fixo", "guardada no arquivo do app");
  await pedir(`${url}/api/compartilhar/ligar`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS } });
  const religado = await esperaEstado("ligado");
  assert.equal(religado.url, FIXO, "o MESMO endereço");
  const t2 = JSON.parse(readFileSync(info, "utf8"));
  assert.equal(t2.publica, FIXO);
  const volta = await sessao(t2);
  assert.equal(volta.status, 200, "a convidada continua entrando, sem link novo: " + volta.texto);
  assert.equal(volta.json.membro.nome, "Bia");

  // 4. o diálogo mostra a escolha e a trava enquanto liga; desligado, a pessoa pode mudar
  const pagina = await janela({ dir: tmp, url }, "dono", `/#/c/${WS}`, filhos);
  await espera(() => pagina.js(`typeof carregado !== "undefined" && carregado && workspaceAtual?.id === ${JSON.stringify(WS)} && document.getElementById("b-compartilhar")?.offsetParent != null`), "Compartilhar visível", 15000);
  await pagina.js(`document.getElementById("b-compartilhar").click()`);
  await espera(() => pagina.js(`!!document.querySelector('input[name="cp-tipo"]:checked') && /Compartilhando/.test(document.getElementById("cp-titulo")?.textContent || "")`), "diálogo aberto, ligado");
  assert.deepEqual(await pagina.js(`[...document.querySelectorAll('input[name="cp-tipo"]')].map((r) => [r.value, r.checked, r.disabled])`), [["temporario", false, true], ["fixo", true, true]], "ligado: mostra o fixo marcado e travado");
  assert.match(await pagina.js(`document.getElementById("cp-dica").textContent`), /endereço é fixo/);
  assert.equal(await pagina.js(`document.getElementById("cp-url").textContent`), FIXO);
  await pedir(`${url}/api/compartilhar/desligar`, { metodo: "POST", cabecalhos: local, corpo: {} }); // o clique em Desligar pede confirmação (coberto em compartilhar-e2e); aqui só o efeito
  await espera(() => pagina.js(`/desligado/i.test(document.getElementById("cp-titulo")?.textContent || "")`), "o diálogo viu o desligamento pelo SSE", 10000);
  assert.deepEqual(await pagina.js(`[...document.querySelectorAll('input[name="cp-tipo"]')].map((r) => [r.value, r.checked, r.disabled])`), [["temporario", false, false], ["fixo", true, false]], "desligado: a escolha guardada está marcada e dá para mudar");
  assert.deepEqual(pagina.erros, [], "nenhum erro no console");
  console.log("endereco-fixo e2e: ok (ligar escolhendo o fixo, convite no endereço fixo, religar sem dizer nada dá o mesmo endereço e a mesma sessão, escolha guardada, diálogo)");
} finally {
  for (const p of filhos) try { p.kill(); } catch {}
  await dormir(800);
  try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {}
}
