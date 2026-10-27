# Celular (iOS e Android): viabilidade

[Voltar ao README](../../README.pt-BR.md) · [English](../mobile.md)

Um levantamento antes de qualquer código: o que o The Blueprint faz no celular, qual caminho técnico serve e como o celular chega ao
desktop com segurança. Código lido em `f65fb76` (versão 0.3.0); fontes da web consultadas em 2026-10-27. A escolha do caminho é do
mantenedor (checkpoint "Caminho técnico do mobile escolhido"); as ações de implementação só são propostas depois dela.

## O que o celular faz, e o que não faz

O celular não roda agente. Ele acompanha e decide de longe. O app do desktop (ou um servidor `blueprint serve --hospedar`) continua sendo
a fonte dos dados; o celular é mais um membro do workspace, com o papel que vem do convite.

| Prioridade | Caso de uso | O que já existe no app | O que falta |
| --- | --- | --- | --- |
| 1 | Acompanhar a folha: ações, planos, checkpoints, progresso, onde está o agente | `GET /api/dados` (filtrado pelo papel), mudanças ao vivo em `GET /api/eventos` (SSE, retoma por `Last-Event-ID`), presença, `GET /api/mcp/agentes` | a folha tem `min-width: 960px` (`web/css/base.css`, `.folha`): não há layout de celular |
| 1 | Aplicar ou descartar uma proposta do agente | a fila (`GET /api/mcp/fila`, `POST /api/mcp/fila/ack`), aplicada pela página via `POST /api/mudancas`; dono e colaborador podem gravar (`docs/colaboracao.md`, tabela de permissões) | uma visão compacta da proposta (a tela Planejar tem tamanho de desktop) |
| 2 | Avisos do agente: proposta nova, tarefa concluída, bloqueio | as regras de aviso em `web/js/notificacao-regras.js` e os eventos da fila | nada chega a um celular com a página fechada: precisa de Web Push (ou push nativo) |
| 2 | Marcar os checkpoints do próprio usuário | `POST /api/mudancas` (checkpoint `feito`), dono e colaborador | alvo de toque de 44 px na folha compacta |
| 3 | Mostrar a apresentação do blueprint a alguém | apresentações (`/api/mcp/apresentacoes`, `/api/mcp/apresentacao?id=`) e convites de visitante que abrem nela (`convites.abre = 'apresentacao'`) | um layout de slide vertical |

Fora do celular: iniciar ou conversar com um agente (`/api/mcp/iniciar`, `/api/mcp/conversa`), o terminal do app, os hooks, editar ações
e planos à mão (telas Editar e Planejar), anexar protótipos, atualizações, a configuração do compartilhamento, Computer Use e voz. Essas
rotas já não existem para um convidado (`docs/compartilhar.md`), então o celular não as alcança nem por engano.

Protótipo das três telas (folha, avisos e proposta, apresentação) com 390 px de largura, nos tokens do app:
[`referencias/mobile-prototipo.html`](../../referencias/mobile-prototipo.html), também anexado à ação. Conferido no Edge headless com
emulação de celular em 390 x 844: o `scrollWidth` é igual à largura nas três telas (sem rolagem horizontal).

## Os caminhos técnicos

