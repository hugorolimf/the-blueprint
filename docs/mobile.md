# Mobile (iOS and Android): viability

[Back to the README](../README.md) · [Português](pt-BR/mobile.md)

A survey before any code: what The Blueprint does on a phone, which technical path fits, and how the phone reaches the desktop safely.
Code read at `f65fb76` (version 0.3.0); web sources consulted on 2026-10-27. The choice of path belongs to the maintainer
(checkpoint "Mobile technical path chosen"); the implementation actions are proposed only after it.

## What the phone does, and what it does not

The phone does not run agents. It follows and decides from afar. The desktop app (or a `blueprint serve --hospedar` server) stays the
source of the data; the phone is one more member of the workspace, with a role from an invite.

| Priority | Use case | What already exists in the app | Gap |
| --- | --- | --- | --- |
| 1 | Follow the sheet: actions, plans, checkpoints, progress, where the agent is | `GET /api/dados` (filtered by role), live changes on `GET /api/eventos` (SSE, resumes with `Last-Event-ID`), presence, `GET /api/mcp/agentes` | the sheet has `min-width: 960px` (`web/css/base.css`, `.folha`): no phone layout |
| 1 | Approve or discard an agent proposal | the queue (`GET /api/mcp/fila`, `POST /api/mcp/fila/ack`), applied by the page through `POST /api/mudancas`; owner and collaborator may write (`docs/colaboracao.md`, permissions table) | a compact proposal view (the Plan screen is desktop-sized) |
| 2 | Agent notices: proposal arrived, task done, blocker | the notice rules in `web/js/notificacao-regras.js` and the queue events | nothing reaches a phone whose page is closed: needs Web Push (or a native push) |
| 2 | Tick the user's own checkpoints | `POST /api/mudancas` (checkpoint `feito`), owner and collaborator | a tap target of 44 px on the compact sheet |
| 3 | Show the blueprint's presentation to someone | presentations (`/api/mcp/apresentacoes`, `/api/mcp/apresentacao?id=`) and visitor invites that open on it (`convites.abre = 'apresentacao'`) | a vertical slide layout |

Out of scope on the phone: starting or talking to an agent (`/api/mcp/iniciar`, `/api/mcp/conversa`), the app's terminal, the hooks,
editing actions and plans by hand (Edit and Plan screens), attaching prototypes, updates, the sharing settings, Computer Use and voice.
Those routes already do not exist for a guest (`docs/compartilhar.md`), so the phone cannot reach them even by mistake.

Prototype of the three screens (sheet, notices and proposal, presentation) at 390 px wide, in the app's tokens:
[`referencias/mobile-prototipo.html`](../referencias/mobile-prototipo.html), also attached to the action. Checked in headless Edge with
mobile emulation at 390 x 844: `scrollWidth` equals the viewport on all three screens (no horizontal scroll).

## The technical paths

