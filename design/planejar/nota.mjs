// O HTML da nota: as classes painel-* do cofre (trecho loja-painel.css) e as bp-* deste protótipo (bp.css e bp-janela.css). As figuras
// próprias são SVG com as cores nos atributos; o exemplo de Alertas usa as classes do SM PRO, recoloridas pelo bp-janela.css.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { CAPA, CINCO, LEGENDA_ALERTAS, BLOCOS, COMO, MCP, ETAPAS, PERGUNTAS } from "./plano.mjs";
import { fluxo } from "./fluxo.mjs";
import { telaPlanejar } from "./tela-planejar.mjs";
import { telaMemoria } from "./tela-memoria.mjs";
import { telaGrafo } from "./tela-grafo.mjs";
import { alertasNoBlueprint } from "./alertas.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const secao = (t) => `<div class="painel-rotulo painel-secao">${t}</div>`;
const fig = (svg, leg) => `<div class="bp-quadro">${svg}</div><p class="bp-leg">${leg}</p>`;
const tabela = (cab, linhas) => `<table><thead><tr>${cab.map((c) => `<th>${c}</th>`).join("")}</tr></thead><tbody>${linhas.map((l) => `<tr>${l.map((c, i) => `<td>${i ? c : `<b>${c}</b>`}</td>`).join("")}</tr>`).join("")}</tbody></table>`;
const lista = (itens, cls) => `<ul class="${cls}">${itens.map(([t, d]) => `<li><b>${t}:</b> ${d}</li>`).join("")}</ul>`;

/** O bloco de imagem de exemplo: um print da folha nativa, como ficaria guardado com a página. */
function imagem() {
  const b64 = readFileSync(join(AQUI, "print-folha.jpg")).toString("base64");
  return `<div class="bp-quadro bp-imagem"><img src="data:image/jpeg;base64,${b64}" alt="Print da folha nativa do Blueprint"></div><p class="bp-leg">Bloco de imagem: um print da folha nativa do Blueprint (01/11), guardado com a página. Na janela do app ele ocupa a largura da página e abre maior no clique.</p>`;
}

export async function painel() {
  const al = await alertasNoBlueprint();
  return [
    `<div class="painel bp-nota">`,
    `<div class="painel-capa" data-tom="espera"><div class="painel-capa-topo"><div class="painel-rotulo">${CAPA.rotulo}</div></div><div class="painel-capa-estado">${CAPA.titulo}</div><p>${CAPA.texto}</p></div>`,
    secao("A ideia em cinco linhas"),
    `<ol class="bp-cinco">${CINCO.map(([t, d]) => `<li><b>${t}.</b> ${d}</li>`).join("")}</ol>`,
    secao(`Tela 1 · Um plano de verdade dentro do Blueprint · Alertas, ${al.figuras ?? 0} figuras`),
    lista(LEGENDA_ALERTAS, "bp-legenda"),
    al.html,
    `<p class="bp-leg">Fim da página de Alertas. A barra de cima fica presa enquanto a página rola, como no app; com a janela estreita, a coluna da esquerda e as etiquetas da margem somem.</p>`,
    secao("Tela 2 · O Planejar: a proposta do agente como página"),
    fig(fluxo(), "O caminho do plano: o agente lê a memória e escreve a proposta como página; você responde as perguntas e aplica; a folha recebe as ações e o agente conta o andamento por ela. O tracejado é o que volta para o agente."),
    fig(telaPlanejar(), "A proposta nº 3 do «Monitor de consumo», escrita pelo Claude: o plano em três linhas, o que muda na folha desenhado como a folha, cada ação marcável, as perguntas P1 e P2 e um botão cheio só, «Aplicar na folha». À direita, a memória que ele usou, as ligações e um pedaço do grafo."),
    secao("Tela 3 · A Memória"),
    fig(telaMemoria(), "A memória do blueprint em cartões, com filtro por tipo (você, como trabalhar, projeto, referência, os mesmos das memórias do Claude Code). A escolhida aparece inteira à direita: o fato, o porquê, como aplicar e as ligações."),
    secao("Tela 4 · O Grafo"),
    fig(telaGrafo(), "O workspace inteiro ligado. A memória «Rede do 2º andar» está escolhida: as ligações dela acendem e o resto apaga. As ações tracejadas são as da proposta que ainda não foi aplicada."),
    secao("Os blocos de uma página"),
    tabela(["Bloco", "Como aparece", "De onde vem nos seus planos"], BLOCOS),
    imagem(),
    secao("Figuras, imagens e texto: como ficam guardados"),
    tabela(["", "Como"], COMO),
    secao("O que muda no MCP"),
    tabela(["Ferramenta", "", "Para quê"], MCP),
    `<p class="bp-leg">Na linha de comando: <code>blueprint pagina publicar plano.json</code>. Nos geradores do SM PRO: <code>node design/alertas/build.mjs --cofre --blueprint</code> grava a nota e publica a página.</p>`,
    secao("Etapas"),
    `<ol class="bp-etapas">${ETAPAS.map(([t, d]) => `<li><b>${t}:</b> ${d}</li>`).join("")}</ol>`,
    secao("Perguntas para você · responda no chat (ex.: «P1: sim»)"),
    tabela(["", "Pergunta", "Minha recomendação"], PERGUNTAS),
    `</div>`,
  ].join("\n");
}
