# Roadmap do The Blueprint

[Read in English](ROADMAP.md) · Atualizado: 2026-10-26

Onde o The Blueprint está e para onde vai. Estados: entregue, fazendo agora, em seguida, depois. Nenhuma data é prometida; vale a ordem. Ideias e votos são bem-vindos nas Issues.

## Como ler

Entregue · Fazendo agora · Em seguida · Depois

## Planejamento

- *Entregue* · **Workspace → Blueprint → Projeto → Ação → Plano/Checkpoint**: Um modelo de planejamento com folha de blueprint, projetos como seções numeradas e numeração por ação.
- *Entregue* · **Prazos no ritmo da IA e tempo real medido**: Planos de 0,5 a 4 h, ações de 1 a 3 dias; o app mede quanto cada plano realmente levou.
- *Entregue* · **Propor, aplicar e provar**: O agente só propõe; você aplica. Marcar um plano como feito exige evidência (o comando ou teste e o resultado).
- *Entregue* · **Exportar o blueprint como arquivos Markdown**: Um arquivo por ação mais um índice, legível no GitHub e versionado com o código. Só leitura: o banco continua sendo a fonte.
- *Entregue* · **Importar issues do GitHub como proposta**: Issues abertas viram ações e planos numa proposta numerada que você revisa e aplica. Nada é gravado antes do clique.
- *Fazendo agora* · **Apresentação e impressão do blueprint**: Transformar um blueprint em proposta de projeto, cronograma ou relatório de andamento para clientes e empresas, imprimível em A4 e exportável em um HTML único. A IA também cria figuras interativas do Anatomy que explicam o projeto em construção, mostradas num sandbox e trocadas por um instantâneo estático na impressão.
- *Depois* · **Importar e exportar Markdown e Issues do GitHub**: Levar planos entre o Blueprint, arquivos Markdown de um repositório e gerenciadores de issues.
- *Depois* · **Um worktree do Git por ação**: Iniciar uma ação no próprio worktree para agentes em paralelo não se atropelarem (ideia vista no ORCA e no Vibe Kanban).

## Integração com agentes

- *Entregue* · **Servidor MCP para qualquer agente**: Ferramentas para listar, ler, propor, relatar progresso, marcar como feito com evidência, anexar protótipos e registrar o trabalho feito fora do plano.
- *Entregue* · **Plugin do Claude Code**: Skill, agente, hooks, status line e /blueprint:status; o Claude aparece na folha, no plano em que trabalha, com os subagentes.
- *Entregue* · **Iniciar uma ação pelo app**: Um clique confirmado abre uma sessão nova do Claude Code, ou envia o pedido a uma sessão já aberta na pasta.
- *Em seguida* · **Claude Code dentro do app**: O comando claude oficial rodando num terminal integrado e com o tema do app, onde os agentes trabalham organizados por ação, com todas as capacidades do Claude Code. Arquitetura a decidir com protótipo medido (pseudoconsole x stream-json); leve e carregado sob demanda.
- *Em seguida* · **Integração nativa com o Codex**: Levar ao Codex o que o plugin do Claude Code faz: hooks, skills, status e sessões, depois de um estudo de paridade documentado. Hoje o Codex recebe só o servidor MCP e a linha do AGENTS.md.
- *Depois* · **Guias para outros agentes MCP**: Guias de configuração testados para outros agentes que falam MCP.
- *Depois* · **Perfis de ferramentas do MCP**: Uma variável de ambiente limita as ferramentas que o agente vê (executor, planejador ou completo), e as descrições ocupam menos do contexto dele.
- *Depois* · **Linha do tempo do agente em cada ação**: Uma lista só de leitura do que o agente fez na ação e quando: começou, mudou de arquivo, concluiu um plano, bloqueou.
- *Depois* · **Enviar comentário do plano ao agente**: Um botão no comentário do plano o envia à sessão do Claude que você escolher, só no seu clique e só no seu computador.
- *Depois* · **Hooks do Codex para o progresso automático**: Confirmado o formato dos hooks na fonte oficial, o Codex ganha o que o Claude Code já tem: o blueprint certo ao iniciar a sessão e as edições registradas como progresso.

