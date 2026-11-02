// O texto da nota «The Blueprint - Planejar (plano e protótipo)»: genérico, sem citar nenhum projeto; o plano aberto nas telas é o que for
// passado ao gerador. As telas vêm de tela-*.mjs e o caminho de fluxo.mjs.

export const CAPA = {
  rotulo: "Plano e protótipo · 01/11/2026 · versão 3.5",
  titulo: "Um formato de plano para qualquer projeto, desenhado pela própria folha",
  texto: "A folha <b>é</b> o plano. A IA não escreve uma página: o <b>orquestrador</b>, agente mestre do Workspace, preenche um <b>formato já composto</b>, peça por peça, e você acompanha em tempo real. Cada figura, tabela e decisão fica presa à ação a que pertence. Marcar a caixinha de um plano chama um agente, e ele mesmo marca quando conclui. O formato não conhece nenhum projeto: serve para um sistema com centenas de telas, uma biblioteca sem tela nenhuma ou uma reforma, escrito por qualquer agente.",
};

export const CINCO = [
  ["A folha é o plano", "não há página separada nem coluna de documentos. Abrir o plano é abrir o blueprint."],
  ["Escrito em tempo real", "as ações aparecem na hora, «Aguardando plano». Depois o plano vai sendo escrito na sua frente: planos, decisões e figuras."],
  ["Formato já composto", "a IA preenche campos conhecidos; não decide layout, não escreve HTML nem CSS. O resultado é sempre nativo, no tema do app."],
  ["Genérico de verdade", "só o nome do blueprint e as ações são obrigatórios. O que o projeto não tiver simplesmente não aparece."],
  ["O check chama o agente", "marcou a caixinha, um agente começa. Quando conclui, ele mesmo marca. Dez planos, dez agentes, sem um pisar no outro."],
  ["Um orquestrador", "o agente mestre do Workspace: escreve o plano, entrega cada um ao seu agente, acompanha todos e fala com você."],
  ["Mudança é um git diff", "plano alterado aparece como diferença, dentro do próprio plano. Aprovar, editar ou reprovar, que volta ao original."],
];

export const PECAS = [
  ["blueprint, resumo", "obrigatório o nome", "o título e o cabeçalho da folha"],
  ["projetos[]", "ao menos um; o nome é opcional", "as seções da folha"],
  ["acoes[]: id, nome", "obrigatórios; o id liga uma versão à seguinte", "a linha da ação"],
  ["de, ate", "opcionais", "a barra na régua; sem data, a ação fica «sem data»"],
  ["texto, checkpoints[]", "opcionais", "a ação aberta"],
  ["planos[]: id, texto, depende_de[], area, instrucoes", "opcionais; pode ser só o texto", "a caixinha que chama o agente"],
  ["detalhes[]: id, titulo, legenda, grupo, figura, pede_aprovacao", "opcionais; SVG, PNG, JPG ou WebP", "miniatura na ação e folha de detalhe"],
  ["tabelas[]: titulo, colunas, linhas", "opcionais", "tabela nativa na ação aberta"],
  ["decisoes[]: id, rotulo, texto, data, acoes[], pede_aprovacao", "opcionais; ligadas às ações", "na ação aberta"],
  ["perguntas[]: id, texto, opcoes, recomendada, acoes[]", "opcionais; ligadas às ações", "na ação aberta; a resposta volta ao agente"],
  ["estilo, tema", "opcionais", "as cores próprias das figuras e a qual cor do tema cada uma corresponde"],
];

