# Linux e macOS: viabilidade

[Voltar ao README](../../README.pt-BR.md) · [English](../linux-macos.md)

Um levantamento, não uma promessa: o que custaria rodar o The Blueprint fora do Windows. Nada aqui entrega instalador nem suporte.
A decisão de apoiar Linux ou macOS (ou não) é do mantenedor; nenhuma implementação começa sem ela.
Código lido em `f65fb76` (versão 0.3.0); fontes da web consultadas em 2026-10-27.

## O que já funciona fora do Windows (medido)

- O job de CI `rust (ubuntu-latest)` roda `cargo test --lib --locked` e passa (execução 40504789617, commit `f65fb76`, 2026-10-27).
  Ou seja, tudo menos a janela compila e é testado no Linux: dados (SQLite), validação, servidor HTTP, MCP, hooks, compartilhar, sincronia.
- `wry`, `tao` e `tray-icon` ficam em `[target."cfg(windows)".dependencies]` e `src/lib.rs` tem `#[cfg(windows)] pub mod janela;`.
  Fora do Windows, `blueprint` sem argumentos mostra o uso (`src/cli.rs`, `abrir_janela`); `blueprint serve` e `blueprint --mcp` rodam.
- Vários módulos já têm o ramo de fora do Windows: `src/idioma.rs` (lê `LC_ALL`/`LC_MESSAGES`/`LANG`), `src/tempo.rs` (dia em UTC),
  `src/atualizador/rede.rs` (`curl` do PATH quando não há `System32\curl.exe`), `src/terminal/mod.rs` (um erro claro).
- O `Cargo.lock` já resolve as dependências do `wry` no Linux e no macOS (webkit2gtk, gtk, objc2-web-kit); tirar o `cfg` da janela não muda o lock.

## Código que só roda no Windows

Todo ponto com `cfg(windows)` e todo caminho ou programa do Windows em `src/` (busca por `cfg(windows)`, `windows_sys`, `APPDATA`, `curl.exe`, `wt`, `cmd`, `.iss`).
Ação: **trocar** (reescrever por sistema), **isolar** (fica atrás do `cfg`, com alternativa) ou **descartar** (sem equivalente; recurso desligado naquele sistema).
Horas de agente por sistema, pela leitura do código; Linux e macOS ficam parecidos, salvo onde indicado.

