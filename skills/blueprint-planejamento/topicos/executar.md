# Executar: ligar a pasta e manter o Blueprint atualizado

## Ligar a pasta ao blueprint

**No Claude Code com o plugin do Blueprint, o app guarda a ligação pasta → blueprint:** a sessão já começa com os ids e a ação atual no contexto. Se o contexto disser que a pasta não está ligada, chame `listar_blueprints`, confirme com o usuário qual é o blueprint e chame `vincular_pasta`; não escreva nada no CLAUDE.md.

Em agentes sem o plugin (Codex e outros), quem guarda a ligação é o `CLAUDE.md` (ou `AGENTS.md`) da própria pasta:

```
Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).
Mantenha este blueprint sempre atualizado: planos e mudanças de plano via propor_blueprint; ao começar um plano, registrar_progresso; ao terminar, marcar_concluido com a evidência; se não deu, marcar_nao_concluido; se travar, reportar_bloqueio.
```

- Se a linha já está lá, use esses ids e não pergunte.
- Se não está, descubra com `listar_blueprints`, pergunte ao usuário se a pasta é de um blueprint e grave as duas linhas, sem mexer no resto do arquivo.
- Blueprint novo só tem `blueprint_id` depois que o usuário aplica a proposta: grave a linha nesse momento.

## Manter atualizado (sem o usuário pedir)

- **Antes de executar uma ação:** `ler_blueprint` com `acao_id` para ter as instruções e as descrições dos planos.
- **Começar um plano:** `registrar_progresso` com uma frase do que vai fazer, mais `acao_id` e `plano_id`: o robô do agente aparece ao lado desse plano na folha.
- **Mudou de arquivo ou de etapa:** `registrar_progresso` de novo, com `estado` e `onde` atualizados (veja abaixo).
- **Terminar:** `marcar_concluido` com a evidência real (comando ou teste rodado e o resultado). Uma chamada por plano, logo que ele termina, sem acumular.
- **Checkpoint cumprido:** `marcar_checkpoint_feito` com a evidência (a demonstração ou o teste que o prova); em geral é o último item da ação.
- **Não deu:** `marcar_nao_concluido` (ou `marcar_checkpoint_nao_feito`) com o motivo. **Travou:** `reportar_bloqueio` com o que falta.
- **Trabalho fora do blueprint:** proponha o plano com `propor_blueprint` antes de executá-lo. Ele só ganha `plano_id` depois que o usuário aplica; até lá, não há o que marcar.
- **O blueprint mudou** (prazo, escopo, ordem): `propor_blueprint` com só o que muda (tópico `editar`). Não deixe a mudança só no terminal.

## Aparecer no painel de agentes (campos opcionais)

O app tem um painel que mostra cada agente do usuário trabalhando, com uma figura própria. Ele só mostra o que você reporta, em `registrar_progresso` e `reportar_bloqueio`. Todos os campos são opcionais (chamada só com `texto` continua valendo, e aparece como "agente"):

| Campo | Valores |
| --- | --- |
| `agente` | seu nome, estável durante o trabalho (até 40 caracteres, sem quebra de linha); **subagentes usam o próprio nome** |
| `papel` | `explorador` (lê), `construtor` (escreve), `revisor` (testa) |
| `estado` | `lendo`, `escrevendo`, `testando`, `aguardando`, `bloqueado`, `concluido` |
| `onde` | até 5 caminhos (arquivos ou pastas, até 120 caracteres cada) em que trabalha agora |
| `percentual` | 0 a 100, quanto do plano já foi feito |
| `acao_id`, `plano_id` | o plano em que está (o painel leva até ele) |

- Reporte ao começar e **toda vez que mudar de arquivo ou de estado**; sem sinal por 10 minutos o agente sai do painel.
- O servidor recusa valores fora disso (estado ou papel inventado, `onde` grande demais, nome com caracteres de controle) e diz o motivo: corrija e envie de novo.
- Subagente sem acesso ao MCP: o agente principal reporta por ele, com o nome do subagente em `agente`.
- Nunca ponha segredos, senhas nem caminhos pessoais em `onde`: o painel mostra o texto como veio.
Avisos do monitor/canal são informativos: consulte `ler_blueprint`. `BLUEPRINT_CHANNEL=1` troca monitor por canal.

## App fechado

Se o The Blueprint estiver fechado, as ferramentas de escrita (`registrar_progresso`, `marcar_concluido`, `marcar_checkpoint_feito`, `propor_blueprint`, `anexar_prototipo`, `vincular_pasta`) **não falham**: o registro fica guardado e entra no app quando ele abrir (a resposta avisa). Não repita a chamada. Os ids e o formato só são validados na abertura; se algo for recusado, o usuário o vê em `pendentes/recusados`. As ferramentas de leitura (`ler_blueprint`, `listar_blueprints`) precisam do app aberto: sem os ids, leia-os do `CLAUDE.md` da pasta ou da sessão.
