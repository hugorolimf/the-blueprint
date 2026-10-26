# The Blueprint v0.1.2 (beta)

[Português](pt-BR/release-v0.1.2.md)

## Fixes

- **The automatic update did not download** in the app opened from the window or the tray: `curl.exe` would not start (Windows "os error 50") because the app, having no console, had no standard output to inherit. Fixed in the download and in launching the installer.
- **An Update button next to the name**, on the Workspaces screen: it shows as "Downloading… N%" and then as "Update to X.Y.Z". "Later" hides only the notice in the corner.

## What is new

- The app records the start and end of each Claude session (the `SessionEnd` hook and a sessions route).

## How to install

Download `Blueprint-0.1.2-instalador.exe` below (it installs for your user only). **Anyone on 0.1.0 or 0.1.1 has to install this one by hand once**, because those versions cannot download updates; from 0.1.2 on the update is automatic and only installs on your click. The `SHA256SUMS` carries the installer's hash.

The installer is not signed: Windows or Defender may warn or block on the first run.
