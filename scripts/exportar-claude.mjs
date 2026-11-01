// Exporta as figuras do Claude (agentes-claude.mjs) para a interface nativa: um SVG por papel e pose, no primeiro quadro, comprimido (ui/assets/claude/*.svg.gz).
// As cores de cada peça vêm de regras CSS com :has() (web/css/claude.css), que o renderizador do egui não entende; aqui entram escritas em cada peça.
// Uso: node scripts/exportar-claude.mjs
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import * as claude from "../agentes-claude.mjs";
import * as emb from "../emblemas.mjs";

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const destino = join(raiz, "ui", "assets", "claude");
mkdirSync(destino, { recursive: true });

// o tema base do emblema (as regras `.emblema …` de marca.css, sem :has) mais o traço do «olho»
const css = readFileSync(join(raiz, "web", "css", "marca.css"), "utf8");
const variaveis = Object.fromEntries([...css.match(/\.emblema\{([^}]*)\}/)[1].matchAll(/(--anatomy-[\w-]+):([^;]+)/g)].map((m) => [m[1], m[2].trim()]));
const regras = [...css.matchAll(/^(\.emblema [^{]+)\{([^}]*)\}$/gm)]
  .filter((m) => !m[1].includes(":has") && !m[1].includes("fantasma") && !m[2].includes("non-scaling"))
  .map((m) => `${m[1].replace(/\.emblema /g, "")}{${m[2].replace(/var\((--anatomy-[\w-]+)\)/g, (_, v) => variaveis[v]).replace(/transform:[^;]+;?/g, "")}}`)
  .join("\n") + '\n.iso-line[data-tone="olho"]{stroke:#4a2f22;stroke-width:.7px}';

// as cores de cada tom do Claude (as variáveis --k0..--kc de claude.css): sombras 0-3, topo, aresta e vinco
const TONS = {
  claude: ["#a8472a", "#bd5733", "#cf6a45", "#d97757", "#f0a07f", "#ffc9b0", "#8e3a21"],
  olho: ["#150e0b", "#1c130f", "#241812", "#2c1f18", "#3a2a22", "#6b5246", "#150e0b"],
  exp: ["#1c7686", "#278a9c", "#32a1b5", "#40b8cd", "#78e0f0", "#bdf4fb", "#145966"],
  con: ["#926212", "#ad761b", "#c98c27", "#e0a238", "#ffd47d", "#ffe8b4", "#74500d"],
  rev: ["#2a8041", "#379653", "#47ae64", "#5ac677", "#9eeeb4", "#cdf8da", "#1f6532"],
  aco: ["#51667c", "#617890", "#748ca4", "#889fb6", "#b9c9d9", "#e1ebf5", "#3d4f62"],
};
/** Pinta cada bloco `<g class="iso-solid">` com a paleta do tom da aresta dele (se for um tom do Claude). */
function colorir(svg) {
  return svg.replace(/<g class="iso-solid">.*?<\/g>/gs, (bloco) => {
    const tom = /class="iso-line iso-edge" data-tone="(\w+)"/.exec(bloco)?.[1];
    const k = TONS[tom];
    if (!k) return bloco;
    return bloco
      .replace(/class="iso-shade" data-shade="(\d)"/g, (_, n) => `class="iso-shade" data-shade="${n}" style="fill:${k[+n]}"`)
      .replace(/class="iso-top"/g, `class="iso-top" style="fill:${k[4]}"`)
      .replace(/class="iso-line iso-edge"/g, `class="iso-line iso-edge" style="stroke:${k[5]}"`)
      .replace(/class="iso-line iso-(crease|bevel)"/g, (m) => `${m} style="stroke:${k[6]}"`);
  });
}

let total = 0;
for (const papel of [null, ...claude.PAPEIS]) {
  for (const pose of claude.POSES) {
    const pecas = claude.pecasDoClaude({ pose, papel, quadro: 0 });
    const e = emb.desenhaEmblema(pecas, { ...claude.OPCOES, largura: 220, altura: 170, pad: 6 });
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}"><style>${regras}</style>${colorir(e.svg).replace(/class="iso-halo"/g, 'class="iso-halo" transform="translate(0 3)"')}</svg>`;
    writeFileSync(join(destino, `${papel ?? "principal"}-${pose}.svg.gz`), gzipSync(svg, { level: 9 }));
    total++;
  }
}
console.log(`${total} figuras em ${destino}`);
