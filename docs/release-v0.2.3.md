# The Blueprint v0.2.3 (beta)

[Português](pt-BR/release-v0.2.3.md)

## What is fixed

- **The update progress window no longer breaks the update.** In 0.2.2 the window was started with the console handles that the app had already released, so clicking Update could show nothing, end the installer with an error without reopening the app, or skip the window. The window now starts with no console handles. Tested on a real update with the real installer: success (the new version reopens and the window closes by itself in about 6 s), installer failure (the window shows the error and the previous version reopens) and a window that cannot open (the installer runs directly).

## If you are on 0.2.2

Your next update still runs the 0.2.2 window, because the window belongs to the version that is leaving. If the app closes without updating, open it again and click Update once more. From 0.2.3 on, updates use the fixed window.

## How to install

Download `Blueprint-0.2.3-instalador.exe` below (it installs for your user only). Anyone on 0.1.2 or later gets the update by itself and installs it with one click on Update. The `SHA256SUMS` carries the installer's hash.

The installer is not signed: Windows or Defender may warn or block on the first run.
