# Colaboração em tempo real: desenho e contrato

Estado: desenho aprovado para construção (ação "Colaboração em tempo real: desenho", 2026-10-26). Este documento é o **contrato** entre as três
frentes que constroem a colaboração em paralelo (ACESSO, SINCRONIA e TELAS). O que está aqui vale mais que a descrição das ações no blueprint;
se o código precisar divergir, mude este documento no mesmo commit e avise as outras frentes.

Princípio: o app local não muda. Tudo o que é de equipe só existe no **modo hospedado** (`blueprint serve --hospedar`). No modo local o usuário
continua dono implícito, sem cookie, sem convite e com o MCP só em loopback sem token.

## 1. O que se mantém, muda ou cai da direção do CLAUDE.md

| Item da direção | Decisão | Origem no código |
| --- | --- | --- |
| Um binário, dois modos | **Muda:** o modo de equipe é `blueprint serve --hospedar` (ou `BLUEPRINT_HOSPEDAR=1`). `blueprint serve` sem a opção continua sendo o servidor local sem janela que o `dev.mjs`, os testes e os scripts usam. | `src/cli.rs` (`serve()`), `src/http/config.rs` |
| SQLite em WAL atrás de `dados/` | **Mantém.** Pessoas, convites e histórico ficam fora do data.json, como `pastas` e `linha_do_tempo`: sem chave estrangeira para a árvore (o `PUT /api/dados` apaga e recria a árvore inteira). | `src/dados/mod.rs`, `src/dados/importar.rs` (`LIMPAR`), migrações 003 e 004 |
| Papéis dono, colaborador, supervisor, visitante | **Mantém** o enum. **Muda:** o papel passa a ser por workspace (ou por blueprint), numa tabela `acessos`; `pode_ler()` sempre verdadeiro **cai**. | `src/contexto/mod.rs` |
| Toda rota recebe um Contexto | **Muda:** hoje cada rota cria `Contexto::local()` por conta própria; passa a ser resolvido uma vez no despacho e entregue à rota. | `src/http/api_dados.rs`, `src/mcp/rotas.rs` (`com_dados`) |
| Acesso por link, sem senha; nome é rótulo | **Mantém.** **Muda:** o segredo vai no **fragmento** (`/convite#<segredo>`), não no caminho: o fragmento nunca chega ao servidor nem ao log do proxy nem ao `Referer`. O "redirecionar para a URL limpa" vira `POST` + `history.replaceState`/`location.replace`. | decisão deste desenho |
| Cookie HttpOnly + SameSite | **Mantém**, com `SameSite=Lax` (Strict quebraria a primeira visita vinda de um link externo) e `Secure` no modo hospedado. | decisão deste desenho |
| Link do dono impresso uma vez | **Mantém**, válido 24 h e de uso único; novo link só por `blueprint serve --hospedar --novo-link-dono` (quem tem o terminal do servidor é a autoridade). | `src/cli.rs` |
| Limite de tentativas, só o hash, `Referrer-Policy`, `Origin` | **Mantém.** A comparação "em tempo constante" é atendida pela busca por hash (o tempo só revela algo sobre o SHA-256, não sobre o segredo). | decisão deste desenho |
| Chave de MCP por membro | **Mantém.** **Muda:** o `--mcp` hoje só fala `http://` (`cliente.rs`); para um servidor `https://` ele usa o `curl` do sistema, como o atualizador já faz, com a chave passada pela entrada padrão (nunca na linha de comando). | `src/mcp/cliente.rs`, `src/atualizador/` |
| `/api/mcp/*` só loopback, sem token | **Mantém no modo local** (inclusive `MCP_REDE`). No modo hospedado a autenticação (cookie ou chave) substitui o loopback; `iniciar`, `conversa`, `hook/*`, `/api/terminal/*`, `/api/atualizacao` e `/api/app/mostrar` **não existem** no modo hospedado. | `src/mcp/permissao.rs`, `src/mcp/rotas.rs` |
| SSE para o tempo real | **Mantém.** **Muda:** rota nova `GET /api/eventos` fora de `/api/mcp` (que é loopback), por workspace, com operações por item e `id:` para retomar pela versão. O SSE do MCP (`/api/mcp/eventos`) continua como está. | `src/mcp/eventos.rs` (`FluxoSse`, batimento de 25 s) |
| Sincronia entre janelas (26/10) | **Mantém** para o `PUT` inteiro e para a recarga longa; **evolui** para operações por item (seção 6). | `src/http/api_dados.rs`, `web/js/sincronia.js`, `web/js/rede.js` |
| Colunas `revisao` por linha (001) | **Cai** como mecanismo de conflito: o `PUT` recria as linhas e zera a revisão. O conflito usa o valor anterior (`antes`) e a tabela `historico_campos`. | `src/dados/migracoes/001_inicial.sql` |
| Fila do MCP em memória | **Muda:** cada item ganha `autor {id, nome}` e a fila é filtrada pelo que o membro pode ler; aplicar/descartar exige gravar. | `src/mcp/fila.rs` |
| Protótipos em sandbox | **Mantém**, sem exceção. | `src/http/prototipos.rs` |
| TLS no proxy reverso | **Mantém**; `X-Forwarded-Proto: https` passa a ser exigido. | decisão deste desenho |
| Supervisor "comenta" | **Cai por ora:** não existe comentário humano no app (os comentários são do agente). Fica para depois. | `web/js/rede.js` (`agente.comentarios`) |
| Troca para Postgres; licença | Postgres **fica para depois** (a camada isolada continua permitindo). A licença já foi decidida (MIT). | `LICENSE` |
| Perfil local (`arquivo.usuario.nome`) | **Mantém no modo local.** No hospedado o nome vem da sessão (`GET /api/sessao`). | `web/js/entrada.js` |

Nota: `Dados::marcar_plano`, `adicionar_comentario` e `gravar_prototipo` não sobem a versão dos dados. Hoje só os testes os chamam; a SINCRONIA
não os usa (toda gravação de item passa por `aplicar_operacoes`, que sobe a versão e avisa).

