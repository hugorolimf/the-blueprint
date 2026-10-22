// Roda, de uma vez, toda a verificação da figura do agente (ação 3.8): auditoria de geometria do anatomy em todas as poses, checagem de pontas de
// linha (lines.mjs) em poses de cada estado e papel nos dois papéis de fundo (escuro e claro), e o peso de 12 figuras. Sai 1 se algo falhar.
// Uso: node scripts/verificar-figura.mjs   Grava referencias/verificacao-figura.json
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { ESTADOS, PAPEIS } from "../agentes-figura.mjs";

const raiz = resolve(import.meta.dirname, "..");
const skill = process.env.ANATOMY_DIR || join(process.env.USERPROFILE || process.env.HOME || "", ".agents", "skills", "anatomy");
const roda = (args, opcoes = {}) => spawnSync(process.execPath, args, { cwd: raiz, encoding: "utf8", timeout: 900000, ...opcoes });

const auditoria = roda(["scripts/auditar-figura.mjs", "--todas"]);
const linhas = (auditoria.stdout || "").trim().split("\n");
const poses = linhas.filter((l) => /sólidos, \d+ falhas$/.test(l));
const resultado = { data: new Date().toISOString(), comando: "node scripts/verificar-figura.mjs", auditoria: { poses: poses.length, com_falha: poses.filter((l) => !/ 0 falhas$/.test(l)).length, resumo: linhas.at(-1) } };
console.log(`auditoria: ${resultado.auditoria.poses} poses, ${resultado.auditoria.com_falha} com falha (${resultado.auditoria.resumo})`);

const casos = [];
for (const estado of ESTADOS) casos.push({ papel: "construtor", estado, posicao: estado === "ocioso" ? 0 : 0.75, tema: "escuro" });
for (const papel of PAPEIS) casos.push({ papel, estado: "lendo", posicao: 1, tema: "escuro" }, { papel, estado: "bloqueado", posicao: 0.25, tema: "claro" }, { papel, estado: "testando", posicao: 0.5, tema: "claro" });
resultado.linhas = [];
for (const c of casos) {
  const pagina = join(raiz, "target", "verificar.html");
  roda(["scripts/pagina-figura.mjs", pagina, "--papel", c.papel, "--estado", c.estado, "--posicao", String(c.posicao), "--progresso", "0.6", "--tema", c.tema]);
  const r = roda([join(skill, "scripts", "lines.mjs"), pagina]);
  const m = (r.stdout || "").match(/line ends: (\d+) of (\d+) ends on (\d+) lines/);
  const falhas = m ? Number(m[1]) : -1;
  resultado.linhas.push({ ...c, pontas_soltas: falhas, pontas: m ? Number(m[2]) : null, linhas: m ? Number(m[3]) : null });
  console.log(`pontas de linha ${c.papel}/${c.estado}/${c.posicao}/${c.tema}: ${falhas}`);
}
const peso = roda(["scripts/medir-figura.mjs"]);
resultado.peso = (peso.stdout || "").trim().split("\n").at(-2) || "";
const ok = auditoria.status === 0 && resultado.linhas.every((l) => l.pontas_soltas === 0);
resultado.passou = ok;
writeFileSync(join(raiz, "referencias", "verificacao-figura.json"), JSON.stringify(resultado, null, 2) + "\n");
console.log(ok ? "figura: auditoria e pontas de linha em zero" : "figura: FALHOU");
process.exitCode = ok ? 0 : 1;
