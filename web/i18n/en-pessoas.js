// Inglês: painéis de equipe do modo hospedado (pessoas.js, pessoas-convites.js, chave-mcp.js, supervisor.js). Chave = texto-fonte em português.
Object.assign(I18N.catalogos.en, {
  // HTML (blueprint.html)
  "Pessoas": "People", "Acompanhar": "Overview", "Minha chave": "My key", "Equipe": "Team",
  // pessoas.js
  "nunca": "never", "agora há pouco": "just now", "há {n} dia": ["{n} day ago", "{n} days ago"],
  "só um blueprint": "one blueprint only", "workspace inteiro": "whole workspace", "Quem tem acesso": "Who has access", "(você)": "(you)",
  "{n} pessoa": ["{n} person", "{n} people"], "Papel de {nome}": "Role of {nome}", "Papel atualizado.": "Role updated.",
  "Remover {nome}?": "Remove {nome}?", "{nome} perde o acesso a este workspace na hora, e a chave de MCP dessa pessoa deixa de valer.": "{nome} loses access to this workspace right away, and their MCP key stops working.",
  "{nome} foi removido.": "{nome} was removed.",
  // pessoas-convites.js
  "ativo": "active", "revogado": "revoked", "vencido": "expired", "esgotado": "used up",
  "1 hora": "1 hour", "1 dia": "1 day", "7 dias": "7 days", "30 dias": "30 days",
  "Links de convite": "Invite links", "Papel": "Role", "Visitante (link público)": "Visitor (public link)", "Escopo": "Scope", "Workspace inteiro": "Whole workspace",
  "Abre": "Opens", "A folha do blueprint": "The blueprint sheet", "A apresentação": "The presentation", "Validade": "Valid for", "Quantas pessoas": "How many people",
  "Uma pessoa": "One person", "Até 10 pessoas": "Up to 10 people", "Sem limite": "No limit", "Criar link": "Create link",
  "O link público mostra um blueprint: escolha qual no Escopo.": "The public link shows one blueprint: pick which one in Scope.", "Link criado.": "Link created.",
  "Este link só aparece agora. Copie e mande para a pessoa; depois dá só para trocar ou revogar.": "This link is shown only now. Copy it and send it to the person; afterwards you can only replace or revoke it.",
  "Selecionei o link: copie com Ctrl+C.": "Link selected: copy it with Ctrl+C.", "Link público": "Public link", "abre a apresentação": "opens the presentation", "abre a folha": "opens the sheet",
  "{n} uso": ["{n} use", "{n} uses"], "{usos} de {max} usos": "{usos} of {max} uses", "vence {data}": "expires {data}", "por {nome}": "by {nome}",
  "Nenhum link ainda.": "No links yet.", "Link trocado: o anterior deixou de valer.": "Link replaced: the old one stopped working.", "Revogar este link?": "Revoke this link?",
  "O link para de funcionar na hora. Quem já entrou continua, menos visitantes, que saem junto.": "The link stops working right away. People who already joined stay, except visitors, who leave with it.",
  "Link revogado.": "Link revoked.", "Revogar": "Revoke", "Copiar": "Copy", "Copiado": "Copied",
  // chave-mcp.js
  "Minha chave de MCP": "My MCP key", "Com a chave, o seu agente lê e atualiza os blueprints deste servidor com o seu papel. Ela é pessoal: não a compartilhe.": "With the key, your agent reads and updates this server's blueprints with your role. It is personal: do not share it.",
  "Gerar chave": "Create key", "Trocar chave": "Replace key", "Chave {prefixo}… criada em {criada}; último uso: {usada}.": "Key {prefixo}… created {criada}; last used: {usada}.",
  "Você ainda não tem chave.": "You do not have a key yet.", "Trocar a chave?": "Replace the key?", "A chave atual deixa de valer na hora; o agente que a usa precisa da nova.": "The current key stops working right away; the agent using it needs the new one.",
  "A chave só aparece agora. Copie o comando do seu agente e rode no terminal.": "The key is shown only now. Copy your agent's command and run it in the terminal.",
  "Um comando": "One command",
  "Conectar meu Claude neste computador": "Connect my Claude on this computer", "Confirme na janela do app.": "Confirm in the app window.",
  "Liga o seu Claude Code (com o progresso automático na folha) e o Codex. Precisa do The Blueprint instalado neste computador. Se o endereço mudar, rode o comando de novo com o endereço novo.": "Connects your Claude Code (with automatic progress on the sheet) and Codex. It needs The Blueprint installed on this computer. If the address changes, run the command again with the new address.",
  "Chave": "Key", "Selecionei o texto: copie com Ctrl+C.": "Text selected: copy it with Ctrl+C.", "Revogar a chave?": "Revoke the key?",
  "O agente que usa esta chave para de acessar o servidor na hora.": "The agent using this key loses access to the server right away.",
  // supervisor.js
  "entrou": "joined", "saiu": "left", "criou um link de convite": "created an invite link", "revogou um link de convite": "revoked an invite link",
  "trocou um link de convite": "replaced an invite link", "mudou o papel de alguém": "changed someone's role", "removeu alguém": "removed someone",
  "gerou uma chave de MCP": "created an MCP key", "revogou uma chave de MCP": "revoked an MCP key", "o servidor imprimiu o link do dono": "the server printed the owner link",
  "Progresso de {nome}": "Progress of {nome}", "{feitas} de {total} ações concluídas": "{feitas} of {total} actions done",
  "{n} em andamento": ["{n} in progress", "{n} in progress"], "Próximo checkpoint: {nome} ({data})": "Next checkpoint: {nome} ({data})", "Sem checkpoint em aberto": "No open checkpoint",
  "Abrir": "Open", "Nenhum blueprint neste workspace.": "No blueprints in this workspace.", "Atividade recente": "Recent activity",
  "Nenhum workspace para acompanhar.": "No workspace to follow.", "Ninguém ainda.": "Nobody yet.", "Servidor": "Server", "Nada ainda.": "Nothing yet.",
});
