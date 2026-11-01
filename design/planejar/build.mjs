// Gera o protótipo do Planejar novo (as páginas, a Memória e o Grafo): `planejar.html` (abre no navegador) e, com `--cofre`, a nota
// do Obsidian `Em andamento/The Blueprint - Planejar (plano e protótipo).md` + a tela em `Em andamento/Painéis/`. A nota fica em
// andamento até o Hugo dar por terminada. O CSS (bp.css) entra no trecho `.obsidian/snippets/loja-painel.css` entre marcas, e a tela
// entra na lista de telas automáticas do plugin Telas (cadeado: só este script mexe nela).
// Uso: node design/planejar/build.mjs [--cofre] [--cofre-em=<pasta do cofre>]. Dados de DEMONSTRAÇÃO.
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";
import { painel } from "./nota.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const outra = process.argv.find((a) => a.startsWith("--cofre-em="));
const COFRE = outra ? outra.slice("--cofre-em=".length) : join(homedir(), "Documents/Obsidian/Brain");
const PASTA = "Em andamento";
const NOME = "The Blueprint - Planejar (plano e protótipo)";

/** O Obsidian encerra o bloco de HTML numa linha em branco: nenhuma pode sobrar. */
const semLinhaEmBranco = (s) => s.split("\n").filter((l) => l.trim() !== "").join("\n");

const html = painel();
const css = readFileSync(join(AQUI, "bp.css"), "utf8");
const base = `body{margin:0;padding:24px;background:#0b1830;color:#c9d6e8;font:14px system-ui,sans-serif;--text-muted:#8ea4c2;--background-modifier-border:#2f4a70;--font-ui-smaller:12px}.painel{max-width:1100px;margin:0 auto}.painel-rotulo{font:11px ui-monospace,monospace;letter-spacing:.14em;text-transform:uppercase;color:#9cc4ee;margin:28px 0 8px}.painel-capa-estado{font-size:24px;color:#eaf4ff}td,th{padding:5px 8px;border-bottom:1px solid #2f4a70;text-align:left;vertical-align:top}table{border-collapse:collapse}`;
const pagina = `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>The Blueprint · Planejar</title><style>${base}${css}</style></head><body>${html}</body></html>`;
writeFileSync(join(AQUI, "planejar.html"), pagina);
console.log("planejar.html", `${(pagina.length / 1024).toFixed(0)} KB`);

if ((process.argv.includes("--cofre") || outra) && existsSync(COFRE)) {
  mkdirSync(join(COFRE, PASTA, "Painéis"), { recursive: true });
  const caminhoTela = `${PASTA}/Painéis/Tela ${NOME}.md`;
  writeFileSync(join(COFRE, caminhoTela), `<!-- Tela da nota «${NOME}», GERADA por design/planejar/build.mjs (repositório the-blueprint): não editar à mão. CSS: design/planejar/bp.css (classes bp-*) -->\n${semLinhaEmBranco(html)}\n`);
  const nota = join(COFRE, PASTA, `${NOME}.md`);
  const minhas = existsSync(nota) ? readFileSync(nota, "utf8").split("## Minhas notas")[1] ?? "\n" : "\n";
  writeFileSync(nota, `---\ncssclasses: [painel-tela]\ntags: [blueprint, prototipo, planejar]\nligadas:\n  - "[[${PASTA}/The Blueprint - interface nativa]]"\n---\n![[${PASTA}/Painéis/Tela ${NOME}]]\n\n## Minhas notas${minhas}`);
  // o CSS no trecho do cofre, entre as marcas deste protótipo (troca o bloco antigo, se houver)
  const trecho = join(COFRE, ".obsidian/snippets/loja-painel.css");
  if (existsSync(trecho)) {
    const [ini, fim] = ["/* >>> the-blueprint design/planejar/bp.css */", "/* <<< the-blueprint design/planejar/bp.css */"];
    const atual = readFileSync(trecho, "utf8"), bloco = `${ini}\n${css.trim()}\n${fim}`;
    const novo = atual.includes(ini) ? atual.slice(0, atual.indexOf(ini)) + bloco + atual.slice(atual.indexOf(fim) + fim.length) : `${atual.trimEnd()}\n\n${bloco}\n`;
    if (novo !== atual) writeFileSync(trecho, novo);
  }
  // a tela é gerada: cadeado do plugin Telas
  const lista = join(COFRE, ".obsidian/plugins/telas/automaticas.json");
  if (existsSync(lista)) {
    const j = JSON.parse(readFileSync(lista, "utf8"));
    if (!j[caminhoTela]) { j[caminhoTela] = "design/planejar/build.mjs (repositório the-blueprint)"; writeFileSync(lista, JSON.stringify(j, null, 2) + "\n"); }
  }
  console.log("nota gravada:", `${PASTA}/${NOME}.md`, "em", COFRE);
}
