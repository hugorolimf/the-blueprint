// Teste no navegador de verdade (Edge ou Chrome headless): uma figura com erro de SINTAXE não derruba a página, as outras figuras seguem vivas
// e a quebrada mostra o cartão de erro. Sem navegador instalado o teste avisa e sai sem falhar (CI sem Edge); com navegador, falha de verdade.
import assert from "node:assert/strict";
import { existsSync, mkdtempSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { montar } from "./montar.mjs";
import { exemplo } from "./teste-comum.mjs";

const candidatos = [
  process.env.CHROME_PATH,
  "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
  "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe", "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
  "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser",
];
const navegador = candidatos.find((c) => c && existsSync(c));
if (!navegador) { console.log("navegador.test.mjs: sem Edge/Chrome, pulado"); process.exit(0); }

const { conteudo, figuras } = exemplo();
const quebrada = { id: "quebrada", titulo: "Figura quebrada de propósito", dica: "d", leitura: "", chaves: [], legenda: "l", rotulo: "r", impressao: "x", codigo: "function (L {  return { svg: '<svg></svg>' }; }" };
const comEtapa = { ...conteudo, etapas: conteudo.etapas.map((e, i) => (i === 1 ? { ...e, figura: "quebrada" } : e)) };
const arquivo = join(mkdtempSync(join(tmpdir(), "ap-nav-")), "pagina.html");
writeFileSync(arquivo, montar({ conteudo: comEtapa, figuras: [...figuras, quebrada] }));

const r = spawnSync(navegador, ["--headless=new", "--disable-gpu", "--no-first-run", "--virtual-time-budget=5000", "--dump-dom", pathToFileURL(arquivo).href], { encoding: "utf8", timeout: 60000, maxBuffer: 64 * 1024 * 1024 });
assert.equal(r.status, 0, "o navegador falhou: " + r.stderr);
const bruto = r.stdout, inicio = bruto.indexOf('id="apresentacao"');
// só a página montada: depois dela vêm os <script>, que também guardam o texto das funções da biblioteca
const dom = bruto.slice(inicio, bruto.indexOf("<script", inicio));
assert.equal((dom.match(/<section class="ap-sec" id="s-[a-z]+"/g) || []).length, 6, "a página montou as seis seções");
assert.ok(dom.includes('data-figura="visao"') && /data-figura="visao"[\s\S]{0,400}<svg/.test(dom) && /data-figura="fichario"[\s\S]{0,400}<svg/.test(dom), "as figuras boas foram desenhadas (a da Visão geral e o fichário de Entregas)");
assert.ok(/data-readout[^>]*>ficha 01/.test(dom), "a figura boa ganhou vida (a leitura foi escrita por `vivo`)");
assert.ok(dom.includes("O código desta figura não carregou."), "a figura quebrada mostra o cartão de erro");
assert.ok(dom.includes("ap-fig-erro") && dom.indexOf("ap-fig-erro") > dom.indexOf('id="e-e2"'), "o cartão de erro está na etapa que cita a figura");
assert.ok(!dom.includes("Esta figura não pôde ser desenhada"), "e não o cartão de erro de execução");
console.log("navegador.test.mjs: ok (" + navegador.split(/[\\/]/).pop() + ")");
