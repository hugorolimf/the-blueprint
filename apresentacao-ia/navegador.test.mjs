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

// ---- o texto faz referência à figura e as duas se comandam (biblioteca/ponte.js), pelo DevTools em tempo real ----
{
  const { abrirNavegador, dormir } = await import("./teste-cdp.mjs");
  const normal = join(mkdtempSync(join(tmpdir(), "ap-ponte-")), "pagina.html");
  writeFileSync(normal, montar({ conteudo, figuras }));
  const url = pathToFileURL(normal).href + "#recebe";
  const nav = await abrirNavegador(navegador);
  const item = (i) => `document.querySelector('[data-sessao="recebe"][data-item="${i}"]')`;
  const estado = async () => JSON.parse(await nav.ev(`(() => { const its = [...document.querySelectorAll('[data-sessao="recebe"]')]; const pl = document.querySelector('[data-figura="fichario"]').closest(".iso-plate"); return JSON.stringify({ marcado: its.findIndex((e) => e.hasAttribute("data-ativo") && e.getAttribute("aria-current") === "true"), varios: its.filter((e) => e.hasAttribute("data-ativo")).length, leitura: pl.querySelector("[data-readout]").textContent }); })()`));
  const centro = async (expr) => JSON.parse(await nav.ev(`(() => { const r = ${expr}.getBoundingClientRect(); return JSON.stringify({ x: r.left + r.width / 2, y: r.top + r.height / 2, l: r.left, w: r.width, t: r.top, b: r.bottom }); })()`));
  try {
    await nav.ir(url);
    assert.equal(await nav.ev(`document.querySelectorAll('[data-sessao="recebe"][tabindex="0"]').length`), 3, "os três itens de Entregas são focáveis");
    assert.match(await nav.ev(`document.querySelector("#s-entregas .ap-ref").textContent`), /^Fig 2 · .+Aponte um item do texto para vê-lo na figura\.$/, "a linha de referência sob o título");
    await dormir(1500);
    let e = await estado();
    assert.ok(e.marcado >= 0 && e.varios === 1, "a figura, no seu passeio, já marca um item: " + JSON.stringify(e));

    // o mouse num item do texto aponta a ficha na figura, e o passeio espera enquanto o item está apontado
    const c2 = await centro(item(2));
    await nav.mouse(c2.x, c2.y); await dormir(1300);
    e = await estado();
    assert.deepEqual([e.marcado, e.varios, /ficha 14/.test(e.leitura)], [2, 1, true], "mouse no item 3: a figura vai à ficha 14: " + JSON.stringify(e));
    await dormir(3800);
    e = await estado();
    assert.equal(e.marcado, 2, "o passeio da figura espera enquanto o item está apontado");
    // a saída devolve a figura ao passeio
    await nav.mouse(3, 3); await dormir(1500);
    e = await estado();
    assert.deepEqual([e.marcado, /ficha 01/.test(e.leitura)], [0, true], "saiu do item: a figura volta ao seu passeio (primeiro passo): " + JSON.stringify(e));

    // mexer na figura marca o item do texto
    const f = await centro(`document.querySelector('[data-figura="fichario"]')`);
    await nav.mouse(f.l + f.w * 0.92, f.y); await dormir(1300);
    e = await estado();
    assert.deepEqual([e.marcado, /ficha 14/.test(e.leitura)], [2, true], "mouse na ponta direita da figura: marca o item 3 do texto: " + JSON.stringify(e));
    await nav.mouse(3, 3); await dormir(500);

    // teclado: o foco que chega seleciona, o que sai solta
    await nav.ev(`${item(1)}.focus()`); await dormir(1000);
    e = await estado();
    assert.deepEqual([e.marcado, /ficha 08/.test(e.leitura)], [1, true], "foco no item 2: ficha 8: " + JSON.stringify(e));
    await nav.ev(`${item(1)}.blur()`); await dormir(1000);
    assert.equal((await estado()).marcado, 0, "o foco saiu: passeio");

    // toque: o toque seleciona; o toque fora solta
    await nav.ev(`(() => { const t = ${item(2)}; t.dispatchEvent(new PointerEvent("pointerdown", { pointerType: "touch", bubbles: true })); t.click(); })()`); await dormir(1000);
    assert.equal((await estado()).marcado, 2, "o toque no item 3 seleciona");
    await nav.ev(`(() => { const t = document.querySelector(".ap-principal"); t.dispatchEvent(new PointerEvent("pointerdown", { pointerType: "touch", bubbles: true })); t.click(); })()`); await dormir(1000);
    assert.equal((await estado()).marcado, 0, "o toque fora solta");

    // o link "Fig 2" rola até a figura e põe o foco nela
    await nav.ev(`window.scrollTo(0, 0)`); await dormir(400);
    await nav.ev(`document.querySelector("#s-entregas .ap-ref-link").click()`); await dormir(900);
    assert.equal(await nav.ev(`document.activeElement.getAttribute("data-figura")`), "fichario", "o foco foi para o palco da figura");
    const v = await centro(`document.querySelector('[data-figura="fichario"]')`);
    assert.ok(v.b > 0 && v.t < 900, "a figura está à vista depois do link");

    // tela estreita: a figura vem abaixo do texto e a ligação continua
    await nav.ir(url, { w: 390, h: 844 });
    const t2 = await centro(item(2)), fg = await centro(`document.querySelector('[data-figura="fichario"]')`);
    assert.ok(fg.t > t2.y, "em tela estreita a figura fica abaixo do texto");
    assert.equal(await nav.ev(`document.documentElement.scrollWidth <= document.documentElement.clientWidth`), true, "sem rolagem lateral");
    await nav.ev(`${item(1)}.focus()`); await dormir(1000);
    e = await estado();
    assert.deepEqual([e.marcado, /ficha 08/.test(e.leitura)], [1, true], "a ligação continua na tela estreita: " + JSON.stringify(e));

    // movimento reduzido: sem passeio, mas a ligação funciona e o item fica marcado
    await nav.ir(url, { reduzido: true });
    await nav.ev(`${item(2)}.focus()`); await dormir(900);
    e = await estado();
    assert.deepEqual([e.marcado, /ficha 14/.test(e.leitura)], [2, true], "movimento reduzido: o foco aponta a ficha: " + JSON.stringify(e));
    await nav.ev(`${item(2)}.blur()`); await dormir(1500);
    assert.equal((await estado()).marcado, 2, "movimento reduzido: sem passeio a figura fica onde está");
  } finally { nav.fechar(); }
}
console.log("navegador.test.mjs: ok (" + navegador.split(/[\\/]/).pop() + ")");
