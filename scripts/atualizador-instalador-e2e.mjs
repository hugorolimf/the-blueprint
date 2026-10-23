// A atualização de verdade, com o instalador do Inno Setup: instala a 0.1.0 numa pasta temporária, gera um instalador 0.1.1, serve-o por um "GitHub" falso e roda
// `blueprint atualizar` + `blueprint atualizar --instalar` do app INSTALADO. Confere que o instalador roda por cima, que a versão registrada muda, que o banco é copiado antes,
// e que (por causa do /ATUALIZACAO=1) o app volta a abrir sozinho. Tudo com APPDATA, CLAUDE_CONFIG_DIR e CODEX_HOME temporários; o app reaberto é encerrado no fim.
// Uso: node scripts/atualizador-instalador-e2e.mjs   (gere antes: node scripts/gerar-release.mjs; leva ~30 s; abre a janela do app por alguns segundos)
import assert from "node:assert/strict";
import { execFileSync, spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre } from "./lib-e2e-rust.mjs";

const raiz = resolve(import.meta.dirname, "..");
const instalador010 = join(raiz, "target", "instalador", "Blueprint-0.1.0-instalador.exe");
assert(existsSync(instalador010), "gere o instalador antes (node scripts/gerar-release.mjs)");
const iscc = [process.env.ISCC, join(process.env.LOCALAPPDATA || "", "Programs/Inno Setup 6/ISCC.exe"), "C:/Program Files (x86)/Inno Setup 6/ISCC.exe"].find((c) => c && existsSync(c));
assert(iscc, "Inno Setup 6 não encontrado");
const tmp = mkdtempSync(join(tmpdir(), "blueprint-atualizacao-real-"));
const destino = join(tmp, "Blueprint"), appdata = join(tmp, "appdata"), saida = join(tmp, "saida");
const porta = await portaLivre();
const iso = { ...process.env, APPDATA: appdata, CLAUDE_CONFIG_DIR: join(tmp, "claude"), CODEX_HOME: join(tmp, "codex") };
for (const d of [appdata, saida, iso.CLAUDE_CONFIG_DIR, iso.CODEX_HOME]) mkdirSync(d, { recursive: true });
const silencioso = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CURRENTUSER"];
const ps = (cmd) => execFileSync("powershell", ["-NoProfile", "-Command", cmd], { encoding: "utf8" }).trim();
const versaoRegistrada = () => { try { return /DisplayVersion\s+REG_SZ\s+(\S+)/.exec(execFileSync("reg", ["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{B5E9C6A2-3C1F-4D1E-9A57-5E2C7B1F8A40}_is1", "/v", "DisplayVersion"], { encoding: "utf8" }))?.[1]; } catch { return null; } };
const doApp = () => ps(`Get-Process -Name blueprint -ErrorAction SilentlyContinue | Where-Object { $_.Path -like '${destino.replaceAll("'", "''")}*' } | Select-Object -ExpandProperty Id`).split(/\s+/).filter(Boolean);
let gh;
try {
  assert.equal(spawnSync(instalador010, [...silencioso, `/DIR=${destino}`, "/TASKS="], { env: iso, timeout: 240000 }).status, 0, "instalar a 0.1.0");
  assert.equal(versaoRegistrada(), "0.1.0");
  const exe = join(destino, "blueprint.exe");
  assert.equal(spawnSync(iscc, ["/DVersao=0.1.1", `/O${saida}`, "/Qp", join(raiz, "instalador", "blueprint.iss")], { cwd: raiz }).status, 0, "gerar o instalador 0.1.1");
  const arquivo = readFileSync(join(saida, "Blueprint-0.1.1-instalador.exe"));
  const hash = createHash("sha256").update(arquivo).digest("hex"), base = `http://127.0.0.1:${porta}`, nome = "Blueprint-0.1.1-instalador.exe";
  gh = createServer((req, res) => {
    if (req.url.startsWith("/releases")) return res.end(JSON.stringify([{ tag_name: "v0.1.1", draft: false, prerelease: true, body: "0.1.1", assets: [{ name: nome, size: arquivo.length, browser_download_url: `${base}/baixar/${nome}` }, { name: "SHA256SUMS", size: 99, browser_download_url: `${base}/baixar/SHA256SUMS` }] }]));
    if (req.url === `/baixar/${nome}`) return res.end(arquivo);
    if (req.url === "/baixar/SHA256SUMS") return res.end(`${hash}  ${nome}\n`);
    res.statusCode = 404; res.end();
  });
  await new Promise((ok) => gh.listen(porta, "127.0.0.1", ok));
  mkdirSync(join(appdata, "Blueprint"), { recursive: true });
  const env = { ...iso, BLUEPRINT_URL: `http://127.0.0.1:${await portaLivre()}`, BLUEPRINT_ATUALIZACAO_URL: base, PORT: String(await portaLivre()), HOST: "127.0.0.1" };
  const cli = (...args) => new Promise((ok) => { const f = spawn(exe, args, { env, windowsHide: true }); let o = "", e = ""; f.stdout.on("data", (d) => (o += d)); f.stderr.on("data", (d) => (e += d)); f.on("close", (status) => ok({ status, o, e })); });
  const baixou = await cli("atualizar");
  assert.equal(baixou.status, 0, baixou.e);
  assert.equal(versaoRegistrada(), "0.1.0", "baixar não instala");
  assert.equal(doApp().length, 0, "ninguém abriu o app");
  // o clique (digitar o comando): sem o app aberto, o CLI cria/abre o banco em %APPDATA%\Blueprint, copia-o para backups\ e roda o instalador
  const instalou = await cli("atualizar", "--instalar");
  assert.equal(instalou.status, 0, instalou.e);
  for (let i = 0; i < 240 && versaoRegistrada() !== "0.1.1"; i++) await dormir(250);
  assert.equal(versaoRegistrada(), "0.1.1", "o instalador devia ter atualizado por cima");
  for (let i = 0; i < 120 && doApp().length === 0; i++) await dormir(250);
  assert(doApp().length > 0, "com /ATUALIZACAO=1 o instalador devia abrir o app de novo");
  assert((readdirSync(join(appdata, "Blueprint", "backups")).some((n) => n.endsWith(".db"))), "o banco foi copiado antes de atualizar");
  console.log("ok: instalada a 0.1.0, baixada e instalada a 0.1.1 pelo comando (o clique), registro em 0.1.1, banco copiado para backups e app reaberto sozinho");
} finally {
  gh?.close();
  try { ps(`Get-Process -Name blueprint -ErrorAction SilentlyContinue | Where-Object { $_.Path -like '${destino.replaceAll("'", "''")}*' } | Stop-Process -Force`); } catch {}
  await dormir(1500);
  const des = existsSync(destino) ? readdirSync(destino).find((f) => /^unins\d+\.exe$/.test(f)) : null;
  if (des) spawnSync(join(destino, des), silencioso, { env: iso, timeout: 120000 });
  await dormir(1500);
  rmSync(tmp, { recursive: true, force: true });
}
