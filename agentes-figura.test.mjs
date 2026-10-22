// Trava a figura do agente: lista de peças válida, SVG sem valores inválidos, determinístico, e igual à referência gravada (hash).
//   node agentes-figura.test.mjs                  -> compara com referencias/agentes.json
//   ATUALIZAR_REF=1 node agentes-figura.test.mjs  -> regrava a referência (só quando o desenho MUDA de propósito)
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { join } from "node:path";
import { desenhaEmblema, sanitizaPecas } from "./emblemas.mjs";
import { pecasDoAgente, descricaoDoAgente, ESTADOS, PAPEIS, GRUPOS, MOVEIS, REDESENHADOS } from "./agentes-figura.mjs";

const ARQ = join(import.meta.dirname, "referencias", "agentes.json");
const LIMITE = 400; // a figura é nossa, não vem de fora: pede mais peças que as 160 do que a IA propõe (decisão da ação 3.7)
const OPCOES = { progresso: 1, sombra: true, limite: LIMITE };
const desenha = (args) => desenhaEmblema(pecasDoAgente(args), OPCOES);
const halo = (svg) => svg.match(/<path class="iso-halo" d="([^"]*)"/)[1];

// grade: todos os estados x os 3 papéis, os extremos de posição e progresso, e entradas inválidas
const CASOS = {};
for (const papel of PAPEIS) for (const estado of ESTADOS) CASOS[`${papel}-${estado}`] = { papel, estado, posicao: 0.5, progresso: 0.6 };
for (const estado of ESTADOS) {
  CASOS[`extremo-0-${estado}`] = { papel: "construtor", estado, posicao: 0, progresso: 0 };
  CASOS[`extremo-1-${estado}`] = { papel: "revisor", estado, posicao: 1, progresso: 1 };
}
CASOS["fora-dos-limites"] = { papel: "explorador", estado: "lendo", posicao: 7, progresso: -3 };
CASOS["lixo"] = { papel: "mago", estado: "dançando", posicao: NaN, progresso: "abc" };
CASOS["vazio"] = {};

const atual = {};
let haloFixo = null;
for (const [nome, args] of Object.entries(CASOS)) {
  const pecas = pecasDoAgente(args);
  const e = desenhaEmblema(pecas, OPCOES);
  assert(e && e.svg, `${nome}: sem svg`);
  assert(!/NaN|undefined|Infinity/.test(e.svg), `${nome}: valor inválido no svg`);
  assert(!/NaN|undefined|Infinity/.test(JSON.stringify(pecas)), `${nome}: valor inválido nas peças`);
  assert.deepEqual(pecasDoAgente(args), pecas, `${nome}: a lista de peças não é determinística`);
  assert.equal(desenha(args).svg, e.svg, `${nome}: o desenho não é determinístico`);
  assert(pecas.length >= 40 && pecas.length <= LIMITE, `${nome}: ${pecas.length} peças (precisa de 40 a ${LIMITE})`);
  assert.equal(sanitizaPecas(pecas, LIMITE).length, pecas.length, `${nome}: o sanitizaPecas descartou peças (limite ${LIMITE})`);
  assert(pecas.every((p) => GRUPOS.includes(p.grupo)), `${nome}: peça sem grupo conhecido`);
  assert.equal(pecas.filter((p) => p.tom === "lit").length, 1, `${nome}: tem de haver exatamente uma peça lit`);
  haloFixo ??= halo(e.svg); // o enquadramento (halo segue a caixa) não pode mudar entre estados, papéis e posições
  assert.equal(halo(e.svg), haloFixo, `${nome}: o enquadramento mudou`);
  atual[nome] = { bytes: e.svg.length, pecas: pecas.length, sha256: createHash("sha256").update(e.svg).digest("hex") };
}

// o que tem de mudar muda, e o que tem de ficar parado fica
const svgs = (f) => f.map((a) => desenha(a).svg);
const distintos = (lista, o) => assert.equal(new Set(lista).size, lista.length, o);
distintos(svgs(ESTADOS.map((estado) => ({ papel: "construtor", estado, posicao: 0.5, progresso: 0.6 }))), "dois estados geram o mesmo SVG");
distintos(svgs(PAPEIS.map((papel) => ({ papel, estado: "lendo", posicao: 0.5, progresso: 0.6 }))), "dois papéis geram o mesmo SVG");
distintos(svgs([0, 0.25, 0.5, 1].map((posicao) => ({ papel: "construtor", estado: "lendo", posicao, progresso: 0.6 }))), "a posição não move o carro");
distintos(svgs([0, 0.3, 0.7, 1].map((progresso) => ({ papel: "construtor", estado: "testando", posicao: 0.5, progresso }))), "o progresso não mexe no mostrador");
assert.equal(desenha({ papel: "revisor", estado: "ocioso", posicao: 1 }).svg, desenha({ papel: "revisor", estado: "ocioso", posicao: 0 }).svg, "parado, o carro tem de ficar recuado");
for (const papel of PAPEIS) for (const estado of ESTADOS) {
  const parado = (a) => pecasDoAgente({ papel, estado, posicao: a, progresso: 0.6 }).filter((p) => !MOVEIS.has(p.grupo) && !REDESENHADOS.has(p.grupo));
  assert.deepEqual(parado(0.2), parado(0.9), `${papel}/${estado}: peça parada mexeu com a posição`);
}
assert(/Revisor/.test(descricaoDoAgente({ papel: "revisor", estado: "bloqueado" })) && /Bloqueado/.test(descricaoDoAgente({ papel: "revisor", estado: "bloqueado" })));

if (process.env.ATUALIZAR_REF === "1") {
  await mkdir(join(import.meta.dirname, "referencias"), { recursive: true });
  await writeFile(ARQ, JSON.stringify(atual, null, 2) + "\n");
  console.log("referência regravada:", ARQ);
} else {
  const ref = JSON.parse(await readFile(ARQ, "utf8"));
  assert.deepEqual(Object.keys(atual), Object.keys(ref), "os casos mudaram: regrave a referência de propósito");
  for (const nome of Object.keys(ref)) assert.deepEqual(atual[nome], ref[nome], `${nome}: o SVG mudou em relação à referência`);
  console.log(`agentes-figura.test.mjs: ${Object.keys(ref).length} casos iguais à referência`);
}
