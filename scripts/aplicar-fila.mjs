// Abre a página do The Blueprint num Edge sem janela até a fila do MCP esvaziar: quem grava as marcações do agente (plano concluído, checkpoint) é a página,
// então, com o usuário longe do navegador, este script faz o papel dele. Só aplica o que é do workspace aberto.
// Uso: node scripts/aplicar-fila.mjs [workspace_id] [url]   (padrão: workspace "sm" em http://127.0.0.1:3000; espera até 30 s e mais 5 s para a página gravar)
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";

const workspace = process.argv[2] || "sm";
const url = (process.argv[3] || "http://127.0.0.1:3000").replace(/\/$/, "");
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
if (!edgeExe) throw new Error("precisa do Microsoft Edge");
const fila = async () => (await (await fetch(url + "/api/mcp/fila", { cache: "no-store" })).json()).itens.filter((i) => i.workspaceId === workspace);
const antes = await fila();
if (!antes.length) { console.log("fila vazia: nada a aplicar"); process.exit(0); }
const depuracao = await livre(), tmp = mkdtempSync(join(tmpdir(), "aplicar-fila-"));
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, `${url}/#/c/${workspace}`], { stdio: "ignore" });
try {
  let restante = antes.length;
  for (let i = 0; i < 60 && restante; i++) { await dormir(500); restante = (await fila()).length; }
  if (!restante) await dormir(5000); // a página grava no servidor com atraso (debounce): espera antes de fechar
  console.log(restante ? `ainda restam ${restante} item(ns) na fila (propostas ficam esperando o usuário aplicar na tela Planejar)` : `fila aplicada: ${antes.length} item(ns)`);
  process.exitCode = 0;
} finally { edge.kill(); setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); }
