# Publicar no GitHub (passo a passo)

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
3. Crie a tag `v0.1.1` (igual à versão do Cargo). O workflow `.github/workflows/release.yml` compila, gera o instalador e o `SHA256SUMS` com `node scripts/gerar-release.mjs` e os anexa a uma release de pré-lançamento. Sem o CI: rode `node scripts/gerar-release.mjs` (arquivos em `target/release-arquivos/`; leva alguns minutos e precisa do Inno Setup 6) e `gh release create v0.1.1 target/release-arquivos/* --prerelease --generate-notes`.
4. Os apps instalados baixam sozinhos em até 24 h depois de abrirem (ou na hora, em "Verificar atualizações" na bandeja) e perguntam antes de instalar.

Autenticidade: o `SHA256SUMS` vem da mesma release do instalador, então prova que o download não corrompeu, mas não quem publicou. Assinar (ed25519 com a chave pública embutida no app, ou Authenticode no instalador) é a melhoria para quando houver uma chave.
