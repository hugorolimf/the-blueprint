// Gera a imagem social do repositório (1280x640, a pré-visualização de links do GitHub): o ícone desenhado com o anatomy (recursos/icone/icone.svg)
// sobre o papel azul do app, com o nome e a frase do README. Usa o Edge sem janela; nada de rede.
//   node scripts/imagem-social.mjs        grava recursos/social.png e confere 1280x640 e menos de 1 MB
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const raiz = resolve(import.meta.dirname, "..");
const saida = join(raiz, "recursos", "social.png");
const edge = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edge, "precisa do Microsoft Edge");
const icone = readFileSync(join(raiz, "recursos", "icone", "icone.svg"), "utf8");
const tokens = readFileSync(join(raiz, "web", "css", "tokens.css"), "utf8"); // as cores e a fonte do app
const html = `<!doctype html><meta charset="utf-8"><style>${tokens}
html,body{margin:0;width:1280px;height:640px;overflow:hidden}
body{font-family:var(--mono);color:var(--ink);display:flex;align-items:center;gap:56px;padding:0 88px;box-sizing:border-box;
  background:linear-gradient(#ffffff1c 1px,transparent 1px) 0 0/64px 64px,linear-gradient(90deg,#ffffff1c 1px,transparent 1px) 0 0/64px 64px,
  linear-gradient(#ffffff0c 1px,transparent 1px) 0 0/16px 16px,linear-gradient(90deg,#ffffff0c 1px,transparent 1px) 0 0/16px 16px,
  radial-gradient(ellipse at 40% 40%,#1250a0 0,#0c3a78 62%,#082b5e 100%)}
.ic{width:360px;height:360px;flex:none}.ic svg{width:100%;height:100%}
h1{margin:0 0 20px;font-size:68px;letter-spacing:.04em;font-weight:700}
p{margin:0;font-size:30px;line-height:1.45;color:var(--mid);max-width:680px}
small{display:block;margin-top:28px;font-size:20px;letter-spacing:.12em;color:var(--lo)}</style>
<div class="ic">${icone}</div><div><h1>The Blueprint</h1><p>Local-first roadmap for AI coding agents: they propose, you approve, completion needs evidence.</p><small>CLAUDE CODE · CODEX · MCP · WINDOWS</small></div>`;
const tmp = mkdtempSync(join(tmpdir(), "social-"));
try {
  writeFileSync(join(tmp, "social.html"), html);
  execFileSync(edge, ["--headless=new", "--disable-gpu", "--hide-scrollbars", `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1280,640", `--screenshot=${saida}`, pathToFileURL(join(tmp, "social.html")).href], { stdio: "ignore" });
} finally { rmSync(tmp, { recursive: true, force: true }); }
const png = readFileSync(saida);
const [largura, altura] = [png.readUInt32BE(16), png.readUInt32BE(20)]; // cabeçalho IHDR do PNG
assert.deepEqual([largura, altura], [1280, 640], `dimensões ${largura}x${altura}`);
assert(statSync(saida).size < 1024 * 1024, "acima de 1 MB");
console.log(`social: recursos/social.png ${largura}x${altura}, ${Math.round(statSync(saida).size / 1024)} KB`);
