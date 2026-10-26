# The Blueprint v0.2.3 (beta)

[English](../release-v0.2.3.md)

## Correção

- **A janela de progresso não atrapalha mais a atualização.** Na 0.2.2 a janela era aberta com as entradas e saídas do console que o app já tinha largado, e o clique em Atualizar podia não mostrar nada, terminar o instalador com erro sem reabrir o app ou pular a janela. Agora a janela abre sem console. Testado numa atualização de verdade, com o instalador real: sucesso (a versão nova reabre e a janela fecha sozinha em cerca de 6 s), falha do instalador (a janela mostra o erro e a versão anterior reabre) e janela que não consegue abrir (o instalador roda direto).

## Se você está na 0.2.2

A sua próxima atualização ainda usa a janela da 0.2.2, porque a janela é da versão que está saindo. Se o app fechar sem atualizar, abra de novo e clique em Atualizar mais uma vez. Da 0.2.3 em diante, as atualizações usam a janela corrigida.

## Como instalar

Baixe o `Blueprint-0.2.3-instalador.exe` abaixo (instala só para o seu usuário). Quem está na 0.1.2 ou mais nova recebe a atualização sozinho e instala com um clique em Atualizar. O `SHA256SUMS` traz o hash do instalador.

O instalador não é assinado: o Windows ou o Defender podem avisar ou bloquear na primeira execução.
