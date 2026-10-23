# Publicar no GitHub (passo a passo)

Tudo abaixo já foi preparado localmente; **nada** foi enviado. Cada passo cria algo público ou irreversível, por isso é seu o comando.

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
