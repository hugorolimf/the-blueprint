// O fragmento e a mensagem `ap-ir` no Edge de verdade, num iframe sandbox servido como o app serve (cabeçalho CSP `sandbox allow-scripts`, origem opaca).
// O pai (página de teste) guarda as mensagens `ap-estado` que a página manda e o teste as lê com --dump-dom. Sem Edge/Chrome avisa e sai sem falhar.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { createServer as livre } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { existsSync } from "node:fs";
import { createServer } from "node:http";
import { montar } from "./montar.mjs";
import { exemplo } from "./teste-comum.mjs";

const candidatos = [process.env.CHROME_PATH, "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Google/Chrome/Application/chrome.exe", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"];
const navegador = candidatos.find((c) => c && existsSync(c));
if (!navegador) { console.log("fragmento-navegador.test.mjs: sem Edge/Chrome, pulado"); process.exit(0); }

const { conteudo, figuras } = exemplo();
// uma sonda só do teste: depois de um tempo diz a rolagem e o fragmento da página (para provar a restauração mesmo se o IntersectionObserver não avisar)
// o mesmo, com history.replaceState quebrado de propósito: a página tem de cair em location.replace sem perder nada
const SEM_REPLACESTATE = `<script>history.replaceState = function () { throw new Error("bloqueado de propósito"); };</script>`;
const SONDA = `<script>setTimeout(function(){parent.postMessage({tipo:"sonda",y:Math.round(scrollY),hash:location.hash,io:typeof IntersectionObserver},"*")},2500)</script>`;
const base0 = montar({ conteudo, figuras }), fim = base0.lastIndexOf("</body>"); // o ÚLTIMO </body>: o kit tem outro dentro de um texto
const pagina = base0.slice(0, fim) + SONDA + base0.slice(fim + 7);
const PAI = `<!doctype html><meta charset="utf-8"><body style="margin:0"><iframe id="f" sandbox="allow-scripts" style="width:1400px;height:850px;border:0"></iframe><pre id="estado">[]</pre>
<script>
const msgs = [], q = new URLSearchParams(location.search), f = document.getElementById("f");
addEventListener("message", (e) => { msgs.push(e.data); document.getElementById("estado").textContent = JSON.stringify(msgs); });
f.src = (q.get("pagina") || "/p") + (q.get("h") ? "#" + q.get("h") : "");
if (q.get("envia")) f.addEventListener("load", () => setTimeout(() => { for (const m of JSON.parse(q.get("envia"))) f.contentWindow.postMessage(m, "*"); }, 1200));
</script>`;
const CSP = "sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:";
const servidor = createServer((req, res) => {
  const url = req.url.split("?")[0];
  if (url === "/p-sem-replacestate") { res.writeHead(200, { "content-type": "text/html; charset=utf-8", "content-security-policy": CSP }); res.end(SEM_REPLACESTATE + pagina); }
  else if (url === "/p") { res.writeHead(200, { "content-type": "text/html; charset=utf-8", "content-security-policy": CSP }); res.end(pagina); }
  else if (url === "/pai") { res.writeHead(200, { "content-type": "text/html; charset=utf-8" }); res.end(PAI); }
  else { res.writeHead(404); res.end(); }
});
await new Promise((ok) => servidor.listen(0, "127.0.0.1", ok));
const base = `http://127.0.0.1:${servidor.address().port}`;

// abre o pai no Edge (DevTools), espera em tempo REAL (o tempo virtual do --dump-dom não alcança o iframe isolado) e lê o que o pai guardou.
// spawn assíncrono: o servidor deste teste roda no mesmo processo e precisa continuar respondendo.
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const portaLivre = () => new Promise((ok) => { const l = livre(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const mensagens = async (consulta) => {
  const porta = await portaLivre(), perfil = mkdtempSync(join(tmpdir(), "ap-frag-"));
  const filho = spawn(navegador, ["--headless=new", "--disable-gpu", "--no-first-run", "--window-size=1440,900", `--remote-debugging-port=${porta}`, `--user-data-dir=${perfil}`, `${base}/pai?${consulta}`], { stdio: "ignore" });
  try {
    let alvo = null;
    for (let i = 0; i < 60 && !alvo; i++) { await dormir(250); try { alvo = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((t) => t.type === "page" && t.url.includes("/pai")); } catch { /* ainda subindo */ } }
    assert.ok(alvo, "o navegador não abriu a página de teste");
    await dormir(5000); // a restauração (350 ms), o aviso (150 ms), a sonda (2,5 s) e a mensagem do pai (1,2 s) cabem com folga
    const ws = new WebSocket(alvo.webSocketDebuggerUrl);
    await new Promise((ok, falha) => { ws.onopen = ok; ws.onerror = () => falha(new Error("DevTools não conectou")); });
    const texto = await new Promise((ok) => { ws.onmessage = (e) => ok(JSON.parse(e.data).result.result.value); ws.send(JSON.stringify({ id: 1, method: "Runtime.evaluate", params: { expression: 'document.getElementById("estado").textContent', returnByValue: true } })); });
    ws.close();
    return JSON.parse(texto).filter((x) => x && (x.tipo === "ap-estado" || x.tipo === "sonda"));
  } finally { filho.kill(); setTimeout(() => { try { rmSync(perfil, { recursive: true, force: true }); } catch (e) { /* o Edge ainda segura o perfil: é pasta temporária */ } }, 2500); }
};
const sonda = (ms) => ms.find((m) => m.tipo === "sonda");
const estados = (ms) => ms.filter((m) => m.tipo === "ap-estado");
const ultima = (ms) => estados(ms)[estados(ms).length - 1];
const via = new Set();
try {
  let ms = await mensagens("h=etapas-3");
  assert.ok(ms.length, "a página avisou o pai da seção (o IntersectionObserver rodou no iframe sandbox)");
  assert.deepEqual([ultima(ms).secao, ultima(ms).etapa, ultima(ms).fragmento], ["etapas", 3, "etapas-3"], "abriu na 3ª etapa: " + JSON.stringify(ms));
  estados(ms).forEach((m) => via.add(m.via));
  assert.ok(sonda(ms).y > 300, "a rolagem restaurada de verdade (a sonda mede o scrollY do iframe): " + JSON.stringify(sonda(ms)));
  assert.equal(sonda(ms).hash, "#etapas-3", "o fragmento da URL do iframe");
  assert.ok(estados(ms).length && estados(ms).every((m) => m.via !== "nenhum"), "o fragmento foi gravado por replaceState ou por location.replace, nunca ficou sem gravar: " + JSON.stringify(ms));

  ms = await mensagens("h=etapas-99");
  assert.deepEqual([ultima(ms).secao, ultima(ms).etapa], ["etapas", 4], "etapa que não existe: a última (4): " + JSON.stringify(ms));

  ms = await mensagens("h=acoes");
  assert.equal(ultima(ms).secao, "decisoes", "Ações não existe no cliente: a seção mais próxima: " + JSON.stringify(ms));

  ms = await mensagens("h=banana");
  assert.ok(estados(ms).every((m) => m.secao === "visao"), "fragmento inválido: abre no começo, sem erro: " + JSON.stringify(ms));

  ms = await mensagens("envia=" + encodeURIComponent(JSON.stringify([{ tipo: "ap-ir", secao: "decisoes" }])));
  assert.equal(ultima(ms).secao, "decisoes", "ap-ir rola até a seção: " + JSON.stringify(ms));

  ms = await mensagens("envia=" + encodeURIComponent(JSON.stringify([{ tipo: "ap-ir", secao: "etapas", etapa: 2 }])));
  assert.deepEqual([ultima(ms).secao, ultima(ms).etapa], ["etapas", 2], "ap-ir com etapa: " + JSON.stringify(ms));

  // replaceState bloqueado: o fragmento é gravado por location.replace e a seção continua certa
  ms = await mensagens("pagina=/p-sem-replacestate&h=linha");
  assert.ok(estados(ms).length && estados(ms).every((m) => m.via === "replace" && m.secao === "linha"), "sem replaceState: grava por location.replace: " + JSON.stringify(ms));
  assert.equal(sonda(ms).hash, "#linha");
  via.add("replace (quando o replaceState falha)");

  // mensagens de outro formato são ignoradas: a seção continua a do começo
  ms = await mensagens("envia=" + encodeURIComponent(JSON.stringify([{ tipo: "outra", secao: "decisoes" }, { tipo: "ap-ir" }, { tipo: "ap-ir", secao: 7 }, "ap-ir", null, { tipo: "ap-ir", secao: "banana" }])));
  assert.ok(estados(ms).every((m) => m.secao === "visao"), "formato errado é ignorado: " + JSON.stringify(ms));
  console.log("fragmento-navegador.test.mjs: ok (" + navegador.split(/[\/]/).pop() + "; gravou por " + [...via].join(", ") + ")");
} finally { servidor.close(); }
