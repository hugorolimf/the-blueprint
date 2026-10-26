# The Blueprint v0.1.2 (beta)

[English](../release-v0.1.2.md)

## Correções

- **A atualização automática não baixava** no app aberto pela janela ou pela bandeja: o `curl.exe` não abria ("os error 50" do Windows) porque o app, sem console, não tinha saída padrão para herdar. Corrigido no download e no lançamento do instalador.
- **Botão Atualizar ao lado do nome**, na tela Workspaces: aparece como "Baixando… N%" e depois como "Atualizar para X.Y.Z". O "Depois" esconde só o aviso do canto.

## Novidades

- O app registra o início e o fim de cada sessão do Claude (hook `SessionEnd` e rota de sessões).

## Como instalar

Baixe `Blueprint-0.1.2-instalador.exe` abaixo (instala só para o seu usuário). **Quem está na 0.1.0 ou 0.1.1 precisa instalar esta à mão uma vez**, porque essas versões não conseguem baixar atualizações; da 0.1.2 em diante a atualização é automática e só instala com o seu clique. O `SHA256SUMS` traz o hash do instalador.

O instalador não é assinado: o Windows ou o Defender pode avisar ou bloquear na primeira execução.
