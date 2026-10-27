# Aplicação automática

[English](../aplicacao-automatica.md)

O contrato que o código (`web/js/auto-aplicar*.js`), as respostas do MCP (`src/mcp/stdio.rs`) e o guia de planejamento (`skills/blueprint-planejamento/topicos/`) seguem. O que você pede ou aprova na conversa com o seu agente entra no blueprint sem um segundo clique na tela Planejar.

## Quem autoriza

- **Você, por blueprint.** O interruptor **Aplicar sozinho: ligado/desligado** fica no cabeçalho do Planejar e é guardado por workspace e blueprint no navegador da janela que aplica (`blueprint.autoAplicar.<workspace>:<blueprint>`).
- **App local:** ligado até você desligar.
- **Modo equipe (`blueprint serve --hospedar`):** desligado por padrão; só o dono vê o interruptor e pode ligar. Colaborador, supervisor e visitante nunca aplicam sozinhos.
- **O agente nunca se autoaprova.** Ele só envia a proposta ou o registro para a fila, como sempre. Quem grava é a página, pelo mesmo caminho do clique (`aplicarAberta`, `registroAdiciona`), com a mesma validação, o controle de versão (`If-Match`) e a confirmação ao servidor.

## O que entra sozinho

- Proposta (`propor_blueprint`) para um blueprint que já existe, no workspace aberto na janela.
- Registro do trabalho feito (`registrar_trabalho`) com pelo menos uma ação.
- Não enquanto você digita no Planejar desse mesmo blueprint: aí espera o seu clique.

## O que nunca entra sozinho

- Proposta com `remover` (apagar ações, planos ou checkpoints) ou `mover` (mover ações entre blueprints). Fica no Planejar com a nota "precisa da sua confirmação".
- Proposta de blueprint novo.
- Nada com o interruptor desligado, nem sem nenhuma janela do app aberta (a v1 aplica só na página; com o app na bandeja o item espera na fila).

## Desfazer

- Toda aplicação automática mostra o aviso **Aplicado sozinho** com **Desfazer** por **60 segundos**, uma vez.
- O Desfazer inverte só as operações daquela aplicação, e só onde o valor ainda é o que ela deixou. O que alguém mexeu depois (outra janela, o agente) fica, e a página diz isso em vez de adivinhar.
- Item desfeito não volta à fila: o agente reenvia só se você pedir.

## Registro de cada aplicação automática

- O Planejar lista as últimas aplicações automáticas do blueprint (hora, texto, Desfazer ou "desfeito"). A página guarda as 30 últimas no navegador.
- O agente fica sabendo qual item entrou e quem aplicou: os eventos SSE `proposta-aplicada` e `registro-adicionado` levam `numero` e `origem` (`automatica` ou `usuario`), nunca texto livre.
- Sem aplicar duas vezes: cada item é identificado por workspace, blueprint, tipo, número e `criadaEm`. Se o servidor mandar de novo um item (a confirmação se perdeu), a página só confirma de novo.

## O que o agente diz

As respostas de `propor_blueprint` e `registrar_trabalho` dizem que o item entra sozinho com o interruptor ligado, e que remoções, movimentos e blueprints novos esperam o clique. O agente confere com `ler_blueprint` antes de dizer que entrou.

## Testes

`node auto-aplicar-logica.test.mjs` (regras e Desfazer), `cargo test mcp::tests` (a resposta do MCP e os primeiros 512 caracteres das instruções) e `node scripts/auto-aplicar-e2e.mjs [exe]` (o agente envia pelo `--mcp`, a página aplica, desfaz, deixa a remoção esperando e não faz nada com o interruptor desligado; pt-BR e en-US).
