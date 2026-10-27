# Publicar no GitHub (passo a passo)

> Portuguese only for now (a technical reference for maintainers). Documentação só em português por enquanto.

> Estado em 2026-11: o repositório `hugorolimf/the-blueprint` já existe (passos 1 a 4 feitos). Para as próximas versões use **"Publicar uma versão nova"** no fim deste arquivo; os passos 1 a 7 ficam como histórico e para quem for publicar a partir de um clone novo.

Cada passo cria algo público ou irreversível, por isso é seu o comando.

1. **Gere o repositório limpo** (um commit, sem dados pessoais; a pasta de destino não pode existir):

   ```
   node scripts/exportar-publico.mjs ../the-blueprint-publico
   ```

   A saída mostra a varredura de dados pessoais (deve dizer 0 achados) e o `git log` com 1 commit.

2. **Crie o repositório público e faça o push** (confirme o nome e a conta; o plano sugeriu `hugorolimf/the-blueprint`):

   ```
   cd ../the-blueprint-publico
   gh repo create hugorolimf/the-blueprint --public --source . --push \
     --description "The Blueprint (beta): roadmap local-first em Rust para agentes de IA (MCP): Workspace, Blueprint, Ação, Plano e Checkpoint"
   ```

3. **Tópicos, Issues e relato privado de vulnerabilidades:**

   ```
   gh repo edit hugorolimf/the-blueprint --add-topic mcp --add-topic roadmap --add-topic claude-code --add-topic codex --add-topic rust --add-topic sqlite --enable-issues
   gh api -X PUT repos/hugorolimf/the-blueprint/private-vulnerability-reporting
   ```

4. **CI:** o push já dispara `.github/workflows/test.yml` (Rust e Node em Ubuntu e Windows). Confira com `gh run list` e `gh run watch`.

5. **Release v0.1.0 como pré-lançamento (beta).** O instalador sai de `cargo build --release` e `ISCC instalador\blueprint.iss` (Inno Setup 6.3 ou mais novo; fica em `%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe`), na pasta `target\instalador` **do repositório privado**, não da cópia pública. Teste antes (`node scripts/testar-instalador.mjs`, leva mais de 1 minuto) e, de dentro da cópia pública:

   ```
   gh release create v0.1.0 <pasta-onde-você-compilou>/target/instalador/Blueprint-0.1.0-instalador.exe --prerelease --title "The Blueprint v0.1.0 (beta)" --notes-file docs/release-v0.1.0.md
   ```

6. **Issues iniciais** (`good first issue`): `docs/issues-iniciais.md` traz o texto de cada uma; crie com `gh issue create --label "good first issue" --title … --body …`.

7. **Divulgação:** `docs/anuncio.md` tem um texto curto para adaptar ao canal que você escolher.

O repositório atual (privado, com os seus dados) continua só local e sem remote.

## Publicar uma versão nova (o que o updater do app enxerga)

O updater (`src/atualizador/`) lê a lista de releases do repositório, pega a primeira que não é rascunho, é mais nova que a instalada e traz **os dois arquivos** `Blueprint-<versão>-instalador.exe` e `SHA256SUMS`. Release sem os dois é ignorada. Pré-lançamentos contam (o app é beta), salvo `BLUEPRINT_ATUALIZACAO_SO_ESTAVEL=1`.

1. Suba a versão em `Cargo.toml` (`version = "0.1.1"`): é a que o app mostra, a que ele compara e a que o instalador recebe (`ISCC /DVersao`).
2. Commit na `main` com os testes verdes (`cargo test`, `node atualizacao.test.mjs`, `node scripts/atualizador-e2e.mjs`).
3. Crie a tag `v0.1.1` (igual à versão do Cargo). O workflow `.github/workflows/release.yml` compila, gera o instalador e o `SHA256SUMS` com `node scripts/gerar-release.mjs` e os anexa a uma release normal ("Latest", com as notas de `docs/release-<tag>.md` quando existirem). Sem o CI: rode `node scripts/gerar-release.mjs` (arquivos em `target/release-arquivos/`; leva alguns minutos e precisa do Inno Setup 6) e `gh release create v0.1.1 target/release-arquivos/* --latest --notes-file docs/release-v0.1.1.md`.
4. Os apps instalados baixam sozinhos em até 24 h depois de abrirem (ou na hora, em "Verificar atualizações" na bandeja) e perguntam antes de instalar.

Autenticidade: o `SHA256SUMS` vem da mesma release do instalador, então prova que o download não corrompeu, mas não quem publicou. Assinar (ed25519 com a chave pública embutida no app, ou Authenticode no instalador) é a melhoria para quando houver uma chave.

