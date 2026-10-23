// Trava a figura do Claude e dos subagentes: lista de peças válida e dentro do limite, SVG sem valores inválidos, determinístico, desenhado em poucos ms
// e igual à referência gravada (hash).
//   node agentes-claude.test.mjs                  -> compara com referencias/claude.json
//   ATUALIZAR_REF=1 node agentes-claude.test.mjs  -> regrava a referência (só quando o desenho MUDA de propósito)
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { join } from "node:path";
import { desenhaEmblema, sanitizaPecas } from "./emblemas.mjs";
import { pecasDoClaude, poseDoEstado, descricaoDoClaude, POSES, PAPEIS, GRUPOS, QUADROS, SEQUENCIA, RITMO, OPCOES, LIMITE, ESCALA, MAX_SUBAGENTES } from "./agentes-claude.mjs";

const ARQ = join(import.meta.dirname, "referencias", "claude.json");
const desenha = (args) => desenhaEmblema(pecasDoClaude(args), OPCOES);
const halo = (svg) => svg.match(/<path class="iso-halo" d="([^"]*)"/)[1];
const COM_LIT = new Set(["trabalhando", "esperando", "bloqueado", "concluido"]); // nas outras poses não há nada para o olhar seguir

// grade: todas as poses x (Claude + 3 subagentes) x todos os quadros da animação, e entradas inválidas
// (o quadro 0 das poses que não andam mantém o nome antigo: o hash dele tem de seguir igual ao desenho aprovado)
const CASOS = {};
for (const papel of [undefined, ...PAPEIS]) for (const pose of POSES) for (let quadro = 0; quadro < QUADROS[pose]; quadro++) {
  CASOS[`${papel ?? "claude"}-${pose}${pose === "andando" || quadro ? "-" + (quadro + 1) : ""}`] = { pose, papel, quadro };
}
CASOS["lixo"] = { pose: "dançando", papel: "mago", quadro: "x" };
CASOS["vazio"] = {};

const atual = {};
let haloFixo = null;
for (const [nome, args] of Object.entries(CASOS)) {
  const pecas = pecasDoClaude(args);
  const e = desenhaEmblema(pecas, OPCOES);
  assert(e && e.svg, `${nome}: sem svg`);
  assert(!/NaN|undefined|Infinity/.test(e.svg), `${nome}: valor inválido no svg`);
  assert(!/NaN|undefined|Infinity/.test(JSON.stringify(pecas)), `${nome}: valor inválido nas peças`);
  assert.deepEqual(pecasDoClaude(args), pecas, `${nome}: a lista de peças não é determinística`);
  assert.equal(desenha(args).svg, e.svg, `${nome}: o desenho não é determinístico`);
  assert(pecas.length >= 30 && pecas.length <= LIMITE, `${nome}: ${pecas.length} peças (precisa de 30 a ${LIMITE})`);
  assert.equal(sanitizaPecas(pecas, LIMITE).length, pecas.length, `${nome}: o sanitizaPecas descartou peças (limite ${LIMITE})`);
  assert(pecas.every((p) => GRUPOS.includes(p.grupo)), `${nome}: peça sem grupo conhecido`);
  const lit = pecas.filter((p) => p.tom === "lit").length;
  const lampadaApagada = args.pose === "bloqueado" && args.quadro >= 2; // a lâmpada pisca: no quadro apagado não há peça lit
  assert.equal(lit, COM_LIT.has(args.pose) && !lampadaApagada ? 1 : 0, `${nome}: ${lit} peças lit`);
  haloFixo ??= halo(e.svg); // o enquadramento não pode mudar entre poses, quadros e papéis (a câmera fica parada)
  assert.equal(halo(e.svg), haloFixo, `${nome}: o enquadramento mudou`);
  atual[nome] = { bytes: e.svg.length, pecas: pecas.length, sha256: createHash("sha256").update(e.svg).digest("hex") };
}

// o que tem de mudar muda: cada pose, cada quadro de "andando" e cada papel geram um desenho próprio
const svgs = (f) => f.map((a) => desenha(a).svg);
const distintos = (lista, o) => assert.equal(new Set(lista).size, lista.length, o);
for (const papel of [undefined, ...PAPEIS]) {
  distintos(svgs(POSES.map((pose) => ({ pose, papel }))), `${papel ?? "claude"}: duas poses geram o mesmo SVG`);
  for (const pose of POSES) distintos(svgs(Array.from({ length: QUADROS[pose] }, (_, quadro) => ({ pose, papel, quadro }))), `${papel ?? "claude"} ${pose}: dois quadros geram o mesmo SVG`);
}
// a animação: toda pose tem sequência que só cita quadros que existem e passa por todos eles, e um ritmo de no máximo ~10 quadros por segundo
for (const pose of POSES) {
  assert(SEQUENCIA[pose].every((q) => Number.isInteger(q) && q >= 0 && q < QUADROS[pose]), `${pose}: a sequência cita um quadro que não existe`);
  assert.equal(new Set(SEQUENCIA[pose]).size, QUADROS[pose], `${pose}: a sequência não passa por todos os quadros`);
  assert(RITMO[pose] >= 100 && RITMO[pose] <= 1000, `${pose}: ritmo fora de 100 a 1000 ms`);
}
assert.deepEqual(pecasDoClaude({ pose: "parado", quadro: 9 }), pecasDoClaude({ pose: "parado", quadro: 0 }), "quadro fora do intervalo vira o quadro 0");
for (const pose of POSES) distintos(svgs([undefined, ...PAPEIS].map((papel) => ({ pose, papel }))), `${pose}: dois papéis geram o mesmo SVG`);

