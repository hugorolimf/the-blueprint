// Mede o custo da figura do agente (tamanho do SVG e tempo de desenho) para decidir como passar do limite de 160 peças (ação 3.7).
// Compara: a figura inteira com o limite subido para 400 (o que adotamos) e cortada em 160 peças (o limite antigo), nos tamanhos de cartão e de captura.
// Desenhar "direto com o kit" chama exatamente as mesmas funções do kit que o desenhaEmblema chama, então o peso do SVG seria o mesmo: a diferença
// é só de organização do código (perde a figura-como-dados que serve qualquer número de agentes), por isso não entra como linha de medição.
// Uso: node scripts/medir-figura.mjs   Grava referencias/medicao-figura.json
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { desenhaEmblema } from "../emblemas.mjs";
import { pecasDoAgente } from "../agentes-figura.mjs";

const raiz = join(import.meta.dirname, "..");
const pose = { papel: "construtor", estado: "escrevendo", posicao: 0.5, progresso: 0.6 };
const todas = pecasDoAgente(pose);
const mediana = (f, n = 40) => { f(); const t = []; for (let i = 0; i < n; i++) { const a = performance.now(); f(); t.push(performance.now() - a); } t.sort((x, y) => x - y); return Math.round(t[n >> 1] * 10) / 10; };
const nos = (svg) => (svg.match(/<(path|circle|text)\b/g) || []).length;
const linhas = [];
for (const [rotulo, pecas, limite] of [["160 peças (limite antigo, figura cortada)", todas.slice(0, 160), 160], [`${todas.length} peças (figura inteira, limite 400)`, todas, 400]]) {
  for (const [tam, largura, altura] of [["cartão 220×170", 220, 170], ["captura 1200×930", 1200, 930]]) {
    const r = desenhaEmblema(pecas, { largura, altura, progresso: 1, sombra: false, limite });
    linhas.push({ alternativa: rotulo, tamanho: tam, pecas: r.total, svg_kb: Math.round(r.svg.length / 1024), nos_svg: nos(r.svg), desenho_ms: mediana(() => desenhaEmblema(pecas, { largura, altura, progresso: 1, sombra: false, limite })) });
  }
}
const doze = (linhas.at(-2).svg_kb * 12) / 1024;
const res = { data: new Date().toISOString(), comando: "node scripts/medir-figura.mjs", pose, linhas, doze_figuras_do_cartao_mb: Math.round(doze * 10) / 10,
  decisao: "Subir o limite por opção (limite, até 400) só para a figura do agente: o padrão de 160 continua valendo para o que vem de fora (IA, proposta). A figura segue sendo só dados." };
writeFileSync(join(raiz, "referencias", "medicao-figura.json"), JSON.stringify(res, null, 2) + "\n");
console.table(linhas);
console.log(`12 figuras do cartão = ${res.doze_figuras_do_cartao_mb} MB de SVG. Gravado em referencias/medicao-figura.json`);