| Módulo | Parte só do Windows | Fora do Windows hoje | Ação | Horas (Linux / macOS) |
| --- | --- | --- | --- | --- |
| `src/janela/mod.rs`, `equipe.rs` | WebView2 por `wry`/`tao`; `MessageBoxW` para confirmar o link de equipe | não compila | trocar: `wry`/`tao` são multiplataforma; a confirmação vira uma página no WebView | 6 / 5 |
| `src/janela/instancia.rs` | instância única com `CreateMutexW` | não compila | trocar: arquivo de trava na pasta de dados, ou o arquivo `porta` que já existe | 2 / 2 |
| `src/janela/icone.rs`, `build.rs` | ícone lido do recurso do `.exe` (`IconExtWindows`, `GetSystemMetrics`) | não compila | trocar: embutir o PNG de `recursos/icone/`; o macOS precisa ainda de um `.icns` no pacote | 1,5 / 2 |
| `src/janela/bandeja.rs` | bandeja pelo `tray-icon` | não compila | isolar: o Linux precisa de `libayatana-appindicator3` e, no GNOME puro, da extensão AppIndicator; o macOS usa a barra de menus | 3 / 2 |
| `src/janela/console.rs`, `atualizando/` | `FreeConsole`; a janela de progresso da atualização | não compila | descartar a parte do console (não há subsistema de console); a janela de progresso segue o atualizador | 0,5 / 0,5 |
| `src/janela/dev/` | vigia de arquivos do `cargo dev` | não compila | nada: só `std::fs` | 0 / 0 |
| `src/terminal/conpty.rs` | o terminal do app no ConPTY, Job Object | a rota responde "só no Windows" | trocar: um PTY (`openpty`/`forkpty`) e um grupo de processos; ou descartar e manter só o terminal externo | 8 / 8 |
| `src/mcp/iniciar.rs` | "Iniciar com o Claude" abre o Windows Terminal (`wt`) ou um console novo | abre sem terminal visível | trocar: `x-terminal-emulator`/`gnome-terminal` no Linux, `open -a Terminal` (AppleScript) no macOS | 3 / 3 |
| `src/http/config.rs`, `equipes.rs`, `mcp/equipe.rs`, `atualizador/mod.rs`, `compartilhar/provedor.rs`, `registrar/` | dados em `%APPDATA%\Blueprint` | cai na **pasta atual** (`config.rs`), o que está errado | trocar: uma função para a pasta de dados (`$XDG_DATA_HOME` ou `~/.local/share`, `~/Library/Application Support`) | 2,5 (os dois) |
| `src/registrar/mod.rs` | procura `claude`/`codex` como `.exe`/`.cmd`/`.bat` | já procura o nome puro | nada além da pasta de dados acima | 0,5 / 0,5 |
| `src/atualizador/` | baixa `Blueprint-<v>-instalador.exe` (Inno Setup) e o roda em silêncio | não acha asset do sistema, então nunca oferece atualização | trocar: um asset por sistema (AppImage, `.dmg`) e um passo de instalação por sistema | 6 / 6 |
| `src/compartilhar/tunel.rs` | `cloudflared-windows-amd64.exe` com SHA-256 fixo, Job Object | baixa o binário do Windows e não consegue rodá-lo | trocar: tabela de asset e hash por sistema; derrubar o grupo de processos | 3 / 3 |
| `src/compartilhar/tailscale.rs`, `repositorio/acesso.rs`, `mcp/cliente_curl.rs` | só `CREATE_NO_WINDOW` | funciona | nada | 0 / 0 |
| `src/tempo.rs` | dia local pelo `GetLocalTime` | usa o dia em UTC (erra perto da meia-noite) | trocar: hora local por `libc::localtime_r` | 1 / 1 |
| `instalador/blueprint.iss`, `.github/workflows/release.yml` | instalador e release do Inno Setup | nenhum | trocar: AppImage ou `.deb` (Linux); `.app` num `.dmg`, assinado e notarizado (macOS) | 8 / 10 |
| `scripts/*.mjs` (36 usam `msedge.exe`) | as verificações de tela dirigem o Microsoft Edge | não rodam | trocar: achar Chromium/Chrome por sistema | 3 (os dois) |

**Total, primeira versão usável:** cerca de **52 h no Linux** e **51 h no macOS**, mais cerca de **6 h em comum** (pasta de dados, scripts).
Descartar o terminal do app (só o terminal externo) economiza 8 h por sistema.

## A janela fora do Windows (job de CI)

O job `janela-fora-do-windows` em `.github/workflows/test.yml` instala WebKitGTK, GTK e a biblioteca da bandeja no Ubuntu, tira o
`cfg(windows)` da janela **só dentro do job**, roda `cargo check --bin blueprint` no Ubuntu e no macOS, mostra os erros por arquivo e
sobe o log como os artefatos `janela-Linux` e `janela-macOS`. Tem `continue-on-error: true`: registra, nunca corrige.

Resultado: **pendente.** O job roda no próximo push para o `main` (ou pull request); a lista de erros dele substitui as estimativas das
cinco primeiras linhas acima. O esperado pela leitura do código: erros só em `instancia.rs`, `icone.rs`, `equipe.rs`, `console.rs` e
`atualizando/` (os imports de `windows_sys`), não nas chamadas do `wry`/`tao` em si.

## Requisitos de cada sistema (fontes oficiais, consultadas em 2026-10-27)

