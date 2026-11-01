// O texto da nota «The Blueprint - Planejar (plano e protótipo)»: a ideia, as decisões a tomar, os blocos de uma página, o que muda
// no MCP e as etapas. As telas vêm de tela-*.mjs e a figura do caminho de fluxo.mjs.

export const CAPA = {
  rotulo: "Plano e protótipo · 01/11/2026 · dados de demonstração",
  titulo: "O Blueprint vira o seu Obsidian de desenvolvimento",
  texto: "O Planejar de hoje é um formulário: a proposta do agente chega como campos para conferir. No novo, cada coisa do Blueprint tem uma <b>página</b> no estilo das suas telas do Obsidian (capa, rótulos, cartões, tabelas, etapas, figuras do anatomy), e o plano do agente é uma dessas páginas. Junto vêm a <b>Memória</b> e o <b>Grafo</b>. Tudo dentro do app, no tema dele. Os nomes, salas e datas são inventados.",
};

export const CINCO = [
  ["Tudo tem página", "O workspace, cada blueprint e cada ação ganham uma página feita de blocos, desenhada pelo app nativo com o visual das suas notas."],
  ["O Planejar é uma página", "O agente escreve a proposta em blocos pelo MCP. Você lê como uma nota, responde as perguntas dele ali mesmo e aplica na folha com um botão só."],
  ["Memória", "Fatos curtos do blueprint e do workspace, no formato das memórias do Claude: o fato, o porquê e como aplicar. O agente lê ao começar e escreve quando aprende; você corrige ou apaga."],
  ["Grafo", "Blueprints, ações, páginas, memórias, decisões e pastas de código ligados. Clicar abre a página; a escolhida acende as ligações dela."],
  ["Tudo no app", "Fica nos dados do Blueprint (o SQLite de hoje), igual no Windows, no Linux e no navegador. O MCP continua sendo a porta do agente."],
];

export const LEGENDA_PLANEJAR = [
  ["À esquerda", "as páginas do workspace, como a árvore de arquivos do Obsidian. Cada blueprint tem a página dele, o Planejar, a Memória, as Decisões e as ações."],
  ["No meio", "a proposta escrita como página: o plano em três linhas, o que muda na folha desenhado como a própria folha (tracejado até você aplicar), cada ação com os planos e o checkpoint, e as perguntas do agente."],
  ["Perguntas", "o agente pergunta em vez de adivinhar (P1, P2…), com a recomendação dele. A sua resposta volta para ele pelo MCP, como no fluxo das perguntas abertas do SM PRO."],
  ["À direita", "a memória que o agente usou para escrever, as ligações da página e um pedaço do grafo."],
  ["Um botão cheio", "«Aplicar na folha». O que você desmarcar não entra; descartar também é um clique."],
];

export const BLOCOS = [
  ["Capa", "o rótulo, o título e o resumo, com o estado (esperando você, aplicada)", "agente e você"],
  ["Texto e rótulo de seção", "parágrafo e o rótulo em caixa alta com a linha embaixo", "agente e você"],
  ["Lista, etapas e cartões", "como as listas numeradas e os cartões das suas telas", "agente e você"],
  ["Tabela", "linhas e colunas, com número à direita", "agente e você"],
  ["Linha do tempo", "a régua e as barras desenhadas como a folha", "o app, a partir das ações"],
  ["Ação proposta", "Nova, Alterada ou Sem mudança, com planos, checkpoint e datas; marcar ou desmarcar", "agente"],
  ["Pergunta", "Pn, as opções e a recomendação; a resposta volta ao agente", "agente; você responde"],
  ["Decisão", "o que foi decidido, quando e por quem", "agente e você"],
  ["Figura", "desenho do anatomy (os emblemas e o kit isométrico do app)", "agente"],
  ["Código e ligação", "trecho de código e a ligação [[ ]] para outra página, que vira linha no grafo", "agente e você"],
];

export const MCP = [
  ["escrever_proposta", "nova", "a proposta como página de blocos (substitui o formulário de propor_blueprint)"],
  ["ler_respostas", "nova", "o agente lê o que você respondeu nas perguntas"],
  ["ler_pagina · escrever_pagina", "nova", "as páginas do workspace, do blueprint e da ação"],
  ["ler_memoria · lembrar · esquecer", "nova", "a memória do blueprint e do workspace"],
  ["buscar", "nova", "procura em páginas e memórias, como o Ctrl+K do app"],
  ["registrar_progresso · marcar_concluido", "fica", "o andamento na folha, como hoje"],
];

export const ETAPAS = [
  ["Protótipo", "esta nota. Você aprova ou muda antes de qualquer código."],
  ["Páginas e blocos", "os dados (tabela de páginas no SQLite), as ferramentas do MCP e o desenho nativo dos blocos."],
  ["Planejar novo", "a proposta vira página: ações marcáveis, perguntas com resposta e «Aplicar na folha»."],
  ["Memória", "ler_memoria, lembrar e a tela de cartões; o agente passa a ler antes de propor."],
  ["Grafo e ligações", "as ligações [[ ]], o painel da direita e a tela do grafo."],
  ["Escrever no app", "você edita bloco a bloco e cria páginas; busca com Ctrl+K."],
  ["Despedida do antigo", "o Planejar da página web sai, e o propor_blueprint antigo vira página por um tempo, até os agentes usarem o novo."],
];

export const PERGUNTAS = [
  ["P1", "O formulário de hoje (a proposta campo a campo) some?", "Sim: o Planejar novo substitui. Por um tempo o MCP ainda aceita o formato antigo e o transforma em página, para nenhum agente quebrar."],
  ["P2", "Quem escreve as páginas: só o agente, ou você também cria do zero, como no Obsidian?", "Os dois. Você edita bloco a bloco, como no plugin Telas, e cria página nova pela coluna da esquerda."],
  ["P3", "A memória é do blueprint e do workspace, ou também uma sua, que vale para todos os workspaces?", "Blueprint e workspace agora; a sua, global, depois, se fizer falta."],
  ["P4", "As páginas ficam só no Blueprint, ou também são copiadas para uma pasta do seu cofre?", "Só no Blueprint no começo, para ter um lugar só. Exportar para o cofre fica para depois."],
  ["P5", "O grafo mostra o código por dentro (arquivos e funções, pelo graphify) ou só as pastas ligadas?", "Só as pastas ligadas agora. O código por dentro depois, lendo o graphify-out que cada repositório já tem."],
  ["P6", "O «aplicar sozinho» de hoje continua valendo para a proposta-página?", "Continua, desligado por padrão, como hoje."],
];
