// A atualização DE VERDADE com a janela de progresso (`blueprint atualizando`): app 0.2.2-teste instalado pelo Inno, aberto como pelo atalho, clique em Atualizar
// (POST /api/atualizacao/instalar), instalador 0.2.3-teste baixado de um "GitHub" falso. Três caminhos:
//   sucesso  a janela aparece (cópia em %TEMP%), o instalador roda, o app novo reabre na versão nova e a janela fecha sozinha;
//   falha    o instalador sai com erro (InitializeSetup devolve False): a janela mostra o erro, o app anterior reabre e o botão Fechar fecha a janela;
//   sem-janela  a cópia para %TEMP% falha (um arquivo ocupa o nome da pasta): o instalador roda direto, como antes da janela, e a atualização acontece.
// Não toca na instalação real: os instaladores saem de uma CÓPIA do .iss com outro AppId, outro nome (atalho) e outra trava (AppMutex), sem `registrar` e sem
// apagar dados ao desinstalar; tudo roda com APPDATA, TEMP, CLAUDE_CONFIG_DIR e CODEX_HOME temporários, PORT livre e BLUEPRINT_INSTANCIA_PARA_TESTE (o app do
// usuário pode estar aberto). Antes e depois compara a chave do desinstalador real, o blueprint.exe instalado, os atalhos e a porta 3000.
// Uso: node scripts/atualizacao-real-e2e.mjs [--recompilar] [--capturas <pasta>] [--so sucesso,falha,sem-janela]
// Leva ~3 min com os binários já compilados (target/e2e-atualizacao/versoes); compilar as duas versões leva mais alguns minutos. Abre janelas por alguns segundos.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { dormir, portaLivre } from "./lib-e2e-rust.mjs";

const raiz = resolve(import.meta.dirname, "..");
const arg = (n) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : undefined; };
const [ANTIGA, NOVA] = ["0.2.2-teste", "0.2.3-teste"];
const ID_TESTE = "{7E57B10E-A7A2-4C1E-9E2E-0000A7E2E000}", NOME_TESTE = "Blueprint Teste Atualizacao", TRAVA = "atualizacao-real-e2e";
const CHAVE = (id) => `HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${id}_is1`;
const CHAVE_REAL = CHAVE("{B5E9C6A2-3C1F-4D1E-9A57-5E2C7B1F8A40}");
const cenarios = (arg("--so") ?? "sucesso,falha,sem-janela").split(",");
const iscc = [process.env.ISCC, join(process.env.LOCALAPPDATA || "", "Programs/Inno Setup 6/ISCC.exe"), "C:/Program Files (x86)/Inno Setup 6/ISCC.exe"].find((c) => c && existsSync(c));
assert(iscc, "Inno Setup 6 não encontrado");
// sem o PSModulePath do pwsh 7 (se o Node foi aberto por ele): o Windows PowerShell 5 não acharia nem os cmdlets básicos
const ps = (cmd, env = process.env) => execFileSync("powershell", ["-NoProfile", "-NonInteractive", "-Command", cmd], { encoding: "utf8", env: { ...env, PSModulePath: "" } }).trim();
const sha = (f) => createHash("sha256").update(readFileSync(f)).digest("hex");
const aspas = (s) => `'${s.replaceAll("'", "''")}'`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-atualizacao-real-"));
const capturas = resolve(arg("--capturas") ?? join(tmp, "capturas"));
mkdirSync(capturas, { recursive: true });

