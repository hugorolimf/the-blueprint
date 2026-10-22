// Acha quais peças da figura do agente deixam ponta de linha solta: desenha cada peça do tipo dado (linha, risco, serrilha) sozinha, junto com
// as lajes, cilindros, rodas, aros e anéis, e roda o lines.mjs da skill anatomy. Depois de achar, corrija a peça (ou marque a ponta como `libre` só se for de propósito).
// Uso: node scripts/achar-pontas.mjs <tipo> [--estado escrevendo] [--papel construtor] [--posicao 0.5]
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { desenhaEmblema } from "../emblemas.mjs";
import { pecasDoAgente } from "../agentes-figura.mjs";

const raiz = resolve(import.meta.dirname, "..");
const skill = process.env.ANATOMY_DIR || join(process.env.USERPROFILE || process.env.HOME || "", ".agents", "skills", "anatomy");
const arg = (nome, padrao) => { const i = process.argv.indexOf(`--${nome}`); return i > 0 ? process.argv[i + 1] : padrao; };
const tipo = process.argv[2];
const todas = pecasDoAgente({ papel: arg("papel", "construtor"), estado: arg("estado", "escrevendo"), posicao: Number(arg("posicao", 0.5)), progresso: 0.6 });
const base = todas.filter((p) => ["laje", "cilindro", "roda", "aro", "anel"].includes(p.tipo));
const css = readFileSync(join(raiz, "web/css/marca.css"), "utf8");
mkdirSync(join(raiz, "target"), { recursive: true });
const pagina = join(raiz, "target", "uma-peca.html");
let ruins = 0;
todas.forEach((p, i) => {
  if (p.tipo !== tipo) return;
  const e = desenhaEmblema([...base, p], { largura: 880, altura: 680, pad: 3, progresso: 1, sombra: false, limite: 400 });
  writeFileSync(pagina, `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:#0c3a78}${css}svg{display:block}</style><svg class="emblema" width="${e.largura}" height="${e.altura}" viewBox="0 0 ${e.largura} ${e.altura}">${e.svg}</svg>`);
  const r = spawnSync(process.execPath, [join(skill, "scripts", "lines.mjs"), pagina], { encoding: "utf8", timeout: 60000 });
  const m = (r.stdout || "").match(/line ends: (\d+) of/);
  if (m && Number(m[1]) > 0) {
    ruins++;
    const detalhes = (r.stdout || "").split(String.fromCharCode(10)).filter((l) => /px from anything/.test(l)).map((l) => l.trim().slice(0, 100));
    console.log(`#${i} (${p.grupo}) ${m[1]} pontas soltas: ${JSON.stringify(p).slice(0, 190)}`, detalhes);
  }
});
console.log(`${tipo}: ${ruins} peças com ponta solta`);
process.exitCode = ruins ? 1 : 0;
