// node scripts/migrar-vocabulario.test.mjs
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { ANTIGOS, NOVOS, contar, migrar } from "./migrar-vocabulario.mjs";

const fase = (id) => ({ id, nome: "F", inicio: "2026-01-01", fim: "2026-01-02", obs: "o", status: "planejado",
  tarefas: [{ id: "t1", texto: "Fazer", feito: true, agente: { estado: "x" } }], marcos: [{ id: "m1", nome: "M", data: "2026-01-02", feito: false }], prototipos: [{ id: "a" }] });
const antigo = { empresa: "E", rev: 3, usuario: { nome: "U" }, projetos: [{ id: "p1", nome: "P", fases: [fase("f1")] }],
  companhias: [{ id: "c1", nome: "C", projetos: [{ id: "p2", nome: "Q", fases: [fase("f2"), fase("f3")] }, { id: "p3", nome: "R" }] }], ordemWorkspaces: ["c1", "sm"],
  chats: [{ id: "x", dadosProjeto: null, faseAlvo: "", proposta: [{ nome: "F", tarefas: [{ texto: "T" }] }] }] };

const novo = migrar(antigo);
assert.deepEqual(Object.keys(novo), ["empresa", "rev", "usuario", "blueprints", "workspaces", "ordemWorkspaces", "chats"], "mantém a ordem e renomeia na raiz");
assert.deepEqual(Object.keys(novo.blueprints[0].acoes[0]), ["id", "nome", "inicio", "fim", "obs", "status", "planos", "checkpoints", "prototipos"]);
assert.deepEqual(novo.workspaces[0].blueprints[0].acoes[1].planos[0], antigo.companhias[0].projetos[0].fases[1].tarefas[0], "valores intactos");
assert.equal(novo.workspaces[0].blueprints[1].acoes, undefined, "blueprint sem ações continua sem a chave");
assert.deepEqual(contar(antigo, ANTIGOS), contar(novo, NOVOS));
assert.deepEqual(contar(novo, NOVOS), { workspaces: 2, blueprints: 3, acoes: 3, planos: 3, checkpoints: 3, feitos: 3 });
assert.deepEqual(novo.chats, [{ id: "x", dadosBlueprint: null, acaoAlvo: "", proposta: [{ nome: "F", planos: [{ texto: "T" }] }] }], "as conversas guardadas também");
assert.equal(JSON.stringify(novo).match(/"(projetos|fases|tarefas|marcos|companhias)"/), null, "nenhuma chave antiga sobra");
assert.throws(() => migrar(novo), /já migrado/, "não migra duas vezes");

// pela linha de comando: grava backup idêntico ao original e o arquivo migrado
const dir = mkdtempSync(join(tmpdir(), "migrar-")), arq = join(dir, "data.json");
writeFileSync(arq, JSON.stringify(antigo));
execFileSync(process.execPath, [fileURLToPath(new URL("./migrar-vocabulario.mjs", import.meta.url)), arq]);
const backup = readdirSync(dir).find((f) => f.includes("pre-vocabulario"));
assert.ok(backup, "backup criado");
assert.equal(readFileSync(join(dir, backup), "utf8"), JSON.stringify(antigo));
assert.deepEqual(JSON.parse(readFileSync(arq, "utf8")), novo);
console.log("ok migrar-vocabulario");