## 2. Modelo de dados

Números reservados: **005 = ACESSO** (`005_acesso.sql`), **006 = SINCRONIA** (`006_historico.sql`). Ninguém mais cria migração até as duas entrarem.
Datas em ISO UTC com milissegundos (`strftime('%Y-%m-%dT%H:%M:%fZ','now')`), como as outras tabelas. Segredos nunca são gravados: só `sha256`
em hexadecimal minúsculo.

### 005_acesso.sql (frente ACESSO)

```sql
CREATE TABLE membros (
    id          TEXT PRIMARY KEY,          -- 16 hex aleatórios
    nome        TEXT NOT NULL,             -- rótulo, 1 a 60 caracteres, sem controle; não precisa ser único
    criado_em   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    removido_em TEXT                       -- removido não entra mais; as linhas de histórico continuam com o nome
);
CREATE TABLE acessos (                     -- o papel de um membro num escopo
    membro_id    TEXT NOT NULL REFERENCES membros (id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL,            -- '*' = todos os workspaces (só o dono do servidor)
    blueprint_id TEXT NOT NULL DEFAULT '', -- '' = o workspace inteiro
    papel        TEXT NOT NULL CHECK (papel IN ('dono', 'colaborador', 'supervisor')),
    convite_id   TEXT,                     -- de qual convite veio (auditoria)
    criado_em    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (membro_id, workspace_id, blueprint_id)
);
CREATE TABLE convites (
    id           TEXT PRIMARY KEY,         -- 16 hex; é o que a tela lista e revoga
    hash         TEXT NOT NULL UNIQUE,     -- sha256 do segredo
    papel        TEXT NOT NULL CHECK (papel IN ('dono', 'colaborador', 'supervisor', 'visitante')),
    workspace_id TEXT NOT NULL,            -- '*' só no convite do dono impresso pelo servidor
    blueprint_id TEXT NOT NULL DEFAULT '', -- obrigatório para visitante
    abre         TEXT NOT NULL DEFAULT 'folha' CHECK (abre IN ('folha', 'apresentacao')),
    criado_por   TEXT,                     -- membro; NULL = impresso pelo servidor
    criado_em    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expira_em    TEXT NOT NULL,
    usos_max     INTEGER,                  -- NULL = sem limite (link público); 1 = uma pessoa
    usos         INTEGER NOT NULL DEFAULT 0,
    revogado_em  TEXT
);
CREATE TABLE sessoes (
    hash       TEXT PRIMARY KEY,           -- sha256 do valor do cookie
    membro_id  TEXT REFERENCES membros (id) ON DELETE CASCADE,
    convite_id TEXT,                       -- sessão de visitante: sem membro, presa ao convite (revogar o convite derruba)
    criada_em  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expira_em  TEXT NOT NULL,              -- membro: 30 dias; visitante: 7 dias ou o fim do convite, o que vier antes
    visto_em   TEXT NOT NULL,              -- atualizado no máximo a cada 10 min por sessão (sem gravação por pedido)
    CHECK (membro_id IS NOT NULL OR convite_id IS NOT NULL)
);
CREATE INDEX sessoes_membro ON sessoes (membro_id);
CREATE TABLE chaves_mcp (
    hash      TEXT PRIMARY KEY,            -- sha256 da chave
    membro_id TEXT NOT NULL UNIQUE REFERENCES membros (id) ON DELETE CASCADE, -- uma chave por membro; gerar outra troca
    prefixo   TEXT NOT NULL,               -- 'bpk_' + 4 primeiros caracteres, só para a tela reconhecer
    criada_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    usada_em  TEXT                         -- no máximo a cada 10 min
);
CREATE TABLE atividade (                   -- eventos de ACESSO (as mudanças de item vão para historico_campos, 006)
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    em           TEXT NOT NULL,
    membro_id    TEXT,                     -- quem fez (NULL = servidor ou visitante)
    nome         TEXT NOT NULL DEFAULT '', -- o nome na hora (o membro pode sair depois)
    tipo         TEXT NOT NULL,            -- entrou, saiu, convite-criado, convite-revogado, convite-trocado, papel-mudado,
                                           -- membro-removido, chave-criada, chave-revogada, link-dono-impresso
    workspace_id TEXT NOT NULL DEFAULT '',
    blueprint_id TEXT NOT NULL DEFAULT '',
    alvo         TEXT NOT NULL DEFAULT ''  -- id do convite ou do membro afetado; nunca segredo, chave ou conteúdo
);
CREATE INDEX atividade_ws ON atividade (workspace_id, id);
CREATE INDEX atividade_em ON atividade (em);
-- retenção a cada gravação: 90 dias e no máximo 20 000 linhas
```

As tentativas com segredo inválido ficam **só na memória** (por IP, janela de 15 min): reiniciar zera, e está tudo bem.

### 006_historico.sql (frente SINCRONIA)

Atende também à sugestão do coordenador (histórico de versões por pessoa, como no Plane): a SINCRONIA já tem o `antes` e o `depois` de cada
campo na operação, então guardar custa um `INSERT` na mesma transação. A mesma tabela responde "quem mudou este campo por último" para o aviso
de conflito, sem mapa em memória. **A tela "histórico do plano" fica fora do escopo** (ação futura); entra só a rota de leitura.

```sql
CREATE TABLE historico_campos (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    versao       INTEGER NOT NULL,         -- a versão dos dados criada pelo lote
    em           TEXT NOT NULL,
    membro_id    TEXT NOT NULL,            -- 'local' no modo local
    nome         TEXT NOT NULL DEFAULT '',
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL DEFAULT '',
    tipo         TEXT NOT NULL,            -- workspace, blueprint, acao, plano, checkpoint, prototipo
    item_id      TEXT NOT NULL,
    campo        TEXT NOT NULL,            -- nome do campo; '*' para criar, mover e apagar
    operacao     TEXT NOT NULL,            -- criar, alterar, mover, apagar
    antes        TEXT,                     -- JSON; cortado em 4 KB (com {"cortado":true})
    depois       TEXT                      -- JSON; idem
);
CREATE INDEX historico_item ON historico_campos (tipo, item_id, campo, id);
CREATE INDEX historico_em ON historico_campos (em);
-- retenção a cada gravação: 100 linhas por item e nada com mais de 180 dias. Apagar grava uma linha '*' para o item e para cada descendente.
```

