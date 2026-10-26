// Confere a tela de workspaces do app local com uma equipe lembrada (src/equipes.rs): o cartão da equipe aparece com o nome do workspace e de quem entrou,
// abrir sem janela mostra o aviso (o `serve` não tem janela) e "Tirar da lista" remove o cartão e o arquivo. Edge sem janela, APPDATA temporário.
// Uso: node scripts/equipes-locais-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; serve a interface da pasta, sem recompilar)
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { janela, espera } from "./lib-e2e-edge.mjs";
import { exePadrao, ligarServidor, pastaDeDados, raiz } from "./lib-e2e-rust.mjs";

const pasta = await pastaDeDados("equipes-locais", {});
const filhos = [];
mkdirSync(join(pasta.dir, "Blueprint"), { recursive: true });
const arquivo = join(pasta.dir, "Blueprint", "equipes.json");
writeFileSync(arquivo, JSON.stringify([{ origem: "https://abc-def.trycloudflare.com", membro: "Ana", workspaces: [{ id: "w2", nome: "App compartilhado" }] }]));
pasta.env.APPDATA = pasta.dir;
pasta.env.BLUEPRINT_ASSETS_DIR = raiz;
try {
  filhos.push(await ligarServidor(exePadrao(), pasta));
  const w = await janela(pasta, "ana", "/", filhos);
  const cartao = `document.querySelector("[data-equipe-abrir]")`;
  await espera(() => w.js(`!!${cartao}`), "o cartão da equipe não apareceu");
  const texto = await w.js(`${cartao}.innerText`);
  for (const parte of ["App compartilhado", "Ana", "abc-def.trycloudflare.com"]) assert(texto.includes(parte), `faltou «${parte}» no cartão: ${texto}`);
  await w.js(`${cartao}.click()`);
  await espera(() => w.js(`document.getElementById("entrada-status").textContent.length > 0 && !document.getElementById("entrada-status").textContent.includes("workspace")`), "abrir sem janela não avisou");
  await w.js(`document.querySelector("[data-equipe-esquecer]").click()`);
  await espera(() => w.js(`!document.querySelector("[data-equipe-abrir]")`), "o cartão não saiu");
  assert.deepEqual(JSON.parse(readFileSync(arquivo, "utf8")), [], "o arquivo ainda guarda a equipe");
  assert.deepEqual(w.erros, [], `erros na página: ${w.erros}`);
  console.log("equipes-locais-e2e: ok (cartão, aviso sem janela, tirar da lista)");
} finally {
  for (const f of filhos) { try { f.kill(); } catch {} }
  setTimeout(() => { try { rmSync(pasta.dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 }); } catch {} process.exit(0); }, 500);
}
