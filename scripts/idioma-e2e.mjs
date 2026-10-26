// O app abre no idioma do Windows e o usuário pode trocar: Edge sem janela com --lang diferentes, dados de exemplo, servidor temporário.
//   node scripts/idioma-e2e.mjs [blueprint.exe]      (~25 s)
// Confere: en-US abre em inglês, pt-BR em português, fr-FR (sem tradução) cai no inglês; o seletor troca e lembra a escolha; ?idioma= vence o Windows.
import assert from "node:assert/strict";
import { join, resolve } from "node:path";
import { abrirAmbiente, dormir, raiz } from "./gravar/nucleo.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const ler = (A) => A.js(`JSON.stringify({ ativo: I18N.ativo, lang: document.documentElement.lang, titulo: document.getElementById("entrada-titulo")?.textContent, botao: document.getElementById("perfil-sair")?.textContent })`).then(JSON.parse);
async function com(idioma, hash, fazer) {
  const A = await abrirAmbiente({ exe, idioma, largura: 1280, altura: 800 });
  try { await A.ir(hash, 2200); await fazer(A); assert.deepEqual(A.erros, [], `sem erro no console (${idioma})`); } finally { A.fechar(); }
}
await com("en-US", "#/", async (A) => {
  const r = await ler(A); assert.deepEqual([r.ativo, r.lang, r.titulo, r.botao], ["en", "en", "Your workspaces", "Change name"], "Windows em inglês: interface em inglês");
});
await com("pt-BR", "#/", async (A) => {
  const r = await ler(A); assert.deepEqual([r.ativo, r.lang, r.titulo, r.botao], ["pt-BR", "pt-BR", "Seus workspaces", "Trocar nome"], "Windows em português: interface em português, igual a antes");
});
await com("fr-FR", "#/", async (A) => {
  const r = await ler(A); assert.deepEqual([r.ativo, r.titulo], ["en", "Your workspaces"], "Windows em francês (sem tradução): inglês");
});
await com("en-US", "#/", async (A) => { // o seletor troca, recarrega e lembra
  assert.equal(await A.js(`document.getElementById("idioma").value`), "auto", "começa em automático");
  await A.js(`(() => { const s = document.getElementById("idioma"); s.value = "pt-BR"; s.dispatchEvent(new Event("change")); })()`);
  await dormir(2500); await A.js(`void 0`).catch(() => {});
  const r = await ler(A); assert.deepEqual([r.ativo, r.titulo], ["pt-BR", "Seus workspaces"], "escolheu português: a tela recarregou em português mesmo com o Windows em inglês");
  assert.equal(await A.js(`document.getElementById("idioma").value`), "pt-BR", "e o seletor mostra a escolha");
  await A.js(`(() => { const s = document.getElementById("idioma"); s.value = "auto"; s.dispatchEvent(new Event("change")); })()`);
  await dormir(2500);
  assert.equal((await ler(A)).ativo, "en", "voltou para automático: segue o Windows (inglês)");
});
{
  const A = await abrirAmbiente({ exe, idioma: "pt-BR" });
  try { await A.cdp("Page.navigate", { url: A.url + "/?idioma=en#/" }); await dormir(2500); await A.js(`void 0`); assert.equal((await ler(A)).titulo, "Your workspaces", "?idioma=en vence o Windows em português"); } finally { A.fechar(); }
}
console.log("idioma e2e: ok (en-US, pt-BR, fr-FR cai no inglês, seletor com persistência, ?idioma= vence)");
