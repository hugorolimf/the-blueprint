// Instala e remove o plugin do Claude Code com pastas temporárias (CLAUDE_CONFIG_DIR, CODEX_HOME e APPDATA), duas vezes cada, e confere que
// a configuração real da pessoa não mudou. Uso: node scripts/testar-plugin-claude.mjs [caminho do blueprint.exe] (padrão: target/debug).
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";

const exe = resolve(process.argv[2] ?? "target/debug/blueprint.exe");
assert.ok(existsSync(exe), `não achei ${exe}`);
const base = mkdtempSync(join(tmpdir(), "blueprint-plugin-"));
const env = { ...process.env, CLAUDE_CONFIG_DIR: join(base, "cfg"), CODEX_HOME: join(base, "codex"), APPDATA: join(base, "appdata") };
for (const p of [env.CLAUDE_CONFIG_DIR, env.CODEX_HOME, env.APPDATA]) mkdirSync(p, { recursive: true });
const rodar = (cmd, args) => execFileSync(cmd, args, { env, encoding: "utf8", shell: (cmd === "claude" || cmd === "codex") && process.platform === "win32" }); // claude e codex do npm são .cmd no Windows

// a configuração real é a de CLAUDE_CONFIG_DIR quando a sessão a define (o Orca define), senão a de ~/.claude
const dirReal = process.env.CLAUDE_CONFIG_DIR ?? join(homedir(), ".claude");
// sem o .claude.json: as sessões do Claude abertas o reescrevem o tempo todo (estado de uso), e isso não é o teste mexendo
const real = [join(dirReal, "settings.json"), join(dirReal, "plugins", "installed_plugins.json"), join(dirReal, "plugins", "known_marketplaces.json")];
const foto = () => real.map((f) => (existsSync(f) ? `${f} ${statSync(f).size} ${statSync(f).mtimeMs}` : `${f} -`)).join("\n");
const antes = foto();
// o Codex só entra no teste se estiver instalado; o MCP dele fica em CODEX_HOME (temporário), então a configuração real dele não é tocada
const temCodex = (() => { try { rodar("codex", ["--version"]); return true; } catch { return false; } })();
const codexMcp = () => rodar("codex", ["mcp", "list"]);

try {
  for (let vez = 1; vez <= 2; vez++) {
    const saida = rodar(exe, ["registrar"]);
    assert.match(saida, /plugin `blueprint` instalado/, `registrar #${vez}: ${saida}`);
    const lista = rodar("claude", ["plugin", "list"]);
    assert.match(lista, /blueprint@blueprint-local[\s\S]*enabled/, `plugin list #${vez}: ${lista}`);
    assert.match(lista, /blueprint-terminal@blueprint-local[\s\S]*enabled/, `o mod do terminal deve estar ligado #${vez}: ${lista}`);
    assert.match(rodar("claude", ["mcp", "list"]), /plugin:blueprint:blueprint[\s\S]*Connected/, "o MCP do plugin deve conectar");
  }
  const pasta = join(env.APPDATA, "Blueprint", "claude-plugin", "plugins", "blueprint");
  // hooks que o registrar grava: sessão, edição, subagentes (início e fim) e notificação; os de HTTP apontam para as rotas do app
  const hooks = JSON.parse(readFileSync(join(pasta, "hooks", "hooks.json"), "utf8")).hooks;
  for (const evento of ["SessionStart", "PostToolUse", "SubagentStart", "SubagentStop", "Notification"]) assert.ok(hooks[evento]?.[0]?.hooks?.[0], `hooks.json sem ${evento}`);
  assert.match(hooks.PostToolUse[0].hooks[0].url, /\/api\/mcp\/hook\/edicao$/);
  for (const evento of ["SubagentStart", "SubagentStop"]) assert.match(hooks[evento][0].hooks[0].url, /\/api\/mcp\/hook\/subagente$/, `${evento} avisa o app`);
  if (temCodex) assert.match(codexMcp(), /blueprint/, "o MCP blueprint deve estar registrado no Codex");
  assert.match(rodar("claude", ["plugin", "validate", pasta]), /Validation passed/);
  assert.match(rodar("claude", ["plugin", "details", "blueprint@blueprint-local"]), /Skills \(2\)[\s\S]*Agents \(1\)[\s\S]*MCP servers \(1\)/);
  for (let vez = 1; vez <= 2; vez++) {
    rodar(exe, ["registrar", "--remover"]);
    assert.match(rodar("claude", ["plugin", "list"]), /No plugins installed/, `remover #${vez}`);
    if (temCodex) assert.doesNotMatch(codexMcp(), /blueprint/, `o MCP não deve sobrar no Codex depois de remover #${vez}`);
    const settings = join(env.CLAUDE_CONFIG_DIR, "settings.json"); // a pasta do plugin em %APPDATA%\Blueprint é dado do app e fica (como o banco); o que não pode sobrar é o registro no Claude
    assert.doesNotMatch(existsSync(settings) ? readFileSync(settings, "utf8") : "", /blueprint/, `sobrou plugin ou marketplace na configuração do Claude depois de remover #${vez}`);
  }
  assert.equal(foto(), antes, "a configuração real do Claude mudou");
  console.log(`ok: instala, repete, valida, hooks (inclui SubagentStart/Stop), remove, repete; ${temCodex ? "MCP no Codex registrado e removido; " : "Codex ausente (pulado); "}configuração real intacta`);
} finally {
  rmSync(base, { recursive: true, force: true });
}
