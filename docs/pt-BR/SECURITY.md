[English](../../SECURITY.md)

# Segurança / Security

## Modelo de ameaça

O app é **local-first**: roda na sua máquina e confia em quem já alcança a sua máquina.

- **Rotas `/api/mcp/*`** só respondem em loopback (`127.0.0.1` ou `localhost`), conferem o cabeçalho `Host` e, quando existe, o `Origin`, e exigem `content-type: application/json` em todo `POST`. Elas **não têm token**: quem alcança o loopback já alcança `/api/dados`.
- **`MCP_REDE=1`** libera essas rotas para a rede. Use só em rede confiável e por sua conta e risco: sem token, qualquer máquina que alcance a porta lê e propõe alterações. Para hospedar, ponha um proxy reverso com TLS e autenticação na frente.
- **O que o agente envia não altera os seus dados sozinho:** planos são propostas que você aplica na tela; remover é um pedido que você aprova item a item; concluir um plano ou checkpoint exige evidência, que fica registrada como comentário.
- **Protótipos de ação** são HTML de terceiros (o seu agente gera). Eles são servidos com `Content-Security-Policy: sandbox allow-scripts; default-src 'none'…` e mostrados num `iframe sandbox="allow-scripts"` de **origem opaca**, sem rede e sem acesso ao app. Nunca se adiciona `allow-same-origin`. Cada arquivo tem um id de 16 caracteres hexadecimais e o servidor recusa qualquer outro caminho.
- **Arquivos estáticos:** o servidor entrega apenas uma lista fixa (`blueprint.html`, `emblemas.mjs`, `agentes-figura.mjs`, `web/**`, `kit/**`); `data.json`, o banco e o código-fonte nunca saem por HTTP.
- **Gravação:** o banco SQLite (WAL) é gravado em transação; uma gravação inválida deixa tudo como estava. O corpo de cada pedido tem limite de tamanho.
- O **MCP por stdio** (`blueprint --mcp`) é um cliente HTTP fino do servidor local; não abre portas.

Essas regras têm testes (`cargo test`) e o script `scripts/mutacoes.mjs` quebra cada uma de propósito para provar que algum teste falha.

## Reportar uma vulnerabilidade

Não abra uma issue pública. Use **GitHub → Security → Report a vulnerability** (private vulnerability reporting) neste repositório e descreva o que viu, como reproduzir e o impacto. Responderemos o quanto antes; correções saem numa nova versão e são citadas nas notas da release.

## Versões suportadas

Só a versão mais recente recebe correções de segurança.

---

**English summary.** Local-first app: `/api/mcp/*` only answers on loopback (Host/Origin checked, JSON-only POSTs, no token). `MCP_REDE=1` exposes them to the network at your own risk. Agent input never changes your data without you applying it; phase prototypes run in an opaque-origin sandboxed iframe with no network. Report vulnerabilities privately through GitHub's *Report a vulnerability*.