export const APROVACAO = [
  ["Onde se decide", "Só dentro do plano: na ação aberta e na folha de detalhe. A visão geral (tela 4) só mostra o que chegou."],
  ["Como aparece", "Um controle de três partes na própria linha da peça, e uma margem colorida que diz a situação à distância. Depois de decidir, o controle vira um carimbo."],
  ["Aprovar", "Vale a versão nova; a peça ganha o carimbo. Dá para desfazer."],
  ["Revisar", "Abre o campo ali mesmo, só nessa hora, para dizer o que mudar. A peça volta ao agente com o seu recado."],
  ["Editar", "Num plano alterado, você corrige o texto ali mesmo, em vez de pedir ao agente."],
  ["Reprovar", "Num plano alterado, volta ao estado original. Numa decisão ou num detalhe, a peça sai e o agente é avisado."],
  ["Quem decide", "Só você. O estado não existe no <code>plano.json</code>: fica no banco do Blueprint, e o agente não tem ferramenta para aprovar."],
];

export const DIFF = [
  ["Plano que já existe", "Salvar de novo compara pelos ids. O que saiu aparece em vermelho, o que entrou em verde, com a palavra exata destacada (<code>comparar.mjs</code>, com testes). Prazo e detalhes novos entram na mesma lista."],
  ["Plano novo", "Aparece na visão geral como «Novo plano», só com o resumo: quantos planos, detalhes e checkpoints. «Colocar» o põe na folha."],
  ["Enquanto não revisa", "O plano alterado não pode ser iniciado. O que não mudou segue normal, inclusive os agentes que já estão trabalhando."],
];

export const AGENTES = [
  ["O check chama", "Marcar a caixinha de um plano chama um agente só para ele. «Iniciar os N» marca todos. Aprovar não inicia nada: são gestos separados."],
  ["O agente marca", "Ao concluir, o agente fecha com <code>marcar_concluido</code> e a evidência, e a caixinha fica marcada sozinha. Com todos os planos prontos, a ação está concluída."],
  ["Sem conflito por área", "O plano diz em que <code>area</code> mexe. Dois planos da mesma área, ou um dentro do outro, nunca rodam juntos: o segundo espera, e a folha diz por quê."],
  ["Sem conflito no código", "Em projeto com git, cada agente trabalha na própria cópia (<code>worktree</code>), no ramo <code>agente/&lt;plano&gt;</code>. O app já reconhece essas cópias e liga cada uma ao blueprint; falta criá-las ao chamar o agente. Sem git, só a regra da área vale."],
  ["Dependência", "<code>depende_de</code> faz o plano esperar. Marcar um plano que depende de outro ainda não iniciado mostra o aviso, sem impedir. Se o outro pediu ajuda, ele fica parado."],
  ["Limite", "No máximo N agentes ao mesmo tempo, que você muda na tela; o resto fica na fila e começa sozinho quando abrir vaga."],
  ["Qualquer agente", "O que inicia é um comando configurável (<code>claude</code>, <code>codex</code>…), que recebe só o plano, as instruções e as decisões da ação dele."],
];

export const ORQUESTRADOR = [
  ["Quem é", "Um por Workspace, acima dos agentes de cada plano. Enxerga todos os blueprints, então a regra da área vale também entre blueprints diferentes."],
  ["Escreve o plano", "É ele que monta as ações e escreve o plano em tempo real (tela 6), e quem recebe as suas revisões e os seus recados."],
  ["Entrega o trabalho", "Marcou a caixinha, ele prepara o agente daquele plano: só o plano, as instruções e as decisões que valem para ele, na cópia própria."],
  ["Acompanha", "Lê o andamento de todos. Se um agente pede ajuda, tenta resolver com o que já está decidido; se a decisão é sua, ele traz para você, na ação certa."],
  ["Fala com você", "Um lugar só para perguntar «como está?» ou pedir uma mudança, em vez de procurar em dez sessões."],
  ["O que ele não faz", "Não aprova por você, não marca a caixinha para iniciar e não conclui no lugar de um agente: concluir continua exigindo a evidência de quem fez."],
  ["Se ele parar", "A fila, as áreas e as dependências são regras do app, não dele. Os agentes continuam, a fila anda e, ao voltar, ele lê a situação e segue."],
];

