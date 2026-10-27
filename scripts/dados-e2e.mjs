// Dados à prova de perda, de ponta a ponta com o binário de verdade, servidor e banco temporários (nunca o banco vivo):
// 1) o agente enfileira uma proposta e um aviso, o processo é morto e, ao reabrir, a fila volta igual;
// 2) o item com ack não volta depois de outra queda;
// 3) o banco é apagado e `blueprint restaurar` o traz de volta de uma cópia (recusando enquanto o servidor responde).
// Uso: node scripts/dados-e2e.mjs [caminho do blueprint.exe]   (padrão: target/debug/blueprint.exe; ~5 s, sem gastar tokens)
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdir, rm } from "node:fs/promises";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { exePadrao, ligarServidor, mcpStdio, pastaDeDados, postJson } from "./lib-e2e-rust.mjs";

const exe = exePadrao();
// trava de instância própria: o `restaurar` não confunde este teste com o app do usuário aberto
const d = await pastaDeDados("blueprint-dados", { BLUEPRINT_INSTANCIA_PARA_TESTE: `dados-e2e-${process.pid}` });
const banco = d.env.BLUEPRINT_DB;
const fila = async () => (await (await fetch(d.url + "/api/mcp/fila")).json()).itens;
const matar = async (proc) => { proc.kill(); await new Promise((ok) => (proc.exitCode !== null ? ok() : proc.once("exit", ok))); };
const restaurar = (...args) => spawnSync(exe, ["restaurar", ...args], { env: d.env, encoding: "utf8", windowsHide: true });
let servidor;
try {
  servidor = await ligarServidor(exe, d);
  const mcp = mcpStdio(exe, d.env);
  await mcp.iniciar();
  const proposta = await mcp.chamar("propor_blueprint", { workspace_id: "sm", blueprint_id: "p1", mensagem: "m", acoes: [{ nome: "Nova", inicio: "2026-11-06", fim: "2026-11-07", planos: [{ texto: "Criar algo", descricao: "d" }] }] });
  const aviso = await mcp.chamar("registrar_progresso", { workspace_id: "sm", blueprint_id: "p1", acao_id: "f1", plano_id: "t1", texto: "Comecei", agente: "Claude" });
  mcp.fechar();
  assert.equal(proposta.erro, false, proposta.texto);
  assert.equal(aviso.erro, false, aviso.texto);
  const antes = await fila();
  assert.deepEqual(antes.map((i) => i.tipo), ["proposta", "progresso"]);

  // 1) queda: o processo morre sem fechar nada; a fila volta igual (ids, número, criadaEm, conteúdo)
  await matar(servidor);
  servidor = await ligarServidor(exe, d);
  assert.deepEqual(await fila(), antes, "a fila sobreviveu à queda");
  console.log("ok: proposta e aviso na fila, processo morto, reaberto com a fila igual");

  // 2) a página confirmou o aviso: depois de outra queda ele não volta; a proposta, que ninguém aplicou, continua
  assert.equal((await postJson(d.url + "/api/mcp/fila/ack", { ids: [antes[1].id] })).status, 200);
  await matar(servidor);
  servidor = await ligarServidor(exe, d);
  assert.deepEqual((await fila()).map((i) => [i.tipo, i.numero]), [["proposta", 1]], "o item com ack não voltou");
  console.log("ok: item com ack não volta depois de reabrir");

  // 3) cópia de segurança (o lugar e o nome das automáticas; quem as faz ao abrir a janela é testado no cargo test) e perda do banco
  await matar(servidor);
  const auto = join(d.dir, "backups", "automaticos");
  await mkdir(auto, { recursive: true });
  const copia = join(auto, `auto-${Date.now()}.db`);
  const db = new DatabaseSync(banco);
  db.exec(`VACUUM INTO '${copia.replaceAll("'", "''")}'`);
  db.close();
  servidor = await ligarServidor(exe, d);
  assert.equal(restaurar("1").status, 1, "com o servidor respondendo, restaurar recusa");
  await matar(servidor);
  servidor = null;
  for (const s of ["", "-wal", "-shm"]) await rm(banco + s, { force: true });
  const lista = restaurar();
  assert.equal(lista.status, 0, lista.stderr);
  assert.match(lista.stdout, /^\s+1\s+.*auto-\d+\.db/m, lista.stdout);
  const volta = restaurar("1");
  assert.equal(volta.status, 0, volta.stderr);
  servidor = await ligarServidor(exe, d);
  const dados = await (await fetch(d.url + "/api/dados")).json();
  assert.equal(dados.blueprints[0].acoes[0].planos[0].texto, "Plano um", "os dados voltaram da cópia");
  assert.deepEqual((await fila()).map((i) => [i.tipo, i.numero]), [["proposta", 1]], "a fila da cópia voltou junto");
  console.log("ok: banco apagado volta pela cópia com blueprint restaurar (e recusa com o servidor de pé)");
} finally {
  if (servidor) await matar(servidor);
  await rm(d.dir, { recursive: true, force: true }).catch(() => {});
}
