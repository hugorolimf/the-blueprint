# Revisão de segurança do modo hospedado

Revisão da frente TELAS (ação "Link público e painel do supervisor", plano "Revisar a segurança do modo servidor"), item por item das regras do
`CLAUDE.md` (seções "Direção: open source, auto-hospedado, colaborativo" e "Regras de segurança") e do contrato `docs/colaboracao.md`.

Estado de cada item:

- **página**: garantido pelo código da interface e provado por teste desta frente (vale igual com o binário real).
- **stub**: o comportamento esperado do servidor está no `scripts/stub-colaboracao.mjs` e os e2e o conferem; falta rodar os mesmos e2e contra o
  binário real.
- **a conferir após integrar**: depende do backend da frente ACESSO, que ainda não está neste branch. Os e2e já aceitam `--rust <exe>` e passam a
  provar o item contra o binário real (`serve --hospedar`, atrás do proxy de teste do próprio stub).

Verificações:

| Comando | O que prova | Tempo |
| --- | --- | --- |
| `node convite.test.mjs` | o segredo sai da barra antes de qualquer pedido e só vai no corpo; destino só da própria página; papel por escopo | < 1 s |
| `node pessoas.test.mjs`, `node supervisor.test.mjs` | corpo e estado dos convites, comandos da chave, lista de pessoas, resumo do supervisor | < 1 s |
| `node scripts/convite-tela-e2e.mjs [--rust exe]` | entrada pelo nome, link copiado uma vez, chave de MCP, remover derruba a sessão, uso único, segredo fora de URL e Referer | ~20 s |
| `node scripts/visitante-supervisor-e2e.mjs [--rust exe]` | visitante só lê um blueprint sem comentários, nunca grava; supervisor só lê; revogar derruba o visitante | ~20 s |
| `node scripts/seguranca-servidor-e2e.mjs [--rust exe]` | cabeçalhos, cookie, Origin, rotas locais ausentes, dono de uso único, limite de tentativas, banco sem segredo (com `--rust`) | ~15 s |

## Itens