export const TEMPO_REAL = [
  ["Primeiro as ações", "O orquestrador diz quais ações o plano terá. Elas entram na folha na hora, em «Aguardando plano»."],
  ["Depois peça por peça", "Ele escreve uma ação de cada vez, e cada plano, decisão e figura aparece assim que chega, pelo canal em tempo real que o app já usa."],
  ["Validação por peça", "Cada peça é validada ao chegar, e o erro volta na hora. O agente não descobre um problema só no fim."],
  ["Se o agente parar", "A ação fica como está e a folha avisa «Escrita parada há 5 min». Ao voltar, ele continua pelos ids, sem começar de novo."],
];

export const COMPATIVEL = [
  ["Qualquer tipo de projeto", "Os exemplos abaixo passam pelo mesmo código: com telas, sem nenhuma figura e sem ser software."],
  ["Qualquer agente", "O agente só precisa chamar as ferramentas do MCP. Não depende de Claude nem de Anatomy."],
  ["Qualquer figura", "Figura que usa as cores do tema (<code>var(--forte)</code>, <code>var(--cartao)</code>…) muda com o app. Figura com cores próprias usa o <code>tema</code> para dizer a correspondência; sem isso, mantém as cores originais."],
  ["Versões futuras", "<code>formato</code> traz a versão. Campo desconhecido é guardado e ignorado, para um plano novo não quebrar num app antigo."],
  ["Segurança", "Validado na entrada: nada de script, foreignObject, evento, arquivo externo, link simbólico ou caminho fora da pasta; limite de 2 MB por SVG e 4 MB por imagem. Nenhum HTML do agente chega à janela."],
];

export const MCP = [
  ["abrir_plano", "nova", "diz o blueprint e as ações; elas entram na folha em «Aguardando plano»"],
  ["escrever_acao", "nova", "manda uma peça da ação: planos, checkpoints, decisões, tabelas, perguntas ou um detalhe com a figura; aparece na hora"],
  ["fechar_plano", "nova", "avisa que terminou; o que mudou em relação à versão anterior vira diff"],
  ["salvar_plano", "nova", "o plano inteiro de uma vez, a partir de uma pasta (substitui propor_blueprint)"],
  ["ler_plano · ler_respostas", "nova", "o agente lê o plano atual, as respostas, as aprovações, as revisões e os recados"],
  ["ler_workspace · falar_com_voce", "nova", "para o orquestrador: a situação de todos os blueprints e agentes, e o recado que aparece na ação certa"],
  ["registrar_progresso · marcar_concluido · reportar_bloqueio", "fica", "o andamento de cada plano, com <code>plano_id</code>; concluir marca a caixinha"],
];

export const TESTE = [
  ["Motor", "resvg 0.45.1, o mesmo da interface nativa, sem navegador"],
  ["Plano real", "54 figuras, 640 KB na pasta"],
  ["Resultado", "RESULTADO_TESTE"],
  ["Cores", "o motor não entende var() nem color-mix(): o Blueprint resolve isso ao carregar o plano (<code>estilo.mjs</code>)"],
  ["Fontes", "sem fonte definida o motor escolheu uma fonte errada e o texto saiu em letras gregas; com a Cascadia embutida no app, saiu certo. O app precisa sempre entregar a própria fonte ao motor"],
  ["Memória", "manter todas desenhadas pesa cerca de 170 MB em 1100 px: desenhar só as visíveis e guardar miniaturas"],
  ["Regras", "<code>agentes.test.mjs</code> e <code>comparar.test.mjs</code> passando: quem pode rodar, área, dependência, ciclo e o git diff"],
];

