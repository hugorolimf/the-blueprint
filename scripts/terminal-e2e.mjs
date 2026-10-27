// O terminal do app de ponta a ponta, no Edge (sem janela) e com um programa FALSO no pseudoconsole (cmd.exe por BLUEPRINT_TERMINAL_COMANDO; nada de
// claude, nada de tokens): o xterm só carrega ao abrir, Iniciar > Terminal do app abre a aba, as teclas chegam e o eco volta, duas sessões com troca
// por teclado, F6 sai do terminal, Delete fecha com confirmação, esconder destrói as telas e a sessão segue no servidor, reabrir recebe o histórico.
// Também mede o atraso do eco, a memória do servidor por sessão e a CPU da página em repouso (números para referencias/relatorio-medicao.md).
// Uso: node scripts/terminal-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; precisa do Microsoft Edge). Captura em target/terminal.png.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dir = mkdtempSync(join(tmpdir(), "blueprint-terminal-"));
const pasta = join(dir, "projeto");
mkdirSync(pasta);
const depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dir, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
let servidor;
const limpar = () => { try { edge.kill(); } catch {} servidor?.parar(); setTimeout(() => { try { rmSync(dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
const memoriaMB = (pid) => Number(execFileSync("powershell", ["-NoProfile", "-Command", `(Get-Process -Id ${pid}).PrivateMemorySize64`], { encoding: "utf8" }).trim()) / 1048576;
try {
  servidor = await sobeServidor(exe, dir, terminalFalso(dir), { BLUEPRINT_TERMINAL_COMANDO: "cmd.exe /q /k prompt $g" });
  assert.equal((await fetch(`${servidor.url}/api/mcp/pasta/vincular`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ cwd: pasta, workspaceId: "sm", blueprintId: "bp1a2b3" }) })).status, 200);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  const espera = async (expr, msg, vezes = 80) => { for (let i = 0; i < vezes && !(await js(expr)); i++) await dormir(100); assert.equal(await js(expr), true, msg); };
  const tecla = (key, code, vk, modifiers = 0) => cdp("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: vk, modifiers }).then(() => cdp("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, modifiers }));
  const telaTexto = `[...document.querySelectorAll(".term-tela .xterm-rows")].map((r) => r.textContent).join("\\n")`;
  await cdp("Runtime.enable");
  await cdp("Performance.enable");
  await cdp("Page.navigate", { url: `${servidor.url}/#/p/bp1a2b3` });
  await espera(`!!document.querySelector('[data-obs="bp1a2b3|ac1a002"]')`, "a folha do blueprint abriu");

  // 1. nada do xterm na página até o terminal abrir
  assert.equal(await js(`typeof globalThis.Terminal`), "undefined", "o xterm não carrega com a página");
  assert.equal(await js(`performance.getEntriesByType("resource").some((e) => e.name.includes("xterm"))`), false, "nenhum pedido do xterm.js antes de abrir");
  assert.equal(await js(`document.getElementById("b-terminal")?.textContent`), "Terminal", "o botão Terminal está na barra");

  const metrica = async () => Object.fromEntries((await cdp("Performance.getMetrics")).result.metrics.map((m) => [m.name, m.value]));
  const cpuEm5s = async () => { await dormir(500); const a = await metrica(); await dormir(5000); return ((await metrica()).TaskDuration - a.TaskDuration) * 1000; };
  const cpuSemTerminal = await cpuEm5s(); // a base: a mesma página, sem o terminal

  // 2. Iniciar com o Claude > Terminal do app (o padrão do diálogo)
  await js(`document.querySelector('[data-obs="bp1a2b3|ac1a002"]').dispatchEvent(new MouseEvent("click", { bubbles: true }))`);
  await dormir(600);
  await js(`[...document.querySelectorAll("#nota-folha [data-nacao]")].find((b) => b.textContent === "Iniciar com o Claude").click()`);
  await espera(`!!document.querySelector("dialog.escolher-sessao[open]")`, "o diálogo de iniciar abre");
  assert.equal(await js(`document.querySelector('dialog.escolher-sessao input[name="sessao"]:checked').value`), "app", "o terminal do app é o padrão");
  await js(`document.querySelector('dialog.escolher-sessao [data-r="1"]').click()`);
  await espera(`!!document.querySelector("#terminal-painel:not([hidden]) .term-tela .xterm-rows")`, "o painel abre com a tela do terminal");
  assert.equal(await js(`typeof globalThis.Terminal`), "function", "o xterm carregou sob demanda");
  assert.match(await js(`document.querySelector("#terminal-painel [role=tab][aria-selected=true]").textContent`), /^1\.2 · /, "a aba tem o número e o nome da ação");
  await espera(`document.activeElement?.classList.contains("xterm-helper-textarea")`, "o foco vai para o terminal");
  assert.equal(await js(`getComputedStyle(document.querySelector(".term-tela .xterm-viewport")).backgroundColor`), await js(`(() => { const d = document.createElement("i"); d.style.color = getComputedStyle(document.documentElement).getPropertyValue("--paper"); document.body.append(d); const c = getComputedStyle(d).color; d.remove(); return c; })()`), "o fundo do terminal é o papel azul do app (--paper)");
  assert.match(await js(`getComputedStyle(document.querySelector(".term-tela .xterm-rows")).fontFamily`), /Cascadia Mono|Consolas/, "a fonte é a mono do app (--mono)");

  // 3. teclas e eco (o programa falso é o cmd)
  await cdp("Input.insertText", { text: "set /a 6*7" });
  await tecla("Enter", "Enter", 13);
  await espera(`${telaTexto}.includes("42")`, "a tecla chegou ao programa e a resposta voltou à tela");

  // 4. atraso do eco: da tecla (evento de entrada no xterm) até o caractere aparecer na tela, 20 vezes
  const atrasos = await js(`(async () => {
    const ta = document.querySelector(".term-tela .xterm-helper-textarea"), linhas = document.querySelector(".term-tela .xterm-rows"), conta = () => (linhas.textContent.match(/q/g) || []).length;
    const r = [];
    for (let i = 0; i < 20; i++) {
      const antes = conta(), t0 = performance.now();
      await new Promise((ok) => { const mo = new MutationObserver(() => { if (conta() > antes) { mo.disconnect(); ok(); } }); mo.observe(linhas, { childList: true, subtree: true, characterData: true }); ta.dispatchEvent(new InputEvent("input", { data: "q", inputType: "insertText", bubbles: true })); });
      r.push(performance.now() - t0);
      await new Promise((ok) => setTimeout(ok, 30));
    }
    return r.sort((a, b) => a - b);
  })()`);
  await tecla("Escape", "Escape", 27);

  // 5. CPU da página em repouso com o terminal aberto
  const cpuRepousoMs = await cpuEm5s();

  // 6. segunda sessão, troca por teclado, F6 sai do terminal, Delete fecha com confirmação
  const mem1 = memoriaMB(servidor.pid);
  await js(`fetch("/api/terminal/iniciar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: "sm", blueprintId: "bp1a2b3", acaoId: "ac1a002", titulo: "segunda" }) }).then((r) => r.json()).then((j) => terminalAbrirSessao(j.sessao))`);
  await espera(`document.querySelectorAll("#terminal-painel [role=tab]").length === 2`, "duas abas");
  await espera(`document.querySelector("#terminal-painel [role=tab][aria-selected=true] .t").textContent === "segunda"`, "a nova sessão fica ativa");
  const mem2 = memoriaMB(servidor.pid);
  await espera(`document.activeElement?.classList.contains("xterm-helper-textarea")`, "o foco está no terminal da segunda");
  await tecla("PageUp", "PageUp", 33, 2);
  await espera(`/^1\\.2/.test(document.querySelector("#terminal-painel [role=tab][aria-selected=true] .t").textContent)`, "Ctrl+PageUp volta para a primeira sessão");
  await espera(`${telaTexto}.includes("42")`, "a primeira sessão volta com o histórico");
  await js(`document.querySelector("#terminal-painel [data-t=dividir]").click()`);
  await espera(`document.querySelectorAll(".term-tela").length === 2 && document.querySelector("#terminal-painel [data-t=dividir]").getAttribute("aria-pressed") === "true"`, "Dividir mostra as duas sessões lado a lado");
  await js(`document.querySelector("#terminal-painel [data-t=dividir]").click()`);
  await espera(`document.querySelectorAll(".term-tela").length === 1`, "e desfazer volta a uma tela (a outra deixa de existir)");
  await js(`document.querySelector(".term-tela .xterm-helper-textarea").focus()`);
  await tecla("F6", "F6", 117);
  assert.equal(await js(`document.activeElement?.getAttribute("role")`), "tab", "F6 tira o foco do terminal e o leva às abas");
  await js(`document.querySelector(".term-tela .xterm-helper-textarea").focus()`);
  await tecla("F6", "F6", 117, 8);
  assert.equal(await js(`document.activeElement?.dataset.t`), "esconder", "Shift+F6 leva o foco às ferramentas do painel");
  await js(`document.querySelector("#terminal-painel [data-t=leitor]").click()`);
  await espera(`!!document.querySelector(".term-tela .xterm-accessibility") && document.querySelector("#terminal-painel [data-t=leitor]").getAttribute("aria-pressed") === "true"`, "Leitor de tela liga a árvore acessível do xterm");
  await js(`document.querySelector("#terminal-painel [data-t=leitor]").click()`);
  await js(`document.querySelector("#terminal-painel [role=tab][aria-selected=true]").focus()`);
  await tecla("ArrowRight", "ArrowRight", 39);
  assert.equal(await js(`document.activeElement?.querySelector(".t")?.textContent`), "segunda", "seta troca de aba");
  await tecla("Delete", "Delete", 46);
  await espera(`!!document.querySelector("dialog.confirmar[open]")`, "Delete pede confirmação antes de encerrar o processo");
  await js(`document.querySelector('dialog.confirmar [data-r="1"]').click()`);
  await espera(`document.querySelectorAll("#terminal-painel [role=tab]").length === 1`, "a sessão fechada sai das abas");
  assert.equal((await (await fetch(`${servidor.url}/api/terminal/sessoes`)).json()).sessoes.length, 1, "e sai do servidor");

  // 7. dividir mostra duas telas; esconder (Ctrl+`) destrói as telas, a sessão segue no servidor; reabrir recebe o histórico
  mkdirSync(join(raiz, "target"), { recursive: true });
  writeFileSync(join(raiz, "target", "terminal.png"), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
  await js(`document.querySelector("#terminal-painel [role=tab]").focus()`);
  await tecla("`", "Backquote", 192, 2);
  await espera(`document.getElementById("terminal-painel").hidden && !document.querySelector(".term-tela")`, "Ctrl+` esconde o painel e destrói as telas");
  assert.equal(await js(`document.activeElement?.id`), "b-terminal", "o foco volta ao botão Terminal");
  assert.equal((await (await fetch(`${servidor.url}/api/terminal/sessoes`)).json()).sessoes[0].viva, true, "a sessão segue rodando no servidor");
  await js(`document.getElementById("b-terminal").click()`);
  await espera(`${telaTexto}.includes("42")`, "reabrir recebe o histórico da sessão");
  assert.deepEqual(erros, [], "sem erro no console");

  const p = (q) => atrasos[Math.min(atrasos.length - 1, Math.floor(q * atrasos.length))].toFixed(1);
  console.log(`terminal do app (cmd falso no ConPTY): ok (sob demanda, Iniciar > Terminal do app, eco, duas sessões, Ctrl+PageUp, F6 e Shift+F6, leitor de tela, Delete com confirmação, Ctrl+\` esconde e destrói, histórico ao reabrir)`);
  console.log(`medidas: eco p50 ${p(0.5)} ms, p95 ${p(0.95)} ms (n=${atrasos.length}); CPU da página em 5 s de repouso: ${cpuSemTerminal.toFixed(1)} ms sem o terminal, ${cpuRepousoMs.toFixed(1)} ms com o terminal aberto; memória privada do servidor: ${mem1.toFixed(1)} MB com 1 sessão, ${mem2.toFixed(1)} MB com 2`);
} finally { limpar(); }
