// O texto da nota «The Blueprint - Planejar (plano e protótipo)»: a ideia, como os planos grandes entram, os blocos, figuras e
// imagens, o MCP, as etapas e as perguntas. As telas vêm de tela-*.mjs e alertas.mjs, e a figura do caminho de fluxo.mjs.

export const CAPA = {
  rotulo: "Plano e protótipo · 01/11/2026 · versão 2",
  titulo: "Planos do tamanho dos do SM PRO, dentro do Blueprint",
  texto: "O Blueprint vira o seu Obsidian de desenvolvimento. Cada coisa tem uma <b>página</b>: o workspace, cada blueprint e cada ação. A página aguenta um plano inteiro do jeito que você já tem no cofre: dezenas de figuras, telas em tamanho real, recortes, tabelas, decisões, etapas e imagens. O plano do agente (o Planejar) é uma dessas páginas, e chegam junto a <b>Memória</b> e o <b>Grafo</b>. Abaixo, o plano de Alertas do SM PRO aberto dentro da janela do app, com o mesmo conteúdo e no tema dele.",
};

export const CINCO = [
  ["A página é feita de blocos", "Os mesmos que os seus planos já usam: capa, seção, texto, rótulo, lista, etapas, tabela, figura, recortes, lado a lado, decidido, nota, caixa, pergunta e imagem. O app desenha cada um no tema dele."],
  ["Figuras entram como estão", "As telas e os desenhos do anatomy são SVG, como no cofre. O app já desenha SVG sem navegador (é assim que ele mostra os emblemas). As figuras pintam por nome de cor, e o app troca esses nomes pelas cores do tema: o mesmo desenho do SM PRO fica azul no Blueprint."],
  ["Imagens também", "Prints e fotos ficam guardados junto da página, nos dados do Blueprint, com limite de tamanho por imagem."],
  ["O mesmo gerador escreve nos dois lugares", "Os <code>design/*/build.mjs</code> do SM PRO ganham <code>--blueprint</code>: o plano sai no cofre e no Blueprint ao mesmo tempo. Os planos que já existem entram por um importador que lê a nota do cofre e separa em blocos."],
  ["O Planejar, a Memória e o Grafo moram nas páginas", "A proposta do agente é uma página com perguntas e «Aplicar na folha»; a memória e as ligações aparecem do lado de cada página."],
];

export const LEGENDA_ALERTAS = [
  ["O conteúdo", "é o HTML que o <code>design/alertas/nota.mjs</code> do SM PRO gera hoje para o cofre, sem nenhuma mudança. Só ganhou a janela e o tema do app."],
  ["As figuras", "são as mesmas; o que mudou foram as cores, trocadas pelo nome (a «linha», o «texto», o «vermelho»…) pelas do tema azul. No app nativo é o mesmo truque, feito na hora de desenhar."],
  ["Na margem", "o tipo de bloco de cada peça (capa, seção, figura, recortes, decidido…), que o app mostraria ao passar o mouse para mover, editar ou apagar."],
  ["À esquerda", "as páginas do workspace SM PRO e o sumário «Nesta página», que acompanha a rolagem."],
];

export const BLOCOS = [
  ["Capa", "rótulo, título e resumo; o estado da página (esperando você, aplicada)", "estudo-capa"],
  ["Seção", "título numerado com a linha embaixo; vira item do sumário", "aprende-h2"],
  ["Texto e rótulo", "parágrafo, com negrito, código, ligação e marca (etapa 2, suposição)", "p · estudo-rotulo · data-marca"],
  ["Lista e etapas", "lista simples e a linha de etapas numerada", "estudo-lista · estudo-etapas"],
  ["Tabela", "linhas e colunas", "estudo-tabela"],
  ["Figura", "um SVG grande com a legenda embaixo (telas inteiras, mapas, desenhos do anatomy)", "al-quadro"],
  ["Recortes e lado a lado", "vários SVG em grade, cada um com o texto em cima", "al-recortes · al-par"],
  ["Decidido", "o que você decidiu, com a data, em linhas rótulo e texto", "fila-linha"],
  ["Nota e caixa", "o aviso destacado e o quadro com borda", "estudo-nota · estudo-caixa"],
  ["Imagem", "print ou foto, guardada com a página", "novo"],
  ["Pergunta", "Pn, opções e recomendação; a resposta volta ao agente", "novo (Planejar)"],
  ["Ação proposta", "Nova, Alterada ou Sem mudança, com planos, checkpoint e datas; marcar ou desmarcar", "novo (Planejar)"],
  ["Linha do tempo", "a régua e as barras desenhadas como a folha", "novo (o app desenha)"],
];

