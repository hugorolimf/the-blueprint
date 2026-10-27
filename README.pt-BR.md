<p align="center">
  <img src="recursos/icone/icone.svg" width="120" alt="The Blueprint">
</p>

<h1 align="center">The Blueprint</h1>

<p align="center"><strong>Roadmap local para agentes de IA de código: eles propõem, você aprova, e concluir exige evidência.</strong></p>

<p align="center">
  <a href="https://github.com/hugorolimf/the-blueprint/releases"><img src="https://img.shields.io/github/v/release/hugorolimf/the-blueprint?include_prereleases" alt="Última release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/hugorolimf/the-blueprint" alt="Licença MIT"></a>
  <a href="https://github.com/hugorolimf/the-blueprint/actions/workflows/test.yml"><img src="https://img.shields.io/github/actions/workflow/status/hugorolimf/the-blueprint/test.yml?label=testes" alt="Testes"></a>
</p>

<p align="center">
  Beta · Windows 10/11 · MIT · <a href="README.md">English</a> · <a href="ROADMAP.pt-BR.md">Roadmap</a> · <a href="docs/pt-BR/comparison.md">Comparação</a>
</p>

![Uma folha de blueprint com projetos, ações, planos e checkpoints](docs/img/blueprint.png)

## Por quê

O seu agente de código (Claude Code, Codex) planeja e constrói rápido. O custo é um plano que você nunca revisou e um "feito" que não dá para conferir. O The Blueprint é um app de desktop pequeno que guarda o projeto inteiro numa folha, e o seu próprio agente conversa com ele pelo [MCP](https://modelcontextprotocol.io):

- **Ele propõe, você aplica.** Tudo o que o agente envia é uma proposta numerada. Nada muda nos seus dados antes de você aplicar na tela **Planejar**.
- **Concluir exige prova.** Um plano só é marcado como feito com o comando ou o teste rodado e o resultado, que vira comentário do plano.
- **Você vê o trabalho.** O Claude aparece na folha, no plano em que está trabalhando, com os subagentes.
- **Pequeno e privado.** Um executável de cerca de 2,5 MB, um arquivo SQLite no seu computador, sem conta e sem telemetria.

## Início rápido (cerca de 5 minutos, Windows)

1. Baixe o `Blueprint-<versão>-instalador.exe` em [Releases](https://github.com/hugorolimf/the-blueprint/releases) e execute. Ele instala só para o seu usuário (sem administrador) e, com a tarefa *conectar* marcada, adiciona o plugin do Claude Code e registra o MCP no Codex.
2. Abra o **The Blueprint** pelo menu Iniciar. Aparecem uma janela e um ícone na bandeja.
3. Abra o Claude Code (ou o Codex) na pasta do seu projeto, como sempre.
4. Peça: *"Planeje este projeto no The Blueprint."* A proposta chega ao app; revise em **Planejar** e aplique.
5. Peça: *"Inicie a primeira ação."* Enquanto o agente trabalha, o Claude anda até o plano na folha e cada plano é marcado com a evidência.

Fora do Windows, ou sem o instalador? Compile com o [Rust](https://rustup.rs): `cargo build --release`, e `blueprint serve` roda o mesmo app como servidor web local em `http://127.0.0.1:3000` ([detalhes](docs/pt-BR/commands.md)).

## Como funciona

```
Workspace
└─ Blueprint            o escopo
   └─ Projeto           agrupa ações (opcional)
      └─ Ação           início, fim, status, instruções
         ├─ Plano       título, descrição, horas, feito + evidência
         └─ Checkpoint  nome, data, feito
```

O agente lê a estrutura, propõe mudanças, relata o andamento e marca planos como feitos por ferramentas MCP (`listar_blueprints`, `ler_blueprint`, `propor_blueprint`, `registrar_progresso`, `marcar_concluido` e outras; [lista completa](docs/pt-BR/mcp.md)). Quem inicia uma ação é você, com um clique confirmado ou o comando `blueprint claude`, então o app nunca manda trabalho a um agente por conta própria. Os planos têm o tamanho do ritmo da IA (de 0,5 a 4 horas) e o app mede quanto cada um realmente levou. [O modelo em detalhe](docs/pt-BR/model.md).

## Documentação

| | |
| --- | --- |
| [Instalação e desempenho](docs/pt-BR/install.md) | detalhes do instalador, pasta de dados, números medidos |
| [Plugin do Claude Code](docs/pt-BR/claude-code.md) | skill, agente, hooks, status line, `/blueprint:status`, o painel no terminal |
| [MCP e outros agentes](docs/pt-BR/mcp.md) | ferramentas, Codex, fila com o app fechado |
| [Atualizações](docs/pt-BR/updates.md) | download em segundo plano, conferência do SHA-256, instala só no seu clique |
| [Servidor e comandos](docs/pt-BR/commands.md) | `blueprint serve`, variáveis de ambiente, todos os subcomandos |
| [Desenvolvimento](docs/pt-BR/development.md) | `cargo dev`, testes, verificações de ponta a ponta |
| [Comparação](docs/pt-BR/comparison.md) | Backlog.md, Task Master, Vibe Kanban, Plane |
| [Solução de problemas](docs/pt-BR/troubleshooting.md) | dois processos, app fechado, plugin ausente, SmartScreen, atualização recusada |

## Perguntas frequentes

**O app roda o meu agente ou gasta tokens sozinho?** Não. Você inicia uma ação e só então o Claude Code roda, como de costume.

**Ele envia meus dados para algum lugar?** Sem conta e sem telemetria. A única chamada de rede é um `GET` às Releases do GitHub para procurar atualização, que dá para desligar ([como](docs/pt-BR/updates.md)).

**Onde ficam os meus dados?** Em `%APPDATA%\Blueprint\blueprint.db`. O `blueprint exportar` grava em JSON.

**O Windows mostra um aviso do SmartScreen ou do Defender.** O instalador ainda não tem assinatura de código, então isso pode acontecer na primeira execução. Compare o SHA-256 dele com o arquivo `SHA256SUMS` da mesma release; releases assinadas estão no [roadmap](ROADMAP.pt-BR.md).

**macOS e Linux?** Ainda não; veja o [roadmap](ROADMAP.pt-BR.md). O modo servidor funciona onde você conseguir compilar.

**Codex?** O MCP funciona hoje; uma integração nativa (hooks, skills, status) está planejada.

## Roadmap, contribuir e segurança

- [ROADMAP.pt-BR.md](ROADMAP.pt-BR.md): o que existe, o que está sendo feito e o que vem depois.
- [CONTRIBUTING.md](docs/pt-BR/CONTRIBUTING.md): ideias e boas primeiras issues são bem-vindas nas [Issues](https://github.com/hugorolimf/the-blueprint/issues).
- [SECURITY.md](docs/pt-BR/SECURITY.md): as rotas do MCP só respondem em loopback; protótipos rodam num sandbox de origem opaca.

## Licença

[MIT](LICENSE). O kit isométrico descende do [Anatomy](https://github.com/wheresryan22/anatomy) (MIT); veja [NOTICE.md](NOTICE.md).