// --- a instalação real: fotografada antes e conferida no fim (se mudar, o teste falha) ---
const fotoReal = async () => {
  const f = ps(`$k = Get-ItemProperty ${aspas(CHAVE_REAL)} -ErrorAction SilentlyContinue; $e = Join-Path $env:LOCALAPPDATA 'Programs\\Blueprint\\blueprint.exe';
    $p = [Environment]::GetFolderPath('Programs'); $d = [Environment]::GetFolderPath('Desktop');
    [pscustomobject]@{ chave = if ($k) { $k | Select-Object * -ExcludeProperty PS* | ConvertTo-Json -Compress } else { $null };
      exe = if (Test-Path $e) { "$e $((Get-Item $e).Length) $((Get-Item $e).LastWriteTimeUtc.Ticks)" } else { $null };
      atalhos = @((Join-Path $p 'The Blueprint (beta).lnk'), (Join-Path $d 'The Blueprint (beta).lnk')) | ForEach-Object { if (Test-Path $_) { "$_ $((Get-Item $_).LastWriteTimeUtc.Ticks)" } else { "$_ -" } };
      teste = [bool](Test-Path ${aspas(CHAVE(ID_TESTE))}); atalhoTeste = [bool](Test-Path (Join-Path $p '${NOME_TESTE}.lnk')) } | ConvertTo-Json -Compress`);
  let porta3000;
  try { porta3000 = (await fetch("http://127.0.0.1:3000/api/atualizacao", { signal: AbortSignal.timeout(2000) })).status; } catch { porta3000 = "sem resposta"; }
  const r = JSON.parse(f);
  return { ...r, sha: r.exe ? sha(r.exe.split(" ").slice(0, -2).join(" ")) : null, porta3000 };
};

// --- os binários: o mesmo código compilado com duas versões (BLUEPRINT_VERSAO_COMPILADA), numa pasta de build própria ---
const alvo = join(raiz, "target", "e2e-atualizacao");
const binario = (v) => {
  const exe = join(alvo, "versoes", v, "blueprint.exe");
  if (process.argv.includes("--recompilar") || !existsSync(exe)) {
    console.log(`compilando ${v} (cargo build --release)...`);
    const r = spawnSync("cargo", ["build", "--release", "--target-dir", alvo], { cwd: raiz, stdio: "inherit", env: { ...process.env, BLUEPRINT_VERSAO_COMPILADA: v } });
    assert.equal(r.status, 0, "cargo build");
    mkdirSync(dirname(exe), { recursive: true });
    copyFileSync(join(alvo, "release", "blueprint.exe"), exe);
  }
  assert.equal(execFileSync(exe, ["--version"], { encoding: "utf8" }).trim(), `blueprint ${v}`);
  return exe;
};