// subagente = o mesmo corpo do Claude + uma ferramenta + a cor de apoio do papel
const COR = { explorador: "exp", construtor: "con", revisor: "rev" };
const corpo = (a) => pecasDoClaude(a).find((p) => p.grupo === "corpo" && p.tipo === "laje" && p.w === 22);
const principal = pecasDoClaude({ pose: "parado" });
assert(!principal.some((p) => p.grupo === "ferramenta"), "o Claude não leva ferramenta");
for (const papel of PAPEIS) {
  const ps = pecasDoClaude({ pose: "parado", papel });
  assert.deepEqual(corpo({ pose: "parado", papel }), corpo({ pose: "parado" }), `${papel}: o corpo tem de ser o do Claude`);
  assert(ps.some((p) => p.grupo === "ferramenta"), `${papel}: sem ferramenta`);
  assert(ps.filter((p) => p.grupo === "ferramenta").every((p) => ["exp", "con", "rev", "aco", "papel", "olho", "hi", "lo"].includes(p.tom)), `${papel}: tom de ferramenta fora da paleta`);
  assert.equal(ps.find((p) => p.grupo === "base" && p.tipo === "laje").tom, COR[papel], `${papel}: a base não tem a cor de apoio`);
  assert(ps.some((p) => p.grupo === "ferramenta" && p.tom === COR[papel]), `${papel}: a ferramenta não usa a cor de apoio`);
}
assert(ESCALA.subagente === ESCALA.claude && MAX_SUBAGENTES === 4, "subagentes: do mesmo tamanho do Claude e no máximo 4 figuras");

// do contrato de registrar_progresso para a pose
const esperado = { lendo: "trabalhando", escrevendo: "trabalhando", testando: "trabalhando", aguardando: "esperando", bloqueado: "bloqueado", concluido: "concluido" };
for (const [estado, pose] of Object.entries(esperado)) assert.equal(poseDoEstado(estado), pose, `estado ${estado}`);
for (const estado of ["lendo", "escrevendo", "testando"]) assert.equal(poseDoEstado(estado, true), "andando", `${estado} mudando de lugar anda`);
for (const estado of ["aguardando", "bloqueado", "concluido"]) assert.equal(poseDoEstado(estado, true), esperado[estado], `${estado} não anda`);
assert(["ocioso", undefined, "xyz"].every((e) => poseDoEstado(e) === "parado"), "sem estado conhecido, o Claude fica parado");
assert(/Subagente revisor/.test(descricaoDoClaude({ pose: "bloqueado", papel: "revisor" })) && /bloqueado/.test(descricaoDoClaude({ pose: "bloqueado" })));

// desenho barato: uma vez só, em poucos ms (mediana de 20)
const mediana = (f, n = 20) => { f(); const t = []; for (let i = 0; i < n; i++) { const a = performance.now(); f(); t.push(performance.now() - a); } t.sort((x, y) => x - y); return t[n >> 1]; };
const pior = Math.max(...Object.keys(CASOS).filter((n) => n !== "lixo").map((n) => mediana(() => desenha(CASOS[n]), 5)));
const ms = mediana(() => desenha({ pose: "trabalhando", papel: "construtor" }));
assert(ms < 30 && pior < 60, `desenho lento: ${ms.toFixed(1)} ms (mediana), ${pior.toFixed(1)} ms (pior pose)`);

if (process.env.ATUALIZAR_REF === "1") {
  await mkdir(join(import.meta.dirname, "referencias"), { recursive: true });
  await writeFile(ARQ, JSON.stringify(atual, null, 2) + "\n");
  console.log("referência regravada:", ARQ);
} else {
  const ref = JSON.parse(await readFile(ARQ, "utf8"));
  assert.deepEqual(Object.keys(atual), Object.keys(ref), "os casos mudaram: regrave a referência de propósito");
  for (const nome of Object.keys(ref)) assert.deepEqual(atual[nome], ref[nome], `${nome}: o SVG mudou em relação à referência`);
  const n = Object.values(atual).map((a) => a.pecas);
  console.log(`agentes-claude.test.mjs: ${Object.keys(ref).length} casos iguais à referência; ${Math.min(...n)} a ${Math.max(...n)} peças; desenho ${ms.toFixed(1)} ms (mediana), ${pior.toFixed(1)} ms (pior pose)`);
}
