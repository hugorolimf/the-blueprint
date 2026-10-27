# Apresentações com IA

[Voltar ao README](../../README.pt-BR.md) · [English](../apresentacao-ia.md)

Estado: em desenvolvimento. Gerar, ver, editar à mão e conversar com o agente estão construídos; o teste com uma sessão real do Claude ainda está pendente, então um pedido pode não completar em todo ambiente.

Uma apresentação feita pelo seu agente (Claude Code) para um cliente ou para você: uma interface de verdade, no visual do app (papel de plotagem azul, cabeçalhos em caixa, navegação por seções), com figuras vivas do anatomy como parte dela, montada a partir do seu blueprint. É diferente dos dois tipos de página que a tela Apresentação também oferece (Andamento por projeto, Plano completo), que o app desenha dos seus dados sem agente e que mostram o lado técnico.

## Escolha o público e o nível

| Público | O que mostra | O que nunca mostra |
| --- | --- | --- |
| **Cliente** | Um plano de projeto: o que o cliente recebe, as etapas e datas, o que se espera do cliente, as decisões em aberto, os riscos em linguagem simples | Tecnologia, rotas, arquivos, horas por plano, as instruções internas das ações |
| **Interno** | A visão técnica completa, para quem constrói | Nada fica escondido |

| Nível | O que você recebe | Tempo estimado |
| --- | --- | --- |
| **Rápido** | Uma figura da IA na visão geral e as páginas do plano | até cerca de 1 h |
| **Padrão** | Uma figura diferente por sessão da página | cerca de 1 a 3 h |
| **Completo** | O conjunto Padrão, mais uma máquina densa do sistema inteiro e uma figura por etapa | cerca de 2 a 4 h |

Os tempos são estimativas que o agente anuncia antes de começar. Um nível maior (Épico, 6 horas ou mais) só existe se você pedir ao agente pelo nome; a tela não o oferece.

## Pela tela

1. Abra o blueprint e clique em **Apresentar**. Se o blueprint já tem apresentações geradas, a tela abre na mais nova; as páginas do próprio app (Andamento por projeto, Plano completo) ficam no grupo recolhido **Visão interna**.
2. No trilho lateral, em **Gerar com IA**, escolha o público e o nível e clique em **Gerar**.
3. Escolha onde a conversa acontece: o **terminal do app** (o padrão, uma aba do painel embutido, onde você acompanha o agente e responde o que ele perguntar), um terminal externo novo ou uma sessão que já está aberta. O pedido já leva as suas escolhas, então o agente não pergunta de novo. Gasta tokens, e o diálogo avisa.
4. Quando o agente terminar, a apresentação aparece em **Apresentações geradas**. Abra no visor, exporte como um único arquivo HTML, gere de novo (ele pergunta antes se você editou à mão) ou remova. O visor é isolado e não imprime sozinho: **Imprimir** exporta o arquivo, que você imprime pelo navegador.

## Editar à mão

**Editar**, ao lado do visor, abre um painel que espelha a página: dados gerais, resumo, o que você recebe, etapas (com datas, situação, figura e marcos), o que precisamos de você, decisões, riscos e, no público interno, ações. Toda lista tem adicionar, remover, subir e descer. Cada figura tem título, dica, leitura, legenda, descrição, chaves e a **Sessão** da página onde ela fica (ou nenhuma); o código fica num campo avançado que só confere a sintaxe e nunca executa. As mudanças são salvas sozinhas, o visor recarrega onde você estava, **Desfazer** reverte a última mudança e os avisos do servidor (por exemplo, um termo técnico numa apresentação para o cliente) aparecem no painel.

## Conversar com o agente

**Conversar**, ao lado do visor, manda um pedido como "deixe o texto da etapa 3 mais curto e troque a data de fim" a uma sessão do Claude (a que você escolher, ou uma aba nova do terminal do app). O agente lê a apresentação e muda **só o que você pediu**, direto no arquivo, com ajustes pequenos: não refaz o resto e mantém o que você editou à mão. Também dá para digitar o pedido no terminal onde ele está. Quando ele muda a apresentação, o visor recarrega sozinho.

## Pelo terminal

Digite `/blueprint:apresentar` numa sessão do Claude Code na pasta do blueprint. O agente pergunta o público e o nível, pergunta o que o cliente precisa fornecer ou decidir, anuncia o tempo, monta a apresentação e diz onde abrir.

## Bom saber

- O agente escreve só o conteúdo e os programas das figuras; o app monta a página com a própria biblioteca, então uma figura grande não custa uma mensagem grande.
- Gerar de novo refaz só as figuras dos projetos que mudaram.
- A página roda isolada: sem rede, sem acesso ao app, sem armazenamento.
- As apresentações geradas ficam na sua máquina, nunca no repositório Git. Leia a apresentação uma vez antes de mostrar: o arquivo exportado sai do app e nada o recolhe.
- O agente só diz o que o blueprint diz: nada de preço, funcionalidade ou data inventados; uma decisão ainda aberta aparece como aberta.
