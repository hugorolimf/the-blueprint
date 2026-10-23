// Gera os assets do mod a partir da mesma marca e CSS do app. Uso: node scripts/marca-terminal.mjs
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { BLUEPRINT, desenhaEmblema } from "../emblemas.mjs";
import { rasterizar } from "./icone/ico.mjs";
import { desenharMarca } from "./marca-pixels.mjs";

const pasta = resolve(import.meta.dirname, "../claude-plugin/plugins/blueprint-terminal/assets");
const css = readFileSync(new URL("../web/css/marca.css", import.meta.url), "utf8");
const e = desenhaEmblema(BLUEPRINT, { largura: 256, altura: 256, sombra: false });
const svg = `<svg xmlns="http://www.w3.org/2000/svg" class="emblema" width="256" height="256" viewBox="0 0 256 256"><style>${css}</style>${e.svg}</svg>`;
// O PNG (terminais com imagem) vem do emblema do app; as células de 32×16 e 16×8 são pixel art própria (marca-pixels.mjs).
const [{ png }] = await rasterizar([[svg, 256]]);
assert(png.length < 2 * 1024 * 1024, "PNG excede o limite de Image");
const rgb = (rgba, i) => rgba[i + 3] < 96 ? 0x01000000 : (rgba[i] << 16) | (rgba[i + 1] << 8) | rgba[i + 2];
const raster = ({ tam, rgba }) => {
  const cells = Buffer.alloc(tam * (tam / 2) * 12);
  for (let y = 0; y < tam / 2; y++) for (let x = 0; x < tam; x++) {
    const i = (y * tam + x) * 12, pixel = ((y * 2) * tam + x) * 4;
    const top = rgb(rgba, pixel), bottom = rgb(rgba, pixel + tam * 4);
    // Frente "default" é branca: áreas transparentes usam espaço ou bloco inferior.
    const vazio = top === 0x1000000;
    cells.writeUInt32LE(vazio ? (bottom === 0x1000000 ? 0x20 : 0x2584) : 0x2580, i);
    cells.writeUInt32LE(vazio ? bottom : top, i + 4);
    cells.writeUInt32LE(vazio ? top : bottom, i + 8);
  }
  assert.equal(cells.length, tam * (tam / 2) * 12);
  return { columns: tam, rows: tam / 2, cells: cells.toString("base64") };
};
mkdirSync(pasta, { recursive: true });
writeFileSync(resolve(pasta, "marca.png"), png);
const celulas = [desenharMarca(32), desenharMarca(16)];
for (const imagem of celulas) {
  const r = raster(imagem);
  writeFileSync(resolve(pasta, `marca-${r.columns}x${r.rows}.cells`), Buffer.from(r.cells, "base64"));
}
writeFileSync(resolve(pasta, "marca.mjs"), `// Gerado por scripts/marca-terminal.mjs; não editar.\nexport const png = ${JSON.stringify(png.toString("base64"))};\nexport const grande = ${JSON.stringify(raster(celulas[0]))};\nexport const pequena = ${JSON.stringify(raster(celulas[1]))};\n`);
console.log(`marca.png: ${png.length} bytes; Raster: 32×16 e 16×8, 12 bytes/célula`);
