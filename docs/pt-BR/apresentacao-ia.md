# Apresentações com IA

[Voltar ao README](../../README.pt-BR.md) · [English](../apresentacao-ia.md)

Estado: em desenvolvimento. Os controles "Gerar com IA" já estão na tela Apresentação; o lado do servidor que guarda o resultado e o comando `/blueprint:apresentar` ainda estão sendo construídos, então um pedido pode ainda não completar.

Uma apresentação feita pelo seu agente (Claude Code) para um cliente ou para você: uma interface de verdade, no visual do app (papel de plotagem azul, cabeçalhos em caixa, navegação por seções), com figuras vivas do anatomy como parte dela, montada a partir do seu blueprint. É diferente dos três tipos de página que a tela Apresentação já oferece (Resumo executivo, Andamento por projeto, Plano completo), que o app desenha dos seus dados sem agente.

## Escolha o público e o nível

| Público | O que mostra | O que nunca mostra |
| --- | --- | --- |
| **Cliente** | Um plano de projeto: o que o cliente recebe, as etapas e datas, o que se espera do cliente, as decisões em aberto, os riscos em linguagem simples | Tecnologia, rotas, arquivos, horas por plano, as instruções internas das ações |
| **Interno** | A visão técnica completa, para quem constrói | Nada fica escondido |

| Nível | O que você recebe | Tempo estimado |
| --- | --- | --- |
| **Rápido** | Uma figura viva na capa e as páginas do plano | até cerca de 1 h |
| **Padrão** | Uma figura por grupo de etapas, cada uma um objeto diferente | cerca de 1 a 3 h |
| **Completo** | O conjunto Padrão mais uma máquina densa do sistema inteiro | cerca de 2 a 4 h |

Os tempos são estimativas que o agente anuncia antes de começar. Um nível maior (Épico, 6 horas ou mais) só existe se você pedir ao agente pelo nome; a tela não o oferece.

## Pela tela

1. Abra o blueprint e clique em **Apresentar**.
2. No trilho lateral, em **Gerar com IA**, escolha o público e o nível e clique em **Gerar**.
3. Escolha a sessão que recebe o pedido, ou deixe o app abrir um terminal novo. O pedido já leva as suas escolhas, então o agente não pergunta de novo. Gasta tokens, e o diálogo avisa.
4. Quando o agente terminar, a apresentação aparece em **Apresentações geradas**. Abra no visor, exporte como um único arquivo HTML, peça de novo ou remova.

## Pelo terminal

Digite `/blueprint:apresentar` numa sessão do Claude Code na pasta do blueprint (em desenvolvimento). O agente pergunta o público e o nível, pergunta o que o cliente precisa fornecer ou decidir, anuncia o tempo, monta a apresentação e diz onde abrir.

## Bom saber

- O agente escreve só o conteúdo e os programas das figuras; o app monta a página com a própria biblioteca, então uma figura grande não custa uma mensagem grande.
- Gerar de novo refaz só as figuras dos projetos que mudaram.
- A página roda isolada: sem rede, sem acesso ao app, sem armazenamento.
- As apresentações geradas ficam na sua máquina, nunca no repositório Git. Leia a apresentação uma vez antes de mostrar: o arquivo exportado sai do app e nada o recolhe.
- O agente só diz o que o blueprint diz: nada de preço, funcionalidade ou data inventados; uma decisão ainda aberta aparece como aberta.
