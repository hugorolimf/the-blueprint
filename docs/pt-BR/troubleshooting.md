# Solução de problemas

[Voltar ao README](../../README.pt-BR.md) · [English](../troubleshooting.md)

Cada problema abaixo tem a causa e uma conferência que você mesmo roda. Um comando `blueprint doctor` que faça essas conferências está no [roadmap](../../ROADMAP.pt-BR.md); até lá, use os comandos daqui.

## A janela mostra dados velhos, ou a proposta do agente não aparece

**Causa:** dois processos usam o mesmo banco (por exemplo, o app e um `blueprint serve` aberto à mão). Cada um guarda a sua cópia dos dados e a sua fila de avisos do agente, e a janela não vê o que o outro recebeu.

**Conferir:** o app grava a porta em que escuta em `%APPDATA%\Blueprint\porta`. Deve ser `3000`:

```
type %APPDATA%\Blueprint\porta
```

Se aparecer outro número, a porta 3000 estava ocupada por outro processo. **Correção:** feche o app (ícone da bandeja → *Sair*), encerre o outro processo `blueprint` (Gerenciador de Tarefas) e abra o app de novo. Rode `blueprint serve` só com o app fechado.

## O agente diz que o app está fechado

**Causa:** o agente fala com o app em `http://127.0.0.1:3000` (ou em `BLUEPRINT_URL`). Com o app fechado, as ferramentas de escrita (`registrar_progresso`, `marcar_concluido`, `propor_blueprint` e as outras) ficam guardadas em arquivos em `%APPDATA%\Blueprint\pendentes\`, e a resposta ao agente avisa. Elas entram na fila, na ordem, quando o app abre. As de leitura (`ler_blueprint`, `listar_blueprints`) precisam do app aberto.

**Conferir:** `dir %APPDATA%\Blueprint\pendentes` lista o que está esperando. O que o app recusar ao abrir vai para `pendentes\recusados\` com o motivo. **Correção:** abra o The Blueprint pelo menu Iniciar.

## O Claude Code não sabe o blueprint da pasta, ou as ferramentas sumiram

**Causa:** o plugin `blueprint@blueprint-local` não está instalado (a tarefa *conectar* do instalador ficou desmarcada, ou o plugin foi removido).

**Conferir:** `claude plugin list` deve mostrar `blueprint@blueprint-local`. Dentro da sessão, `/blueprint:status` só responde com o plugin instalado. **Correção:** `blueprint registrar` instala o plugin de novo (e o MCP no Codex). Numa pasta que nunca foi vinculada, peça ao agente para vinculá-la, ou escolha o blueprint em **Editar → Dados do blueprint → Pasta do projeto**. Detalhes no [guia do plugin](claude-code.md).

O **Codex** não tem plugin: `codex mcp list` deve mostrar `blueprint`, e o `AGENTS.md` da pasta precisa da linha `Blueprint: workspace_id …, blueprint_id …` ([guia do MCP](mcp.md)).

## O Windows mostra um aviso do SmartScreen ou do Defender

**Causa:** o instalador ainda não tem assinatura de código, então o Windows não conhece o editor na primeira execução.

**Conferir:** compare o hash do instalador com o arquivo `SHA256SUMS` anexado à mesma release:

```
certutil -hashfile Blueprint-<versão>-instalador.exe SHA256
```

Se os dois forem iguais, o arquivo é o que o workflow de release gerou. Releases assinadas estão no [roadmap](../../ROADMAP.pt-BR.md).

## A atualização se recusa a instalar

O botão **Atualizar e reiniciar** (ou `blueprint atualizar --instalar`) fecha o app, então ele recusa enquanto fechar perderia alguma coisa. O aviso diz qual é o caso:

| Aviso | Correção |
| --- | --- |
| avisos do agente que a tela ainda não aplicou | abra o blueprint, aplique ou descarte no **Planejar** e clique de novo |
| um workspace está sendo compartilhado | desligue o compartilhamento (botão **Compartilhar**) e clique de novo |
| sessões rodando no terminal do app | feche-as no painel **Terminal** e clique de novo |
| não há atualização pronta para instalar | `blueprint atualizar` procura e baixa |

Como funciona a atualização: [Atualizações](updates.md).

## O terminal não reconhece `blueprint`

**Causa:** o instalador não põe `blueprint` no PATH (o plugin do Claude Code não precisa disso). **Correção:** chame pelo caminho completo, `%LOCALAPPDATA%\Programs\Blueprint\blueprint.exe`, ou ponha essa pasta no PATH ([Instalação](install.md)).

## Ainda travado?

Abra uma issue com o `blueprint --version`, o que você fez e o que viu: [Issues](https://github.com/hugorolimf/the-blueprint/issues). Problemas de segurança vão pelo [SECURITY.md](SECURITY.md), não por issue pública.
