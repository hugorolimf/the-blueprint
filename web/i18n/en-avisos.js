// Inglês: entrada, avisos da tela, mensagens do MCP, atualização e notificações do agente. Chave = texto-fonte em português (web/js/i18n.js).
Object.assign(I18N.catalogos.en, {
  // entrada e workspaces
  "Workspaces": "Workspaces", "Seus workspaces": "Your workspaces",
  "Salvando seu nome…": "Saving your name…", "Não foi possível salvar seu nome. Tente novamente.": "Could not save your name. Try again.",
  "Digite o nome do workspace.": "Enter the workspace name.", "Já existe um workspace com esse nome.": "A workspace with that name already exists.",
  "salvando…": "saving…", "erro": "error", "Não foi possível concluir. Tente novamente.": "Could not finish. Try again.",
  "Excluir workspace?": "Delete workspace?", "Excluir workspace": "Delete workspace",
  "O workspace \"{nome}\" será apagado{extra}. Não dá para desfazer.": "The workspace \"{nome}\" will be deleted{extra}. This cannot be undone.",
  ", junto com {blueprints} e todas as ações": ", along with {blueprints} and all actions",
  "Não foi possível excluir o workspace.": "Could not delete the workspace.",
  "{nome} movido para {pos} de {total}. Salvando…": "{nome} moved to {pos} of {total}. Saving…",
  "Ordem dos workspaces salva.": "Workspace order saved.", "A ordem mudou, mas não foi possível salvá-la.": "The order changed, but it could not be saved.",
  // rede e salvamento
  "Não foi possível carregar os dados. Tentando novamente…": "Could not load the data. Trying again…",
  "salvo ✓": "saved ✓", "erro ao salvar": "error saving",
  "“{nome}” chegou a 100%: está em Concluídos na lista de blueprints.": "“{nome}” reached 100%: it is now under Completed in the blueprint list.",
  "“{nome}” voltou para Em andamento.": "“{nome}” is back in progress.",
  "Concluído pelo agente, com base no resultado da execução.": "Completed by the agent, based on the execution result.",
  "Relatório revisado pelo usuário; plano marcado como concluído.": "Report reviewed by the user; plan marked as completed.",
  // agente (MCP): o que a fila avisa
  "Registro nº {n} recebido: veja em ✦ Planejar e adicione ao blueprint o que quiser.": "Record no. {n} received: see it in ✦ Plan and add to the blueprint whatever you like.",
  "O seu agente enviou um registro do que fez: abra ✦ Planejar para adicioná-lo ao blueprint.": "Your agent sent a record of what it did: open ✦ Plan to add it to the blueprint.",
  "Proposta recebida para {alvo}: revise ao lado e aplique.": "Proposal received for {alvo}: review it alongside and apply it.",
  "um blueprint novo": "a new blueprint",
  "O seu agente enviou uma proposta: abra ✦ Planejar para revisar.": "Your agent sent a proposal: open ✦ Plan to review it.",
  "Protótipo “{titulo}” atualizado na ação “{acao}”: veja no ícone de protótipo da ação.": "Prototype “{titulo}” updated on action “{acao}”: see the action's prototype icon.",
  "Protótipo “{titulo}” anexado na ação “{acao}”: veja no ícone de protótipo da ação.": "Prototype “{titulo}” attached to action “{acao}”: see the action's prototype icon.",
  "O seu agente atualizou o protótipo “{titulo}”.": "Your agent updated the prototype “{titulo}”.", "O seu agente anexou o protótipo “{titulo}”.": "Your agent attached the prototype “{titulo}”.",
  "Não concluído: {texto}": "Not completed: {texto}", "O seu agente travou: {texto}": "Your agent got stuck: {texto}",
  // atualização
  "Atualização": "Update", "Atualizar e reiniciar": "Update and restart", "Depois": "Later",
  "Não consegui atualizar ({status}).": "Could not update ({status}).", "Não consegui falar com o app.": "Could not talk to the app.",
  "Baixando… {pct}%": "Downloading… {pct}%", "Atualizar para {versao}": "Update to {versao}",
  "Baixando a versão{v}… {pct}%": "Downloading version{v}… {pct}%",
  "Atualizando: o app fecha e abre de novo sozinho.": "Updating: the app closes and reopens by itself.",
  "A versão{v} está pronta (você tem a {atual}).": "Version{v} is ready (you have {atual}).",
  // notificações do agente
  "Ir até o blueprint": "Go to the blueprint", "Aviso do agente · atenção": "Agent notice · attention", "Aviso do agente": "Agent notice",
  "Próximo ({mais})": "Next ({mais})", "Abrir o plano": "Open the plan", "Blueprint": "Blueprint", "Ação": "Action", "Plano": "Plan",
  "O agente travou": "The agent got stuck", "Tarefa não concluída": "Task not completed", "Checkpoint não atingido": "Checkpoint not reached",
  "Nova proposta do agente": "New proposal from the agent", "Tarefa concluída": "Task completed", "Checkpoint atingido": "Checkpoint reached",
  "Protótipo anexado": "Prototype attached", "Revise a proposta em ✦ Planejar e aplique o que quiser.": "Review the proposal in ✦ Plan and apply whatever you like.",
});
