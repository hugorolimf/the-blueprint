# The Blueprint v0.2.1 (beta)

[English](../release-v0.2.1.md)

## Novidades

- **O Claude do convidado se conecta com um comando.** Na tela Minha chave, o convidado copia `blueprint conectar <endereço> <chave>` (precisa do The Blueprint instalado no computador dele). O Claude Code dele passa a ler e propor no workspace compartilhado com o papel dele, e os hooks dele chegam ao seu app: você vê "Claude · nome" trabalhando na sua folha. Quando o endereço do compartilhamento muda, ele roda o mesmo comando de novo, e vale na hora, até nas sessões abertas. A chave fica num arquivo do app, nunca nos argumentos dos processos; `blueprint conectar --remover` desfaz. Veja [Compartilhar pelo app](compartilhar.md).
- **Vários workspaces compartilhados ao mesmo tempo** pelo mesmo endereço, e o diálogo Compartilhar sempre fala do workspace aberto.
- **`ler_blueprint` mais enxuto para os agentes:** em blueprints grandes, as ações concluídas e as planejadas distantes viram uma linha cada (de cerca de 89 mil para 20 mil caracteres com 100 ações), com os parâmetros opcionais `incluir` e `projeto`.
- **Mais do plugin do Claude Code:** o vínculo da pasta segue as worktrees do git até a pasta principal; hooks novos (pedido e recusa de permissão, falha ao parar, compactação) viram estados do agente sem nunca bloquear o Claude; o subagente recebe o contexto do plano ao começar; três agentes de papel vêm com o plugin (explorador, construtor, revisor); as status lines mostram contexto, limites de uso, worktree e agente, e a dos subagentes alimenta o modo foco.
- O subagente que termina sai da folha em 10 s; só o Claude principal fica como Concluído.
- **Decisões do blueprint:** um card no Editar registra decisões em aberto e decididas (título, escolha, motivo), e o agente pode propô-las.

## Como instalar

Baixe o `Blueprint-0.2.1-instalador.exe` abaixo (instala só para o seu usuário). Quem está na 0.1.2 ou mais nova recebe a atualização sozinho e instala com um clique em Atualizar. O `SHA256SUMS` traz o hash do instalador.

O instalador não é assinado: o Windows ou o Defender podem avisar ou bloquear na primeira execução.
