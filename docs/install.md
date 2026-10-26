# Install

[Back to the README](../README.md) · [Português](pt-BR/install.md)

## Windows 10/11 (window, tray and installer)

- Download the installer `Blueprint-<version>-instalador.exe` from the Releases page (it installs for your user only, no administrator), or
- build it: install [Rust](https://rustup.rs) and run `cargo build --release`; the executable lands in `target/release/blueprint.exe` (about 2.5 MB, the interface is embedded). The installer comes from `ISCC instalador\blueprint.iss` (Inno Setup 6).

Your data lives in `%APPDATA%\Blueprint\` (the SQLite database `blueprint.db` and `prototipos\`). To start with sample data: `blueprint importar data.exemplo.en.json %APPDATA%\Blueprint\blueprint.db` (or `data.exemplo.json` for the Portuguese sample).

The installer does not put `blueprint` on the PATH: the Claude Code plugin is already installed for your user and your usual `claude` works. To use `blueprint claude` from any folder, call it by its full path (`%LOCALAPPDATA%\Programs\Blueprint\blueprint.exe claude`) or add that folder to the PATH.

## Performance (measured)

A 2.5 MB binary, a server with ~11 MB of memory and 0% CPU at rest, `GET /api/dados` under 5 ms, a window visible in ~0.1 s. Details in [`referencias/relatorio-medicao.md`](../referencias/relatorio-medicao.md) (in Portuguese).
