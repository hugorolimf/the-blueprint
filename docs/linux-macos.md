# Linux and macOS: viability

[Back to the README](../README.md) · [Português](pt-BR/linux-macos.md)

A survey, not a promise: what it would take to run The Blueprint outside Windows. Nothing here ships an installer or support.
The decision to support Linux or macOS (or not) belongs to the maintainer; no implementation work starts without it.
Code read at `f65fb76` (version 0.3.0); web sources consulted on 2026-10-27.

## What already works outside Windows (measured)

- The CI `rust (ubuntu-latest)` job runs `cargo test --lib --locked` and passes (run 40504789617, commit `f65fb76`, 2026-10-27).
  So everything except the window compiles and is tested on Linux: data (SQLite), validation, HTTP server, MCP, hooks, sharing, sync.
- `wry`, `tao` and `tray-icon` sit under `[target."cfg(windows)".dependencies]` and `src/lib.rs` has `#[cfg(windows)] pub mod janela;`.
  Outside Windows, `blueprint` with no arguments prints the usage (`src/cli.rs`, `abrir_janela`); `blueprint serve` and `blueprint --mcp` run.
- Several modules already have a non-Windows branch: `src/idioma.rs` (reads `LC_ALL`/`LC_MESSAGES`/`LANG`), `src/tempo.rs` (UTC day),
  `src/atualizador/rede.rs` (`curl` from the PATH when there is no `System32\curl.exe`), `src/terminal/mod.rs` (a clear error).
- `Cargo.lock` already resolves the Linux and macOS dependencies of `wry` (webkit2gtk, gtk, objc2-web-kit), so un-gating the window does not change the lock file.

## Windows-only code

Every `cfg(windows)` site and every Windows path or program in `src/` (grep for `cfg(windows)`, `windows_sys`, `APPDATA`, `curl.exe`, `wt`, `cmd`, `.iss`).
Action: **swap** (rewrite per system), **isolate** (keep behind `cfg`, with a fallback) or **drop** (no equivalent; feature off on that system).
Hours are agent hours per system, from reading the code; Linux and macOS are close except where noted.

| Module | Windows-only piece | Outside Windows today | Action | Hours (Linux / macOS) |
| --- | --- | --- | --- | --- |
| `src/janela/mod.rs`, `equipe.rs` | WebView2 through `wry`/`tao`; `MessageBoxW` to confirm a team link | not compiled | swap: `wry`/`tao` are cross-platform; the confirmation becomes a page in the WebView | 6 / 5 |
| `src/janela/instancia.rs` | single instance with `CreateMutexW` | not compiled | swap: a lock file in the data folder, or the port file that already exists | 2 / 2 |
| `src/janela/icone.rs`, `build.rs` | icon read from the `.exe` resource (`IconExtWindows`, `GetSystemMetrics`) | not compiled | swap: embed the PNG from `recursos/icone/`; macOS also needs an `.icns` in the bundle | 1.5 / 2 |
| `src/janela/bandeja.rs` | tray through `tray-icon` | not compiled | isolate: Linux needs `libayatana-appindicator3` and, on plain GNOME, the AppIndicator extension; macOS uses the menu bar | 3 / 2 |
| `src/janela/console.rs`, `atualizando/` | `FreeConsole`; the update progress window | not compiled | drop the console part (no console subsystem); the progress window follows the updater | 0.5 / 0.5 |
| `src/janela/dev/` | file watcher for `cargo dev` | not compiled | none: plain `std::fs` | 0 / 0 |
| `src/terminal/conpty.rs` | the app's terminal on ConPTY, Job Object | the route answers "only on Windows" | swap: a PTY (`openpty`/`forkpty`) and a process group; or drop and keep only the external terminal | 8 / 8 |
| `src/mcp/iniciar.rs` | "Start with Claude" opens Windows Terminal (`wt`) or a new console | spawns without a visible terminal | swap: `x-terminal-emulator`/`gnome-terminal` on Linux, `open -a Terminal` (AppleScript) on macOS | 3 / 3 |
| `src/http/config.rs`, `equipes.rs`, `mcp/equipe.rs`, `atualizador/mod.rs`, `compartilhar/provedor.rs`, `registrar/` | data under `%APPDATA%\Blueprint` | falls back to the **current folder** (`config.rs`), which is wrong | swap: one function for the data folder (`$XDG_DATA_HOME` or `~/.local/share`, `~/Library/Application Support`) | 2.5 (both) |
| `src/registrar/mod.rs` | looks for `claude`/`codex` as `.exe`/`.cmd`/`.bat` | already searches the bare name | none beyond the data folder above | 0.5 / 0.5 |
| `src/atualizador/` | downloads `Blueprint-<v>-instalador.exe` (Inno Setup) and runs it silently | finds no asset for the system, so it never offers an update | swap: one asset per system (AppImage, `.dmg`) and an install step per system | 6 / 6 |
| `src/compartilhar/tunel.rs` | `cloudflared-windows-amd64.exe` with a fixed SHA-256, Job Object | downloads the Windows binary and fails to run it | swap: a table of asset and hash per system; kill the process group | 3 / 3 |
| `src/compartilhar/tailscale.rs`, `repositorio/acesso.rs`, `mcp/cliente_curl.rs` | only `CREATE_NO_WINDOW` | works | none | 0 / 0 |
| `src/tempo.rs` | local day from `GetLocalTime` | uses the UTC day (off by one near midnight) | swap: local time through `libc::localtime_r` | 1 / 1 |
| `instalador/blueprint.iss`, `.github/workflows/release.yml` | Inno Setup installer and release | none | swap: AppImage or `.deb` (Linux); `.app` in a `.dmg`, signed and notarized (macOS) | 8 / 10 |
| `scripts/*.mjs` (36 use `msedge.exe`) | screen checks drive Microsoft Edge | do not run | swap: find Chromium/Chrome by system | 3 (both) |

