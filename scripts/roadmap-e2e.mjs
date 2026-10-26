// A tela Roadmap do app (web/js/roadmap.js): abre pelo botão e pela rota, mostra todos os itens de roadmap.json, filtra, busca, expande, obedece ao teclado,
// enche os anéis de progresso, respeita movimento reduzido e segue o idioma. Edge sem janela, servidor temporário. ~25 s
//   node scripts/roadmap-e2e.mjs [blueprint.exe]
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { abrirAmbiente, dormir, raiz } from "./gravar/nucleo.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const R = JSON.parse(readFileSync(join(raiz, "roadmap.json"), "utf8"));
const visiveis = (A) => A.js(`[...document.querySelectorAll("#roadmap-tela .rm-card:not([hidden])")].filter((c) => !c.closest("[hidden]")).map((c) => c.dataset.id)`);
const tecla = async (A, key, extra = {}) => {
  const base = { key, code: key.length === 1 ? "Key" + key.toUpperCase() : key, windowsVirtualKeyCode: { Escape: 27, Enter: 13, ArrowDown: 40, ArrowUp: 38 }[key] || key.toUpperCase().charCodeAt(0), ...extra };
  await A.cdp("Input.dispatchKeyEvent", { type: "keyDown", ...(key.length === 1 ? { text: key } : {}), ...base });
  await A.cdp("Input.dispatchKeyEvent", { type: "keyUp", ...base }); await dormir(250);
};