O `PUT /api/dados` (importação) não grava histórico. Visitante nunca lê esta tabela.

## 3. Papéis

Modo local: `Contexto::local()` é dono de tudo e nada abaixo se aplica. Modo hospedado: o membro tem um papel por escopo (tabela `acessos`);
vale o maior papel que cobre o alvo (`'*'` > workspace > blueprint). Visitante não é membro: a sessão dele carrega o convite (um blueprint, só leitura).

| Recurso (rota) | Dono | Colaborador | Supervisor | Visitante |
| --- | --- | --- | --- | --- |
| Ler blueprints, ações, planos, checkpoints (`GET /api/dados`, filtrado) | ler | ler | ler | ler só o blueprint do link |
| Comentários do agente (`agente.comentarios`), `chats`, `usuario` | ler | ler | ler | **não** (removidos da resposta) |
| Gravar itens (`POST /api/mudancas`) | gravar | gravar | não | não |
| Criar/apagar blueprint no workspace | gravar | gravar | não | não |
| Criar/apagar workspace, campos do workspace raiz | administrar (`'*'`) | não | não | não |
| `PUT /api/dados` inteiro (importação) | só dono `'*'` | não (`use-mudancas`) | não | não |
| Fila do MCP: ver / aplicar / descartar | ler / gravar | ler / gravar | ler | não |
| MCP com chave: ler, propor, marcar, protótipo | sim | sim | só ler | sem chave |
| Protótipos: ver / anexar / remover | ler / gravar | ler / gravar | ler | ler os do blueprint |
| Linha do tempo do agente, histórico de campos | ler | ler | ler | não |
| Presença: aparecer e ver | sim | sim | sim | não |
| Pessoas do workspace: listar / mudar papel / remover | administrar | não | listar | não |
| Convites: criar / listar / revogar / trocar | administrar | não | não | não |
| Atividade de acesso (`GET /api/atividade`) | ler | não | ler | não |
| Chave de MCP própria (`/api/chave`) | sim | sim | sim | não |
| Terminal, iniciar, hooks, atualização, mostrar janela | só modo local | — | — | — |

Regras: nunca remover nem rebaixar o último dono de um workspace (`ultimo-dono`); o dono de um workspace convida dono, colaborador, supervisor
e visitante desse workspace; só o dono `'*'` cria workspace. A função única de decisão é `Contexto::pode(Acao, workspace, blueprint)` (seção 7).

## 4. Fluxos de acesso (modo hospedado)

**Configuração.** `blueprint serve --hospedar` lê `BLUEPRINT_URL_PUBLICA` (obrigatória, `https://...`, sem barra no fim), `HOST` (padrão
`127.0.0.1`), `PORT`, `BLUEPRINT_DB`, `PROTOTIPOS_DIR`, `BLUEPRINT_PROXY` (IP do proxy quando ele não está na mesma máquina) e `BLUEPRINT_HSTS=1`
(opcional). Recusa subir com `HOST` fora de loopback sem `BLUEPRINT_PROXY`. Sem janela, bandeja, atualizador, terminal, pendentes nem hooks.

