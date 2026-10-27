// Inglês: o que sobrou das outras telas (cartão do carimbo, pastas, projetos, arte, protótipos, agentes, markdown, avisos pequenos). Chave = texto-fonte em português (web/js/i18n.js).
Object.assign(I18N.catalogos.en, {
  "Dias": "Days", "Horas": "Hours", "Dia": "Day", "AGORA": "NOW", "Desvio": "Difference",
  "Escala de tempo": "Time scale", "Alternar Dias e Horas (Alt+H)": "Toggle Days and Hours (Alt+H)",
  "Previsto: {horas} h": "Estimated: {horas} h", "Real: {horas} h": "Elapsed: {horas} h", "Em andamento: {horas} h": "In progress: {horas} h",
  // folha
  "Data": "Date", "Progresso": "Progress", "Folha": "Sheet", "Ações": "Actions", "Planejar este blueprint": "Plan this blueprint",
  "Ver o que {agente} está fazendo": "See what {agente} is doing", "o agente": "the agent",
  "Agente": "Agent",
  "(mais {n} agente junto aqui)": ["(plus {n} agent here)", "(plus {n} agents here)"],
  "Blueprint movido para a posição {n}": "Blueprint moved to position {n}",
  "Pressione F11 para uma melhor visualização desta tela.": "Press F11 for a better view of this screen.", "Fechar dica": "Close tip",
  // pastas
  "Erro {status}": "Error {status}", "Pasta ligada ao blueprint": "Folder linked to the blueprint", "Trocar": "Change", "Remover": "Remove",
  "Ligar outra pasta": "Link another folder", "C:\\caminho\\da\\pasta": "C:\\path\\to\\folder", "Ligar pasta": "Link folder",
  "Não foi possível carregar as pastas: {erro}": "Could not load the folders: {erro}", "Pastas sem blueprint": "Folders without a blueprint",
  "Pasta sem blueprint: {cwd}": "Folder without a blueprint: {cwd}", "Blueprint para {cwd}": "Blueprint for {cwd}",
  "Escolha o blueprint para ligar": "Choose the blueprint to link", "Agora não": "Not now",
  // projetos
  "Nome do projeto": "Project name", "Descrição (uma linha, aparece sob o nome na folha)": "Description (one line, shown under the name on the sheet)",
  "Descrição do projeto": "Project description", "Remover projeto": "Remove project", "Projetos": "Projects",
  "Sem projetos: todas as ações ficam juntas. Crie projetos para agrupar as ações do blueprint na folha.": "No projects: all actions stay together. Create projects to group the blueprint's actions on the sheet.",
  "+ Projeto": "+ Project",
  "Na folha cada projeto é uma seção numerada (01, 02…) com o cabeçalho de um blueprint e as ações dele; a ordem segue o início da primeira ação. A ação sem projeto fica em “Geral”; escolha o projeto dela em Período e situação.": "On the sheet each project is a numbered section (01, 02…) with a blueprint-style header and its actions; the order follows the start of the first action. An action without a project stays under “General”; pick its project in Period and status.",
  "Projeto": "Project", "Geral (sem projeto)": "General (no project)", "Geral": "General", "{nome} (novo)": "{nome} (new)",
  "Projetos que esta proposta cria: {nomes}.": "Projects this proposal creates: {nomes}.",
  // arte
  "Blueprint (padrão)": "Blueprint (default)", "Moedas": "Coins", "Livros": "Books", "Chip": "Chip", "Caminhão": "Truck",
  "A prévia da arte ainda está carregando.": "The art preview is still loading.", "Digite o texto da arte.": "Enter the art text.",
  "Cole as peças da arte em JSON.": "Paste the art parts as JSON.", "O texto da arte não é um JSON válido.": "The art text is not valid JSON.",
  "Nenhuma peça válida na arte.": "No valid part in the art.",
  // protótipos
  "Protótipo · {nome}": "Prototype · {nome}", "ação": "action", "Excluir protótipo?": "Delete prototype?", "Excluir protótipo": "Delete prototype",
  "O protótipo \"{titulo}\" será apagado. Não dá para desfazer.": "The prototype \"{titulo}\" will be deleted. This cannot be undone.",
  // agentes
  "Lendo": "Reading", "Escrevendo": "Writing", "Testando": "Testing", "Aguardando": "Waiting", "Bloqueado": "Blocked", "Concluído": "Done", "Parado": "Idle",
  "agora": "now", "há {n} s": "{n} s ago", "há {n} min": "{n} min ago", "há {n} h": "{n} h ago",
  // markdown
  "A prévia aparece aqui enquanto você escreve.": "The preview appears here as you write.", "Prévia": "Preview",
  // primeiro uso (primeiro-uso.js): a entrada vazia e o exemplo
  "Primeiros passos": "First steps", "Nenhum blueprint ainda. Por onde começar?": "No blueprints yet. Where do you want to start?",
  "Abrir o exemplo": "Open the example", "Um workspace com dois blueprints fictícios, para ver o app cheio. Os seus dados não mudam e ele sai com um clique.": "A workspace with two made-up blueprints, to see the app full. Your data does not change and it goes away with one click.",
  "Conectar seu agente": "Connect your agent", "Registre o MCP no Claude Code ou no Codex e peça um plano: a proposta aparece aqui para você aplicar.": "Register the MCP in Claude Code or Codex and ask for a plan: the proposal shows up here for you to apply.",
  "Criar um blueprint": "Create a blueprint", "Escreva o primeiro blueprint à mão, com ações, planos e checkpoints.": "Write the first blueprint by hand, with actions, plans and checkpoints.",
  "Exemplo": "Example", "Dados fictícios para conhecer o app. Remova quando quiser.": "Made-up data to get to know the app. Remove it whenever you want.",
  "O workspace «{nome}» é o exemplo: dados fictícios, separados dos seus.": "The workspace «{nome}» is the example: made-up data, kept apart from yours.", "Remover exemplo": "Remove example",
  "Abrindo o exemplo…": "Opening the example…", "Não foi possível abrir o exemplo. Tente novamente.": "Could not open the example. Try again.",
});
