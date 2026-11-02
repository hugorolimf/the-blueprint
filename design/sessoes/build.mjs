// Gera o protótipo interativo da tela de sessões: `sessoes.html` (uma página só, com fonte e figuras embutidas) e, com `--cofre`,
// a nota `Em andamento/The Blueprint - Sessões do Claude (protótipo).md`, a tela em `Em andamento/Painéis/` e a página em `Em andamento/Anexos/`.
// As figuras saem de agentes-claude.mjs, as mesmas peças que o app desenha.
// Uso: node design/sessoes/build.mjs [--cofre] [--cofre-em=<pasta>]
import { readFileSync, writeFileSync, existsSync, mkdirSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";
import { desenhaEmblema } from "../../emblemas.mjs";
import { pecasDoClaude, OPCOES } from "../../agentes-claude.mjs";
import { nota } from "./nota.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const RAIZ = join(AQUI, "../..");
const lido = (n) => readFileSync(join(AQUI, n), "utf8");

const desenho = (o) => { const e = desenhaEmblema(pecasDoClaude({ quadro: 0, ...o }), { ...OPCOES, largura: 120, altura: 93, pad: 2 }); return `<svg class="emblema" viewBox="0 0 ${e.largura} ${e.altura}">${e.svg}</svg>`; };
const FIG = {};
for (const pose of ["parado", "trabalhando", "esperando", "concluido"]) FIG[pose] = desenho({ pose });
for (const papel of ["explorador", "construtor", "revisor"]) FIG[papel] = desenho({ pose: "trabalhando", papel });

const fonte = readFileSync(join(RAIZ, "web/fontes/CascadiaMono.woff2")).toString("base64");
const pagina = `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Sessões</title><style>@font-face{font-family:"Blueprint Mono";src:url(data:font/woff2;base64,${fonte}) format("woff2");font-weight:200 700}\n${readFileSync(join(RAIZ, "web/css/marca.css"), "utf8")}\n${lido("tela.css")}</style></head><body>${lido("tela.html")}<script>const FIG=${JSON.stringify(FIG)};\n${lido("dados.js")}\n${lido("desenho.js")}\n${lido("tela.js")}</script></body></html>`;
const saida = join(AQUI, "sessoes.html");
writeFileSync(saida, pagina);
console.log("sessoes.html", `${(pagina.length / 1024).toFixed(0)} KB`);

const outra = process.argv.find((a) => a.startsWith("--cofre-em="));
if (process.argv.includes("--cofre") || outra) {
  const COFRE = outra ? outra.slice("--cofre-em=".length) : join(homedir(), "Documents/Obsidian/Brain");
  const PASTA = "Em andamento", NOME = "The Blueprint - Sessões do Claude (protótipo)";
  mkdirSync(join(COFRE, PASTA, "Painéis"), { recursive: true });
  mkdirSync(join(COFRE, PASTA, "Anexos"), { recursive: true });
  const destino = join(COFRE, PASTA, "Anexos", "sessoes.html");
  copyFileSync(saida, destino);
  const caminhoTela = `${PASTA}/Painéis/Tela ${NOME}.md`;
  writeFileSync(join(COFRE, caminhoTela), `<!-- Tela da nota «${NOME}», GERADA por design/sessoes/build.mjs (repositório the-blueprint): não editar à mão. -->\n${nota(destino)}\n`);
  const arquivo = join(COFRE, PASTA, `${NOME}.md`);
  const minhas = existsSync(arquivo) ? readFileSync(arquivo, "utf8").split("## Minhas notas")[1] ?? "\n" : "\n";
  writeFileSync(arquivo, `---\ncssclasses: [painel-tela, prancha-larga]\ntags: [blueprint, prototipo]\nligadas:\n  - "[[The Blueprint/Pesquisa/reports/Integração de agentes no Blueprint]]"\n---\n![[${PASTA}/Painéis/Tela ${NOME}]]\n\n## Minhas notas${minhas}`);
  const lista = join(COFRE, ".obsidian/plugins/telas/automaticas.json");
  if (existsSync(lista)) {
    const j = JSON.parse(readFileSync(lista, "utf8"));
    if (!j[caminhoTela]) { j[caminhoTela] = "design/sessoes/build.mjs (repositório the-blueprint)"; writeFileSync(lista, JSON.stringify(j, null, 2) + "\n"); }
  }
  console.log("nota gravada:", `${PASTA}/${NOME}.md`, "em", COFRE);
}