## Interface

- *Entregue* · **Folha, menu de contexto, animações e notificações**: A folha do blueprint com expandir e recolher, menu de contexto, notificações do agente e o Claude animado.
- *Fazendo agora* · **Este roadmap dentro do app**: Uma tela Roadmap animada e interativa, alimentada pelo mesmo arquivo desta página.
- *Em seguida* · **Primeiros cinco minutos**: Uma tela vazia com os próximos passos, um blueprint de exemplo dentro do app e o comando blueprint doctor para diagnosticar a instalação.

## Dados e segurança

- *Entregue* · **Local primeiro, pequeno e privado**: Banco SQLite na sua máquina, binário de cerca de 2,5 MB, sem conta e sem telemetria; as rotas do MCP só respondem em loopback; protótipos rodam num sandbox de origem opaca.
- *Entregue* · **Janelas sempre sincronizadas**: Gravação versionada com 409 em dados velhos, junção de três vias por id e atualização ao vivo a cada gravação, para duas janelas nunca se sobrescreverem.
- *Entregue* · **Atualização no seu clique**: Baixa em segundo plano, confere o SHA-256 e só instala quando você clica, depois de copiar o seu banco.
- *Em seguida* · **Dados à prova de perda**: Persistir a fila do agente, backups automáticos rotativos e o comando blueprint restaurar.
- *Em seguida* · **Releases assinadas**: Assinar os checksums das releases para o app provar que o instalador é do autor, e não só que está íntegro.
- *Depois* · **Guardas de desempenho no CI**: Reprovar o build quando o binário, a latência da API ou a memória em repouso regredirem.

## Plataformas

- *Entregue* · **Windows 10/11**: Janela nativa, bandeja, instância única e instalador; o blueprint serve roda o mesmo app como servidor web local em qualquer sistema.
- *Depois* · **Linux e macOS**: Primeiro um estudo de viabilidade: o que hoje é só do Windows e o que uma compilação no Ubuntu encontra.
- *Depois* · **App para iOS e Android**: Acompanhar os agentes, revisar propostas e mostrar uma apresentação pelo celular. O desktop continua sendo a fonte dos dados.

## Colaboração e apresentação

- *Depois* · **Modo colaborativo**: Um modo servidor com membros, papéis (dono, colaborador, supervisor, visitante) e links de convite, ao vivo entre pessoas. Primeiro o desenho; a correção de sincronia entregue agora é o primeiro tijolo.
- *Depois* · **Link público de leitura**: Compartilhar uma proposta ou o andamento com um cliente por um link revogável.
- *Depois* · **Resumo de progresso por período**: Um clique copia um resumo em Markdown do que foi concluído, do que está em andamento e do que bloqueou, com as evidências. Calculado dos seus dados, sem IA e sem rede.
- *Depois* · **Responsável por plano: pessoa ou o agente dela**: Com várias pessoas no mesmo blueprint, cada plano pode dizer de quem é; filtro Meus planos, e o agente de cada pessoa pega os planos dela pelo MCP.

## Comunidade

- *Entregue* · **Código aberto desde o começo**: Licença MIT, CONTRIBUTING, SECURITY, CI no Windows e no Ubuntu, releases com checksums.
- *Entregue* · **Completar comandos no terminal**: blueprint completions imprime um script para PowerShell e Bash que completa os subcomandos.
- *Fazendo agora* · **README, vídeos de demonstração e guias que valem a leitura**: Um README novo, no modelo dos melhores projetos do gênero, demonstrações gravadas, guias passo a passo e um FAQ de problemas.
- *Em seguida* · **winget e Scoop**: Instalar com um único comando pelos gerenciadores de pacotes do Windows.

---

Tem uma ideia ou um voto? [Abra uma issue](https://github.com/hugorolimf/the-blueprint/issues). Fonte desta página: `roadmap.json` (a tela Roadmap do app lê o mesmo arquivo).