**HTTPS e cabeçalhos.** O app confia em `X-Forwarded-Proto` e `X-Forwarded-For` só quando o par da conexão é loopback ou `BLUEPRINT_PROXY`.
Pedido sem `X-Forwarded-Proto: https` leva 403 `https-obrigatorio` (inclusive a página). `Host` diferente do host da URL pública: 403
`host-invalido`. Todo pedido que não é `GET`/`HEAD` e se autentica por cookie precisa de `Origin` igual à origem pública (403 `origem-invalida`);
pedido com chave (agente) não tem `Origin` e não precisa (o navegador nunca manda a chave sozinho). Toda resposta leva `Referrer-Policy:
no-referrer`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` (`SAMEORIGIN` em `/prototipos/`), `Cache-Control: no-store` em `/api/*`
e `Strict-Transport-Security: max-age=31536000` com `BLUEPRINT_HSTS=1`. IP do cliente: último endereço de `X-Forwarded-For` se o par é o proxy,
senão o par.

**Segredos.** Convite, sessão e chave: 32 bytes do gerador do sistema (crate `getrandom`, a única dependência nova; `RandomState` não serve),
em base64url sem `=` (43 caracteres). Chave com prefixo `bpk_`. No banco só `sha256`. Nunca em log, em mensagem de erro nem em `Location`.

**Link do dono.** Ao subir, se não há acesso `dono` em `'*'` nem convite de dono pendente, cria um (24 h, uso único) e imprime **uma vez**
`<URL_PUBLICA>/convite#<segredo>`. Se há convite pendente, imprime só "o link do dono já foi impresso; para um novo use --novo-link-dono" (que
revoga o pendente e imprime outro). Atividade `link-dono-impresso`. Aviso no guia: o journal do sistema guarda essa linha; por isso uso único e 24 h.

**Entrada pelo link.**
1. A pessoa abre `https://equipe.exemplo.com/convite#<segredo>`. O servidor entrega a página (`/convite` serve `blueprint.html`); o segredo não sai do navegador.
2. A página (TELAS) lê o fragmento, apaga-o da barra (`history.replaceState(null, "", "/convite")`) e chama `POST /api/convite/conferir {segredo}`.
3. Resposta com papel, workspace e blueprint. Membro: a página pede o nome. Visitante: segue direto.
4. `POST /api/convite/entrar {segredo, nome?}`: o servidor confere de novo, cria o membro (ou reaproveita o da sessão atual e só soma o acesso),
   cria a sessão, soma 1 em `usos`, grava a atividade `entrou` e responde `Set-Cookie` + `{ok, destino}`.
5. A página faz `location.replace(destino)` (`/#/c/<workspace>`, as rotas que a página já usa; visitante: `/#/p/<blueprint>`): a URL fica limpa e o histórico não guarda o convite.

Segredo inexistente, vencido, revogado ou esgotado: a mesma resposta, 404 `convite-invalido`. Cada falha conta: 10 por IP em 15 min, ou 200 no
total em 15 min, e as rotas de convite respondem 429 `muitas-tentativas` com `params.segundos`.

**Cookie.** `bp_sessao=<43 caracteres>; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000; Secure`. Sair (`POST /api/sessao/sair`) apaga a sessão e
devolve o cookie vencido. Revogar convite derruba as sessões de visitante dele; remover membro derruba sessões e chave dele; o SSE dessa pessoa
recebe `sessao-encerrada` e fecha.

**Chave de MCP.** A pessoa gera na tela (`POST /api/chave`), vê a chave uma vez e copia o comando:
`claude mcp add blueprint-equipe --env BLUEPRINT_URL=https://equipe.exemplo.com --env BLUEPRINT_CHAVE=bpk_... -- blueprint --mcp`
(Codex: `codex mcp add` com os mesmos `--env`). O `--mcp` manda `Authorization: Bearer <chave>` em todo pedido. Para `https://` chama
`curl --silent --show-error --config -` e escreve `url`, `request`, `header` e `data` na entrada padrão do curl (a chave nunca vai para os
argumentos do processo). Chave errada ou revogada: 401 `chave-invalida`. Limite conhecido: os hooks do plugin apontam para o app local
(`127.0.0.1:3000`) e não seguem para o servidor hospedado.

## 5. API para a frente TELAS

Erros sempre como hoje: `{"erro": "<texto no idioma do pedido, tr!>", "codigo": "<estável>", "params": {...}?}`. A página traduz por `codigo`.
Códigos novos: `sem-sessao` (401), `chave-invalida` (401), `proibido` (403, já existe), `https-obrigatorio`, `host-invalido`, `origem-invalida`
(403), `so-servidor` e `so-local` (404: a rota não existe neste modo), `convite-invalido` (404), `muitas-tentativas` (429, `params.segundos`),
`falta-nome`, `nome-invalido` (400, `params.max: 60`), `papel-invalido`, `validade-invalida`, `usos-invalidos` (400), `ultimo-dono` (409),
`use-mudancas` (403), `operacao-invalida` (400, `params.indice`, `params.motivo`).

| Rota | Modo | Corpo | Resposta 200 | Erros |
| --- | --- | --- | --- | --- |
| `GET /api/sessao` | ambos | — | local: `{"modo":"local"}`; hospedado: `{"modo":"servidor","membro":{"id","nome"}\|null,"visitante":bool,"acessos":[{"workspaceId","blueprintId","papel","abre"}]}` | 401 `sem-sessao` |
| `POST /api/convite/conferir` | servidor | `{"segredo"}` | `{"papel","workspace":{"id","nome"},"blueprint":{"id","nome"}\|null,"abre","pedeNome":bool}` | 404 `convite-invalido`, 429 |
| `POST /api/convite/entrar` | servidor | `{"segredo","nome"?}` | `{"ok":true,"destino":"/#/c/<ws>"}` (visitante: `/#/p/<bp>`) + `Set-Cookie` | 400 `falta-nome`/`nome-invalido`, 404, 429 |
| `POST /api/sessao/sair` | servidor | `{}` | `{"ok":true}` + cookie vencido | — |
| `GET /api/pessoas?workspace=` | servidor | — | `{"membros":[{"id","nome","papel","blueprintId","ultimaAtividade","voce":bool}]}` (`ultimaAtividade` = maior `visto_em`) | 403 |
| `POST /api/pessoas/papel` | servidor | `{"workspaceId","membroId","papel"}` | `{"ok":true}` | 400 `papel-invalido`, 403, 404, 409 `ultimo-dono` |
| `POST /api/pessoas/remover` | servidor | `{"workspaceId","membroId"}` | `{"ok":true}` | 403, 404, 409 `ultimo-dono` |
| `GET /api/convites?workspace=` | servidor | — | `{"convites":[{"id","papel","blueprintId","abre","criadoPor":{"id","nome"}\|null,"criadoEm","expiraEm","usosMax","usos","revogado":bool}]}` | 403 |
| `POST /api/convites` | servidor | `{"workspaceId","blueprintId"?,"papel","abre"?,"validadeHoras"(1 a 720, padrão 168),"usosMax"(1 a 1000 ou null; padrão 1, visitante null)}` | 201 `{"convite":{...},"link":"https://.../convite#<segredo>"}` (o link só aparece aqui) | 400, 403 |
| `POST /api/convites/revogar` | servidor | `{"id"}` | `{"ok":true}` | 403, 404 |
| `POST /api/convites/trocar` | servidor | `{"id"}` | 201 `{"convite","link"}` (revoga o antigo, mesmos parâmetros) | 403, 404 |
| `GET /api/chave` | servidor | — | `{"chave":{"prefixo","criadaEm","usadaEm"}\|null}` | 401, 403 (visitante) |
| `POST /api/chave` | servidor | `{}` | 201 `{"chave":"bpk_...","prefixo","comandos":{"claude","codex"}}` (troca a anterior) | 401, 403 |
| `POST /api/chave/revogar` | servidor | `{}` | `{"ok":true}` | 401 |
| `GET /api/atividade?workspace=&limite=` | servidor | — | `{"itens":[{"em","membro":{"id","nome"}\|null,"tipo","blueprintId","alvo"}]}` (limite 1 a 500, padrão 100) | 403 |
| `GET /api/historico?workspace=&blueprint=&item=` | ambos | — | `{"itens":[{"versao","em","membro":{"id","nome"},"tipo","itemId","campo","operacao","antes","depois"}]}` (SINCRONIA; sem tela por ora) | 403 |
| `POST /api/mudancas`, `GET /api/eventos`, `POST /api/presenca` | ambos | seção 6 | seção 6 | seção 6 |

O link público (ação "Link público e painel do supervisor") é um convite com `papel: "visitante"`, `blueprintId` e `abre`. O painel do supervisor
não ganha rota própria: usa `/api/dados` (filtrado), `/api/pessoas` e `/api/atividade`. Toda rota de servidor em modo local responde 404
`so-servidor`; `GET /api/sessao`, `/api/historico` e as rotas da seção 6 existem nos dois modos.

## 6. Sincronização entre pessoas

**Versão.** Continua um contador global (`Estado::versao_dados`, começa na hora de abertura): sobe 1 por lote de operações ou por `PUT`. A página
guarda a última versão vista.

**Operações.** `POST /api/mudancas` (corpo até 2 MB, até 500 operações):

```json
{ "base": 1730000000123, "cliente": "c-k3j9x2", "ops": [
  { "op": "criar",   "tipo": "plano", "id": "p8x2kq1", "pai": "a1", "posicao": 3, "valor": { "texto": "Novo", "feito": false } },
  { "op": "alterar", "tipo": "plano", "id": "p1", "campos": { "texto": "Título novo" }, "antes": { "texto": "Título velho" } },
  { "op": "mover",   "tipo": "plano", "id": "p1", "pai": "a2", "posicao": 0 },
  { "op": "apagar",  "tipo": "acao",  "id": "a9" }
] }
```

- `tipo`: `workspace`, `blueprint`, `acao`, `plano`, `checkpoint`, `prototipo`. O workspace raiz é `sm` (campos `empresa`, `titulo`, `rev`...).
- `pai`: workspace do blueprint, blueprint da ação, ação do plano, checkpoint ou protótipo. `posicao` é o índice entre os irmãos (cortado ao tamanho).
- `valor` de `criar` é o item no formato do data.json e pode trazer os filhos (subárvore). `campos` de `alterar` aceita qualquer chave do item
  **menos** `id` e as listas filhas (`blueprints`, `acoes`, `planos`, `checkpoints`, `prototipos`, `workspaces`). Campos inteiros são a unidade:
  `projetos` (no blueprint) e `agente` (no plano, com `comentarios`, que vão para a tabela `comentarios`) se trocam por inteiro.
- `antes` traz o valor que a página tinha (na base) para cada campo de `campos` (ausente = `null`).
- O lote é **uma transação**. Operação malformada, de tipo desconhecido ou sem permissão derruba o lote inteiro (400 `operacao-invalida` ou 403
  `proibido`, com `params.indice`). Conflito e item apagado **não** derrubam: aparecem na resposta.

Resposta 200:

```json
{ "versao": 1730000000201,
  "conflitos": [ { "indice": 1, "tipo": "plano", "id": "p1", "campo": "texto", "deles": "Título da Ana", "seu": "Título novo",
                   "quem": { "id": "m1", "nome": "Ana" }, "em": "2027-01-15T10:00:00.000Z" } ],
  "ignoradas": [ { "indice": 3, "codigo": "item-apagado", "tipo": "plano", "id": "p9", "quem": { "id": "m1", "nome": "Ana" } } ] }
```

**Regra de conflito, campo a campo.** Para cada campo de `alterar`: se o valor atual no servidor é igual a `antes`, aplica sem aviso. Se é
diferente (alguém mudou depois da base da página) e diferente do novo, **a última gravação do campo vence** (aplica o novo) e a resposta lista o
conflito com o valor que foi sobrescrito e `quem` (a última linha de `historico_campos` daquele campo; `null` se foi um `PUT`, um script ou o
banco antigo). Item (ou pai) que não existe mais: a operação é ignorada com `item-apagado` ou `pai-apagado` e `quem` da linha de apagar. `criar`
com id que já existe: ignorada com `id-existe` (a página gera outro id e tenta de novo). Dois `mover` do mesmo item: vale o último. Sem edição
cooperativa de texto: um campo de texto é um valor só.

**Cenários (com o resultado definido; cada um vira teste de `aplicar_operacoes` e do e2e de duas pessoas):**

1. *Campos diferentes do mesmo plano.* Ana muda `texto`, Bruno muda `descricao`, ambos sobre a versão 100. Ana grava (101), Bruno grava (102) com
   `antes.descricao` igual ao atual. Resultado: o plano fica com o texto da Ana e a descrição do Bruno; ninguém recebe aviso.
2. *O mesmo campo do mesmo plano.* Ana muda `texto` para "A" (101). Bruno, ainda na 100, manda `texto: "B"` com `antes: "velho"`. O servidor
   aplica "B" (102) e responde a Bruno um conflito com `deles: "A"`, `quem: Ana`; a página do Bruno avisa "Ana tinha mudado o título; ficou o seu"
   com o botão Desfazer (que manda outro `alterar` com "A"). A página da Ana recebe a operação pelo SSE, com `autor: Bruno`, e avisa "Bruno mudou
   o título que você editou". Resultado: "B".
3. *Apagar contra editar.* Ana apaga a ação a9 (101). Bruno, na 100, altera o plano p9 dessa ação. A operação do Bruno é ignorada com
   `item-apagado`, `quem: Ana`; a página dele avisa "Ana apagou este item; sua mudança não foi gravada" e oferece copiar o texto que ele tinha
   escrito. Resultado: a ação continua apagada. (Diferente do `sincMesclar` de hoje, em que editar vence apagar: com operações o servidor não tem
   mais o item inteiro para ressuscitar.)
4. *Duas janelas da mesma pessoa* (plano "Definir a gravação por mudança"). Igual ao cenário 2, com `quem` sendo a própria pessoa; o aviso diz
   "Você mudou isto em outra janela; ficou o desta". Teste proposto: `aplicar_operacoes` com o mesmo membro e dois `cliente` (Rust) e o e2e
   `scripts/duas-pessoas-e2e.mjs` abrindo duas abas do mesmo membro além das duas pessoas.

**Como a página gera as operações** (`web/js/operacoes.js`, puro e testado). Ela continua editando o objeto `arquivo` como hoje; ao salvar, em
vez de mandar o arquivo inteiro, calcula `opsEntre(base, atual)` comparando por id a base (`ultimo`) com a tela: item novo vira `criar` (o mais
alto da subárvore), item sumido vira `apagar` (o mais alto), campo diferente vira `alterar` com `antes`, pai ou ordem relativa diferente vira
`mover`. Assim nenhum ponto de edição da interface precisa mudar. Depois de 200, a base passa a ser a tela.

**Operações de outras pessoas na página.** `aplicarOp(arvore, op)` (mesmo arquivo) aplica na base **e** na tela, exceto nos campos que esta tela
mudou e ainda não mandou (tela diferente da base): esses ficam, vão no próximo lote e vencem como última gravação, com o aviso de conflito.
A página ignora o eco do próprio lote pelo `cliente`. Atualização por item na folha, nunca a folha inteira.

**Eventos.** `GET /api/eventos?workspace=<id>&desde=<versao>` (SSE, só quem lê o workspace ou o blueprint do link):

```
retry: 3000
id: 1730000000201
event: mudancas
data: {"versao":1730000000201,"workspaceId":"w","blueprintId":"b","autor":{"id":"m2","nome":"Bruno"},"cliente":"c-k3j9x2","ops":[...]}

id: 1730000000202
event: substituido
data: {"versao":1730000000202}

event: presenca
data: {"workspaceId":"w","pessoas":[{"id":"m1","nome":"Ana","blueprintId":"b","acaoId":"a1","editando":{"tipo":"plano","id":"p1"}}]}

event: recarregar
data: {"versao":1730000000300}

event: sessao-encerrada
data: {}
```

Um lote vira um evento por blueprint tocado. Quem lê o workspace inteiro recebe tudo; quem só lê um blueprint (visitante) recebe só as operações
dele, sem `agente.comentarios`, e um `mover` entre blueprints vira `recarregar` para ele. O batimento continua o do `FluxoSse` (comentário a cada
25 s): zero trabalho em repouso.

**Reconexão pela versão.** O servidor guarda na memória os últimos lotes (até 1000 ou 15 min). O navegador reenvia `Last-Event-ID` sozinho
ao reconectar; na primeira conexão a página manda `?desde=` com o `x-dados-versao` do `GET /api/dados`. Se tudo depois dessa versão está na
memória, o servidor reenvia em ordem; se não está (servidor reiniciado, fila curta, ou um `PUT` no meio), manda `recarregar` e a página faz o
`GET` inteiro e junta com `sincMesclar(base, minha, deles)`, como hoje, antes de mandar o que faltava.

**Sem rede.** A página continua editando; o salvar falha e espera. Como as operações saem da diferença entre base e tela, não há fila para
perder: ao reconectar, primeiro aplica o que perdeu (ou recarrega e junta), depois manda a diferença.

**Presença** (só memória, só modo hospedado). Entrada por (membro, cliente) com nome, workspace, blueprint, ação aberta e `editando`. Nasce
quando o SSE abre e some quando ele fecha; `POST /api/presenca {"workspaceId","blueprintId","acaoId"?,"editando":{"tipo","id"}|null}` atualiza
(a página manda ao trocar de blueprint ou ação e ao entrar e sair de um campo). `editando` vence em 120 s sem nova mensagem. O servidor manda
`presenca` (lista do workspace) no máximo a cada 500 ms por workspace. Cor: a página deriva do id do membro numa paleta de tokens do tema.
Visitante não aparece nem vê presença. No modo local a rota responde `{"ok":true}` e não há presença.

**Convivência com o PUT e o If-Match de hoje.** `PUT /api/dados` com ou sem `If-Match` continua igual para scripts, `aplicar-fila.mjs`,
importação e o modo local antigo; ele sobe a versão, limpa a cópia em memória, avisa `dados` no SSE do MCP (como hoje) e `substituido` no
`/api/eventos`, e marca o buffer (reconexão que cruza um `PUT` recebe `recarregar`). No modo hospedado o `PUT` é só do dono `'*'`; os outros
recebem 403 `use-mudancas` (um `PUT` com dados filtrados apagaria o que a pessoa não vê). `blueprint importar`/`exportar` não mudam. A página
passa a salvar por operações nos dois modos (um caminho só); `sincronia.js` fica para o `substituido`/`recarregar` e o 409 do `PUT`.

**Migração sem perder dados.** As operações gravam nas mesmas tabelas; a 006 só acrescenta `historico_campos`. Ordem: (1) servidor ganha
`/api/mudancas` e `/api/eventos` com o `PUT` intacto; (2) a página troca o salvar; (3) testes de navegador e e2e. Um binário antigo aberto num
banco novo ignora a tabela nova. Sem plano B silencioso: se o lote falha, a página mostra "erro ao salvar" (um defeito no diff não pode cair num
`PUT` que sobrescreve os outros).

## 7. Divisão de trabalho e arquivos

Três worktrees a partir do `main`. **Ordem de integração: ACESSO, depois SINCRONIA, depois TELAS**; cada frente faz rebase sobre a anterior
antes de juntar e resolve os conflitos de acréscimo mantendo os dois lados. Regras do CLAUDE.md valem para todas: 350 linhas por arquivo,
um teste por módulo, sem emojis, textos por `tr!`/`tr()` com inglês, `blueprint.html` em CRLF e editado com cirurgia, `cargo test` e os
testes curtos antes de cada commit, nada de push.

### Frente ACESSO (ações "Modo servidor para hospedar", "Membros, papéis e links de convite", "Permissões por papel em todas as rotas")

Faz: modo hospedado, HTTPS e cabeçalhos, link do dono, membros, acessos, convites, sessões, limite de tentativas, permissões por papel em todas
as rotas (inclusive `/api/mcp/*` e o filtro de `GET /api/dados`), chave de MCP e `--mcp` remoto, atividade de acesso e as rotas da seção 5 menos
`/api/historico`. Também o lado servidor do link público (convite visitante com `abre`) e do filtro do visitante.

- Novos: `src/acesso/` (`mod.rs` resolve o Contexto do pedido; `segredo.rs`; `convites.rs`; `sessoes.rs`; `chaves.rs`; `limite.rs`;
  `cabecalhos.rs`; `rotas.rs`; `filtro.rs` para o que o papel lê; `tests/`), `src/dados/membros.rs`, `src/dados/convites.rs`,
  `src/dados/sessoes.rs`, `src/dados/atividade.rs`, `src/dados/migracoes/005_acesso.sql`, `docs/hospedar.md` (inglês) e `docs/pt-BR/hospedar.md`,
  `scripts/proxy-teste.mjs`, `scripts/servidor-e2e.mjs`, `scripts/convites-e2e.mjs`.
- Dono (pode mudar): `src/contexto/`, `src/mcp/permissao.rs`, `src/mcp/cliente.rs`, `src/http/config.rs`, `src/cli.rs`, `src/http/api_dados.rs`
  (filtro do GET e `use-mudancas`), `src/mcp/fila.rs` (autor e filtro).
- Compartilhados, só acréscimo: `src/http/rotas.rs` (porta de entrada que resolve o Contexto, cabeçalhos e os ramos `/convite`, `/api/sessao`,
  `/api/convite/`, `/api/pessoas`, `/api/convites`, `/api/chave`, `/api/atividade`), `src/http/pedido.rs` (campo `ip`, códigos 401/429 em
  `codigo_do_status`, leitura de cookie), `src/http/servidor.rs` (preenche `ip`; campo novo no `Estado { ... }`), `src/http/estado.rs` (campo
  `acesso`, logo depois de `config`), `src/mcp/rotas.rs` (Contexto em `com_dados`, bloqueio das rotas só locais), `src/dados/mod.rs`,
  `src/dados/migracoes.rs` (linha 005), `src/lib.rs`, `Cargo.toml` (`getrandom`), `scripts/mutacoes.mjs` (mutações das regras novas).
- Sem catálogo web (só mensagens do servidor por `tr!`).
- Entrega para as outras frentes, **no primeiro commit**: o formato final do `Contexto` e a assinatura das rotas:

```rust
pub enum Acao { Ler, Gravar, Administrar }
pub struct Acesso { pub workspace_id: String /* '*' = todos */, pub blueprint_id: String /* '' = workspace */, pub papel: Papel, pub abre: String }
pub struct Contexto { pub usuario: String /* id do membro; "local" */, pub nome: String, pub papel: Papel /* o maior; compatível */,
                      pub acessos: Vec<Acesso>, pub local: bool, pub visitante: bool }
