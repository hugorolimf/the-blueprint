// Mede e confere o app com janela (blueprint.exe sem argumentos): tempo até a janela visível (partida fria e quente), memória com a janela
// aberta e depois de ir para a bandeja, MCP respondendo com a janela fechada, instância única, porta de reserva e uma captura da janela.
// Roda o app de verdade com APPDATA e dados temporários (nada do seu %APPDATA% é tocado). Uso: node scripts/medir-janela.mjs [blueprint.exe]
// Grava referencias/medicao-janela.json e a captura em target/janela.png.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const r1 = (n) => Math.round(n * 10) / 10;
const tmp = mkdtempSync(join(tmpdir(), "medir-janela-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, APPDATA: tmp, DADOS_ARQ: join(tmp, "data.json") };
delete env.PORT; delete env.BLUEPRINT_DB; delete env.PROTOTIPOS_DIR; delete env.MCP_REDE;
const ps = (cmd, extraEnv = env) => execFileSync("powershell", ["-NoProfile", "-Command", cmd], { encoding: "utf8", env: extraEnv }).trim();
const limpar = () => { try { ps(`Get-Process blueprint -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${exe.replaceAll("'", "''")}' } | Stop-Process -Force`); } catch {} try { rmSync(tmp, { recursive: true, force: true }); } catch {} };
process.on("exit", limpar);

// o app tenta a porta 3000 primeiro; se algo (o servidor Node antigo, por exemplo) já ocupa, ele usa outra: é o que acontece aqui
const ocupante = await new Promise((ok) => { const s = createServer(); s.on("error", () => ok(null)); s.listen(3000, "127.0.0.1", () => ok(s)); });
if (ocupante) console.log("a porta 3000 estava livre: vou ocupá-la para provar o plano B");
else console.log("a porta 3000 já está ocupada por outro programa: o app deve usar outra porta");

const iniciar = `
$sw=[Diagnostics.Stopwatch]::StartNew()
$p=Start-Process -FilePath '${exe.replaceAll("'", "''")}' -PassThru
$v=-1
while($sw.ElapsedMilliseconds -lt 20000){ $p.Refresh(); if($p.MainWindowHandle -ne 0){ $v=$sw.ElapsedMilliseconds; break }; Start-Sleep -Milliseconds 3 }
"$v $($p.Id)"`;
const memoria = (pid) => ps(`$ids=@(${pid})+@(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" | Where-Object { $_.CommandLine -and $_.CommandLine.Contains('${tmp}') } | ForEach-Object { $_.ProcessId }); $n=$ids.Count-1; $s=0; foreach($i in $ids){ $q=Get-Process -Id $i -ErrorAction SilentlyContinue; if($q){ $s+=$q.WorkingSet64 } }; "$([math]::Round($s/1MB,1)) $n"`).split(" ").map(Number);

const resultados = {};
try {
  const visiveis = [];
  let pid;
  for (const rotulo of ["fria", "quente 1", "quente 2"]) {
    if (pid) { ps(`Stop-Process -Id ${pid} -Force`); await dormir(1500); }
    const [ms, p] = ps(iniciar).split(" ").map(Number);
    assert(ms > 0, `a janela não apareceu em 20 s (${rotulo})`);
    pid = p;
    visiveis.push({ partida: rotulo, ate_janela_visivel_ms: ms });
    console.log(`janela visível (${rotulo}): ${ms} ms`);
    if (rotulo === "fria") { // espera a página assentar, confere a porta e captura a janela
      await dormir(3000);
      const porta = Number(readFileSync(join(tmp, "Blueprint", "porta"), "utf8"));
      assert(porta !== 3000 || !ocupante, "com a 3000 ocupada o app tinha de usar outra porta");
      resultados.porta_usada = porta;
      // os dados do app ficam em %APPDATA%Blueprint (aqui, o APPDATA temporário), importados por cópia, e o data.json de origem não muda
      assert(existsSync(join(tmp, "Blueprint", "blueprint.db")), "o banco devia estar em %APPDATA%\Blueprint\blueprint.db");
      assert.equal(readFileSync(join(tmp, "data.json"), "utf8"), readFileSync(join(raiz, "data.exemplo.json"), "utf8"), "o data.json de origem não pode mudar");
      const lista0 = await (await fetch(`http://127.0.0.1:${porta}/api/dados`)).json();
      assert.deepEqual(lista0, JSON.parse(readFileSync(join(tmp, "data.json"), "utf8")), "o banco importado devolve os mesmos dados do data.json");
      resultados.dados_em_appdata = { banco: "%APPDATA%\Blueprint\blueprint.db", importado_igual_ao_data_json: true, original_intacto: true };
      console.log(`porta do servidor: ${porta}${porta === 3000 ? "" : " (a 3000 estava ocupada: plano B funcionou)"}`);
      const png = join(raiz, "target", "janela.png");
      mkdirSync(join(raiz, "target"), { recursive: true });
      ps(`Add-Type -AssemblyName System.Drawing; Add-Type @'
using System; using System.Runtime.InteropServices;
public class W { [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r); [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h); public struct RECT { public int L,T,R,B; } }
'@
$p=Get-Process -Id ${pid}; [W]::SetForegroundWindow($p.MainWindowHandle) | Out-Null; Start-Sleep -Milliseconds 400; $r=New-Object W+RECT; [W]::GetWindowRect($p.MainWindowHandle,[ref]$r) | Out-Null
$b=New-Object System.Drawing.Bitmap ($r.R-$r.L),($r.B-$r.T); $g=[System.Drawing.Graphics]::FromImage($b); $g.CopyFromScreen($r.L,$r.T,0,0,$b.Size); $b.Save('${png.replaceAll("\\", "\\\\")}')`);
      console.log(`captura da janela: ${png} (${statSync(png).size} bytes)`);
      const [mbAberta, n] = memoria(pid);
      resultados.com_janela_aberta = { memoria_mb_somando_os_processos: mbAberta, processos_webview2: n };
      console.log(`com a janela aberta: ${mbAberta} MB somando o blueprint.exe e ${n} processos do WebView2 (a soma conta páginas compartilhadas mais de uma vez)`);
      // fechar a janela manda o app para a bandeja: o WebView é destruído e só o servidor fica
      ps(`(Get-Process -Id ${pid}).CloseMainWindow() | Out-Null`);
      await dormir(4000);
      const [mbBandeja, nb] = memoria(pid);
      const vivo = ps(`if(Get-Process -Id ${pid} -ErrorAction SilentlyContinue){'sim'}else{'nao'}`);
      assert.equal(vivo, "sim", "o app não pode sair ao fechar a janela: vai para a bandeja");
      const lista = await (await fetch(`http://127.0.0.1:${porta}/api/mcp/blueprints`)).json();
      assert(lista.workspaces.length > 0, "o MCP precisa responder com a janela na bandeja");
      resultados.na_bandeja = { memoria_mb: mbBandeja, processos_webview2: nb, processo_vivo: true, mcp_responde: true };
      console.log(`na bandeja: ${mbBandeja} MB, ${nb} processos do WebView2; o MCP respondeu (${lista.workspaces.length} workspaces)`);
      assert(mbBandeja < mbAberta / 2, "a memória tem de cair bastante ao ir para a bandeja");
      // a segunda execução só traz a janela existente de volta
      const t = performance.now();
      const antes = ps(`Get-Process blueprint -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${exe.replaceAll("'", "''")}' } | Measure-Object | ForEach-Object Count`);
      ps(`Start-Process -FilePath '${exe.replaceAll("'", "''")}' -Wait`);
      const depois = ps(`Get-Process blueprint -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${exe.replaceAll("'", "''")}' } | Measure-Object | ForEach-Object Count`);
      assert.equal(antes, depois, "a segunda execução não pode ficar aberta");
      let volta = false;
      for (let i = 0; i < 100 && !volta; i++) { volta = ps(`(Get-Process -Id ${pid}).MainWindowHandle -ne 0`) === "True"; if (!volta) await dormir(50); }
      assert(volta, "a segunda execução tinha de trazer a janela de volta");
      resultados.instancia_unica = { segunda_execucao_encerrou: true, janela_voltou_ms: Math.round(performance.now() - t) };
      console.log(`instância única: a segunda execução encerrou e a janela voltou (${resultados.instancia_unica.janela_voltou_ms} ms, contando o PowerShell)`);
      await dormir(2000);
      const [mbVolta] = memoria(pid);
      resultados.apos_voltar_da_bandeja = { memoria_mb: mbVolta };
    }
  }
  resultados.partida = visiveis;
  const quentes = visiveis.filter((v) => v.partida !== "fria").map((v) => v.ate_janela_visivel_ms);
  resultados.partida_quente_media_ms = r1(quentes.reduce((a, b) => a + b, 0) / quentes.length);
  resultados.data = new Date().toISOString();
  resultados.nota = "Medido com Start-Process e o PowerShell olhando MainWindowHandle a cada 3 ms (a janela só fica visível com a página carregada). O WebView2 usa uma pasta de dados nova na partida fria.";
  writeFileSync(join(raiz, "referencias", "medicao-janela.json"), JSON.stringify(resultados, null, 2) + "\n");
  console.log(`\nmeta de partida: < 500 ms. Fria ${visiveis[0].ate_janela_visivel_ms} ms; quente em média ${resultados.partida_quente_media_ms} ms.`);
  console.log("Gravado em referencias/medicao-janela.json");
} finally { ocupante?.close(); limpar(); }
