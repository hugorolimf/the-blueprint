# Desenvolver e testar

[Voltar ao README](../../README.pt-BR.md) · [English](../development.md)

```
cargo dev                                    # o app em modo dev, com recarga automática (precisa do cargo-watch)
```

`cargo dev` abre a janela do app com os seus dados (`%APPDATA%\Blueprint`), então feche o app antes. Salvou `web/`, `kit/` ou `blueprint.html`: a janela recarrega (CSS troca sem perder a tela; F12 abre as ferramentas). Salvou `src/`, `skills/` ou `Cargo.toml`: o `cargo-watch` (`cargo install cargo-watch`, uma vez) recompila e reabre. Sem ele: `cargo run -- dev`. Para trabalhar no navegador com dados de exemplo, sem Node na janela: `node dev.mjs` (http://127.0.0.1:3917; opções `--porta N`, `--limpar`, `--abrir`).

```
cargo test                                   # mais de 230 testes: dados, validação, MCP, HTTP
node emblemas.test.mjs                       # marcas e figuras (kit do anatomy)
node agentes-figura.test.mjs
node i18n.test.mjs                           # a camada de idiomas
node scripts/i18n-completude.mjs             # toda chave usada no código tem tradução
node scripts/migrar-vocabulario.test.mjs
```

Verificações de ponta a ponta (precisam do `target/release/blueprint.exe` e, as de tela, do Microsoft Edge): `scripts/contrato-http.mjs`, `fumaca-rust.mjs`, `robo-e2e.mjs`, `painel-agentes-e2e.mjs`, `idioma-e2e.mjs`, `roadmap-e2e.mjs`, `sincronia-e2e.mjs`, `capturas.mjs --comparar`, `mutacoes.mjs` (quebra as regras de segurança de propósito e confere que algum teste falha) e `testar-instalador.mjs`. Veja [CONTRIBUTING.md](CONTRIBUTING.md).

## Idiomas

A interface segue o idioma do Windows e cai no inglês. No código, todo texto que o usuário lê passa por `tr("texto em português")`; o texto-fonte em português é a chave, então texto sem tradução cai no português e nunca quebra. Os catálogos ficam em `web/i18n/en-<área>.js`. `node scripts/i18n-varrer.mjs` abre cada tela em inglês e lista o que ainda está em português; `node scripts/i18n-inventario.mjs` lista candidatos a converter.

## Idiomas da documentação

A documentação é escrita primeiro em inglês, com o português do Brasil como segunda língua:

- O arquivo com o nome simples é o inglês (`docs/mcp.md`); a tradução fica em `docs/pt-BR/` com o mesmo nome (`docs/pt-BR/mcp.md`). `CONTRIBUTING.md`, `SECURITY.md` e `AGENTS.md` ficam na raiz, com as traduções em `docs/pt-BR/`. Os READMEs e o roadmap usam o sufixo `.pt-BR.md` ao lado do arquivo em inglês (`README.pt-BR.md`, `ROADMAP.pt-BR.md`).
- Cada arquivo abre com um seletor de idioma abaixo do título: `[Português](pt-BR/<nome>.md)` no inglês, `[English](../<nome>.md)` no português.
- As duas versões têm os mesmos títulos, na mesma ordem e nos mesmos níveis; o texto pode mudar na forma, nunca no conteúdo.
- Os nomes de arquivo são em inglês. Quando um arquivo muda de nome, o nome antigo fica como uma página curta que aponta para o novo, para os links de fora continuarem funcionando.
- As notas só do mantenedor (`docs/interno/`) seguem a mesma regra. Algumas referências técnicas antigas ainda dizem "Portuguese only for now" no topo; elas são traduzidas quando alguém mexe nelas.

`node scripts/conferir-links.mjs` (roda no CI) confere todo link relativo e, para cada arquivo em `docs/pt-BR/`, que o inglês existe, que um aponta para o outro e que os títulos batem.

Glossário (use estes nomes em inglês; o código, o banco e os campos do MCP mantêm os nomes em português):

| Inglês | Português | No código e no MCP |
| --- | --- | --- |
| Workspace | Workspace | `workspace_id`, `workspaces[]` |
| Blueprint | Blueprint | `blueprint_id`, `blueprints[]` |
| Project | Projeto | `projetos[]`, `projetoId`, `projeto` |
| Action | Ação | `acao_id`, `acoes[]` |
| Plan | Plano | `plano_id`, `planos[]` |
| Checkpoint | Checkpoint | `checkpoint_id`, `checkpoints[]` |
| Proposal | Proposta | `propor_blueprint` |
| Record (trabalho feito fora do plano) | Registro | `registrar_trabalho` |
| Evidence | Evidência | `evidencia` |

Não use "fase", "tarefa" ou "marco" para essas ideias. A única exceção é o aviso do agente na tela, que chama o plano concluído de "Tarefa concluída".

## Vídeos e GIFs

`node scripts/gravar/index.mjs` regrava todos os vídeos do app em inglês e em português do Brasil (`--idioma en-US` para um idioma só, `03 04` para alguns roteiros, `--lista` para listá-los). Precisa do Microsoft Edge, do `ffmpeg` no PATH e de um `blueprint.exe` (`BLUEPRINT_EXE`, padrão `target/debug/blueprint.exe`); os dados são sempre os de exemplo, num servidor temporário. Os arquivos vão para `videos/<roteiro>.<en|pt-BR>.mp4` e `.gif` (os primeiros 12 s, abaixo de 3 MB), pasta que o Git ignora. Antes de usar um vídeo em qualquer lugar, confira os quadros por dados pessoais. O GIF do README, `docs/img/visao-geral.gif`, é cópia de `videos/01-visao-geral.en.gif`.
