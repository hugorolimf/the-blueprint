// A tela da nota no cofre: o protótipo interativo num iframe, o caminho para testar, a ideia e o que falta decidir. Sem linha em branco.
const url = (caminho) => `file://${encodeURI(caminho)}`;
const lista = (itens) => itens.map((t) => `<div class="painel-bloco" data-bloco>${t}</div>`).join("");

export const nota = (destino) => [
  `<div class="painel" data-tela="automatica" data-origem="design/sessoes/build.mjs">`,
  `<p class="painel-resumo">Cada workspace vira um ambiente de trabalho: as sessões do Claude daquele trabalho, a pasta, o modelo e o que já ficou permitido. Cada pessoa usa o próprio Claude, com o próprio login, e o app só abre e acompanha. O protótipo abaixo funciona com dados de exemplo, nenhum Claude de verdade roda aqui e o app continua igual.</p>`,
  `<div class="painel-rotulo painel-secao">Protótipo: clique e digite</div>`,
  `<iframe src="${url(destino)}" title="Protótipo da tela de sessões" style="display:block;width:100%;aspect-ratio:1280/760;border:1px solid #5b93d0;background:#0c264b"></iframe>`,
  `<div class="painel-rotulo painel-secao">Teste assim</div>`,
  `<div class="painel-blocos">`,
  lista([
    `<strong>Responder.</strong> Na sessão aberta, aperte Permitir: ela volta a trabalhar e termina em «Sua vez».`,
    `<strong>Interromper com pedido aberto.</strong> Em «Site da padaria», aperte Interromper: o pedido é retirado e o cartão não fica pendurado.`,
    `<strong>Escrever durante o trabalho.</strong> Em «Tela de sessões», mande uma mensagem: ela fica «na fila» até o turno acabar.`,
    `<strong>Sair para o terminal.</strong> Aperte «No terminal»: a sessão sai do app, porque a mesma sessão não pode estar aberta em dois lugares.`,
    `<strong>Ver o ambiente.</strong> Aperte Ambiente e veja a regra entrar quando você usar «Sempre nesta pasta».`,
  ]),
  `</div>`,
  `<div class="painel-rotulo painel-secao">Como isso vira realidade</div>`,
  `<div class="painel-blocos">`,
  lista([
    `<strong>Cada sessão é um processo.</strong> O app abre o <code>claude</code> instalado pela pessoa em stream-json e guarda o ID da sessão junto da ação.`,
    `<strong>A conversa é um log do app.</strong> O que aparece vem dos eventos gravados pelo próprio Blueprint, e não dos arquivos internos do Claude.`,
    `<strong>O MCP e os hooks continuam.</strong> A folha do blueprint segue atualizada do mesmo jeito, e sessões abertas no terminal aparecem em «Fora do app».`,
    `<strong>Ainda não testado.</strong> Nada disso rodou com um Claude real. Depende dos nove testes do relatório <a class="internal-link" data-href="The Blueprint/Pesquisa/reports/Integração de agentes no Blueprint.md">Integração de agentes no Blueprint</a>.`,
  ]),
  `</div>`,
  `<div class="painel-rotulo painel-secao">O que falta você decidir</div>`,
  `<div class="painel-blocos">`,
  lista([
    `<strong>Fechar o app.</strong> As sessões continuam na bandeja ou são encerradas? Fechar de vez sempre encerra, e a conversa pode ser retomada depois.`,
    `<strong>Duas sessões na mesma pasta.</strong> Deixar, avisar, ou dar a cada uma a própria cópia de trabalho (worktree)? O protótipo deixa, sem avisar.`,
    `<strong>«Sempre nesta pasta».</strong> A regra fica no banco do app, ou em <code>.claude/settings.local.json</code> para valer também no terminal?`,
    `<strong>Sessão sem ação.</strong> Pode existir conversa solta no workspace, ou toda sessão nasce de uma ação? O protótipo permite as duas.`,
    `<strong>A TUI completa fica de fora.</strong> Seletores e telas de plugin só existem no terminal, e por isso o botão «No terminal» está em toda sessão.`,
  ]),
  `</div>`,
  `</div>`,
].join("\n");
