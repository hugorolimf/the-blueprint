// O que o agente registra com o app FECHADO fica em arquivo (pendentes/) e entra na fila do app quando ele abre, sem precisar do servidor de pé.
// Uso: node scripts/pendentes-e2e.mjs [caminho do blueprint.exe]   (padrão: target/debug/blueprint.exe; roda em ~3 s, sem gastar tokens)
import assert from "node:assert/strict";
import { readdir, rm } from "node:fs/promises";
import { join } from "node:path";
import { exePadrao, ligarServidor, mcpStdio, pastaDeDados, portaLivre } from "./lib-e2e-rust.mjs";

const exe = exePadrao();
const d = await pastaDeDados("blueprint-pendentes");
const fechado = `http://127.0.0.1:${await portaLivre()}`; // ninguém escuta aqui: o app está fechado
const ids = { workspace_id: "sm", blueprint_id: "p1", acao_id: "f1", plano_id: "t1" };
const pendentes = join(d.dir, "pendentes");
let servidor;
try {
  const mcp = mcpStdio(exe, { ...d.env, BLUEPRINT_URL: fechado });
  await mcp.iniciar();
  const a = await mcp.chamar("registrar_progresso", { ...ids, texto: "Comecei o plano com o app fechado", agente: "Claude" });
  const b = await mcp.chamar("marcar_concluido", { ...ids, evidencia: "cargo test: 10 passaram" });
  const ruim = await mcp.chamar("marcar_concluido", { ...ids, plano_id: "nao-existe", evidencia: "x" });
  const proposta = await mcp.chamar("propor_blueprint", { workspace_id: "sm", blueprint_id: "p1", mensagem: "m", acoes: [{ nome: "Nova", inicio: "2026-11-06", fim: "2026-11-07", planos: [{ texto: "Criar algo", descricao: "d" }] }] });
  const leitura = await mcp.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" });
  mcp.fechar();
  for (const r of [a, b, ruim, proposta]) { assert.equal(r.erro, false); assert.match(r.texto, /fechado|guardad/i, r.texto); }
  assert.equal(leitura.erro, true, "ler precisa do app: com ele fechado volta o erro de sempre, sem guardar nada");
  assert.equal((await readdir(pendentes)).filter((n) => n.endsWith(".json")).length, 4, "um arquivo por registro, e só os de escrita");
  console.log("ok: 4 registros guardados com o app fechado; a leitura continua pedindo o app");

  servidor = await ligarServidor(exe, d);
  const fila = (await (await fetch(d.url + "/api/mcp/fila")).json()).itens;
  assert.deepEqual(fila.map((i) => i.tipo), ["progresso", "concluir", "proposta"], "na ordem em que o agente registrou; o inválido não entra");
  assert.equal(fila[1].texto, "cargo test: 10 passaram");
  assert.equal(fila[1].plano, "Plano um", "validado contra os dados só quando o app abriu");
  const sobrou = await readdir(pendentes);
  assert.deepEqual(sobrou, ["recusados"], "o que entrou saiu da pasta; o recusado ficou separado");
  assert.equal((await readdir(join(pendentes, "recusados"))).length, 2, "o registro recusado e o motivo");
  console.log("ok: ao abrir, 3 registros entraram na fila e 1 foi recusado com o motivo (pendentes/recusados)");
} finally {
  servidor?.kill();
  await rm(d.dir, { recursive: true, force: true }).catch(() => {});
}