**Total, first usable build:** about **52 h for Linux** and **51 h for macOS**, plus about **6 h shared** (data folder, scripts).
Dropping the app's terminal (external terminal only) saves 8 h per system.

## The window outside Windows (CI job)

The `janela-fora-do-windows` job in `.github/workflows/test.yml` installs WebKitGTK, GTK and the tray library on Ubuntu, removes the
`cfg(windows)` gate from the window **only inside the job**, runs `cargo check --bin blueprint` on Ubuntu and macOS, prints the errors per
file and uploads the log as the `janela-Linux` and `janela-macOS` artifacts. It has `continue-on-error: true`: it records, never fixes.

Result: **pending.** The job runs on the next push to `main` (or pull request); its error list replaces the estimates of the first five
rows above. Expected from reading the code: errors only in `instancia.rs`, `icone.rs`, `equipe.rs`, `console.rs` and `atualizando/`
(the `windows_sys` imports), not in the `wry`/`tao` calls themselves.

## Platform requirements (official sources, consulted 2026-10-27)

| Point | Fact | Source |
| --- | --- | --- |
| Linux WebView | wry uses WebKitGTK; Ubuntu/Debian package `libwebkit2gtk-4.1-dev`; tao needs `libgtk-3-dev` | [wry README](https://github.com/tauri-apps/wry), [tao README](https://github.com/tauri-apps/tao) |
| WebKitGTK version | Ubuntu 22.04: 2.36 at release, 2.50 in updates; 24.04: 2.44 at release, 2.52 in updates | [Launchpad webkit2gtk](https://launchpad.net/ubuntu/+source/webkit2gtk) |
| Web features | WebKitGTK 2.36 is roughly Safari 16, so ES modules, SSE and WebGL are expected to work (not tested here) | [Tauri webview versions](https://v2.tauri.app/reference/webview-versions/), [webkitgtk.org](https://webkitgtk.org/) |
| Known Linux issue | blank window or Wayland "Error 71" with NVIDIA (DMABUF renderer); workaround `WEBKIT_DISABLE_DMABUF_RENDERER=1` | [Tauri: Linux graphics](https://v2.tauri.app/develop/debug/linux-graphics/) |
| Linux tray | `tray-icon` uses GTK with `libayatana-appindicator3`; plain GNOME needs the AppIndicator extension | [tray-icon README](https://github.com/tauri-apps/tray-icon), [GNOME extension 615](https://extensions.gnome.org/extension/615/appindicator-support/) |
| macOS WebView | WKWebView, part of the system; Tauri 2 requires macOS 10.15 or later | [wry README](https://github.com/tauri-apps/wry), [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) |
| macOS distribution | outside the App Store the app must be signed and notarized; notarization needs the Apple Developer Program (US$99 per year) | [Tauri: macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Apple Developer Program](https://developer.apple.com/programs/whats-included/) |
| Linux packaging | AppImage, `.deb`, RPM, Flatpak and Snap are the usual formats; the app does not use the Tauri bundler, so this is done by hand or with a separate tool | [Tauri: distribute](https://v2.tauri.app/distribute/) |

## Risks to the performance goals

The goals in `referencias/relatorio-medicao.md` (window visible in under 500 ms, binary under 5 MB, backend at rest under 20 MB) were
measured on Windows with WebView2. Outside Windows:

- **Binary:** the Rust side stays small, but the Linux app depends on WebKitGTK from the system (tens of MB, not in our binary). An
  AppImage that bundles it would be far above 5 MB. macOS uses the system WKWebView, so the binary goal should hold.
- **Window in 500 ms:** not measured. WebKitGTK starts its own web and network processes; the NVIDIA workaround turns off the fast
  rendering path. Needs a measurement with `scripts/medir-janela.mjs` ported to the system.
- **Tray memory:** the app destroys the WebView when it goes to the tray; that holds on both systems, since WebKit runs in separate processes.
- **Testing:** the screen checks (36 scripts) drive Edge; without porting them, a Linux or macOS release would ship with fewer checks.

## Recommendation

**Postpone, keep the door open.** The server, the MCP and the hooks already compile and pass tests on Linux, so a headless use
(`blueprint serve --hospedar` on a Linux server, the agent on any system) is supported today and costs nothing more. The desktop window
on Linux or macOS is about 50 agent hours per system plus the signing account on macOS, and adds a second WebView engine to test every
release. Worth doing when there is demand (issues from users on those systems); the cheapest first step is the shared data folder
function (2.5 h), which also helps the headless use: today `pasta_do_app()` falls back to the current folder without `APPDATA`, so the
`--mcp` pending queue, `equipes.json` and `registrar` land wherever the command ran (`blueprint serve` itself takes its paths from
`BLUEPRINT_DB` and `PROTOTIPOS_DIR`).

What the maintainer decides: support Linux, macOS, both, or neither for now (checkpoint "decided whether to support Linux and macOS").
