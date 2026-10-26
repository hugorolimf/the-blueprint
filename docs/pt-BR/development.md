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
node scripts/migrar-vocabulario.test.mjs
```

Verificações de ponta a ponta (precisam do `target/release/blueprint.exe` e, as de tela, do Microsoft Edge): `scripts/contrato-http.mjs`, `fumaca-rust.mjs`, `ui-e2e.mjs`, `robo-e2e.mjs`, `painel-agentes-e2e.mjs`, `capturas.mjs --comparar`, `mutacoes.mjs` (quebra as regras de segurança de propósito e confere que algum teste falha) e `testar-instalador.mjs`. Veja [CONTRIBUTING.md](../../CONTRIBUTING.md).