// --- os instaladores: uma cópia do .iss, isolada da instalação real ---
const issDeTeste = (falhar) => {
  let t = readFileSync(join(raiz, "instalador", "blueprint.iss"), "utf8");
  const troca = (de, para) => { const antes = t; t = t.replace(de, para); assert.notEqual(t, antes, `o .iss mudou: não achei ${de}`); };
  troca(/AppId=\{\{B5E9C6A2-3C1F-4D1E-9A57-5E2C7B1F8A40\}/, `AppId={${ID_TESTE}`);
  troca(/#define Nome "The Blueprint \(beta\)"/, `#define Nome "${NOME_TESTE}"`);
  troca(/#define Pasta "Blueprint"/, '#define Pasta "BlueprintTesteAtualizacao"');
  troca(/AppMutex=Local\\Blueprint\.InstanciaUnica/, `AppMutex=Local\\Blueprint.Teste.${TRAVA}`);
  troca(/SetupIconFile=\.\.\\recursos/, `SetupLogging=yes\r\nSetupIconFile=${join(raiz, "recursos")}`); // o log do Inno vai para o %TEMP% do teste
  troca(/^.*Parameters: "registrar.*$/gm, ""); // nada de mexer na configuração do Claude Code e do Codex
  troca(/if Passo = usPostUninstall then/, "if False then"); // {userappdata} é a pasta REAL (não lê o APPDATA do ambiente): nunca apagar
  if (falhar) troca(/function InitializeSetup: Boolean;(\r?\n)begin(\r?\n)  Result := True;/, "function InitializeSetup: Boolean;$1begin$2  Result := False; Exit;");
  const arquivo = join(tmp, falhar ? "falha.iss" : "teste.iss");
  writeFileSync(arquivo, t);
  return arquivo;
};
const compilar = (iss, versao, exe, saida) => {
  const r = spawnSync(iscc, [`/DVersao=${versao}`, `/DOrigem=${dirname(exe)}`, `/O${saida}`, "/Qp", iss], { encoding: "utf8" });
  assert.equal(r.status, 0, `ISCC ${versao}: ${r.stdout}${r.stderr}`);
  return join(saida, `Blueprint-${versao}-instalador.exe`);
};

// --- o "GitHub" falso: lista a versão nova com o instalador e o SHA256SUMS da vez ---
let servir = null;
const gh = createServer((req, res) => {
  const { nome, dados } = servir, base = `http://${req.headers.host}`;
  if (req.url.startsWith("/releases")) return res.end(JSON.stringify([{ tag_name: `v${NOVA}`, draft: false, prerelease: true, body: "teste", assets: [{ name: nome, size: dados.length, browser_download_url: `${base}/baixar/${nome}` }, { name: "SHA256SUMS", size: 99, browser_download_url: `${base}/baixar/SHA256SUMS` }] }]));
  if (req.url === `/baixar/${nome}`) return res.end(dados);
  if (req.url === "/baixar/SHA256SUMS") return res.end(`${createHash("sha256").update(dados).digest("hex")}  ${nome}\n`);
  res.statusCode = 404; res.end();
});

const silencioso = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CURRENTUSER"];
const pastaTeste = (pasta) => ps(`ConvertTo-Json -Compress -InputObject @(Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -like ${aspas(pasta + "\\*")} } |
  ForEach-Object { [pscustomobject]@{ pid = $_.ProcessId; pai = $_.ParentProcessId; exe = $_.ExecutablePath; cmd = $_.CommandLine } });
  Write-Output '|'; (Get-ItemProperty ${aspas(CHAVE(ID_TESTE))} -ErrorAction SilentlyContinue).DisplayVersion`).split("|");
const observar = (pasta) => {
  try { const [p, v] = pastaTeste(pasta); return { procs: JSON.parse(p || "[]"), versao: v.trim() || null }; } catch (e) { console.error(`observar: ${e.message}`); return { procs: [], versao: "?" }; }
};
const matar = (pasta) => { try { ps(`Get-CimInstance Win32_Process -Filter "Name='blueprint.exe' or Name='blueprint-atualizando.exe'" | Where-Object { $_.ExecutablePath -like ${aspas(pasta + "\\*")} } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`); } catch {} };
const app = async (appdata) => { // a porta que o app anotou e o que ele responde
  try { const porta = readFileSync(join(appdata, "Blueprint", "porta"), "utf8").trim(); return { porta, mtime: statSync(join(appdata, "Blueprint", "porta")).mtimeMs, ...(await (await fetch(`http://127.0.0.1:${porta}/api/atualizacao`, { signal: AbortSignal.timeout(1500) })).json()) }; } catch { return null; }
};
const capturar = (pid, nome) => { // só a janela de progresso (o retângulo dela), nunca a tela inteira
  const arquivo = join(capturas, nome);
  try {
    ps(`Add-Type -AssemblyName System.Drawing; Add-Type 'using System;using System.Runtime.InteropServices;public class Jan{[DllImport("user32.dll")]public static extern bool GetWindowRect(IntPtr h,out R r);[DllImport("user32.dll")]public static extern bool SetProcessDPIAware();public struct R{public int L,T,D,B;}}';
      [Jan]::SetProcessDPIAware() | Out-Null; $h = (Get-Process -Id ${pid}).MainWindowHandle; if ($h -eq 0) { throw 'sem janela' }; $r = New-Object Jan+R; [Jan]::GetWindowRect($h, [ref]$r) | Out-Null;
      $b = New-Object System.Drawing.Bitmap ($r.D - $r.L), ($r.B - $r.T); [System.Drawing.Graphics]::FromImage($b).CopyFromScreen($r.L, $r.T, 0, 0, $b.Size); $b.Save(${aspas(arquivo)})`);
    return arquivo;
  } catch (e) { return `sem captura (${e.message.split("\n")[0]})`; }
};
const lerJanela = async (temp) => { // o conteúdo da página de progresso, pelo DevTools do WebView2 (WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS)
  try {
    const pasta = readdirSync(temp).find((n) => n.startsWith("blueprint-atualizando-") && statSync(join(temp, n)).isDirectory());
    const f = readdirSync(join(temp, pasta), { recursive: true }).find((n) => n.endsWith("DevToolsActivePort"));
    const porta = readFileSync(join(temp, pasta, f), "utf8").split("\n")[0].trim();
    const pagina = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((p) => p.type === "page");
    return await new Promise((ok, falha) => {
      const ws = new WebSocket(pagina.webSocketDebuggerUrl);
      const expr = `JSON.stringify({fase: document.body.dataset.fase, titulo: document.querySelector("h1").textContent, nota: document.getElementById("nota").textContent, passos: [...document.querySelectorAll("#passos li")].map((l) => l.dataset.s).join(","), fechar: !document.getElementById("fechar").hidden})`;
      ws.onopen = () => ws.send(JSON.stringify({ id: 1, method: "Runtime.evaluate", params: { expression: expr } }));
      ws.onmessage = (m) => { ws.close(); ok(JSON.parse(JSON.parse(m.data).result.result.value)); };
      ws.onerror = () => falha(new Error("ws"));
      setTimeout(() => falha(new Error("ws: sem resposta")), 2000);
    });
  } catch { return null; }
};
const clicarFechar = async (temp) => { // o botão Fechar da janela de erro (mesmo caminho: DevTools)
  const pasta = readdirSync(temp).find((n) => n.startsWith("blueprint-atualizando-"));
  const f = readdirSync(join(temp, pasta), { recursive: true }).find((n) => n.endsWith("DevToolsActivePort"));
  const porta = readFileSync(join(temp, pasta, f), "utf8").split("\n")[0].trim();
  const pagina = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((p) => p.type === "page");
  const ws = new WebSocket(pagina.webSocketDebuggerUrl);
  await new Promise((ok) => { ws.onopen = () => { ws.send(JSON.stringify({ id: 1, method: "Runtime.evaluate", params: { expression: 'document.getElementById("fechar").click()' } })); setTimeout(ok, 300); }; ws.onerror = ok; });
};

async function cenario(nome, instAntigo, instNovo) {
  const base = join(tmp, nome), destino = join(base, "app"), appdata = join(base, "appdata"), temp = join(base, "temp");
  for (const d of [appdata, temp, join(base, "claude"), join(base, "codex")]) mkdirSync(d, { recursive: true });
  const env = { ...process.env, APPDATA: appdata, TEMP: temp, TMP: temp, CLAUDE_CONFIG_DIR: join(base, "claude"), CODEX_HOME: join(base, "codex"), PORT: String(await portaLivre()), HOST: "127.0.0.1",
    BLUEPRINT_ATUALIZACAO_URL: `http://127.0.0.1:${gh.address().port}`, BLUEPRINT_INSTANCIA_PARA_TESTE: TRAVA, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: "--remote-debugging-port=0" };
  delete env.BLUEPRINT_URL; delete env.BLUEPRINT_SEM_ATUALIZACAO; delete env.BLUEPRINT_VERSAO_PARA_TESTE; delete env.BLUEPRINT_ATUALIZACAO_SEM_EXECUTAR;
  servir = { nome: `Blueprint-${NOVA}-instalador.exe`, dados: readFileSync(instNovo) };
  const r = { nome, linha: [] }, marca = (t) => { r.linha.push(`${((Date.now() - r.t0) / 1000).toFixed(1)} s ${t}`); };
  try {
    assert.equal(spawnSync(instAntigo, [...silencioso, `/DIR=${destino}`, "/TASKS=", `/LOG=${join(base, "instalar.log")}`], { env, timeout: 240000 }).status, 0, "instalar a antiga");
    assert.equal(observar(base).versao, ANTIGA);
    const exe = join(destino, "blueprint.exe");
    ps(`Start-Process -FilePath ${aspas(exe)} -WorkingDirectory ${aspas(base)}`, env); // como o atalho: console próprio que o app solta (FreeConsole)
    let a = null;
    for (let i = 0; i < 240 && a?.fase !== "pronta"; i++) { await dormir(250); a = await app(appdata); }
    assert.equal(a?.fase, "pronta", `o app antigo devia baixar a ${NOVA}: ${JSON.stringify(a)}`);
    assert.equal(a.atual, ANTIGA);
    const pidAntigo = observar(base).procs.find((p) => p.exe === exe)?.pid;
    if (nome === "sem-janela") writeFileSync(join(temp, `blueprint-atualizando-${pidAntigo}`), "ocupa o nome da pasta da cópia"); // lancar() falha
    r.t0 = Date.now();
    const clique = await fetch(`http://127.0.0.1:${a.porta}/api/atualizacao/instalar`, { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
    marca(`clique: HTTP ${clique.status}`);
    assert.equal(clique.status, 200, await clique.text());
    let janela = null, viuJanela = false, capturada = false, novo = null, fim = Date.now() + 150000, ultimoTexto = "";
    while (Date.now() < fim) {
      const o = observar(base), j = o.procs.find((p) => p.exe.endsWith("blueprint-atualizando.exe"));
      if (j && !viuJanela) { viuJanela = true; janela = j; marca(`janela: pid ${j.pid} em ${j.exe}`); }
      if (!j && viuJanela && !r.janelaFechou) { r.janelaFechou = true; marca("janela fechou"); }
      if (j) {
        const t = await lerJanela(temp), texto = t && `${t.fase} | ${t.passos} | ${t.nota}`;
        if (texto && texto !== ultimoTexto) { ultimoTexto = texto; marca(`página: ${texto}`); r.pagina = t; }
        if (t && !capturada) { await dormir(400); capturada = true; marca(`captura: ${capturar(j.pid, `${nome}-progresso.png`)}`); }
        if (t?.fase === "erro" && !r.capturaErro) { r.capturaErro = capturar(j.pid, `${nome}-erro.png`); marca(`captura do erro: ${r.capturaErro}`); }
      }
      if (o.versao !== r.versao) { r.versao = o.versao; marca(`registro: ${o.versao}`); }
      if (!r.antigoSaiu && !o.procs.some((p) => p.pid === pidAntigo)) { r.antigoSaiu = true; marca("o app antigo saiu"); }
      const outros = o.procs.filter((p) => p.pid !== pidAntigo && p.pid !== janela?.pid).map((p) => `${p.pid} (pai ${p.pai}) ${p.cmd.replaceAll(base, "<teste>")}`).join(" ; ");
      if (outros !== (r.outros ?? "")) { r.outros = outros; marca(`processos: ${outros || "-"}`); }
      const n = await app(appdata);
      if (n && n.mtime > r.t0 && !novo) { novo = n; marca(`app reaberto na porta ${n.porta}: atual ${n.atual}, fase ${n.fase}`); }
      const pronto = nome === "falha" ? novo && r.pagina?.fase === "erro" : novo && o.versao === NOVA && (!viuJanela || r.janelaFechou);
      if (pronto) break;
      await dormir(150);
    }
    r.viuJanela = viuJanela; r.novo = novo; r.janela = janela;
    if (nome === "falha" && viuJanela) {
      await clicarFechar(temp);
      for (let i = 0; i < 40 && observar(base).procs.some((p) => p.exe.endsWith("blueprint-atualizando.exe")); i++) await dormir(250);
      r.fecharFunciona = !observar(base).procs.some((p) => p.exe.endsWith("blueprint-atualizando.exe"));
      marca(`botão Fechar: ${r.fecharFunciona ? "fechou" : "não fechou"}`);
    }
    r.versaoFinal = observar(base).versao;
    for (const f of readdirSync(temp).filter((n) => n.startsWith("Setup Log"))) copyFileSync(join(temp, f), join(capturas, `${nome}-${f}`));
    if (existsSync(join(appdata, "Blueprint", "erro.txt"))) marca(`erro.txt do app: ${readFileSync(join(appdata, "Blueprint", "erro.txt"), "utf8")}`);
    r.backups = existsSync(join(appdata, "Blueprint", "backups")) ? readdirSync(join(appdata, "Blueprint", "backups")) : [];
    return r;
  } finally {
    matar(base);
    await dormir(1000);
    const des = existsSync(destino) ? readdirSync(destino).find((f) => /^unins\d+\.exe$/.test(f)) : null;
    if (des) spawnSync(join(destino, des), silencioso, { env, timeout: 120000 });
    for (let i = 0; i < 40 && observar(base).versao; i++) await dormir(250);
    r.desinstalou = !observar(base).versao;
  }
}

const antes = await fotoReal();
assert(!antes.teste && !antes.atalhoTeste, "sobrou uma instalação de teste de uma execução anterior: desinstale antes");
const resultados = [];
try {
  const exeAntigo = binario(ANTIGA), exeNovo = binario(NOVA);
  const iss = issDeTeste(false), instAntigo = compilar(iss, ANTIGA, exeAntigo, join(tmp, "inst-antigo"));
  const instNovo = compilar(iss, NOVA, exeNovo, join(tmp, "inst-novo")), instFalha = compilar(issDeTeste(true), NOVA, exeNovo, join(tmp, "inst-falha"));
  await new Promise((ok) => gh.listen(0, "127.0.0.1", ok));
  for (const c of cenarios) {
    const r = await cenario(c, instAntigo, c === "falha" ? instFalha : instNovo);
    resultados.push(r);
    console.log(`\n== ${c}\n${r.linha.join("\n")}\n  desinstalado: ${r.desinstalou}`);
  }
} finally {
  gh.close();
  const depois = await fotoReal();
  console.log(`\ninstalação real antes:  ${JSON.stringify(antes)}\ninstalação real depois: ${JSON.stringify(depois)}`);
  // a porta 3000 só pode deixar de responder; passar a responder é o usuário abrindo o app dele durante o teste (o teste nunca usa a 3000)
  if (antes.porta3000 !== depois.porta3000) console.log(`porta 3000: ${antes.porta3000} -> ${depois.porta3000}`);
  for (let i = 0; i < 20; i++) { try { rmSync(tmp, { recursive: true, force: true }); break; } catch { await dormir(500); } } // o WebView2 da janela pode segurar a pasta por um instante
  // shortcut: se o usuário atualizar ou reinstalar o app DELE durante o teste, isto acusa mudança também (rode com o app real parado ou confira a versão)
  assert.deepEqual({ ...depois, porta3000: antes.porta3000 === 200 ? depois.porta3000 : 0 }, { ...antes, porta3000: antes.porta3000 === 200 ? 200 : 0 }, "A INSTALAÇÃO REAL (chave, exe, atalhos ou porta 3000) MUDOU: pare e confira");
}

for (const r of resultados) {
  assert(r.desinstalou, `${r.nome}: a instalação de teste devia ter saído`);
  assert(r.backups.some((n) => n.endsWith(".db")), `${r.nome}: o banco devia ser copiado antes de instalar`);
  if (r.nome === "sucesso") {
    assert(r.viuJanela, "sucesso: a janela de progresso devia aparecer");
    assert.match(r.janela.exe, /\\temp\\blueprint-atualizando-\d+\\blueprint-atualizando\.exe$/i, "a janela roda da cópia em %TEMP%");
    assert.equal(r.versaoFinal, NOVA); assert.equal(r.novo?.atual, NOVA, "o app reaberto devia ser o novo");
    assert(r.janelaFechou, "a janela devia fechar sozinha");
  }
  if (r.nome === "falha") {
    assert(r.viuJanela, "falha: a janela devia aparecer"); assert.equal(r.pagina?.fase, "erro"); assert.match(r.pagina.nota, /1/);
    assert.equal(r.versaoFinal, ANTIGA); assert.equal(r.novo?.atual, ANTIGA, "o app anterior devia reabrir"); assert(r.fecharFunciona, "o botão Fechar devia fechar a janela");
  }
  if (r.nome === "sem-janela") {
    assert(!r.viuJanela, "sem-janela: a janela não devia abrir"); assert.equal(r.versaoFinal, NOVA); assert.equal(r.novo?.atual, NOVA);
  }
}
console.log(`\nok: ${cenarios.join(", ")}; instalação real intacta`);