## Assinar as releases (proposta; espera a sua decisão)

Nada abaixo está implementado: é a preparação da ação "Releases assinadas". Criar a chave, o secret e publicar uma release assinada são passos seus.

### Opções comparadas

| Esquema | O que entra no app | Assinar no CI | Troca de chave | Limite |
| --- | --- | --- | --- | --- |
| Ed25519 puro (crate `ed25519-compact`) | 1 crate pequena, sem dependências obrigatórias | `node:crypto` já assina Ed25519: nenhum programa a instalar | lista de chaves públicas no binário | formato próprio; ninguém confere à mão sem um script nosso |
| Formato do minisign (crate `minisign-verify`) | 1 crate sem dependências, só verificação | o binário `minisign` com hash fixo, ou o mesmo formato gerado pelo `node:crypto` (Ed25519 sobre BLAKE2b-512; a conferir com `minisign -V`) | id da chave na assinatura; o "comentário confiável" assinado pode levar a versão, e uma assinatura velha não serve para outra release | 1 crate a mais (vai contra "poucas crates") |
| Atestações do GitHub (Sigstore) | verificar exige `gh attestation verify` ou uma biblioteca Sigstore, pesada e com rede | `actions/attest-build-provenance` | a do GitHub | prova de onde saiu o build, não cabe no updater offline e enxuto |
| Authenticode no instalador | sem crate nova: o `WinVerifyTrust` do `windows-sys` (mais uma feature, `Win32_Security_WinTrust`) | `signtool`, chamado pelo Inno Setup | certificado novo a cada 1 a 3 anos | custa dinheiro e identidade: certificado OV de US$ 150 a 300 por ano; o Azure Artifact Signing (cerca de US$ 9,99 por mês) só aceita pessoa física nos EUA e no Canadá (pesquisa de 26/10, `docs/interno/pesquisa-horaria.md`); o aviso do SmartScreen só some com reputação |

Tamanho do binário: não medido. A estimativa para as duas primeiras é de dezenas de KB; o plano "Decidir o esquema" mede com `cargo build --release` antes de aprovar (meta: abaixo de 5 MB).

**Recomendação:** o formato do minisign com `minisign-verify` no app, assinado no CI pelo `node:crypto` (sem programa novo) e conferível à mão por qualquer pessoa com o `minisign`. O Authenticode resolve outra coisa (o aviso do Windows) e pode vir depois, sem conflito.

### Como ficaria

1. Par de chaves gerado fora do repositório; a privada vira o secret `BLUEPRINT_CHAVE_RELEASE` do GitHub Actions e uma cópia cifrada fica com você, fora do computador de desenvolvimento. A pública entra no código do `src/atualizador/`.
2. O `release.yml` (via `scripts/gerar-release.mjs`) gera `SHA256SUMS.minisig` ao lado do `SHA256SUMS`; o segredo chega por variável de ambiente e nunca vai a log.
3. O updater baixa os três arquivos, confere a assinatura do `SHA256SUMS` antes do hash e recusa com motivo (estado "recusada"); o clique em Atualizar confere de novo.

**Decisão pendente:** a ação supõe a verificação "obrigatória só quando a release trouxer a assinatura". Isso deixa quem troca o instalador também apagar o `.minisig`. A alternativa: a primeira versão que verifica passa a exigir assinatura de toda release mais nova que ela. Recomendo a alternativa.

### Troca de chave e o pior caso

- **Troca planejada:** o app aceita até duas chaves públicas. A release N embute a atual e a próxima; a N+1 sai assinada com a próxima; a N+2 tira a antiga.
- **Chave vazada:** apague o secret, gere um par novo e publique logo uma release de ponte assinada com a chave vazada (é a única que as instalações aceitam), que só embute a nova. Até essa ponte chegar, quem tiver a chave e acesso para publicar na release consegue instalar o que quiser: o risco volta ao de hoje, não piora. Avise nas notas da release e no `SECURITY.md`, e peça a quem quiser certeza que reinstale baixando da página da release.
- **Chave perdida (sem vazamento):** sem a privada não sai release que as instalações aceitem. Com a regra recomendada acima, quem já tem a verificação precisa reinstalar à mão; guarde a cópia cifrada.
- **Instalações antigas (até a última versão sem verificação):** não conferem nada e aceitam qualquer release até atualizarem para a primeira que verifica. Essa primeira versão é a ponte; vale publicá-la cedo.
