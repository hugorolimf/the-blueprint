// O HTML da nota: as classes painel-* do cofre (trecho loja-painel.css) e as bp-* deste protótipo (bp.css). As figuras são SVG
// com as cores nos atributos, porque o Obsidian tira <style> da nota.
import { CAPA, CINCO, LEGENDA_PLANEJAR, BLOCOS, MCP, ETAPAS, PERGUNTAS } from "./plano.mjs";
import { fluxo } from "./fluxo.mjs";
import { telaPlanejar } from "./tela-planejar.mjs";
import { telaMemoria } from "./tela-memoria.mjs";
import { telaGrafo } from "./tela-grafo.mjs";

const secao = (t) => `<div class="painel-rotulo painel-secao">${t}</div>`;
const fig = (svg, leg) => `<div class="bp-quadro">${svg}</div><p class="bp-leg">${leg}</p>`;
const tabela = (cab, linhas) => `<table><thead><tr>${cab.map((c) => `<th>${c}</th>`).join("")}</tr></thead><tbody>${linhas.map((l) => `<tr>${l.map((c, i) => `<td>${i ? c : `<b>${c}</b>`}</td>`).join("")}</tr>`).join("")}</tbody></table>`;

export function painel() {
  return [
    `<div class="painel bp-nota">`,
    `<div class="painel-capa" data-tom="espera"><div class="painel-capa-topo"><div class="painel-rotulo">${CAPA.rotulo}</div></div><div class="painel-capa-estado">${CAPA.titulo}</div><p>${CAPA.texto}</p></div>`,
    secao("A ideia em cinco linhas"),
    `<ol class="bp-cinco">${CINCO.map(([t, d]) => `<li><b>${t}.</b> ${d}</li>`).join("")}</ol>`,
    secao("O caminho do plano"),
    fig(fluxo(), "O agente lê a memória e escreve a proposta como página. Você responde as perguntas e aplica; a folha recebe as ações e o agente conta o andamento por ela, como já faz hoje. A linha tracejada é o que volta para o agente."),
    secao("Tela 1 · O Planejar"),
    fig(telaPlanejar(), "A proposta nº 3 do «Monitor de consumo», escrita pelo Claude, na janela do app (1280 de largura; a página rola)."),
    `<ul class="bp-legenda">${LEGENDA_PLANEJAR.map(([t, d]) => `<li><b>${t}:</b> ${d}</li>`).join("")}</ul>`,
    secao("Tela 2 · A Memória"),
    fig(telaMemoria(), "A memória do blueprint em cartões, com o filtro por tipo. A escolhida aparece inteira à direita: o fato, o porquê, como aplicar e as ligações. Os tipos são os mesmos das memórias do Claude Code: você, como trabalhar, projeto e referência."),
    secao("Tela 3 · O Grafo"),
    fig(telaGrafo(), "O workspace inteiro ligado. A memória «Rede do 2º andar» está escolhida: as ligações dela acendem e o resto apaga. As ações tracejadas são as da proposta que ainda não foi aplicada."),
    secao("Os blocos de uma página"),
    tabela(["Bloco", "Como aparece", "Quem escreve"], BLOCOS),
    `<p class="bp-leg">O agente manda blocos com tipo e texto, nunca HTML solto: o app desenha cada bloco no tema dele, e nada que o agente escreva roda como código na janela.</p>`,
    secao("O que muda no MCP"),
    tabela(["Ferramenta", "", "Para quê"], MCP),
    secao("Etapas"),
    `<ol class="bp-etapas">${ETAPAS.map(([t, d]) => `<li><b>${t}:</b> ${d}</li>`).join("")}</ol>`,
    secao("Perguntas para você · responda no chat (ex.: «P1: sim»)"),
    tabela(["", "Pergunta", "Minha recomendação"], PERGUNTAS),
    `</div>`,
  ].join("\n");
}