| | PWA (a página atual, instalável) | Tauri 2 mobile | Flutter | React Native |
| --- | --- | --- | --- | --- |
| O que se reaproveita | a página `web/` inteira, o servidor, convites, cookie, papéis, SSE | a página `web/` no WebView do sistema (WKWebView, Android WebView) e o código Rust | só servidor e API; a interface inteira é reescrita em Dart | só servidor e API; a interface é reescrita em componentes React |
| A folha em SVG e o kit do anatomy | como estão | como estão | portar, ou um WebView dentro do app | portar, ou um WebView dentro do app |
| Esforço para os casos acima (horas de agente, estimativa) | 25 a 35 (layout de celular, manifest, service worker, Web Push) | 50 a 70 (o trabalho do PWA mais a casca mobile, push nativo, publicação nas lojas) | 120 ou mais | 120 ou mais |
| Push | iOS/iPadOS 16.4 ou mais novo, só para web apps adicionados à Tela de Início, com `display: standalone`, permissão pedida num toque; Android pelo service worker ([WebKit](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/), [web.dev](https://web.dev/articles/push-notifications-overview)) | o plugin oficial só faz notificação local; push remoto (APNs, FCM) exige plugin da comunidade ou código nativo ([notificação no Tauri](https://v2.tauri.app/plugin/notification/)) | APNs/FCM por plugins (não verificado aqui) | APNs/FCM por bibliotecas; o ORCA usa tokens APNs/FCM crus e um gateway de push próprio (`mobile/src/notifications/`) |
| Tamanho | nada a instalar além do ícone | o Tauri não publica ([tamanho no Tauri](https://v2.tauri.app/concept/size/)); não verificado | app de demonstração com cerca de 5,4 MB para baixar e 13,7 MB instalado no iOS ([tamanho no Flutter](https://docs.flutter.dev/perf/app-size)) | cerca de 23 MB para um APK universal "hello world" (fonte secundária: [dev.to](https://dev.to/srajesh636/how-we-reduced-our-production-apk-size-by-70-in-react-native-1lci)) |
| Ferramentas | nenhuma nova | o iOS exige macOS com Xcode; o Android, Android Studio, SDK e NDK ([pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/)) | SDK do Flutter, Xcode, Android Studio | Node, Xcode, Android Studio |
| Lojas | nenhuma (instala pelo navegador) | App Store e Google Play | App Store e Google Play | App Store e Google Play |
| Maturidade | Web Push no iOS desde o 16.4 (2023) | mobile estável desde o Tauri 2.0 (2024-10-02), com ressalvas sobre plugins e CI mobile ([Tauri 2.0](https://v2.tauri.app/blog/tauri-20/)) | madura | madura (o ORCA tem um: React Native 0.83 + Expo 55) |

Custos e regras das lojas (páginas oficiais, 2026-10-27):

- Apple Developer Program: **US$ 99 por ano**; uma conta gratuita só instala nos próprios aparelhos, os apps vencem em 7 dias, sem App
  Store nem TestFlight ([o que inclui](https://developer.apple.com/programs/whats-included/), [comparar contas](https://developer.apple.com/support/compare-memberships/)).
- Google Play Console: **US$ 25 uma vez**, com verificação de identidade ([cadastro](https://support.google.com/googleplay/android-developer/answer/6112435)).
  Contas pessoais criadas depois de 2023-11-13 precisam de um teste fechado com **12 testadores por 14 dias** antes da produção
  ([requisitos de teste](https://support.google.com/googleplay/android-developer/answer/14151465)).
- Diretriz 4.2 da App Store: o app precisa ser mais que um site reempacotado, e a 4.2.3 pede que funcione sem outro app instalado. Um
  app que só funciona pareado com o nosso app do desktop é um risco de revisão para Tauri, Flutter e React Native
  ([diretrizes](https://developer.apple.com/app-store/review/guidelines/)). O PWA não passa por revisão.

O armazenamento de um web app na Tela de Início é de melhor esforço e pode ser apagado ([política de armazenamento do WebKit](https://webkit.org/blog/14403/updates-to-storage-policy/)):
aqui não importa, porque o servidor é a fonte e a página já recarrega dele.

## Conexão segura com o desktop

Regras que não mudam: `/api/mcp/*` na porta local só responde em loopback e não tem token; nada que o celular manda muda dados sem um
papel que possa gravar; o segredo do convite nunca chega ao log de um servidor.

O celular usa o que o "Compartilhar pelo app" já construiu (`docs/compartilhar.md`): um segundo listener em 127.0.0.1 onde tudo é
convidado (as rotas locais respondem 404), alcançado por HTTPS por um quick tunnel da Cloudflare ou por um endereço fixo do Tailscale
Funnel (`src/compartilhar/tailscale.rs`). O que é novo é o QR code e o layout de celular.

```mermaid
sequenceDiagram
    participant D as App do desktop (janela local)
    participant G as Listener dos convidados 127.0.0.1
    participant T as Túnel HTTPS (Cloudflare ou Tailscale)
    participant P as Celular (navegador ou PWA instalado)
    D->>D: Compartilhar: liga o túnel, cria um convite (papel, validade, usos)
    D->>D: Mostra o QR de https://<endereço>/convite#<segredo> (só na janela local)
    P->>P: Lê o QR; o segredo fica no fragmento da URL
    P->>T: POST /api/convite/conferir {segredo} (no corpo, nunca na URL)
    T->>G: repassado; o servidor compara hashes SHA-256, com limite de tentativas
    P->>T: POST /api/convite/entrar {segredo, nome}
    G-->>P: cookie bp_sessao (HttpOnly, SameSite), URL limpa
    P->>T: GET /api/dados, GET /api/eventos (SSE), POST /api/mudancas
    Note over G: hooks, iniciar, conversa, terminal, atualização: 404 nesta porta
    D->>G: Tela Pessoas: remove o membro ou revoga o convite
    G-->>P: no pedido seguinte: sessão encerrada
```

O que trafega: os dados do workspace que o papel pode ler (`GET /api/dados`, filtrado), as mudanças ao vivo (SSE), as gravações do
usuário (`POST /api/mudancas`) e, para dono e colaborador, a fila de propostas. O que não trafega: os hooks, o terminal, a conversa com o
agente, o atualizador e tudo da porta local.

Por que não a rede local direto: uma página servida por `http://<IP da rede>` não é contexto seguro, então não tem service worker nem Web
Push; o Android bloqueia HTTP sem TLS por padrão desde o Android 9 ([network security config](https://developer.android.com/privacy-and-security/security-config));
o iOS pede a permissão de rede local. Certificado autoassinado na rede local é um aviso de certificado em cada celular. O túnel dá HTTPS
sem nada disso; o custo é o tráfego passar pela Cloudflare ou pelo Tailscale.

Revogar: remover o membro na tela Pessoas encerra a sessão no pedido seguinte; revogar o convite impede novas entradas; desligar o
compartilhamento fecha o túnel. Celular perdido é tratado do mesmo jeito (remover o membro). O endereço do quick tunnel muda a cada vez,
então um PWA instalado precisa do endereço fixo (Tailscale Funnel) ou de um servidor hospedado para continuar funcionando entre reinícios.

Referência lida (não copiada): o `mobile/` do ORCA (MIT) pareia por um QR com uma oferta `orca://pair`, guarda um token por aparelho no
chaveiro, cifra de ponta a ponta (Curve25519 + XSalsa20-Poly1305) e usa um relay opcional na nuvem que exige login no ORCA. O nosso
convite + túnel HTTPS dá o mesmo "pareia uma vez, revoga por aparelho" sem relay próprio; cifrar de ponta a ponta por cima do TLS só faria
diferença se o provedor do túnel não fosse confiável.

## Recomendação

**PWA primeiro.** Reaproveita tudo de que o celular precisa (convites, papéis, cookie, SSE, o túnel), não tem taxa de loja, revisão nem
ferramenta nova, e mantém a folha em SVG e as figuras do anatomy como estão. Em ordem de risco:

1. Layout de celular da folha, da proposta e da apresentação (o protótipo acima), atrás de uma media query de largura, sem mudar o desktop.
2. QR do convite na tela Compartilhar (um codificador de QR pequeno em `web/`; hoje não há biblioteca).
3. Manifest e um service worker mínimo (instalável; sem dados offline, o servidor é a fonte).
4. Web Push para "proposta nova", "tarefa concluída", "bloqueio": o servidor precisa de chaves VAPID (assinatura ES256, uma dependência
   de criptografia nova a pesar) e de uma tabela de inscrições; push sem carga evita cifrar o conteúdo, e o service worker busca o aviso.

Reavaliar o Tauri 2 mobile só se o Web Push no iOS não bastar (ele exige a instalação na Tela de Início) ou se estar nas lojas virar
objetivo. Flutter e React Native reescrevem a interface e perdem o kit do anatomy; não recomendados.

Não verificado: tamanho de apps Tauri mobile, maturidade dos plugins de push da comunidade para o Tauri e se o iOS aplica a permissão de
rede local ao WKWebView. Nenhum muda a recomendação.
