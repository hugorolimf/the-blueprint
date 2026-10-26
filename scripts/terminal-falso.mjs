// Auxiliar dos e2e de "Iniciar com o Claude": um `wt` FALSO (nenhum terminal de verdade abre) e um servidor `blueprint serve` de teste com ele no PATH.
// O `wt.cmd` falso chama o node, que grava os argumentos EXATOS que o app passou (um JSON por linha) em `chamadas.jsonl`; `chamadas()` os devolve.
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { delimiter, join, resolve } from "node:path";

export const raiz = resolve(import.meta.dirname, "..");
export const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
export const portaLivre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });

// `dir`: pasta temporária do teste. Devolve { bin, chamadas }.
export function terminalFalso(dir) {
  const bin = join(dir, "bin-falso"), log = join(dir, "chamadas.jsonl");
  mkdirSync(bin, { recursive: true });
  writeFileSync(join(bin, "grava.mjs"), `import { appendFileSync } from "node:fs";\nappendFileSync(${JSON.stringify(log)}, JSON.stringify(process.argv.slice(2)) + "\\n");\n`);
  writeFileSync(join(bin, "wt.cmd"), `@echo off\r\nnode "%~dp0grava.mjs" %*\r\n`);
  const chamadas = () => (existsSync(log) ? readFileSync(log, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l)) : []);
  return { bin, chamadas };
}

// Liga `<exe> serve` com os dados de exemplo e o `wt` falso na frente do PATH. `env` acrescenta variáveis (ex.: MCP_REDE=1). Devolve { url, porta, pid, parar }.
export async function sobeServidor(exe, dir, falso, env = {}) {
  const porta = await portaLivre();
  copyFileSync(join(raiz, "data.exemplo.json"), join(dir, "data.json"));
  const servidor = spawn(exe, ["serve"], {
    env: { ...process.env, PATH: `${falso.bin}${delimiter}${process.env.PATH}`, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos"), ...env },
    stdio: "ignore", windowsHide: true,
  });
  const url = `http://127.0.0.1:${porta}`;
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  return { url, porta, pid: servidor.pid, parar: () => { try { servidor.kill(); } catch {} } };
}
