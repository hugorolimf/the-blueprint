# Editar um blueprint que já existe

Mande **só o que muda**. O servidor completa o resto com o que já existe, então uma proposta parcial nunca apaga conteúdo.

- **Ação existente:** o `id` e apenas os campos que mudam. Nome, datas, status, instruções, planos e checkpoints que não vierem ficam como estão.
- **Plano ou checkpoint existente:** o `id` e só o que muda (ex.: `{ "id": "t2", "texto": "Novo título" }` mantém a descrição).
- **Item novo** (sem `id`): completo, como no tópico `planejar` (plano com `texto` e `descricao`; checkpoint com `nome` e `data`).
- Ações que você não citar ficam como estão. **Nada é apagado** por omissão, e itens já feitos continuam feitos.
- **Mover** plano ou checkpoint para outra ação: cite o `id` dele na lista da ação de destino (existente ou nova). O item sai da origem e vai com o mesmo id e o que já estava feito; num checkpoint, mande a `data` nova, dentro do período do destino. A tela mostra "← ação de origem".
- **Mover uma ação inteira de OUTRO blueprint do mesmo workspace** (para unir blueprints): lista `mover` em `propor_blueprint` com `{acao_id, motivo}`, no blueprint de destino (existente). A ação chega com planos, checkpoints, feito e protótipos; o usuário revisa e aplica. Não recrie a ação com texto igual: isso perderia o que já está feito.
- **Projetos** (grupos de ações dentro do blueprint): `blueprint.projetos` lista os projetos que devem existir (`[{ "nome": "Interface" }]`; um nome que já existe é o mesmo projeto, e os existentes vêm em `projetos` de `ler_blueprint`). Em cada ação, `projeto` aceita o id ou o nome de um projeto existente ou listado na proposta; `""` ou `"Geral"` tira a ação do projeto; sem o campo, ela fica onde está. Projeto desconhecido é erro. Para distribuir ações já existentes, mande só `{ "id", "projeto" }` de cada uma. A proposta nunca apaga nem renomeia projetos (isso é feito na tela Editar), e a numeração das ações não muda.
- **Apagar:** lista `remover` em `propor_blueprint`, com `{acao_id, plano_id | checkpoint_id, motivo}` (só `acao_id` apaga a ação inteira). É um pedido: a tela mostra os itens desmarcados e só some o que o usuário marcar e aplicar. Sempre dê o `motivo`; para um item já feito, só se o usuário pediu.

- **Fila de propostas e ajuste:** cada `propor_blueprint` entra numa **fila** do blueprint com um número (1, 2, 3…) e **nunca apaga outra**; o usuário aplica ou descarta uma de cada vez em ✦ Planejar. A resposta traz o número e os **ids provisórios** da proposta: `+a1` (ação nova), `+a1.p2` (plano), `+a1.c1` (checkpoint); numa ação existente, `+<id da ação>.p1`. Para **corrigir uma proposta que ainda está na fila** (um título, um plano a mais), mande `ajustar: <número>` com só o que muda: cite a ação/plano/checkpoint pelo id provisório (ou pelo id real) e os campos novos; item sem `id` entra no fim. Nada some por omissão (para tirar algo, `substituir`), e a proposta inteira é revalidada: se der erro, a da fila fica como estava. `substituir: <número>` troca a proposta inteira, só quando o usuário pedir para refazê-la. Número que não está na fila (já aplicada, descartada ou o app reiniciou) dá erro: mande uma proposta nova. A fila fica na memória do app e some ao reiniciá-lo.
- Nunca invente um `id`: use só os de `ler_blueprint` e os provisórios devolvidos por `propor_blueprint`.
- Para **reescrever as instruções** de uma ação, leia-a antes com `ler_blueprint(acao_id)` e envie o texto novo inteiro em `observacao`.
- **Revisar prazos:** aponte os conflitos (ações sobrepostas sem motivo, checkpoint fora da ação) e ajuste só o que fizer sentido, explicando.

Exemplo: adiar uma ação e acrescentar um plano, sem reenviar o resto:

```json
{ "acoes": [{ "id": "a1", "fim": "2026-12-05", "planos": [{ "texto": "Cobrir erros com testes", "descricao": "Escopo: … Resultado esperado: … Validação: node x.test.mjs passa." }] }] }
```
