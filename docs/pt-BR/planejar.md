# Planejar: a folha é o plano

[Voltar ao README](../../README.pt-BR.md) · [English](../planejar.md)

Um agente escreve o plano peça por peça num formato fixo, e você vê chegar na folha do blueprint. Nada do que ele manda muda o conteúdo aprovado até você decidir. Marcar um plano inicia um agente para ele, numa cópia própria do repositório, e você integra o resultado quando estiver pronto.

**Onde existe:** só no app local com a janela nativa (Linux). No Windows, no navegador, em `blueprint serve`, na porta de convidados e no modo hospedado, as rotas e ferramentas não aparecem ou são recusadas, e `propor_blueprint` continua sendo o caminho para planejar.

## O formato

Um plano é uma pasta com um `plano.json` no formato `blueprint.plano/1`, mais as figuras:

```json
{
  "formato": "blueprint.plano/1",
  "blueprint": "Monitor de consumo",
  "projetos": [{ "acoes": [{
    "id": "1.1", "nome": "Coleta dos medidores", "de": "2026-11-02", "ate": "2026-11-09",
    "planos": [
      { "id": "1.1.1", "texto": "Ler os medidores pela rede local", "area": "coleta" },
      { "id": "1.1.2", "texto": "Guardar as leituras", "area": "banco", "depende_de": ["1.1.1"] }
    ]
  }]}]
}
```

Só o nome do blueprint e as ações são obrigatórios. A ação também pode trazer `texto`, `checkpoints`, `tabelas` e `detalhes` (figuras em SVG, PNG, JPG ou WebP); o plano como um todo pode trazer `decisoes` e `perguntas`. Exemplos completos: `src/plano/fixtures/`. Campos como `aprovado`, `estado` ou `feito` no JSON não têm efeito.

- **`id`** liga uma versão do plano à seguinte. Mantenha estável.
- **`area`** diz que parte do projeto o plano toca. Dois planos com áreas que se sobrepõem (`app` e `app/barra`) nunca rodam ao mesmo tempo, mesmo em blueprints diferentes do workspace.
- **`depende_de`** segura o plano até aqueles planos ficarem prontos.

## As ferramentas do agente

| Ferramenta | O que faz |
| --- | --- |
| `abrir_plano`, `escrever_acao`, `fechar_plano` | escrevem peça por peça: as ações aparecem na hora como "Aguardando plano" e vão sendo preenchidas. Chamar `abrir_plano` de novo retoma a escrita aberta |
| `salvar_plano` | manda uma pasta inteira de uma vez |
| `ler_plano`, `ler_respostas`, `ler_workspace` | leem o plano, suas revisões e respostas, e a situação do workspace. Ler não tem efeito nenhum |
| `falar_com_voce` | deixa um recado para você |

Nenhuma ferramenta aprova mudança, marca plano, integra ramo ou liga agente, em perfil nenhum. Isso são gestos da janela do app, e só dela.

## O que você decide

- **Ações novas.** Escritas peça por peça, entram na folha quando a escrita fecha. Vindas de `salvar_plano`, esperam **Colocar** na tela "O que chegou".
- **Ações alteradas.** O conteúdo aprovado fica na folha, com a mudança como diff por palavra dentro da ação. **Aprovar** aplica, **Editar** muda o texto antes, **Reprovar** deixa tudo como estava. Plano que mudou enquanto você olhava (409) pede nova revisão.
- **Decisões e detalhes** que pedem aprovação têm controle próprio. Reescrever um deles derruba a aprovação.
- **Plano pronto** cujo texto, instruções ou área mudou volta a não feito quando você aprova; o diff avisa naquela linha.
- **Com agente rodando** em plano alterado ou removido, a aprovação é recusada até você cancelar ou ele terminar.

## Agentes rodando

1. Clique na caixinha do plano, ou em **Iniciar os N**. Só esse gesto cria trabalho.
2. O app inicia os planos livres, um por tique, até o limite (4 por padrão, de 1 a 16). Os outros esperam vaga, dependência ou área.
3. Cada agente abre num terminal externo, numa cópia do repositório fora da sua pasta (`git worktree`), no ramo `agente/<id>`. Plano que vem depois de um plano pronto e ainda não integrado parte do ramo dele. Pasta sem git não tem cópia: um agente por pasta.
4. O agente termina com `marcar_concluido` e evidência, gravados no servidor. Só o agente com o segredo daquela tentativa consegue.
5. **Integrar**, na tela de agentes, faz o merge do ramo na pasta principal com `--no-ff` e remove a cópia e o ramo. Recusa se a pasta principal foi para outro ramo ou tem merge em andamento. Conflito é abortado e relatado. Suas alterações locais nunca são descartadas.

Agente sem sinal por 120 segundos aparece como sumido, e **Retomar** o traz de volta na mesma cópia. **Cancelar** o encerra no próximo sinal e mantém o ramo e a cópia.

O **orquestrador** é um agente opcional do workspace. Fica desligado até você ligar ou mandar uma mensagem, porque acordar gasta tokens. Ele não tem Bash, vê a situação e deixa recados; não aprova, não inicia e não conclui nada.

## O que isto não garante

- Cópia não é sandbox: o agente alcança a pasta principal por caminho absoluto, e `.git/hooks` é compartilhado. "Sem conflito entre agentes" depende de as áreas serem declaradas com honestidade.
- Processo hostil rodando com o seu usuário lê o banco e a memória do app, e pode editar o SQLite. Agente com Bash alcança as rotas de agente sem o token de gesto, mas não as de gesto.
- Um `plano.json` escrito por outra pessoa pode tentar induzir o agente. O texto vai ao agente marcado como dado, o que ajuda, mas não é imunidade.
- Com a janela nativa aberta, a página web antiga não grava: as escritas dela pedem o token da janela.
- Agente parado em pedido de permissão ou de confiança na pasta fica esperando no terminal. Se ele pergunta ou não depende da sua configuração do Claude Code.
- O backup copia o banco. Figuras em disco não são copiadas, como já acontece com protótipos.