const A = await abrirAmbiente({ exe, idioma: "en-US", largura: 1440, altura: 900 });
try {
  await A.ir("#/", 2200);
  await A.clicarEm("#b-roadmap"); await dormir(1800);
  assert.equal(await A.js(`!document.getElementById("roadmap-tela").hidden`), true, "o botão da entrada abre o Roadmap");
  assert.equal((await visiveis(A)).length, R.itens.length, "mostra todos os itens do roadmap.json");
  assert.match(await A.js(`document.querySelector("#roadmap-tela h1").textContent`), /Roadmap/);
  assert(await A.js(`document.querySelector("#roadmap-tela").textContent.includes("Native Codex integration")`), "conteúdo em inglês no Windows em inglês");

  // anel de progresso: enche até a fração entregue do tema
  const tema = R.temas[0], itens = R.itens.filter((i) => i.tema === tema.id), p = itens.filter((i) => i.estado === "entregue").length / itens.length;
  const off = await A.js(`(() => { const c = document.querySelector('#roadmap-tela .rm-faixa[data-tema="${tema.id}"] .rm-anel-feito'); return { off: parseFloat(getComputedStyle(c).strokeDashoffset.replace(/[^0-9.]/g, "")), c: parseFloat(getComputedStyle(c).getPropertyValue("--c")) }; })()`);
  assert(Math.abs(off.off - off.c * (1 - p)) < 1.5, `o anel do tema '${tema.id}' encheu até ${(p * 100).toFixed(0)}% (offset ${off.off.toFixed(1)} de ${off.c.toFixed(1)})`);

  // filtro por estado (botão) e por tema
  await A.clicarEm('#roadmap-tela .rm-filtros [data-estado="proximo"]'); await dormir(300);
  assert.deepEqual((await visiveis(A)).sort(), R.itens.filter((i) => i.estado === "proximo").map((i) => i.id).sort(), "filtra 'Next'");
  await A.clicarEm('#roadmap-tela .rm-filtros [data-estado=""]');
  await A.clicarEm(`#roadmap-tela .rm-filtros [data-tema="agentes"]`); await dormir(300);
  assert.deepEqual((await visiveis(A)).sort(), R.itens.filter((i) => i.tema === "agentes").map((i) => i.id).sort(), "filtra o tema Agent integration");
  await A.clicarEm('#roadmap-tela .rm-filtros [data-tema=""]');

  // busca pelo teclado: "/" foca, digita, o que não casa some, Esc limpa
  await tecla(A, "/");
  assert.equal(await A.js(`document.activeElement.type`), "search", "'/' foca a busca");
  await A.cdp("Input.insertText", { text: "codex" }); await dormir(400);
  assert.deepEqual(await visiveis(A), ["codex"], "a busca por 'codex' deixa só o item do Codex");
  await tecla(A, "Escape");
  assert.equal((await visiveis(A)).length, R.itens.length, "Esc limpa a busca");
  assert.equal(await A.js(`!document.getElementById("roadmap-tela").hidden`), true, "e não fecha a tela");

  // cartão com detalhe abre e fecha por clique e por Enter
  const comDetalhe = R.itens.find((i) => i.en.detalhe).id;
  await A.clicarEm(`#roadmap-tela .rm-card[data-id="${comDetalhe}"]`); await dormir(300);
  assert.equal(await A.js(`document.querySelector('#roadmap-tela .rm-card[data-id="${comDetalhe}"]').getAttribute("aria-expanded")`), "true");
  assert.equal(await A.js(`!document.querySelector('#roadmap-tela .rm-card[data-id="${comDetalhe}"] .rm-detalhe').hidden`), true, "o detalhe aparece");
  await tecla(A, "Escape");
  assert.equal(await A.js(`document.querySelector('#roadmap-tela .rm-card[data-id="${comDetalhe}"]').getAttribute("aria-expanded")`), "false", "Esc recolhe o cartão aberto antes de fechar a tela");

  // atalhos de estado: 1 filtra entregues, 0 limpa; setas movem o foco; Enter abre
  await A.js(`document.activeElement.blur()`);
  await tecla(A, "1");
  assert.deepEqual((await visiveis(A)).sort(), R.itens.filter((i) => i.estado === "entregue").map((i) => i.id).sort(), "'1' filtra o que foi entregue");
  await tecla(A, "0");
  await tecla(A, "ArrowDown"); await tecla(A, "ArrowDown");
  assert.equal(await A.js(`document.activeElement.classList.contains("rm-card")`), true, "as setas movem o foco entre os cartões");
  await tecla(A, "Enter");
  assert.equal(await A.js(`document.activeElement.getAttribute("aria-expanded")`), "true", "Enter abre o cartão focado");

  // fechar: Esc recolhe, Esc fecha; a tela volta à entrada
  await tecla(A, "Escape"); await tecla(A, "Escape");
  assert.equal(await A.js(`document.getElementById("roadmap-tela").hidden`), true, "Esc fecha o Roadmap");
  assert.equal(await A.js(`document.body.classList.contains("rm-aberto")`), false);

  // rota direta e movimento reduzido: tudo aparece de uma vez, sem animação
  await A.cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await A.cdp("Page.navigate", { url: A.url + "/#/roadmap" }); await dormir(1000); await A.js(`location.reload()`).catch(() => {}); await dormir(3000);
  assert.equal(await A.js(`!document.getElementById("roadmap-tela").hidden`), true, "a rota #/roadmap abre a tela");
  const mov = await A.js(`(() => { const c = document.querySelector("#roadmap-tela .rm-card"), s = getComputedStyle(c); return { op: s.opacity, anim: s.animationName, trans: s.transitionDuration }; })()`);
  assert.equal(mov.op, "1", "movimento reduzido: o cartão já está visível"); assert.equal(mov.trans, "0s", "e sem transição");
  assert.deepEqual(A.erros, [], "sem erro no console");
} finally { A.fechar(); }

const B = await abrirAmbiente({ exe, idioma: "pt-BR", largura: 1440, altura: 900 });
try {
  await B.ir("#/roadmap", 3000);
  assert(await B.js(`document.querySelector("#roadmap-tela").textContent.includes("Integração nativa com o Codex")`), "no Windows em português o conteúdo vem em português");
  assert(await B.js(`document.querySelector("#roadmap-tela .rm-voltar").textContent.includes("Voltar")`), "e a interface também");
  assert.deepEqual(B.erros, [], "sem erro no console (pt-BR)");
} finally { B.fechar(); }
console.log("roadmap e2e: ok (abrir por botão e rota, filtros, busca, expandir, teclado, anéis, movimento reduzido, en-US e pt-BR)");
