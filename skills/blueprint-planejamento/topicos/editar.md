# Editar um blueprint que já existe

Mande **só o que muda**. O servidor completa o resto com o que já existe, então uma proposta parcial nunca apaga conteúdo.

- **Ação existente:** o `id` e apenas os campos que mudam. Nome, datas, status, instruções, planos e checkpoints que não vierem ficam como estão.
- **Plano ou checkpoint existente:** o `id` e só o que muda (ex.: `{ "id": "t2", "texto": "Novo título" }` mantém a descrição).
- **Item novo** (sem `id`): completo, como no tópico `planejar` (plano com `texto` e `descricao`; checkpoint com `nome` e `data`).
- Ações que você não citar ficam como estão. **Nada é apagado** por omissão, e itens já feitos continuam feitos.
- **Mover** plano ou checkpoint para outra ação: cite o `id` dele na lista da ação de destino (existente ou nova). O item sai da origem e vai com o mesmo id e o que já estava feito; num checkpoint, mande a `data` nova, dentro do período do destino. A tela mostra "← ação de origem".
- **Apagar:** lista `remover` em `propor_blueprint`, com `{acao_id, plano_id | checkpoint_id, motivo}` (só `acao_id` apaga a ação inteira). É um pedido: a tela mostra os itens desmarcados e só some o que o usuário marcar e aplicar. Sempre dê o `motivo`; para um item já feito, só se o usuário pediu.

- Nunca invente um `id`: use só os de `ler_blueprint`.
- Para **reescrever as instruções** de uma ação, leia-a antes com `ler_blueprint(acao_id)` e envie o texto novo inteiro em `observacao`.
- **Revisar prazos:** aponte os conflitos (ações sobrepostas sem motivo, checkpoint fora da ação) e ajuste só o que fizer sentido, explicando.

Exemplo: adiar uma ação e acrescentar um plano, sem reenviar o resto:

```json
{ "acoes": [{ "id": "a1", "fim": "2026-12-05", "planos": [{ "texto": "Cobrir erros com testes", "descricao": "Escopo: … Resultado esperado: … Validação: node x.test.mjs passa." }] }] }
```
