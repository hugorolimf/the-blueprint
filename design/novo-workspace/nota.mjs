// A tela da nota no cofre: o protótipo interativo num iframe, a tela de hoje para comparar, o que muda e o que falta decidir. Sem linha em branco.
const url = (caminho) => `file://${encodeURI(caminho)}`;
const lista = (itens) => itens.map((t) => `<div class="painel-bloco" data-bloco>${t}</div>`).join("");

export const nota = (destino) => [
  `<div class="painel" data-tela="automatica" data-origem="design/novo-workspace/build.mjs">`,
  `<p class="painel-resumo">A tela deixa de ser um formulário ao lado de uma caixa vazia e vira uma prancha: o que você digita aparece na hora no carimbo da prancha. O protótipo abaixo funciona: digite um nome e aperte Criar. Só o protótipo mudou; o app continua igual até você aprovar.</p>`,
  `<div class="painel-rotulo painel-secao">Protótipo: clique e digite</div>`,
  `<iframe src="${url(destino)}" title="Protótipo da tela Novo workspace" style="display:block;width:100%;aspect-ratio:1280/760;border:1px solid #5b93d0;background:#0c264b"></iframe>`,
  `<div class="painel-rotulo painel-secao">O que muda</div>`,
  `<div class="painel-blocos">`,
  lista([
    `<strong>O nome vem primeiro e grande.</strong> Ele é a decisão da tela, então é escrito direto na folha, sem caixa e sem caixa alta.`,
    `<strong>A marca sai da tela.</strong> Somem os dois seletores e a escolha de arte. Todo workspace novo nasce com a figura do Blueprint.`,
    `<strong>A figura ocupa o palco.</strong> Sem a caixa tracejada. Ao abrir a tela, ela aparece no traço e fica sólida, como no resto do app.`,
    `<strong>O carimbo mostra o resultado.</strong> O canto de prancha técnica se preenche enquanto você digita.`,
    `<strong>Menos barra.</strong> Saem a linha grossa, o «pronto» e o botão de tela cheia. Fica só voltar e o Criar como único botão forte.`,
  ]),
  `</div>`,
  `<div class="painel-rotulo painel-secao">Hoje</div>`,
  `<img src="${url(destino.replace("novo-workspace.html", "novo-workspace-hoje.png"))}" alt="A tela Novo workspace como está hoje" style="display:block;width:100%;border:1px solid #5b93d0">`,
  `</div>`,
].join("\n");
