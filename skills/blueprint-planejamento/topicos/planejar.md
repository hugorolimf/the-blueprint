# Planejar: ações, planos e checkpoints

## Ações

- `nome`: curto e específico (ex.: "Terminal real do agente"), não "Ação 1".
- `inicio` e `fim`: sequência sem buracos nem conflitos. Ações novas começam **hoje ou depois**, salvo se o usuário pedir datas passadas.
- `status`: `planejado` ou `andamento` (andamento só se o trabalho já começou).
- `observacao` (no app: **Instruções da ação**): o **roteiro completo da ação**. Pode ser longo (até 20000 caracteres) e deve trazer: objetivo, entrega, passo a passo na ordem, **dependências** ("depende da ação X"), decisões e suposições, riscos e critério de pronto. Os planos são os itens marcáveis; as instruções contam como e por quê.
- De 3 a 8 planos e de 1 a 3 checkpoints por ação.
- `projeto` (opcional): grupo da ação no blueprint, por nome ou id; declare-os em `blueprint.projetos` (`[{ "nome": "Interface" }]`, vale também em blueprint existente). Só com frentes distintas (2 a 5); sem o campo a ação fica em "Geral" (nome reservado). Detalhes: tópico `editar`.

## Planos

**Título (`texto`)**: até **45 caracteres**, começando com **verbo no infinitivo**, acionável.

| Bom | Ruim |
| --- | --- |
| Validar a pasta e a raiz Git | Pasta |
| Criar rota de parar o agente | Fazer a parte do servidor |
| Cobrir erros com testes | Testes e outras coisas diversas |

**Descrição**: obrigatória em plano novo, concreta, com três partes:

```
Escopo: o que entra (e o que não entra).
Resultado esperado: o que existirá quando estiver pronto.
Validação: como provar que está pronto — de preferência um comando ou teste ("node x.test.mjs passa"), ou um comportamento observável.
```

**Tamanho**: cabe em **uma sessão de trabalho** (poucas horas). Se precisa de "e depois...", divida; se é trivial demais, junte.

**Ordem**: na ordem em que devem ser feitas; dependências entre ações vão nas instruções da ação.

Cite arquivos ou módulos **somente** se você os viu na pasta. Nunca invente caminhos, APIs ou fatos do negócio: quando faltar informação, proponha algo razoável e diga que é suposição.

## Checkpoints

- **Checkpoints verificáveis**: uma entrega, uma decisão ou uma demonstração ("Fluxo ponta a ponta validado"), não uma atividade ("Trabalhar no fluxo").
- `data` dentro do período da ação, normalmente perto do fim ou de um risco importante.
- Nome curto, que alguém responde com sim ou não.

## Prazos no ritmo da IA

Quem executa é a IA: prazos curtos, em sequência, a partir de hoje, sem buracos de mais de 2 dias.

- **Plano:** de 0,5 a 4 h de trabalho do agente (acima de 8 h, divida). Informe `horas` (0,5 a 16) em cada plano novo.
- **Ação:** de 1 a 3 dias corridos. Mais que isso só com motivo escrito nas instruções (espera por pessoa, CI, publicação, terceiro).
- **Tempo do usuário** (revisar, aprovar, testar em máquina real) vira checkpoint com data, nunca folga escondida no prazo.
- Bom: ação de 2 dias com 4 planos de 1 a 3 h. Ruim: ação de 3 semanas com 5 planos de 1 dia.

## Nível de detalhe

**Enxuto** (poucos planos, descrições de uma linha), **padrão** (o descrito acima) ou **detalhado** (mais planos pequenos, validação por comando em todos, riscos nas instruções). Sem pedido, use o padrão.

## Blueprint novo

Sem `blueprint_id`, envie também `blueprint: { nome, descricao }` (nome curto, descrição de uma ou duas frases). Proponha o nome; não exija que o usuário escolha.

## Antes de enviar, confira

- [ ] Títulos com verbo no infinitivo e até 45 caracteres; nenhum repetido na ação.
- [ ] Todo plano novo com descrição (escopo, resultado, validação).
- [ ] Todo plano novo com `horas`; ações de até 3 dias (ou com motivo) e sem buracos entre elas.
- [ ] Datas `aaaa-mm-dd`, `fim` depois de `inicio`, checkpoints dentro da ação, sem datas passadas sem pedido.
- [ ] Nada marcado como feito; ids só os que você leu em `ler_blueprint`.
- [ ] Fatos inventados? Se sim, vire suposição declarada.
