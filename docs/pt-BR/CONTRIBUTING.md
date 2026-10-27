[English](../../CONTRIBUTING.md)

# Como contribuir

Obrigado! O projeto é pequeno de propósito. Antes de uma mudança grande, abra uma issue para conversarmos. O que está planejado e o que está sendo feito fica no [ROADMAP](../../ROADMAP.pt-BR.md).

## Vocabulário (importante)

O modelo é **Workspace → Blueprint → Projeto (opcional) → Ação → Plano / Checkpoint**. "Projeto" é o grupo de ações dentro de um blueprint (campos `projetos`, `projetoId` e `projeto`); não o use como nome do blueprint. Não use "fase", "tarefa" nem "marco" no código, na interface, no MCP ou nos documentos (existem só no script de migração e na migração SQL `002_vocabulario.sql`). Em português, "plano" é o item marcável; a proposta inteira do agente é uma **proposta** (`propor_blueprint`).

## Regras de código

- **Arquivos pequenos:** no máximo 350 linhas; o teto absoluto é 500 e só com o motivo escrito no topo do arquivo. Um arquivo, uma responsabilidade; nada de `utils` genérico.
- **Um teste por módulo** (`tests.rs` ou `tests/` ao lado do módulo Rust). Mudou uma regra de segurança ou de dados? Escreva o teste que falha sem a mudança.
- **Poucas dependências.** Rust: só crates que valem o peso (nada de `tokio` se um servidor síncrono resolve). Navegador: sem framework e sem bundler; scripts clássicos carregados na ordem de `blueprint.html` (`inicio.js` por último).
- **O kit isométrico** (`kit/`, `emblemas.mjs`) roda no navegador e não vai para Rust; mudou o desenho? Rode `node emblemas.test.mjs` e `node agentes-figura.test.mjs` e só regrave as referências de propósito.
- **Protótipos** nunca ganham `allow-same-origin`. As rotas `/api/mcp/*` continuam só em loopback.
- `blueprint.html` usa fim de linha CRLF; edite com cuidado, sem converter o arquivo inteiro.
- **Sem emojis** em nenhum lugar do projeto. Precisa de uma figura? Desenhe com o Anatomy. Tudo o que é visual usa o tema do próprio app (`web/css/tokens.css`).
- **Todo texto que o usuário lê passa por `tr()`** e precisa de uma entrada em inglês em `web/i18n/` (veja [Desenvolver e testar](development.md#idiomas)); `node scripts/i18n-completude.mjs` reprova o que faltar.
- **A documentação é escrita primeiro em inglês**; a versão em português fica em `docs/pt-BR/` com o mesmo nome de arquivo (regra e glossário em [Idiomas da documentação](development.md#idiomas-da-documentação)).

## Desenvolver

`cargo dev` abre o app em modo dev (janela nativa, com os seus dados; feche o app antes): salvar `web/` ou `kit/` recarrega a janela; salvar `src/` recompila e reabre (precisa de `cargo install cargo-watch`, uma vez; sem ele, `cargo run -- dev`). `node dev.mjs` é a alternativa no navegador, com banco de exemplo em `.dev/`. Nenhum dos dois roda testes.

## Rodar os testes

```
cargo test
node emblemas.test.mjs
node agentes-figura.test.mjs
node i18n.test.mjs
node scripts/i18n-completude.mjs
node scripts/migrar-vocabulario.test.mjs
cargo build --release
node scripts/contrato-http.mjs       # as respostas da API contra referencias/api-snapshots.json
node scripts/fumaca-rust.mjs
node scripts/idioma-e2e.mjs          # precisa do Microsoft Edge
node scripts/roadmap-e2e.mjs         # precisa do Microsoft Edge
node scripts/capturas.mjs --comparar # as 7 telas, pixel a pixel (hash)
```

Mudou de propósito uma resposta da API ou a aparência de uma tela? Regrave a referência (`contrato-http.mjs --gravar`, `capturas.mjs --gravar`), confira a diferença e diga no pull request o que mudou e por quê.

## Pull requests

1. Uma mudança por PR, com o porquê na descrição.
2. `cargo test` e os testes `node` verdes (a CI roda os mesmos em Linux e Windows).
3. Atualize a documentação e o guia do agente (`skills/blueprint-planejamento/`) se mudou o MCP.

Issues boas para começar têm o rótulo **good first issue**.
