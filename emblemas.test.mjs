// Trava o desenho do kit do anatomy: o SVG de cada caso tem de bater com a referência gravada (hash).
//   node emblemas.test.mjs                  -> compara com referencias/emblemas.json
//   ATUALIZAR_REF=1 node emblemas.test.mjs  -> regrava a referência (só quando o desenho MUDA de propósito)
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { join } from "node:path";
import { BLUEPRINT, desenhaEmblema } from "./emblemas.mjs";

const ARQ = join(import.meta.dirname, "referencias", "emblemas.json");

// uma peça de cada tipo, para exercitar também o lathe (cilindro, serrilha, roda, aro, anel)
const TODOS_OS_TIPOS = [
  { tipo: "laje", x: -60, y: -40, w: 120, d: 80, z: 0, h: 12, r: 8, tom: "hi" },
  { tipo: "cilindro", cx: -20, cy: 0, r: 22, z: 12, h: 18, borda: 2, tom: "mid" },
  { tipo: "prisma", pontos: [[10, -20], [50, -20], [40, 20], [5, 15]], z: 12, h: 14, tom: "lo" },
  { tipo: "anel", cx: 30, cy: 0, r: 14, z: 26 },
  { tipo: "serrilha", cx: -20, cy: 0, r: 24, z: 30, h: 6, n: 24 },
  { tipo: "linha", x: -50, y: -30, x2: 50, y2: 30, z: 12 },
  { tipo: "risco", x: -50, y: 30, x2: 50, y2: -30, z: 12, z2: 40 },
  { tipo: "pontos", pts: [[-30, -30, 12], [0, 0, 12], [30, 30, 12]] },
  { tipo: "texto", x: -10, y: 10, z: 12, texto: "SM", tam: 12 },
  { tipo: "roda", cx: 20, cy: 10, z: 40, r: 18, w: 10, eixo: "y" },
  { tipo: "aro", cx: -30, cy: -10, z: 40, r: 8, eixo: "x" },
];

const CASOS = {
  "blueprint": () => desenhaEmblema(BLUEPRINT),
  "blueprint-metade": () => desenhaEmblema(BLUEPRINT, { progresso: 0.5 }),
  "blueprint-sem-sombra-azimute-30": () => desenhaEmblema(BLUEPRINT, { sombra: false, azimute: 30, largura: 260, altura: 200, pad: 2 }),
  "todos-os-tipos": () => desenhaEmblema(TODOS_OS_TIPOS),
  "todos-os-tipos-parcial": () => desenhaEmblema(TODOS_OS_TIPOS, { progresso: 0.4, azimute: 60 }),
};

const atual = {};
for (const [nome, desenha] of Object.entries(CASOS)) {
  const e = desenha();
  assert(e && e.svg, `${nome}: sem svg`);
  assert(!/NaN|undefined|Infinity/.test(e.svg), `${nome}: valor inválido no svg`);
  assert.equal(desenha().svg, e.svg, `${nome}: o desenho não é determinístico`);
  atual[nome] = { bytes: e.svg.length, sha256: createHash("sha256").update(e.svg).digest("hex"), feitas: e.feitas, total: e.total };
}

if (process.env.ATUALIZAR_REF === "1") {
  await mkdir(join(import.meta.dirname, "referencias"), { recursive: true });
  await writeFile(ARQ, JSON.stringify(atual, null, 2) + "\n");
  console.log("referência regravada:", ARQ);
} else {
  const ref = JSON.parse(await readFile(ARQ, "utf8"));
  assert.deepEqual(Object.keys(atual), Object.keys(ref), "os casos mudaram: regrave a referência de propósito");
  for (const nome of Object.keys(ref)) assert.deepEqual(atual[nome], ref[nome], `${nome}: o SVG mudou em relação à referência`);
  console.log(`emblemas.test.mjs: ${Object.keys(ref).length} casos iguais à referência`);
}
