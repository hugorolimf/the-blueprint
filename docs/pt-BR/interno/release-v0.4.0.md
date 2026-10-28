# The Blueprint v0.4.0 (beta)

[English](../../interno/release-v0.4.0.md)

## Novidades

- **Nada se perde ao fechar o app.** Propostas, registros de trabalho e as marcações que o seu agente envia agora esperam no banco, não na memória, e voltam iguais depois de reiniciar.
- **Cópias diárias e `blueprint restaurar`.** Ao abrir, o app copia o banco no máximo uma vez por dia e guarda as 7 últimas. `blueprint restaurar` lista as cópias e `blueprint restaurar <número>` volta a uma delas (com o app fechado; o banco atual é guardado antes).
- **`blueprint doctor`.** Um comando confere se o app responde onde os agentes procuram, a versão, o banco, o plugin do Claude Code, o MCP do Codex e se a pasta atual está ligada a um blueprint, e diz o que fazer em cada problema.
- **Primeiro uso.** O app vazio mostra o próximo passo e pode abrir um blueprint de exemplo, para você ver o ciclo antes de planejar o seu.
- **Perfis de ferramentas do MCP.** `BLUEPRINT_MCP_PERFIL=executor` ou `planejador` dá ao agente só as ferramentas de que ele precisa (até 50% menos contexto).
- **Barra do topo da folha.** Os botões não cobrem mais o título do blueprint; em janela estreita viram ícones, com o nome ao passar o mouse.
- **Documentação.** Guia de solução de problemas, guia do Codex e de outros agentes MCP e uma demonstração do ciclo no README.

## Limites conhecidos

- O instalador continua sem assinatura (a proposta de assinatura está no repositório e espera uma chave).

## Como instalar

Baixe `Blueprint-0.4.0-instalador.exe` abaixo (instala só para o seu usuário). Quem está na 0.1.2 ou mais nova recebe a atualização sozinho e instala com um clique em Atualizar. O `SHA256SUMS` traz o hash do instalador.

O instalador não é assinado: o Windows ou o Defender podem avisar ou bloquear na primeira execução.
