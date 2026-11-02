// Gera o protótipo interativo da tela «Novo workspace»: `novo-workspace.html` (uma página só, com fonte e figuras embutidas) e, com `--cofre`,
// a nota `Em andamento/The Blueprint - Novo workspace (protótipo).md`, a tela em `Em andamento/Painéis/` e a página em `Em andamento/Anexos/`.
// A figura sai de emblemas.mjs, as mesmas peças que o app desenha, em dois quadros: só o traço e pronta.
// Uso: node design/novo-workspace/build.mjs [--cofre] [--cofre-em=<pasta>]
import { readFileSync, writeFileSync, existsSync, mkdirSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";
import * as E from "../../emblemas.mjs";
import { nota } from "./nota.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const RAIZ = join(AQUI, "../..");
const lido = (n) => readFileSync(join(AQUI, n), "utf8");

const desenho = (o) => { const e = E.desenhaEmblema(E.BLUEPRINT, { largura: 660, altura: 410, pad: 14, ...o }); return `<svg class="emblema" viewBox="0 0 ${e.largura} ${e.altura}">${e.svg}</svg>`; };
const figura = { traco: desenho({ progresso: 0, sombra: false }), pronta: desenho({}) };

const fonte = readFileSync(join(RAIZ, "web/fontes/CascadiaMono.woff2")).toString("base64");
const pagina = `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Novo workspace</title><style>@font-face{font-family:"Blueprint Mono";src:url(data:font/woff2;base64,${fonte}) format("woff2");font-weight:200 700}\n${readFileSync(join(RAIZ, "web/css/marca.css"), "utf8")}\n${lido("tela.css")}</style></head><body>${lido("tela.html")}<script>const FIGURA=${JSON.stringify(figura)};\n${lido("tela.js")}</script></body></html>`;
const saida = join(AQUI, "novo-workspace.html");
writeFileSync(saida, pagina);
console.log("novo-workspace.html", `${(pagina.length / 1024).toFixed(0)} KB`);

const outra = process.argv.find((a) => a.startsWith("--cofre-em="));
if (process.argv.includes("--cofre") || outra) {
  const COFRE = outra ? outra.slice("--cofre-em=".length) : join(homedir(), "Documents/Obsidian/Brain");
  const PASTA = "Em andamento", NOME = "The Blueprint - Novo workspace (protótipo)";
  mkdirSync(join(COFRE, PASTA, "Painéis"), { recursive: true });
  mkdirSync(join(COFRE, PASTA, "Anexos"), { recursive: true });
  const destino = join(COFRE, PASTA, "Anexos", "novo-workspace.html");
  copyFileSync(saida, destino);
  const hoje = join(AQUI, "hoje.png");
  if (existsSync(hoje)) copyFileSync(hoje, join(COFRE, PASTA, "Anexos", "novo-workspace-hoje.png"));
  const caminhoTela = `${PASTA}/Painéis/Tela ${NOME}.md`;
  writeFileSync(join(COFRE, caminhoTela), `<!-- Tela da nota «${NOME}», GERADA por design/novo-workspace/build.mjs (repositório the-blueprint): não editar à mão. -->\n${nota(destino)}\n`);
  const arquivo = join(COFRE, PASTA, `${NOME}.md`);
  const minhas = existsSync(arquivo) ? readFileSync(arquivo, "utf8").split("## Minhas notas")[1] ?? "\n" : "\n";
  writeFileSync(arquivo, `---\ncssclasses: [painel-tela, prancha-larga]\ntags: [blueprint, prototipo]\nligadas:\n  - "[[${PASTA}/The Blueprint - interface nativa]]"\n---\n![[${PASTA}/Painéis/Tela ${NOME}]]\n\n## Minhas notas${minhas}`);
  // a nota é mais larga que as outras, para o protótipo caber: o tema Minimal limita pela --line-width
  const trecho = join(COFRE, ".obsidian/snippets/loja-painel.css");
  if (existsSync(trecho)) {
    const [ini, fim] = ["/* >>> the-blueprint design/novo-workspace */", "/* <<< the-blueprint design/novo-workspace */"];
    const atual = readFileSync(trecho, "utf8"), bloco = `${ini}\n.prancha-larga { --file-line-width: 1180px; --line-width: 1180px; --line-width-wide: 1180px; --max-width: 96%; }\n${fim}`;
    const novo = atual.includes(ini) ? atual.slice(0, atual.indexOf(ini)) + bloco + atual.slice(atual.indexOf(fim) + fim.length) : `${atual.trimEnd()}\n\n${bloco}\n`;
    if (novo !== atual) writeFileSync(trecho, novo);
  }
  const lista = join(COFRE, ".obsidian/plugins/telas/automaticas.json");
  if (existsSync(lista)) {
    const j = JSON.parse(readFileSync(lista, "utf8"));
    if (!j[caminhoTela]) { j[caminhoTela] = "design/novo-workspace/build.mjs (repositório the-blueprint)"; writeFileSync(lista, JSON.stringify(j, null, 2) + "\n"); }
  }
  console.log("nota gravada:", `${PASTA}/${NOME}.md`, "em", COFRE);
}
