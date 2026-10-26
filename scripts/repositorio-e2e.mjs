// O repositório GitHub do blueprint de ponta a ponta (docs/repositorio-github.md), no Edge sem janela, com um git FALSO e terminais FALSOS (nada de rede,
// de Claude nem de tokens): o dono conecta o repositório em Editar (endereço ruim é recusado), o estado do acesso aparece (sem acesso, com acesso),
// Configurar projeto abre o terminal do app na pasta escolhida, o terminal externo (wt falso) recebe o prompt exato, a pasta ligada some com o botão,
// o aviso do canto da tela aparece uma vez e "Agora não" é lembrado, e desconectar tira o repositório dos dados. Também confere as rotas por HTTP.
// Uso: node scripts/repositorio-e2e.mjs [exe] [pasta-com-web]   (precisa do Microsoft Edge; serve web/ da pasta dada, por padrão a do repositório)
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { janela, espera } from "./lib-e2e-edge.mjs";
import { dormir, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const assets = resolve(process.argv[3] || raiz);
const dir = mkdtempSync(join(tmpdir(), "blueprint-repo-e2e-"));
const destino = join(dir, "projetos");
mkdirSync(destino);
const modo = (m) => writeFileSync(join(dir, "modo.txt"), m);
modo("sem");
writeFileSync(join(dir, "git-falso.cmd"), `@echo off\r\nset /p modo=<"%~dp0modo.txt"\r\nif "%modo%"=="ok" exit /b 0\r\nexit /b 128\r\n`);
const filhos = [];
let servidor;
const limpar = () => { for (const p of filhos) try { p.kill(); } catch {} servidor?.parar(); setTimeout(() => { try { rmSync(dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800); };
process.on("exit", limpar);
const falso = terminalFalso(dir);
const post = (rota, corpo, tipo = "application/json") => fetch(servidor.url + rota, { method: "POST", headers: { "content-type": tipo }, body: typeof corpo === "string" ? corpo : JSON.stringify(corpo) }).then(async (r) => ({ status: r.status, j: await r.json().catch(() => ({})) }));
try {
  servidor = await sobeServidor(exe, dir, falso, { BLUEPRINT_ASSETS_DIR: assets, BLUEPRINT_GIT_COMANDO: join(dir, "git-falso.cmd"), BLUEPRINT_TERMINAL_COMANDO: "cmd.exe /q /k prompt $g" });
  const dados = async () => (await fetch(servidor.url + "/api/dados")).json();
  const bp = (await dados()).blueprints.find((p) => p.acoes?.length);
  const ids = { workspaceId: "sm", blueprintId: bp.id };
  const repositorioGuardado = async () => (await dados()).blueprints.find((p) => p.id === bp.id).repositorio;

  // ---- as rotas por HTTP
  assert.equal((await post("/api/repositorio/verificar", ids)).j.estado, "sem-repositorio");
  assert.equal((await post("/api/repositorio/verificar", ids, "text/plain")).status, 415);
  assert.equal((await fetch(servidor.url + "/api/repositorio/verificar")).status, 405);
  assert.equal((await post("/api/repositorio/verificar", {})).j.codigo, "faltam-ids");
  assert.equal((await post("/api/repositorio/verificar", { workspaceId: "sm", blueprintId: "nao" })).status, 404);
  const sem = await post("/api/repositorio/configurar", { ...ids, pasta: destino });
  assert.deepEqual([sem.status, sem.j.codigo], [409, "sem-repositorio"]);

  // ---- o dono, na tela Editar
  const { js, erros } = await janela({ dir, url: servidor.url }, "dono", `/#/p/${bp.id}`, filhos);
  await espera(() => js(`typeof editar === "function" && typeof D !== "undefined" && !!D.blueprints?.length`), "a página não carregou", 15000);
  await js(`editar(true, { proj: ${JSON.stringify(bp.id)} })`);
  const card = `document.querySelector("[data-repositorio]")`;
  await espera(() => js(`!!${card}?.querySelector("input[data-repo]")`), "o card do repositório não apareceu");
  const preenche = (v) => js(`(() => { const c = ${card}.querySelector("input[data-repo]"); c.value = ${JSON.stringify(v)}; ${card}.querySelector('[data-ra="conectar"]').click(); })()`);
  await preenche("http://evil.com/a/b");
  assert.equal(await js(`!${card}.querySelector("[data-erro]").hidden`), true, "endereço ruim mostra o erro");
  assert.equal(await js(`D.blueprints.find((p) => p.id === ${JSON.stringify(bp.id)}).repositorio`), undefined, "e não grava nada");
  await preenche("https://github.com/dono/repo.git");
  await espera(async () => (await repositorioGuardado())?.url === "https://github.com/dono/repo", "o repositório não chegou aos dados (forma limpa, sem .git)");
  await espera(() => js(`/não tem acesso/.test(${card}.querySelector("[data-status]").textContent)`), "o estado sem acesso não apareceu");
  assert.equal(await js(`!!${card}.querySelector('[data-configurar] a[href$="/settings/access"]')`), true, "sem acesso: link dos colaboradores no GitHub");
  assert.equal(await js(`!!${card}.querySelector("[data-configurar] button")`), false, "sem acesso não há Configurar projeto");
  assert.equal((await post("/api/repositorio/verificar", ids)).j.estado, "sem-acesso");

  modo("ok");
  await js(`${card}.querySelector('[data-ra="verificar"]').click()`);
  await espera(() => js(`/Você tem acesso/.test(${card}.querySelector("[data-status]").textContent)`), "o estado com acesso não apareceu");
  await espera(() => js(`!!${card}.querySelector("[data-configurar] button")`), "com acesso e sem pasta: o botão Configurar projeto");

  // ---- Configurar projeto: pasta ruim fica no diálogo, pasta boa abre o terminal do app
  const dialogo = `document.querySelector("dialog.escolher-sessao")`;
  await js(`${card}.querySelector("[data-configurar] button").click()`);
  await espera(() => js(`!!${dialogo}`), "o diálogo Configurar projeto não abriu");
  assert.match(await js(`${dialogo}.textContent`), /github\.com\/dono\/repo/, "o diálogo mostra a URL");
  await js(`(() => { ${dialogo}.querySelector("[data-pasta]").value = "projetos"; ${dialogo}.querySelector('[data-r="1"]').click(); })()`);
  assert.equal(await js(`!${dialogo}.querySelector("[data-erro]").hidden`), true, "pasta relativa: o erro aparece e o diálogo fica");
  await js(`(() => { ${dialogo}.querySelector("[data-pasta]").value = ${JSON.stringify(destino)}; ${dialogo}.querySelector('[data-r="1"]').click(); })()`);
  await espera(() => js(`!${dialogo}`), "o diálogo não fechou");
  const sessoes = async () => (await (await fetch(servidor.url + "/api/terminal/sessoes")).json()).sessoes;
  await espera(async () => (await sessoes()).length === 1, "o terminal do app não abriu");
  const s = (await sessoes())[0];
  assert.deepEqual([s.pasta.toLowerCase(), s.titulo, s.blueprintId], [destino.toLowerCase(), "Configurar · repo", bp.id]);

  // ---- terminal externo (wt falso): o prompt exato, num argumento só
  await js(`${card}.querySelector("[data-configurar] button").click()`);
  await espera(() => js(`!!${dialogo}`), "o diálogo não reabriu");
  assert.equal(await js(`${dialogo}.querySelector("[data-pasta]").value`), destino, "a pasta escolhida da última vez fica lembrada");
  await js(`(() => { ${dialogo}.querySelector('input[name="terminal"][value="externo"]').checked = true; ${dialogo}.querySelector('[data-r="1"]').click(); })()`);
  await espera(() => falso.chamadas().length === 1, "o terminal externo não foi chamado");
  const [args] = falso.chamadas();
  assert.deepEqual([args[0], args[1].toLowerCase(), args[3], args[5]], ["-d", destino.toLowerCase(), "claude", "--canal"]);
  const prompt = args[4];
  assert.ok(prompt.includes(`git clone -- https://github.com/dono/repo "${join(destino, "repo")}"`), prompt);
  assert.ok(prompt.includes(`blueprint_id ${bp.id}`) && prompt.includes("vincular_pasta") && !prompt.includes(";"), prompt);

  // ---- pasta ligada ao blueprint: o botão some
  assert.equal((await post("/api/mcp/pasta/vincular", { cwd: join(destino, "repo"), ...ids })).status, 200);
  await js(`${card}.querySelector('[data-ra="verificar"]').click()`);
  await espera(() => js(`/Você tem acesso/.test(${card}.querySelector("[data-status]").textContent) && !${card}.querySelector("[data-configurar] button")`), "com a pasta ligada não há mais Configurar projeto");
  assert.equal((await post("/api/mcp/pasta/desvincular", { cwd: join(destino, "repo") })).status, 200);

  // ---- o aviso do canto da tela: aparece uma vez, "Agora não" é lembrado
  await js(`editar(false)`);
  await js(`location.hash = ""`);
  await dormir(300);
  await js(`location.hash = "/p/${bp.id}"`);
  await espera(() => js(`!!document.getElementById("repo-aviso")`), "o aviso Configurar projeto não apareceu", 8000);
  assert.match(await js(`document.getElementById("repo-aviso").textContent`), /repo/);
  await js(`document.querySelector('#repo-aviso [data-r="0"]').click()`);
  assert.equal(await js(`!!document.getElementById("repo-aviso")`), false);
  await js(`location.reload()`);
  await dormir(3000);
  assert.equal(await js(`!!document.getElementById("repo-aviso")`), false, "Agora não é lembrado por blueprint");

  // ---- janela da equipe (modo hospedado + IPC do app instalado): o card e o Iniciar mandam o pedido ao app no lugar das rotas locais, que a página remota não tem
  const sessaoDe = (papel, visitante = false) => `sessao = { modo: "servidor", membro: { id: "m1", nome: "Bia" }, visitante: ${visitante}, acessos: [{ papel: ${JSON.stringify(papel)}, workspaceId: "sm", blueprintId: "" }] }`;
  const f0 = bp.acoes[0];
  await js(`window.__ipc = []; window.ipc = { postMessage: (m) => window.__ipc.push(m) }; window.__sessaoLocal = sessao; ${sessaoDe("colaborador")}; editar(true, { proj: ${JSON.stringify(bp.id)} })`);
  await espera(() => js(`!!${card}?.querySelector('[data-ra="configurar-app"]')`), "na janela da equipe o card mostra Configurar projeto");
  assert.equal(await js(`!${card}.querySelector('[data-ra="verificar"]')`), true, "a página remota não chama as rotas locais");
  await js(`${card}.querySelector('[data-ra="configurar-app"]').click()`);
  const msg = async (i) => { const m = await js(`window.__ipc[${i}] || ""`); assert.ok(m.startsWith("projeto "), `mensagem ${i}: ${m}`); return JSON.parse(m.slice(8)); };
  assert.deepEqual(await msg(0), { tipo: "configurar", workspaceId: "sm", blueprintId: bp.id, nome: bp.nome, repositorio: "https://github.com/dono/repo" });
  await js(`editar(false)`);
  await js(`iniciarComClaude(D.blueprints.find((x) => x.id === ${JSON.stringify(bp.id)}), D.blueprints.find((x) => x.id === ${JSON.stringify(bp.id)}).acoes[0])`);
  const ini = await msg(1);
  assert.deepEqual([ini.tipo, ini.acaoId, ini.workspaceId, ini.blueprintId, ini.repositorio], ["iniciar", f0.id, "sm", bp.id, "https://github.com/dono/repo"]);
  assert.match(ini.numero, /^\d+(\.\d+)?$/, "o número da ação como na folha");
  assert.match(await js(`document.getElementById("avisoed").textContent + document.getElementById("ag-aviso").textContent`), /janela do Windows/, "o aviso diz para confirmar na janela do Windows");
  assert.equal(await js(`document.querySelector("dialog.escolher-sessao") === null`), true, "o diálogo de sessões do app local não abre na janela da equipe");
  await js(`${sessaoDe("supervisor")}; iniciarComClaude(D.blueprints.find((x) => x.id === ${JSON.stringify(bp.id)}), D.blueprints.find((x) => x.id === ${JSON.stringify(bp.id)}).acoes[0])`);
  assert.equal(await js(`window.__ipc.length`), 2, "supervisor só lê: nenhum pedido ao app");
  await js(`${sessaoDe("visitante", true)}; editar(true, { proj: ${JSON.stringify(bp.id)} })`);
  await espera(() => js(`!!${card}?.querySelector("a[href]")`), "visitante só vê o endereço");
  assert.equal(await js(`!${card}.querySelector('[data-ra="configurar-app"]') && !${card}.querySelector("input[data-repo]")`), true, "visitante não configura nem edita");
  await js(`editar(false); sessao = window.__sessaoLocal; delete window.ipc`);

  // ---- sem acesso a pessoa não recebe o aviso; desconectar tira o repositório dos dados
  modo("sem");
  await js(`localStorage.clear()`);
  await js(`location.reload()`);
  await dormir(3500);
  assert.equal(await js(`!!document.getElementById("repo-aviso")`), false, "sem acesso não há aviso");
  await js(`editar(true, { proj: ${JSON.stringify(bp.id)} })`);
  await espera(() => js(`!!${card}?.querySelector('[data-ra="desconectar"]')`), "o botão Desconectar não apareceu");
  await js(`${card}.querySelector('[data-ra="desconectar"]').click()`);
  await espera(async () => (await repositorioGuardado()) === undefined, "desconectar não tirou o repositório dos dados");
  assert.equal((await post("/api/repositorio/verificar", ids)).j.estado, "sem-repositorio");
  assert.deepEqual(erros, [], "nenhum erro no console da página");
  console.log("repositorio-e2e: ok");
} catch (e) {
  console.error(e);
  process.exitCode = 1;
} finally {
  limpar();
}