| # | Regra (origem) | Como fica | Estado |
| --- | --- | --- | --- |
| 1 | HTTPS obrigatório, TLS no proxy reverso (CLAUDE.md) | Servidor: 403 `https-obrigatorio` sem `X-Forwarded-Proto: https` vindo do proxy confiável (seção 4). A página não monta URL absoluta: usa caminhos relativos e o link que o servidor devolve. O proxy de teste manda `https` e Host/Origin da URL pública. | a conferir após integrar (o e2e com `--rust` só passa se o binário aceitar o proxy; falta um teste negativo sem o cabeçalho, que é da ACESSO: `servidor-e2e.mjs`) |
| 2 | `Referrer-Policy: no-referrer` (CLAUDE.md) | Toda resposta leva o cabeçalho; o segredo vive no fragmento, que o navegador nunca põe no `Referer`; o iframe de protótipo já tem `referrerpolicy="no-referrer"`. `seguranca-servidor-e2e` lê o cabeçalho na API e na página; os dois e2e conferem no registro do proxy que nenhum `Referer` levou um segredo. | página + stub; a conferir após integrar |
| 3 | Segredo de convite nunca em log nem em mensagem de erro (CLAUDE.md) | `/convite#<segredo>`: o fragmento não chega ao servidor nem ao log do proxy. `convite.js` faz `history.replaceState(null, "", "/convite")` antes do primeiro pedido e manda o segredo só no corpo do POST; no fim `location.replace(destino)` (o histórico não guarda o convite). Erros mostrados na tela vêm do `codigo` (`textoDoErro`), nunca ecoam o pedido. O link com segredo aparece **uma vez**, só na resposta da criação ou da troca, e some da tela ao fechar o painel. | página (convite.test, convite-tela-e2e); o que o binário escreve no próprio log: a conferir após integrar |
| 4 | Só o hash do segredo no banco (CLAUDE.md) | Convite, sessão e chave guardados como `sha256` (seção 2). `seguranca-servidor-e2e --rust` procura cada segredo e a chave gerada nos bytes de `b.db`, `-wal` e `-shm`. | a conferir após integrar |
| 5 | Limite de tentativas com segredo inválido (CLAUDE.md) | 10 falhas por IP em 15 min: 429 `muitas-tentativas` com `params.segundos`; a tela mostra "Espere N s". O e2e faz 11 tentativas e confere o 429 e o texto. Segredo inexistente, vencido, revogado ou esgotado têm a mesma resposta (404 `convite-invalido`). | página + stub; a conferir após integrar |
| 6 | Conferência de `Origin` (CLAUDE.md) | Pedido com cookie que não é GET/HEAD precisa de `Origin` igual à origem pública (403 `origem-invalida`). O e2e manda, com o cookie da sessão, um POST de `https://mal.exemplo` e outro sem `Origin`. | stub; a conferir após integrar |
| 7 | Cookie HttpOnly + SameSite (CLAUDE.md) | `bp_sessao`, 43 caracteres, `HttpOnly; SameSite=Lax; Path=/` e `Secure` no hospedado. O e2e lê o `Set-Cookie` como o servidor mandou (antes de o proxy de teste tirar o `Secure`) e confere pelo DevTools que `document.cookie` não o vê. | página + stub; `Secure`: a conferir após integrar (só com `--rust`) |
| 8 | Segredo >= 128 bits, link revogável, trocável e com validade (CLAUDE.md) | 32 bytes do gerador do sistema (`getrandom`), base64url. Tela: validade de 1 hora a 30 dias, uma pessoa, até 10 ou sem limite, Trocar (revoga e cria igual) e Revogar com confirmação. `convite.js` só aceita fragmento base64url de 20 a 200 caracteres. | página + stub; tamanho e gerador: a conferir após integrar |
| 9 | Link do dono impresso uma vez (CLAUDE.md, seção 4) | Uso único e 24 h. O e2e abre o mesmo link numa segunda pessoa e espera "Convite indisponível". | stub; a conferir após integrar (o stub lê o link na saída do binário) |
| 10 | Toda rota checa o papel no servidor (CLAUDE.md) | A página só esconde o que o papel não usa (`poderesDaSessao`, classe `so-leitura`); quem decide é o servidor. Os e2e chamam as rotas proibidas direto: visitante `PUT /api/dados`, `POST /api/convites` e `POST /api/chave` (403); supervisor `POST /api/convites` (403); visitante lê só o blueprint do link. | stub; a conferir após integrar |
| 11 | Visitante não vê comentários internos nem a chave de ninguém (ação) | `GET /api/dados` do visitante sem `agente.comentarios`, `chats` e `usuario`; sem botão de chave; `POST /api/chave` 403. `GET /api/chave` nunca devolve a chave, só o prefixo. | stub; a conferir após integrar |
| 12 | Nome é só rótulo (CLAUDE.md) | A tela diz "não é senha"; 1 a 60 caracteres sem controle, conferido na página (`problemaDoNome`) e no servidor (`falta-nome`, `nome-invalido`). Sempre escrito com `textContent` ou `esc()`. | página + stub |
| 13 | Registro de atividade (CLAUDE.md) | Tabela `atividade` sem segredo; o painel Acompanhar mostra as últimas 20 (quem, o quê, quando). O e2e confere "entrou". | stub; a conferir após integrar |
| 14 | MCP com chave por membro, com o papel dele (CLAUDE.md) | Tela Minha chave: gerar (troca a anterior), revogar, chave e comandos só uma vez; os comandos levam a chave por `--env`, e o `--mcp` a manda pela entrada padrão do `curl` (seção 4). Supervisor só lê pelo MCP. | página; `--mcp` remoto e 401 `chave-invalida`: a conferir após integrar |
| 15 | `/api/mcp/*` só em loopback no modo local; rotas de processo não existem no hospedado (Regras de segurança) | No local nada muda (`mutacoes.mjs` já cobre). No hospedado, `iniciar`, `hook/*`, `/api/terminal/*` e `/api/atualizacao` respondem 404; o e2e confere quatro delas. | stub; a conferir após integrar |
| 16 | Protótipos em sandbox com origem opaca e CSP, sem exceção (CLAUDE.md, ação) | Nenhuma mudança de tela; `X-Frame-Options: SAMEORIGIN` só em `/prototipos/` (seção 4); visitante lê os protótipos do blueprint do link. | sem mudança (mutação existente "protótipo sem allow-same-origin"); a conferir após integrar |
| 17 | Cabeçalhos extras (seção 4) | `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` na página, `Cache-Control: no-store` na API, HSTS com `BLUEPRINT_HSTS=1`. O e2e confere os três primeiros. | stub; HSTS: a conferir após integrar |
| 18 | Nada que o agente envia altera dados sem a página aplicar (Regras de segurança) | Sem mudança: as telas de equipe não tocam a fila do MCP. | sem mudança |
| 19 | Sem sessão, nada carrega (seção 4) | `sessao.js` pergunta `/api/sessao` antes de `carregar()`; 401 leva à tela de entrada (`semSessao()`, global para a SINCRONIA chamar em 401 e em `sessao-encerrada`). Remover a pessoa ou revogar o link do visitante: ao recarregar, a página volta à entrada (e2e). | página + stub |

## Mutações (`scripts/mutacoes.mjs`)

O `mutacoes.mjs` quebra regras no código Rust e exige que algum `cargo test` falhe. As regras novas vivem no backend da ACESSO (`src/acesso/`),
que não está neste branch: escrever a mutação sem ver o trecho exato faria o script acusar "o trecho não existe mais". **A conferir após
integrar** (rebase sobre a ACESSO), uma mutação por regra, com o trecho real:

- `X-Forwarded-Proto` exigido (403 `https-obrigatorio`) e só do par confiável (loopback ou `BLUEPRINT_PROXY`);
- `Origin` conferida em pedido com cookie que não é GET/HEAD;
- `Referrer-Policy: no-referrer` em toda resposta;
- segredo guardado como `sha256` (convite, sessão, chave) e comparado pelo hash;
- limite de tentativas (10 por IP em 15 min) e resposta igual para convite inexistente, vencido, revogado ou esgotado;
- cookie com `HttpOnly`, `SameSite=Lax` e `Secure` no hospedado;
- `Contexto::pode`: visitante nunca grava, supervisor nunca grava, último dono não sai (`ultimo-dono`);
- filtro do visitante: só o blueprint do link, sem `agente.comentarios`;
- rotas só locais ausentes no hospedado.

## Pendências desta frente

- Rodar os três e2e com `--rust target/debug/blueprint.exe` depois do merge da ACESSO (e da SINCRONIA) e marcar aqui o que passou.
- Se o binário imprimir o link do dono em outro formato, ajustar a leitura em `subirStub` (procura `/convite#<segredo>` na saída).
