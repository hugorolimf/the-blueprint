# Updates

[Back to the README](../README.md) · [Português](pt-BR/updates.md)

The app looks for a new version on the GitHub Releases by itself (when it opens, at most once a day), **downloads the installer in the background** and checks its SHA-256 against the release's `SHA256SUMS`. **Nothing is installed without your click:** when the installer is ready a notice appears in the window (and an "Update to X…" item in the tray); only when you click **Update** does the app copy the database to `%APPDATA%\Blueprint\backups`, run the installer silently, close and open again. If there is an agent notice the screen has not applied yet, it refuses and says why (closing the app would lose it). On the command line: `blueprint atualizar` (looks for it and downloads), `blueprint atualizar --verificar` (only checks) and `blueprint atualizar --instalar` (the click, from the terminal).

The app only makes one `GET` to GitHub (no token, no identifier, no telemetry), through Windows' `curl.exe`. To turn it off: the "Check on its own when opening" item in the tray or `BLUEPRINT_SEM_ATUALIZACAO=1`; when off, the app neither checks nor downloads anything. `BLUEPRINT_ATUALIZACAO_SO_ESTAVEL=1` ignores releases marked as pre-releases.
