# The Blueprint v0.2.2 (beta)

[Português](../pt-BR/interno/release-v0.2.2.md)

## What is new

- **An invite link opens the installed app first.** The installer registers the `blueprint://` protocol. When a guest opens an invite link, the page tries the installed app (the browser and the app each ask once, and the app shows the address it is opening); if the app is not installed or does not answer in 2.5 s, the web version continues in the same tab. The app shows the shared workspace in a window of its own. See [Share from the app](../compartilhar.md).
- **A guest's Claude connects from that window.** On My key, **Connect my Claude on this computer** does what `blueprint conectar` does, after a native question with the address, with no command to copy.
- **Progress window when the app updates itself.**

## How to install

Download `Blueprint-0.2.2-instalador.exe` below (it installs for your user only). Anyone on 0.1.2 or later gets the update by itself and installs it with one click on Update. The `SHA256SUMS` carries the installer's hash. The `blueprint://` protocol is registered by the installer, so update (or reinstall) to get the invite flow.

The installer is not signed: Windows or Defender may warn or block on the first run.