impl Contexto {
    pub fn local() -> Self;                                                     // continua igual por fora
    pub fn pode(&self, acao: Acao, workspace: &str, blueprint: Option<&str>) -> bool; // a função única de decisão
    pub fn exige(&self, acao: Acao, workspace: &str, blueprint: Option<&str>) -> Resultado<()>; // Erro::Proibido
}
// toda rota nova: fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta
// Acesso também expõe: fn sessao_ativa(&self, ctx: &Contexto) -> bool (lida da memória, para o SSE fechar quem foi removido)
```

### Frente SINCRONIA (ações "Gravação por mudança e conflitos", "Presença e mudanças ao vivo")

Faz: `POST /api/mudancas` com a regra de conflito, `historico_campos` e `GET /api/historico`, `GET /api/eventos` com buffer e retomada, `POST
/api/presenca`, a troca do salvar da página por operações, a aplicação ao vivo por item, a presença e o contorno do item em edição, a medição com
vários clientes.

- Novos: `src/sincronia/` (`mod.rs` com o estado em memória: buffer de lotes, assinantes e presença; `operacoes.rs` lê e valida o lote;
  `eventos.rs` o fluxo SSE com `id:` e `Last-Event-ID`; `presenca.rs`; `rotas.rs`; `tests/`), `src/dados/operacoes.rs` (aplica o lote numa
  transação; único ponto que sobe a versão por item), `src/dados/historico.rs`, `src/dados/migracoes/006_historico.sql`, `web/js/operacoes.js`
  (`opsEntre`, `aplicarOp`), `web/js/ao-vivo.js` (EventSource, retomada, envio da presença), `web/js/presenca.js` (avatares e contorno numa
  camada própria, como `claude-camada.js`, sem mexer em `render.js`), `web/css/presenca.css`, `web/i18n/en-sincronia.js`,
  `operacoes.test.mjs`, `presenca.test.mjs`, `scripts/duas-pessoas-e2e.mjs`, `scripts/reconectar-e2e.mjs`, `scripts/medir-sincronia.mjs`
  (resultado em `referencias/relatorio-medicao.md`).
- Dono (pode mudar): `web/js/rede.js`, `web/js/sincronia.js`.
- Compartilhados, só acréscimo: `src/http/rotas.rs` (ramos `/api/mudancas`, `/api/eventos`, `/api/presenca`, `/api/historico` e o limite de
  2 MB em `limite_corpo`), `src/http/estado.rs` (campo `sincronia`, logo depois de `versao_dados`), `src/http/servidor.rs` (o campo no
  `Estado { ... }`), `src/http/api_dados.rs` (depois do `notificar("dados")`: `substituido` e marca no buffer; uma ou duas linhas),
  `src/dados/mod.rs`, `src/dados/migracoes.rs` (linha 006), `src/lib.rs`, `blueprint.html` (tags de `operacoes.js`, `ao-vivo.js`,
  `presenca.js` logo depois de `rede.js`, o CSS e um `<div id="presenca">` no cabeçalho), `web/js/i18n.js` (área `"sincronia"` em `I18N.areas`).
- Antes da ACESSO entrar: as rotas recebem `&Contexto::local()` no despacho; o autor é `ctx.usuario` e o nome, `ctx.usuario`; a permissão é
  `ctx.exige_editar()`. Ao integrar (rebase sobre a ACESSO): trocar por `ctx.nome` e `ctx.exige(Acao::Gravar, ws, Some(bp))` por operação,
  filtrar os assinantes por `ctx.pode(Acao::Ler, ...)` e fechar o fluxo quando `sessao_ativa` for falso.

### Frente TELAS (ações "Tela de pessoas e convites", "Link público e painel do supervisor")

Faz: tela de entrada pelo convite (nome, papel explicado, tema do app), sessão e modo na página, painel Pessoas (lista, papel, remover),
convites (criar com papel e validade, copiar uma vez, revogar, trocar), tela da chave de MCP com os comandos, link público (diálogo de convite
visitante com "abre folha ou apresentação"), painel do supervisor e a revisão de segurança do modo hospedado.

- Novos: `web/js/sessao.js` (`GET /api/sessao`, modo, a função global `semSessao()` que leva à tela de entrada), `web/js/convite.js` (lê o
  fragmento, conferir, entrar, `replaceState`/`location.replace`), `web/js/pessoas.js` (módulo carregado por `import()` quando o painel abre),
  `web/js/pessoas-convites.js`, `web/js/chave-mcp.js`, `web/js/supervisor.js`, `web/css/pessoas.css`, `web/css/convite.css`,
  `web/i18n/en-pessoas.js`, `web/i18n/en-convite.js`, `pessoas.test.mjs`, `convite.test.mjs`, `supervisor.test.mjs`,
  `scripts/stub-colaboracao.mjs` (servidor Node falso com as rotas da seção 5, para testar antes da ACESSO), `scripts/convite-tela-e2e.mjs`,
  `scripts/visitante-supervisor-e2e.mjs`, `docs/seguranca-servidor.md` (a revisão, item por item).
- Dono (pode mudar): `web/js/entrada.js` (o perfil usa o nome da sessão no modo hospedado).
- Compartilhados, só acréscimo: `blueprint.html` (tags dos scripts novos antes de `inicio.js`, CSS, botão Pessoas e os contêineres do painel e
  da entrada), `web/js/inicio.js` (uma guarda no começo: no modo hospedado sem sessão, ou em `/convite`, mostra a entrada e não chama
  `carregar()`), `web/js/i18n.js` (áreas `"pessoas"` e `"convite"`), `scripts/mutacoes.mjs` (no fim da revisão de segurança).
- Consome só a API da seção 5 (e `GET /api/sessao` para saber o modo); testa contra `scripts/stub-colaboracao.mjs` até a ACESSO entrar e troca
  para o binário real no rebase. Não chama `/api/mudancas` nem `/api/eventos` (são da SINCRONIA).

### Pontos de integração

| Ponto | Quem entrega | Quem usa | Como, antes da integração |
| --- | --- | --- | --- |
| `Contexto` final, `Acao`, `pode`/`exige`, assinatura `rota(estado, pedido, ctx)` | ACESSO (primeiro commit) | SINCRONIA | `Contexto::local()` e `exige_editar()` |
| `sessao_ativa(ctx)` para fechar o SSE | ACESSO | SINCRONIA | sem fechamento |
| Filtro do que o papel lê (`acesso::filtro`) | ACESSO | SINCRONIA (eventos do visitante) | todo assinante lê tudo |
| `semSessao()` global | TELAS (`sessao.js`) | SINCRONIA (`rede.js`, `ao-vivo.js` em 401) | `typeof semSessao === "function" && semSessao()` |
| API da seção 5 | ACESSO | TELAS | `scripts/stub-colaboracao.mjs` |
| Nome e id do autor nas operações e na presença | ACESSO (`ctx.nome`) | SINCRONIA | `ctx.usuario` |
| Linha `I18N.areas` em `web/js/i18n.js` | SINCRONIA e TELAS | — | conflito certo de uma linha: manter todas as áreas |
| Construtor `Estado { ... }` em `servidor.rs` (uma linha só) | ACESSO e SINCRONIA | — | conflito certo de uma linha: manter os dois campos |

## 8. Diagramas

Entrada pelo link (modo hospedado):

```
 navegador                         proxy (Caddy)                  blueprint serve --hospedar
    |  GET /convite#segredo  ------->  GET /convite  ------------->  entrega blueprint.html (o #segredo não viaja)
    |  replaceState("/convite")
    |  POST /api/convite/conferir {segredo}  --- X-Forwarded-Proto: https --->  sha256 -> convites (válido? limite por IP)
    |  <-------------------------------------------------------------  {papel, workspace, pedeNome}
    |  [pessoa digita o nome]
    |  POST /api/convite/entrar {segredo, nome}  ------------------>  cria membro + acesso + sessão, usos+1, atividade "entrou"
    |  <---------------------------  Set-Cookie: bp_sessao=...; HttpOnly; SameSite=Lax; Secure  +  {destino}
    |  location.replace("/#/c/<ws>")   (URL limpa; o histórico não guarda o convite)
    |  GET /api/dados (cookie)  ----------------------------------->  Contexto do membro -> dados filtrados pelo papel
```

Uma mudança ao vivo:

```
 página da Ana                        servidor                                   página do Bruno
    | edita o título; salvar (400 ms)
    | opsEntre(base, tela) -> [alterar p1 texto, antes]
    | POST /api/mudancas {base, cliente, ops} -->  exige(Gravar) por operação
    |                                              transação: compara antes, aplica, historico_campos
    |                                              versao+1, limpa a cópia, guarda o lote no buffer
    | <-- {versao, conflitos, ignoradas}           SSE /api/eventos  -- event: mudancas (id: versao) -->  aplicarOp na base e na tela
    | base = tela                                                                                    (campo com edição local fica;
    |                                                                                                  aviso "Ana mudou o título")
    |                                     [Bruno cai da rede e volta]
    |                                              <-- reconecta com Last-Event-ID ------------------
    |                                              reenvia os lotes depois da versão, ou "recarregar" -->  GET /api/dados + sincMesclar
```