| | PWA (the existing page, installable) | Tauri 2 mobile | Flutter | React Native |
| --- | --- | --- | --- | --- |
| What is reused | the whole `web/` page, the server, invites, cookie, roles, SSE | the `web/` page in the system WebView (WKWebView, Android WebView) and Rust code | server and API only; the whole interface is rewritten in Dart | server and API only; the interface is rewritten in React components |
| The SVG sheet and the anatomy kit | as they are | as they are | port, or a WebView inside the app | port, or a WebView inside the app |
| Effort for the cases above (agent hours, our estimate from the gaps table, not a sourced figure) | 25 to 35 (phone layout, manifest, service worker, Web Push) | 50 to 70 (the PWA work plus the mobile shell, native push, store releases) | 120 or more | 120 or more |
| Push | iOS/iPadOS 16.4 or later, only for web apps added to the Home Screen, with `display: standalone`, permission asked on a tap; Android through the service worker ([WebKit](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/), [web.dev](https://web.dev/articles/push-notifications-overview)) | the official plugin does local notifications only; remote push (APNs, FCM) needs a community plugin or native code ([Tauri notification](https://v2.tauri.app/plugin/notification/)) | FCM, with an APNs key for iOS, through the FlutterFire plugin ([Firebase: FCM in Flutter](https://firebase.google.com/docs/cloud-messaging/flutter/client)) | APNs/FCM through libraries; ORCA uses raw APNs/FCM tokens and its own push gateway (`mobile/src/notifications/`) |
| Size | nothing to install beyond the icon | not published by Tauri ([Tauri size](https://v2.tauri.app/concept/size/)); not verified | demo app about 5.4 MB download, 13.7 MB installed on iOS ([Flutter app size](https://docs.flutter.dev/perf/app-size)) | about 23 MB for a universal hello-world APK (secondary source: [dev.to](https://dev.to/srajesh636/how-we-reduced-our-production-apk-size-by-70-in-react-native-1lci)) |
| Toolchain | none new | iOS needs macOS with Xcode; Android needs Android Studio, SDK and NDK ([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)) | Flutter SDK, Xcode, Android Studio ([install Flutter](https://docs.flutter.dev/get-started/install)) | Node, JDK, Android Studio; iOS needs a Mac with Xcode ([environment setup](https://reactnative.dev/docs/set-up-your-environment)) |
| Stores | none (installed from the browser) | App Store and Google Play | App Store and Google Play | App Store and Google Play |
| Maturity | Web Push on iOS since 16.4 (2023) | mobile stable since Tauri 2.0 (2024-10-02), with caveats on plugins and mobile CI ([Tauri 2.0](https://v2.tauri.app/blog/tauri-20/)) | mature | mature (ORCA ships one: React Native 0.83 + Expo 55) |

Store costs and rules (official pages, 2026-10-27):

- Apple Developer Program: **US$99 per year**; a free account only installs on your own devices, apps expire after 7 days, no App
  Store or TestFlight ([what's included](https://developer.apple.com/programs/whats-included/), [compare memberships](https://developer.apple.com/support/compare-memberships/)).
- Google Play Console: **US$25 once**, with identity check ([registration](https://support.google.com/googleplay/android-developer/answer/6112435)).
  Personal accounts created after 2023-11-13 must run a closed test with **12 testers for 14 days** before production
  ([testing requirements](https://support.google.com/googleplay/android-developer/answer/14151465)).
- App Store Review Guideline 4.2: an app must be more than a repackaged website, and 4.2.3 asks that it work without another app
  installed. A companion that only works paired with our desktop app is a review risk for Tauri, Flutter and React Native alike
  ([guidelines](https://developer.apple.com/app-store/review/guidelines/)). The PWA does not go through review.

Storage on a Home Screen web app is best-effort and can be evicted ([WebKit storage policy](https://webkit.org/blog/14403/updates-to-storage-policy/)):
fine here, because the server is the source of truth and the page already reloads from it.

## Connecting to the desktop safely

Rules that do not change: `/api/mcp/*` on the local port answers only on loopback and has no token; nothing the phone sends changes
data without a role that may write; the invite secret never reaches a server log.

The phone uses what "Share from the app" already built (`docs/compartilhar.md`): a second listener on 127.0.0.1 where everything is a
guest (local routes answer 404), reached through HTTPS by a Cloudflare quick tunnel or a fixed Tailscale Funnel address
(`src/compartilhar/tailscale.rs`). The new parts are the QR code and the phone layout.

```mermaid
sequenceDiagram
    participant D as Desktop app (local window)
    participant G as Guest listener 127.0.0.1
    participant T as HTTPS tunnel (Cloudflare or Tailscale)
    participant P as Phone (browser or installed PWA)
    D->>D: Share: turn on the tunnel, create an invite (role, expiry, uses)
    D->>D: Show QR of https://<address>/convite#<secret> (local window only)
    P->>P: Scan the QR; the secret stays in the URL fragment
    P->>T: POST /api/convite/conferir {secret} (body, never the URL)
    T->>G: forwarded; the server compares SHA-256 hashes, rate limited
    P->>T: POST /api/convite/entrar {secret, name}
    G-->>P: cookie bp_sessao (HttpOnly, SameSite), clean URL
    P->>T: GET /api/dados, GET /api/eventos (SSE), POST /api/mudancas
    Note over G: hooks, iniciar, conversa, terminal, updates: 404 on this port
    D->>G: People screen: remove the member or revoke the invite
    G-->>P: next request: session ended
```

What travels: the workspace data the role may read (`GET /api/dados`, filtered), live changes (SSE), the user's writes
(`POST /api/mudancas`) and, for owner and collaborator, the proposal queue. What does not: the hooks, the terminal, the conversation with
the agent, the updater and anything under the local port.

Why not the local network directly: a page served over plain `http://<LAN IP>` is not a secure context, so it gets no service worker and
no Web Push; Android blocks cleartext HTTP by default from Android 9 ([network security config](https://developer.android.com/privacy-and-security/security-config));
iOS asks for the local network permission. Self-signed certificates on the LAN mean a certificate warning on every phone. The tunnel
gives HTTPS without any of that; the cost is that the traffic goes through Cloudflare or Tailscale.

Revoking: removing the member on the People screen ends the session on the next request; revoking the invite stops new entries; turning
sharing off closes the tunnel. A lost phone is handled the same way (remove the member). The quick tunnel address changes every time, so
an installed PWA needs the fixed address (Tailscale Funnel) or a hosted server to keep working across restarts.

Reference read (not copied): ORCA's `mobile/` (MIT) pairs through a QR with an `orca://pair` offer, keeps a per-device token in the
keychain, encrypts end to end (Curve25519 + XSalsa20-Poly1305) and uses an optional cloud relay that needs an ORCA sign-in. Our invite +
HTTPS tunnel gives the same "pair once, revoke per device" without a relay of our own; end-to-end encryption on top of TLS would only
matter if the tunnel provider were not trusted.

## Recommendation

**PWA first.** It reuses everything the phone needs (invites, roles, cookie, SSE, the tunnel), costs no store fee, no review and no
new toolchain, and keeps the SVG sheet and the anatomy figures as they are. In order of risk:

1. Phone layout of the sheet, the proposal and the presentation (the prototype above), behind a width media query, without changing the desktop.
2. QR of the invite on the Share screen (a small QR encoder in `web/`; no library today).
3. Manifest and a minimal service worker (installable; no offline data, the server is the source).
4. Web Push for "proposal arrived", "task done", "blocker": the server needs VAPID keys (ES256 signature, a new crypto dependency to
   weigh) and a subscription table; payload-less pushes avoid the payload encryption, the service worker fetches the notice.

Revisit Tauri 2 mobile only if Web Push on iOS proves insufficient (it needs the Home Screen install) or a store presence becomes a goal.
Flutter and React Native rewrite the interface and lose the anatomy kit; not recommended.

Not verified: Tauri mobile app sizes, the maturity of community push plugins for Tauri, and whether iOS applies the local network
permission to WKWebView. They do not change the recommendation.
