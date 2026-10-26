# Instalar

[Voltar ao README](../README.pt-BR.md)

## Windows 10/11 (janela, bandeja e instalador)


- Baixe o instalador `Blueprint-<versão>-instalador.exe` da página de Releases (instala só para o seu usuário, sem administrador), ou
- compile: instale o [Rust](https://rustup.rs) e rode `cargo build --release`; o executável sai em `target/release/blueprint.exe` (cerca de 2,5 MB, a interface vai embutida). O instalador sai de `ISCC instalador\blueprint.iss` (Inno Setup 6).

Os dados ficam em `%APPDATA%\Blueprint\` (banco SQLite `blueprint.db` e `prototipos\`). Para começar com dados de exemplo: `blueprint importar data.exemplo.json %APPDATA%\Blueprint\blueprint.db`.

O instalador não coloca o `blueprint` no PATH: o plugin do Claude Code já fica instalado para o seu usuário e o `claude` de sempre funciona. Para usar `blueprint claude` de qualquer pasta, chame pelo caminho completo (`%LOCALAPPDATA%ProgramsBlueprintlueprint.exe claude`) ou ponha essa pasta no PATH.

## Desempenho (medido)

Binário de 2,5 MB, servidor com ~11 MB de memória e 0% de CPU em repouso, `GET /api/dados` abaixo de 5 ms, janela visível em ~0,1 s. Detalhes em [`referencias/relatorio-medicao.md`](../referencias/relatorio-medicao.md).
