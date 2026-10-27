// dist/ é gerado: este teste falha se alguém mexeu em kit/, biblioteca/*.js ou estilo/*.css sem rodar o empacotador, e confere o aviso de "arquivo gerado".
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const AQUI = dirname(fileURLToPath(import.meta.url));
const r = spawnSync(process.execPath, [join(AQUI, "empacotar.mjs"), "--conferir"], { encoding: "utf8" });
assert.equal(r.status, 0, `dist/ desatualizado: rode node apresentacao-ia/empacotar.mjs\n${r.stderr}`);

for (const nome of ["biblioteca.js", "estilo.css"]) {
  const topo = readFileSync(join(AQUI, "dist", nome), "utf8").split("\n").slice(0, 3).join("\n");
  assert.match(topo, /ARQUIVO GERADO/, `${nome}: falta o aviso de arquivo gerado no topo`);
  assert.match(topo, /limite de 350 linhas/, `${nome}: falta o motivo de passar do limite de linhas`);
}
// as fontes seguem o limite de 350 linhas
for (const nome of ["textos", "laco", "auditoria", "pecas", "secoes", "casca"]) {
  const linhas = readFileSync(join(AQUI, "biblioteca", nome + ".js"), "utf8").split("\n").length;
  assert.ok(linhas <= 350, `biblioteca/${nome}.js tem ${linhas} linhas (limite 350)`);
}
console.log("empacotar.test.mjs: ok");