| Ponto | Fato | Fonte |
| --- | --- | --- |
| WebView no Linux | o wry usa o WebKitGTK; pacote no Ubuntu/Debian `libwebkit2gtk-4.1-dev`; o tao precisa de `libgtk-3-dev` | [README do wry](https://github.com/tauri-apps/wry), [README do tao](https://github.com/tauri-apps/tao) |
| Versão do WebKitGTK | Ubuntu 22.04: 2.36 no lançamento, 2.50 nas atualizações; 24.04: 2.44 no lançamento, 2.52 nas atualizações | [Launchpad webkit2gtk](https://launchpad.net/ubuntu/+source/webkit2gtk) |
| Recursos web | o WebKitGTK 2.36 equivale mais ou menos ao Safari 16, então módulos ES, SSE e WebGL devem funcionar (não testado aqui) | [Versões de webview do Tauri](https://v2.tauri.app/reference/webview-versions/), [webkitgtk.org](https://webkitgtk.org/) |
| Problema conhecido no Linux | janela em branco ou "Error 71" no Wayland com NVIDIA (renderizador DMABUF); contorno `WEBKIT_DISABLE_DMABUF_RENDERER=1` | [Tauri: gráficos no Linux](https://v2.tauri.app/develop/debug/linux-graphics/) |
| Bandeja no Linux | o `tray-icon` usa GTK com `libayatana-appindicator3`; o GNOME puro precisa da extensão AppIndicator | [README do tray-icon](https://github.com/tauri-apps/tray-icon), [extensão GNOME 615](https://extensions.gnome.org/extension/615/appindicator-support/) |
| WebView no macOS | WKWebView, parte do sistema; o Tauri 2 exige macOS 10.15 ou mais novo | [README do wry](https://github.com/tauri-apps/wry), [Pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/) |
| Distribuição no macOS | fora da App Store o app precisa ser assinado e notarizado; notarizar exige o Apple Developer Program (US$ 99 por ano) | [Tauri: assinatura no macOS](https://v2.tauri.app/distribute/sign/macos/), [Apple Developer Program](https://developer.apple.com/programs/whats-included/) |
| Pacotes no Linux | AppImage, `.deb`, RPM, Flatpak e Snap são os formatos usuais; o app não usa o empacotador do Tauri, então isso é feito à mão ou com outra ferramenta | [Tauri: distribuir](https://v2.tauri.app/distribute/) |

## Riscos para as metas de desempenho

As metas de `referencias/relatorio-medicao.md` (janela visível em menos de 500 ms, binário abaixo de 5 MB, servidor em repouso abaixo de
20 MB) foram medidas no Windows com o WebView2. Fora do Windows:

- **Binário:** a parte Rust continua pequena, mas o app no Linux depende do WebKitGTK do sistema (dezenas de MB, fora do nosso binário).
  Um AppImage que o leve junto ficaria muito acima de 5 MB. O macOS usa o WKWebView do sistema, então a meta do binário deve valer.
- **Janela em 500 ms:** não medido. O WebKitGTK sobe seus próprios processos de web e de rede; o contorno da NVIDIA desliga o caminho
  rápido de desenho. Precisa de medição com o `scripts/medir-janela.mjs` portado para o sistema.
- **Memória na bandeja:** o app destrói o WebView ao ir para a bandeja; isso vale nos dois sistemas, porque o WebKit roda em processos à parte.
- **Testes:** as verificações de tela (36 scripts) dirigem o Edge; sem portá-las, uma versão para Linux ou macOS sairia com menos verificações.

## Recomendação

**Adiar, sem fechar a porta.** O servidor, o MCP e os hooks já compilam e passam nos testes no Linux, então o uso sem janela
(`blueprint serve --hospedar` num servidor Linux, o agente em qualquer sistema) já funciona hoje e não custa mais nada. A janela no
Linux ou no macOS custa cerca de 50 horas de agente por sistema, mais a conta de assinatura no macOS, e acrescenta um segundo motor de
WebView para testar a cada versão. Vale a pena quando houver demanda (issues de quem usa esses sistemas); o primeiro passo mais barato é
a função única da pasta de dados (2,5 h), que também ajuda o uso sem janela: hoje `pasta_do_app()` cai na pasta atual sem `APPDATA`,
então a fila de pendentes do `--mcp`, o `equipes.json` e o `registrar` vão parar onde o comando rodou (o `blueprint serve` em si pega os
caminhos de `BLUEPRINT_DB` e `PROTOTIPOS_DIR`).

O que o mantenedor decide: apoiar Linux, macOS, os dois, ou nenhum por enquanto (checkpoint "Usuário decidiu se vale apoiar Linux e macOS").
