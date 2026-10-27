# The Blueprint v0.3.0 (beta)

[English](../release-v0.3.0.md)

## Novidades

- **Apresentações com IA.** Na tela Apresentar você escolhe o público (Cliente: um plano de projeto em linguagem simples, sem tecnologia, horas ou instruções internas; Interno: a visão técnica completa) e o nível (Rápido, Padrão, Completo), e o seu agente monta uma interface de verdade no visual do próprio app, com figuras vivas do anatomy. Cada seção da página tem a sua figura, e apontar um item do texto o destaca na figura e vice-versa. Detalhes: [Apresentações com IA](apresentacao-ia.md).
- **Gerar no terminal do app.** O pedido roda numa aba do terminal embutido, já com o público e o nível que você escolheu.
- **Editar à mão ou pedir ajustes pontuais ao agente.** A apresentação inteira é editável na tela. Quando você pede uma mudança, o agente edita o arquivo com operações pontuais em vez de reescrever tudo.
- **A tela Apresentar abre na apresentação gerada.** As páginas do próprio app (Andamento por projeto, Plano completo) ficam no grupo recolhido Visão interna. A antiga página Resumo saiu.
- **Aplicação automática.** O que você pede ou aprova no chat entra no blueprint sem um segundo clique, com aviso de Desfazer por 60 s. Um interruptor por blueprint desliga (desligado por padrão no modo equipe); remover e mover continuam esperando o seu clique. O agente é avisado de qual item foi aplicado e se foi você ou a aplicação automática.

## Limites conhecidos

- O fluxo da apresentação foi testado de ponta a ponta com um agente simulado, mas ainda não com uma sessão real do Claude; por isso um pedido pode não completar em todo ambiente.
- **Imprimir** dentro da página gerada é bloqueado pelo sandbox: use **Exportar HTML** e imprima pelo navegador.

## Como instalar

Baixe `Blueprint-0.3.0-instalador.exe` abaixo (instala só para o seu usuário). Quem está na 0.1.2 ou mais nova recebe a atualização sozinho e instala com um clique em Atualizar. O `SHA256SUMS` traz o hash do instalador.

O instalador não é assinado: o Windows ou o Defender podem avisar ou bloquear na primeira execução.