export const ETAPAS = [
  ["Protótipo", "Esta nota. Você aprova ou muda antes de qualquer código."],
  ["Formato e validação", "O <code>blueprint.plano/1</code> em Rust, com testes dos três exemplos, a documentação em inglês e o guia do agente."],
  ["Detalhes na folha", "As tabelas no banco, a folha nova, a ação aberta e a folha de detalhe, com o motor de figuras."],
  ["Escrita em tempo real", "<code>abrir_plano</code>, <code>escrever_acao</code> e <code>fechar_plano</code>, com os estados «Aguardando plano» e «Escrevendo»."],
  ["Git diff e revisão", "As versões no banco, a comparação pelos ids, o diff no plano, aprovar, editar, revisar e reprovar, e a visão geral."],
  ["Agentes por plano", "A caixinha que chama o agente, a cópia própria, a fila, os avisos e a marcação automática ao concluir."],
  ["Orquestrador", "A sessão do mestre do Workspace, o campo para falar com ele, o diário «Agora» e a entrega do plano a cada agente."],
  ["Kit para o agente", "As cores do tema no kit Anatomy e o guia em <code>skills/</code>."],
  ["Planos existentes", "Um adaptador por projeto, fora do Blueprint, para quem já tem planos em outro formato."],
];

export const PERGUNTAS = [
  ["P1", "O nome do formato: «plano» confunde com o Plan da folha (a caixinha da ação). Trocar?", "Manter <code>plano.json</code> para a IA e não criar um nome novo na tela."],
  ["P3", "Texto longo de raciocínio (o «porquê» de um estudo) entra no formato?", "Só o <code>texto</code> curto da ação e as decisões; o resto fica como arquivo anexo. Se fizer falta, vira uma peça na versão 2 do formato."],
  ["P4", "Memória e grafo entram agora?", "Não: ficam para depois do plano funcionar, para não misturar as duas entregas."],
  ["P5", "Os nomes dos campos ficam em português, sendo um projeto aberto para o mundo?", "Decisão sua. Eu aceitaria os dois: português como está e sinônimos em inglês no formato."],
  ["P6", "Reprovar uma decisão ou um detalhe exige escrever o motivo?", "Não: reprovar é só reprovar. Quem quer explicar usa «Revisar»."],
  ["P8", "Dá para exigir revisão em tudo, mesmo quando o agente não pediu?", "Sim, como opção do blueprint, desligada por padrão: útil em projeto de equipe."],
  ["P9", "A folha nativa muda para este desenho (a ação e os números à esquerda, a régua à direita, o hoje no começo)?", "Sim: é o que faz a folha aguentar um plano grande. Hoje a régua ocupa a largura toda, com o hoje no meio."],
  ["P10", "Quantos agentes ao mesmo tempo, por padrão?", "4, ajustável na tela. Cada agente gasta a sua cota."],
  ["P11", "Plano pronto: o ramo do agente entra sozinho no principal?", "Não: fica pronto com a evidência e você integra com um botão, ou liga «integrar sozinho» se os testes passarem."],
  ["P12", "O agente de um plano pode chamar subagentes?", "Pode: é um por plano na sua tela, e os subagentes dele aparecem como hoje, ao lado."],
  ["P13", "«Colocar» um plano novo na folha: é isso que você quis dizer, ou o resumo só aparece, sem botão?", "Com o tempo real, as ações já entram sozinhas. Eu deixaria o «Colocar» só quando o plano novo chega inteiro por <code>salvar_plano</code>."],
  ["P14", "O orquestrador escreve o plano sozinho, ou pode dividir a escrita com um agente por ação?", "Sozinho, nesta versão: o plano fica coerente. Dividir vira opção depois."],
  ["P15", "Entendi certo o orquestrador: ele coordena, mas quem inicia é a sua caixinha e quem aprova é você?", "Sim. Ele pode ganhar mais poder depois, como opção, mas começar sem isso protege a sua cota e o seu código."],
  ["P16", "O orquestrador fica sempre ligado, ou só acorda quando algo acontece?", "Acorda quando algo acontece (plano concluído, pedido de ajuda, recado seu). Sempre ligado gastaria cota à toa, porque a fila já é do app."],
];