export const COMO = [
  ["Figuras", "SVG guardado com a página, com as cores por nome (as mesmas variáveis dos protótipos do SM PRO: page, card, text, strong, muted, line, faint, red, amber, green, blue…). O app aplica o tema na hora de desenhar e guarda o desenho pronto, para rolar uma página de 80 figuras sem pesar. SVG desenhado assim não roda código nem busca nada fora."],
  ["Imagens", "PNG, JPG ou WebP guardados no banco do Blueprint, ligados à página; até 4 MB cada. O app reduz para a largura da página na hora de mostrar."],
  ["Texto", "Blocos com tipo e texto, nunca HTML solto: negrito, código, ligação [[ ]] e marca são as únicas formatações. Por isso o app desenha tudo no tema e nada que o agente escreva roda na janela."],
  ["Tamanho", "Sem limite de seções; cada figura até 2 MB. O plano de Alertas, o maior que você tem, tem 54 figuras (82 desenhos, contando os de dentro) e cerca de 570 KB."],
];

export const MCP = [
  ["publicar_pagina", "nova", "a página inteira em blocos, com as figuras; é o que o <code>--blueprint</code> dos geradores chama"],
  ["anexar_imagem", "nova", "manda um print ou foto para uma página"],
  ["escrever_proposta · ler_respostas", "nova", "a proposta como página (substitui o formulário de propor_blueprint) e as suas respostas às perguntas"],
  ["ler_pagina · buscar", "nova", "o agente lê páginas e procura em páginas e memórias"],
  ["ler_memoria · lembrar · esquecer", "nova", "a memória do blueprint e do workspace"],
  ["registrar_progresso · marcar_concluido", "fica", "o andamento na folha, como hoje"],
];

export const ETAPAS = [
  ["Protótipo", "esta nota. Você aprova ou muda antes de qualquer código."],
  ["Páginas e blocos", "a tabela de páginas e de imagens no banco, o desenho nativo de cada bloco (texto, seção, tabela, lista, etapas, decidido) e o sumário."],
  ["Figuras e imagens", "SVG com o tema do app e imagem guardada; rolar uma página de 80 figuras sem travar."],
  ["Publicar", "publicar_pagina e anexar_imagem no MCP, o <code>blueprint pagina publicar</code> na linha de comando, o <code>--blueprint</code> no gerador de Alertas e o importador das notas do cofre."],
  ["Planejar novo", "a proposta vira página: ações marcáveis, perguntas com resposta e «Aplicar na folha»."],
  ["Memória", "ler_memoria e lembrar, e a tela de cartões; o agente passa a ler antes de propor."],
  ["Grafo e ligações", "as ligações [[ ]], o painel da direita e a tela do grafo."],
  ["Escrever no app", "você edita bloco a bloco e cria páginas; busca com Ctrl+K."],
];

export const PERGUNTAS = [
  ["P1", "O formulário de hoje (a proposta campo a campo) some?", "Sim: o Planejar novo substitui. Por um tempo o MCP ainda aceita o formato antigo e o transforma em página."],
  ["P2", "Quem escreve as páginas: só o agente, ou você também cria do zero, como no Obsidian?", "Os dois. Você edita bloco a bloco, como no plugin Telas, e cria página nova pela coluna da esquerda."],
  ["P3", "A memória é do blueprint e do workspace, ou também uma sua, que vale para todos os workspaces?", "Blueprint e workspace agora; a sua, global, depois."],
  ["P4", "Os planos do SM PRO passam a morar no Blueprint, ou continuam no cofre e o Blueprint recebe uma cópia?", "Os dois por um tempo (o gerador grava nos dois); quando o Blueprint estiver bom, o cofre fica só com o link."],
  ["P5", "As figuras de outro sistema (o SM PRO) aparecem no tema do Blueprint ou nas cores originais?", "No tema do Blueprint, como no exemplo, com um botão por figura para ver as cores originais."],
  ["P6", "O grafo mostra o código por dentro (arquivos e funções, pelo graphify) ou só as pastas ligadas?", "Só as pastas agora; o código por dentro depois."],
  ["P7", "Os planos que já estão no cofre (Alertas, Relatórios, WhatsApp, Plano do sistema…) entram todos de uma vez?", "Sim, pelo importador, logo depois da etapa de figuras: são o teste de verdade da página."],
  ["P8", "O «aplicar sozinho» de hoje continua valendo para a proposta-página?", "Continua, desligado por padrão."],
];
