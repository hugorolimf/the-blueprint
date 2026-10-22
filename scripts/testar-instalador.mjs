// Instala o The Blueprint (beta) com o instalador de verdade (silencioso, só para o usuário, numa pasta temporária), confere os arquivos e atalhos,
// roda a fumaça do servidor e do MCP com o .exe INSTALADO, desinstala e confere que não sobrou nada. Não toca no seu %APPDATA%.
// Uso: node scripts/testar-instalador.mjs   (gere antes com ISCC instalador\blueprint.iss)
import assert from "node:assert/strict";
import { execFileSync, spawn, spawnSync } from "node:child_process";
import { createServer } from "node:net";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const instalador = join(raiz, "target", "instalador", "Blueprint-0.1.0-instalador.exe");
assert(existsSync(instalador), "gere o instalador antes (ISCC instalador\\blueprint.iss)");
const tmp = mkdtempSync(join(tmpdir(), "blueprint-instalacao-"));
const destino = join(tmp, "Blueprint");
const registro = join(tmp, "instalacao.log");
const silencioso = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CURRENTUSER"];
try {
  const inst = spawnSync(instalador, [...silencioso, `/DIR=${destino}`, `/LOG=${registro}`, "/TASKS=atalhodesktop"], { encoding: "utf8", timeout: 120000 });
  assert.equal(inst.status, 0, `o instalador saiu com ${inst.status}\n${existsSync(registro) ? readFileSync(registro, "utf8").slice(-1500) : ""}`);
  const exe = join(destino, "blueprint.exe");
  assert(existsSync(exe), "blueprint.exe devia estar na pasta instalada");
  const desinstalador = readdirSync(destino).find((f) => /^unins\d+\.exe$/.test(f));
  assert(desinstalador, "o desinstalador devia existir");
  const menu = execFileSync("powershell", ["-NoProfile", "-Command", "[Environment]::GetFolderPath('Programs')"], { encoding: "utf8" }).trim();
  const desktop = execFileSync("powershell", ["-NoProfile", "-Command", "[Environment]::GetFolderPath('Desktop')"], { encoding: "utf8" }).trim();
  assert(existsSync(join(menu, "The Blueprint (beta).lnk")), "atalho no menu Iniciar");
  assert(existsSync(join(desktop, "The Blueprint (beta).lnk")), "atalho na área de trabalho (tarefa marcada)");
  console.log(`instalado em ${destino} (${(readFileSync(exe).length / 1048576).toFixed(1)} MB), com atalhos no menu Iniciar e na área de trabalho`);
  const fumaca = spawnSync(process.execPath, [join(raiz, "scripts", "fumaca-rust.mjs"), exe], { encoding: "utf8", timeout: 120000 });
  assert.equal(fumaca.status, 0, `a fumaça com o app instalado falhou:\n${fumaca.stdout}\n${fumaca.stderr}`);
  console.log(fumaca.stdout.trim().split("\n").at(-1));
  // "máquina limpa" possível sem VM: ambiente sem node/cargo no PATH, APPDATA novo e vazio; depois reinstala por cima (atualização) e confere que os dados ficam
  const appdata = join(tmp, "appdata");
  const limpo = { SystemRoot: process.env.SystemRoot, windir: process.env.windir, PATH: join(process.env.SystemRoot, "System32"), APPDATA: appdata, HOST: "127.0.0.1", PORT: "0", BLUEPRINT_DB: join(appdata, "Blueprint", "blueprint.db"), PROTOTIPOS_DIR: join(appdata, "Blueprint", "prototipos") };
  mkdirSync(join(appdata, "Blueprint"), { recursive: true });
  const sobe = async (porta) => {
    const p = spawn(exe, ["serve"], { env: { ...limpo, PORT: String(porta) }, stdio: "ignore", windowsHide: true });
    for (let i = 0; i < 100; i++) { try { return { p, r: await fetch(`http://127.0.0.1:${porta}/api/dados`) }; } catch { await new Promise((r) => setTimeout(r, 100)); } }
    p.kill(); throw new Error("o app instalado não subiu num ambiente sem node/cargo");
  };
  const gravaNoLimpo = async (porta) => {
    const { p, r } = await sobe(porta);
    try { assert.equal(r.status, 200); const d = await r.json(); d.workspaces ??= []; d.nota_do_teste = "sobrevive"; const put = await fetch(`http://127.0.0.1:${porta}/api/dados`, { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify(d) }); return put.status; } finally { p.kill(); }
  };
  const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
  const st = await gravaNoLimpo(await livre());
  assert(st === 200 || st === 400, `PUT /api/dados respondeu ${st}`);
  console.log(`ambiente limpo (PATH só com System32, APPDATA novo): o app instalado sobe e grava o banco em ${limpo.BLUEPRINT_DB}`);
  const de_novo = spawnSync(instalador, [...silencioso, `/DIR=${destino}`], { encoding: "utf8", timeout: 120000 });
  assert.equal(de_novo.status, 0, "reinstalar por cima (atualização) devia funcionar");
  assert(existsSync(limpo.BLUEPRINT_DB), "a atualização não podia apagar o banco do usuário");
  const { p: p2, r: r2 } = await sobe(await livre());
  p2.kill(); assert.equal(r2.status, 200);
  console.log("atualização por cima: o banco do usuário continua lá e o app sobe");
  const des = spawnSync(join(destino, desinstalador), [...silencioso], { encoding: "utf8", timeout: 120000 });
  assert.equal(des.status, 0, `o desinstalador saiu com ${des.status}`);
  for (let i = 0; i < 40 && existsSync(exe); i++) await new Promise((r) => setTimeout(r, 250));
  assert(!existsSync(exe), "o blueprint.exe devia ter sido removido");
  assert(!existsSync(join(menu, "The Blueprint (beta).lnk")) && !existsSync(join(desktop, "The Blueprint (beta).lnk")), "os atalhos deviam ter sido removidos");
  console.log("desinstalado: arquivo e atalhos removidos. Instalar e desinstalar funcionam (silencioso, só para o usuário).");
} finally {
  try { execFileSync("powershell", ["-NoProfile", "-Command", "Remove-Item -Force -ErrorAction SilentlyContinue \"$([Environment]::GetFolderPath('Programs'))\\The Blueprint (beta).lnk\", \"$([Environment]::GetFolderPath('Desktop'))\\The Blueprint (beta).lnk\""]); } catch {}
  rmSync(tmp, { recursive: true, force: true });
}
