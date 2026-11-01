// A janela do Blueprint com uma página longa dentro, em HTML (a página rola como rolaria no app): a barra de cima, a coluna da
// esquerda com as páginas do workspace e o sumário «Nesta página», e o conteúdo. Cada peça do conteúdo ganha `data-bp` com o tipo
// de bloco que ela seria no Blueprint, e o CSS (bp.css) escreve esse tipo na margem, para ver o modelo de blocos na própria página.
// As cores: o CSS troca as variáveis do Obsidian e as das figuras (--al-*) pelas do tema do app, sem tocar no HTML de origem.

/** Classe de origem (as das telas do SM PRO no cofre) → bloco do Blueprint. */
export const BLOCO_DA_CLASSE = {
  "estudo-capa": "capa", "aprende-h2": "seção", "estudo-rotulo": "rótulo", "estudo-etapas": "etapas", "estudo-lista": "lista",
  "al-quadro": "figura", "al-recortes": "recortes", "fila-linha": "decidido", "estudo-caixa": "caixa", "estudo-nota": "nota",
  "estudo-tabela": "tabela", "al-par": "lado a lado",
};

const semTags = (s) => s.replace(/<[^>]+>/g, "").replace(/\s+/g, " ").trim();

/** Marca os blocos do HTML de origem e devolve [html marcado, títulos das seções, contagem por bloco]. */
export function marcarBlocos(html) {
  const conta = {};
  const marcado = html.replace(/<(\w+) class="([a-z0-9-]+)"/g, (todo, tag, cls) => {
    const tipo = BLOCO_DA_CLASSE[cls];
    if (!tipo) return todo;
    conta[tipo] = (conta[tipo] ?? 0) + 1;
    return `<${tag} class="${cls}" data-bp="${tipo}"`;
  });
  const secoes = [...html.matchAll(/<h2 class="aprende-h2">([\s\S]*?)<\/h2>/g)].map((m) => semTags(m[1]));
  return [marcado, secoes, conta];
}

const botao = (t, cheio = false) => `<span class="bp-j-btn${cheio ? " cheio" : ""}">${t}</span>`;
const item = (t, { nivel = 0, ativo = false, num = "", sinal = "·" } = {}) =>
  `<div class="bp-j-item n${nivel}${ativo ? " ativo" : ""}"><span><i>${sinal}</i>${t}</span>${num ? `<em>${num}</em>` : ""}</div>`;

/**
 * A janela inteira. `arvore`: [[nome, {nivel, ativo, num, sinal}]...] da coluna da esquerda; `meta`: a linha de cima da página;
 * `pagina`: o HTML do conteúdo (já marcado).
 */
export function janela({ rotulo, titulo, estado, botoes, workspace, arvore, secoes, meta, pagina }) {
  return [
    `<div class="bp-janela">`,
    `<div class="bp-j-barra">${botao("← Voltar à folha")}<div class="bp-j-tit"><small>${rotulo}</small><b>${titulo}</b></div><span class="bp-j-estado">${estado}</span>${botoes.map(([t, c]) => botao(t, c)).join("")}</div>`,
    `<div class="bp-j-corpo">`,
    `<div class="bp-j-lado"><div class="bp-j-fixo">`,
    `<div class="bp-j-rot">${workspace}</div>`,
    arvore.map(([t, o]) => item(t, o)).join(""),
    `<div class="bp-j-rot bp-j-rot2">Nesta página · ${secoes.length}</div>`,
    secoes.map((s, i) => `<div class="bp-j-sum${i === 0 ? " ativo" : ""}">${s}</div>`).join(""),
    `</div></div>`,
    `<div class="bp-j-pagina"><div class="bp-j-meta">${meta}</div>${pagina}</div>`,
    `</div></div>`,
  ].join("\n");
}
