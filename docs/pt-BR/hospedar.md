# Hospedar o The Blueprint para uma equipe

[Voltar ao README](../../README.pt-BR.md) · [English](../hospedar.md)

O mesmo binário `blueprint` que roda o app de mesa também roda um servidor de equipe: `blueprint serve --hospedar`. As pessoas entram por links
de convite (sem senha: a URL é o convite), cada uma com um papel por workspace, e o agente de cada pessoa se conecta com uma chave de MCP
própria. O app local não muda: sem `--hospedar`, nada desta página vale. Desenho e contrato: [colaboracao.md](../colaboracao.md).

## O que você precisa

- Uma máquina sempre ligada (um VPS Linux pequeno basta) e um domínio apontando para ela, por exemplo `equipe.exemplo.com`.
- O binário: no Windows, o `blueprint.exe` instalado; no Linux, compile com `cargo build --release` (o resultado é `target/release/blueprint`).
- Um proxy reverso que termina o TLS. O app nunca fala HTTPS sozinho: o [Caddy](https://caddyserver.com) pega e renova o certificado sozinho.

## Configuração

| Variável | Obrigatória | O que faz |
| --- | --- | --- |
| `BLUEPRINT_URL_PUBLICA` | sim | A origem pública, `https://equipe.exemplo.com`, sem caminho e sem barra no fim. É também a única `Origin` e o único `Host` aceitos. |
| `HOST` | não | Onde o app escuta. Padrão `127.0.0.1` (o proxy na mesma máquina). |
| `PORT` | não | Padrão `3000`. |
| `BLUEPRINT_DB` | não | O banco SQLite. Padrão `blueprint.db` na pasta atual. |
| `PROTOTIPOS_DIR` | não | Onde ficam os protótipos das ações. Padrão `prototipos`. |
| `BLUEPRINT_PROXY` | quando o proxy está em outra máquina | O IP do proxy. O app se recusa a subir com `HOST` fora de loopback sem ele. |
| `BLUEPRINT_HSTS` | não | `1` acrescenta `Strict-Transport-Security: max-age=31536000`. Ligue depois que o HTTPS funcionar. |
| `DADOS_ARQ` | não | Um `data.json` copiado para o banco vazio na primeira execução (o arquivo não muda). |

`BLUEPRINT_HOSPEDAR=1 blueprint serve` é o mesmo que `blueprint serve --hospedar`. `MCP_REDE` não tem efeito aqui: no modo hospedado o cookie
da sessão ou a chave do membro substituem a regra de loopback.

O app confia em `X-Forwarded-Proto` e `X-Forwarded-For` só vindos do proxy (par em loopback ou `BLUEPRINT_PROXY`). Pedido sem
`X-Forwarded-Proto: https` leva `403 https-obrigatorio`, até a página; `Host` diferente leva `403 host-invalido`; pedido do navegador que muda
algo precisa da `Origin` pública (`403 origem-invalida`). Toda resposta sai com `Referrer-Policy: no-referrer`, `X-Content-Type-Options: nosniff`
e `X-Frame-Options: DENY`.

## Caddy

Um `Caddyfile` mínimo (o Caddy repassa o `Host` original e põe `X-Forwarded-Proto` e `X-Forwarded-For` por padrão):

```
equipe.exemplo.com {
    reverse_proxy 127.0.0.1:3000
}
```

Com nginx, repasse os mesmos cabeçalhos à mão: `proxy_set_header Host $host;`, `proxy_set_header X-Forwarded-Proto $scheme;`,
`proxy_set_header X-Forwarded-For $remote_addr;`, e `proxy_buffering off;` para as atualizações ao vivo (server-sent events) não ficarem presas.

## Rodar como serviço

Uma unidade do systemd (`/etc/systemd/system/blueprint.service`):

```
[Unit]
Description=The Blueprint (servidor de equipe)
After=network.target

[Service]
User=blueprint
WorkingDirectory=/var/lib/blueprint
Environment=BLUEPRINT_URL_PUBLICA=https://equipe.exemplo.com
Environment=BLUEPRINT_DB=/var/lib/blueprint/blueprint.db
Environment=PROTOTIPOS_DIR=/var/lib/blueprint/prototipos
Environment=BLUEPRINT_HSTS=1
ExecStart=/usr/local/bin/blueprint serve --hospedar
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Depois, `systemctl enable --now blueprint`. No Windows, rode o mesmo comando pelo Agendador de Tarefas ao iniciar, com as variáveis definidas
para esse usuário.

## O link do dono

Na primeira subida, sem dono ainda, o servidor imprime **uma vez** um link como `https://equipe.exemplo.com/convite#...`, que vale 24 horas e um
uso só. Abra, digite o seu nome e você é dono de todos os workspaces. O segredo fica depois do `#`, que o navegador nunca manda ao servidor nem
ao log do proxy. O journal do sistema guarda essa linha impressa; por isso o link é de uso único e vence.

Perdeu antes de usar? Pare o serviço e rode uma vez `blueprint serve --hospedar --novo-link-dono`: ele revoga o link pendente e imprime outro.
Quem tem o terminal do servidor é a autoridade.

## Convites, papéis e agentes

Pelo app, um dono convida pessoas para um workspace como **dono**, **colaborador** (edita), **supervisor** (lê e acompanha) ou **visitante**
(link público, só leitura, de um blueprint). Os links vencem (de 1 a 720 horas, 168 por padrão), podem ter limite de usos, ser revogados ou
trocados. Só o SHA-256 de cada segredo é guardado; dez tentativas erradas do mesmo endereço em 15 minutos levam `429 muitas-tentativas`.

Cada membro gera uma chave de MCP na própria tela e liga o agente com o comando que ela mostra:

```
claude mcp add blueprint-equipe --env BLUEPRINT_URL=https://equipe.exemplo.com --env BLUEPRINT_CHAVE=bpk_... -- blueprint --mcp
```

O agente passa a agir com o papel desse membro (o agente de um supervisor lê, mas não grava). Para `https://` o `--mcp` usa o `curl` do sistema e
passa a chave pela entrada padrão dele, nunca na linha de comando. Gerar outra chave troca a anterior; revogar leva a `401 chave-invalida`.
Limite conhecido: os hooks do plugin do Claude Code (sessão, edições) falam com o app local e não seguem o servidor de equipe, e as rotas que
abrem processos na máquina local (terminal, "Iniciar com o Claude", conversa, atualização) não existem no modo hospedado.

## Backup

O banco é um arquivo SQLite em modo WAL. Para uma cópia consistente com o servidor rodando, use o backup online do SQLite:

```
sqlite3 /var/lib/blueprint/blueprint.db ".backup '/var/backups/blueprint-$(date +%F).db'"
```

Ou pare o serviço e copie `blueprint.db` junto com `blueprint.db-wal` e `blueprint.db-shm`. Guarde também a `PROTOTIPOS_DIR`.
`blueprint exportar <banco> <arquivo.json>` grava os planos em JSON (pessoas, convites e sessões ficam de fora).

## Atualizar

O servidor de equipe não tem atualizador automático. Faça o backup, troque o binário e reinicie o serviço. As migrações novas do banco rodam ao
subir e nunca mudam uma já aplicada.

## Testar na sua máquina

`node scripts/servidor-e2e.mjs` e `node scripts/convites-e2e.mjs` sobem um servidor hospedado com banco temporário atrás do
`scripts/proxy-teste.mjs`, um proxy de teste que faz o papel do Caddy sem TLS.

Para experimentar à mão, com duas pessoas: `node scripts/equipe-local.mjs` (depois de `cargo build`). Ele sobe uma equipe de teste com
banco próprio em `.dev/equipe` e imprime o link do dono já apontando para `http://127.0.0.1:8443`. Abra esse link num navegador, crie um
convite em Pessoas e abra o convite numa janela InPrivate ou em outro perfil do navegador: cada um guarda o próprio cookie. Opções:
`--porta N`, `--novo-link-dono` (o link do dono só vale uma vez) e `--limpar` (recomeça do zero). Só serve nesta máquina, porque não tem
TLS; para outra pessoa em outro computador, use HTTPS de verdade (Caddy, ou um túnel que entregue HTTPS com a `BLUEPRINT_URL_PUBLICA`
igual ao endereço do túnel).
