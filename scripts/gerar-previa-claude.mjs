// Gera referencias/claude-previa.html: o Claude e os três subagentes, em todas as poses, lado a lado sobre o papel azul do blueprint (SVG inline, animado só com CSS, sem script).
// É para o usuário aprovar o desenho (ação "Figura do Claude e dos subagentes"). Uso: node scripts/gerar-previa-claude.mjs
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { desenhaEmblema } from "../emblemas.mjs";
import { pecasDoClaude, POSES, PAPEIS, OPCOES, ESCALA, QUADROS, SEQUENCIA, RITMO, descricaoDoClaude } from "../agentes-claude.mjs";

const raiz = join(import.meta.dirname, "..");
const css = ["marca.css", "claude.css"].map((f) => readFileSync(join(raiz, "web", "css", f), "utf8")).join("\n");
const LARG = 220, ALT = 170;
const fig = (args, largura, quadro, cls = "", rotulado = true) => {
  const e = desenhaEmblema(pecasDoClaude({ ...args, quadro }), { ...OPCOES, largura: LARG, altura: ALT, pad: 6 });
  const nome = rotulado ? `role="img" aria-label="${descricaoDoClaude(args)}"` : `aria-hidden="true"`;
  return `<svg class="emblema${cls ? " " + cls : ""}" width="${largura}" height="${Math.round((largura * ALT) / LARG)}" viewBox="0 0 ${LARG} ${ALT}" ${nome}>${e.svg}</svg>`;
};
// animação só com CSS: os quadros da pose ficam empilhados na mesma célula e cada um fica visível nos passos da SEQUENCIA em que é o seu (step-end)
const ciclo = (pose) => SEQUENCIA[pose].length * RITMO[pose];
const quadros = (pose) => Array.from({ length: QUADROS[pose] }, (_, k) => k);
const keyframes = POSES.flatMap((pose) => quadros(pose).map((k) => {
  const seq = SEQUENCIA[pose], passos = seq.map((q, i) => `${+((i / seq.length) * 100).toFixed(3)}%{visibility:${q === k ? "visible" : "hidden"}}`).join("");
  return `@keyframes q-${pose}-${k}{${passos}}.f-${pose}-${k}{animation:q-${pose}-${k} ${ciclo(pose)}ms step-end infinite}`;
})).join("");
const pilha = (args, largura) => `<div class="pilha">${quadros(args.pose).map((k) => fig(args, largura, k, `f-${args.pose}-${k}`, k === 0)).join("")}</div>`;
const legenda = (pose) => `${QUADROS[pose]} quadros · ${RITMO[pose]} ms`;
const celula = (conteudo, rotulo) => `<figure><div class="papel">${conteudo}</div><figcaption>${rotulo}</figcaption></figure>`;
const linha = (titulo, args, escala) => {
  const larg = Math.round(LARG * escala);
  return `<h2>${titulo}</h2><div class="linha">${POSES.flatMap((pose) => pose === "andando"
    ? [celula(fig({ ...args, pose }, larg, 0), "andando · quadro 1"), celula(fig({ ...args, pose }, larg, 1), "andando · quadro 2"), celula(pilha({ ...args, pose }, larg), `andando · animado<br>${legenda(pose)}`)]
    : [celula(pilha({ ...args, pose }, larg), `${pose}<br>${legenda(pose)}`)]).join("")}</div>`;
};
const pequeno = (args) => [24, 32, 48].map((px) => `<div class="papel mini">${pilha(args, px)}</div>`).join("");
const tira = (args, rotulo) => `<div class="tira"><span>${rotulo}</span>${pequeno(args)}</div>`;

const html = `<!doctype html>
<html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Prévia: o Claude na folha</title>
<style>
${css}
body{margin:0;padding:24px;background:#082b5e;color:#eaf4ff;font:12px/1.5 ui-monospace,"Cascadia Mono",Consolas,monospace}
h1{font-size:15px;letter-spacing:.08em;text-transform:uppercase;margin:0 0 4px}h2{font-size:11px;letter-spacing:.16em;text-transform:uppercase;color:#8dbaea;font-weight:400;margin:26px 0 8px}
p{margin:0;color:#8dbaea;max-width:900px}
.linha{display:flex;flex-wrap:wrap;gap:10px}
figure{margin:0;display:grid;gap:4px;justify-items:center}figcaption{font-size:10px;letter-spacing:.1em;text-transform:uppercase;color:#8dbaea}
.papel{display:flex;justify-content:center;border:1px solid #2d63a6;background:linear-gradient(#ffffff1c 1px,transparent 1px) 0 0/60px 60px,linear-gradient(90deg,#ffffff1c 1px,transparent 1px) 0 0/60px 60px,linear-gradient(#ffffff0c 1px,transparent 1px) 0 0/12px 12px,linear-gradient(90deg,#ffffff0c 1px,transparent 1px) 0 0/12px 12px,radial-gradient(ellipse at 50% 40%,#1250a0 0,#0c3a78 62%,#082b5e 100%)}
.papel svg{display:block}.mini{padding:2px}
.pilha{display:grid}.pilha>svg{grid-area:1/1}
${keyframes}
@media (prefers-reduced-motion:reduce){.pilha>svg{animation:none!important;visibility:hidden}.pilha>svg:first-child{visibility:visible}}
.tiras{display:flex;flex-wrap:wrap;gap:18px}.tira{display:flex;align-items:center;gap:8px}.tira span{width:92px;font-size:10px;letter-spacing:.1em;text-transform:uppercase;color:#8dbaea}
</style></head><body>
<h1>O Claude na folha · prévia do desenho</h1>
<p>Personagem feito com o anatomy (lista de peças, uma vez desenhado). O Claude é o corpo laranja; os subagentes são o mesmo corpo, do mesmo tamanho, com a ferramenta do papel na mão esquerda e a cor de apoio na base, na cinta e na ferramenta: explorador = lupa (ciano), construtor = martelo (âmbar), revisor = prancheta (verde). Só indicador, sem clique para mandar trabalho.</p>
${linha("Claude (agente principal) · mesmo tamanho", {}, 1)}
${PAPEIS.map((papel) => linha(`Subagente ${papel} · mesmo tamanho`, { papel }, ESCALA.subagente)).join("")}
<h2>Legibilidade em 24, 32 e 48 px de largura (parado, respirando e piscando)</h2>
<div class="tiras">${tira({ pose: "parado" }, "Claude")}${PAPEIS.map((papel) => tira({ pose: "parado", papel }, papel)).join("")}</div>
<h2>Os três papéis trabalhando, em 24, 32 e 48 px (animados)</h2>
<div class="tiras">${PAPEIS.map((papel) => tira({ pose: "trabalhando", papel }, papel)).join("")}</div>
</body></html>`;
writeFileSync(join(raiz, "referencias", "claude-previa.html"), html);
console.log(`referencias/claude-previa.html: ${(html.length / 1024).toFixed(0)} KB`);
