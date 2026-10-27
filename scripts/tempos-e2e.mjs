// Registra início e fim sem navegador e reabre o servidor com o mesmo SQLite.
import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const dir = mkdtempSync(join(tmpdir(), "blueprint-tempos-"));
const falso = terminalFalso(dir);
let servidor;
try {
  servidor = await sobeServidor(exe, dir, falso);
  const dados = await (await fetch(servidor.url + "/api/dados")).json();
  const bp = dados.blueprints.find((p) => p.acoes?.some((a) => a.planos?.some((t) => !t.feito)));
  const acao = bp.acoes.find((a) => a.planos?.some((t) => !t.feito));
  const plano = acao.planos.find((t) => !t.feito);
  const evento = async (tipo) => {
    const r = await fetch(servidor.url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo, texto: "teste", workspaceId: "sm", blueprintId: bp.id, acaoId: acao.id, planoId: plano.id }) });
    assert.equal(r.status, 200);
    return r.json();
  };
  const inicio = (await evento("progresso")).em, fim = (await evento("concluir")).em;
  await evento("progresso");
  servidor.parar();
  await dormir(500);
  servidor = await sobeServidor(exe, dir, falso);
  const depois = await (await fetch(servidor.url + "/api/dados")).json();
  const salvo = depois.blueprints.find((p) => p.id === bp.id).acoes.find((a) => a.id === acao.id).planos.find((t) => t.id === plano.id);
  assert.equal(salvo.agente.inicioEm, inicio);
  assert.equal(salvo.agente.feitoEm, fim);
  assert(!salvo.feito, "a medição não aprova a conclusão");
  console.log("tempos e2e: ok (sem página, após reinício, sem aprovar conclusão)");
} finally {
  servidor?.parar();
  setTimeout(() => { try { rmSync(dir, { recursive: true, force: true }); } catch {} }, 500);
}
