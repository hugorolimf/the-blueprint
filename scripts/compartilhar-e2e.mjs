// Compartilhar ponta a ponta (ação "Compartilhar um workspace pelo próprio app"): no app local o dono abre o diálogo Compartilhar, liga o
// túnel, vê o endereço, cria um link de colaborador; a pessoa entra pelo endereço do túnel, só vê o workspace compartilhado, as rotas só
// locais dão 404 nela, e o dono a vê na lista; com o binário real, uma mudança vai e volta ao vivo; o dono desliga e o túnel fecha.
//   node scripts/compartilhar-e2e.mjs                         contra o stub (scripts/stub-compartilhar.mjs)          (~15 s)
//   node scripts/compartilhar-e2e.mjs --rust <blueprint.exe>  contra `<exe> serve` (modo local) com BLUEPRINT_TUNEL_COMANDO apontando para
//                                                             scripts/tunel-falso.mjs (um .cmd numa pasta temporária); banco temporário
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { abrirNavegador, dormir } from "./colaboracao-navegador.mjs";
import { portaLivre } from "./lib-e2e-rust.mjs";
import { subirStubCompartilhar } from "./stub-compartilhar.mjs";

const raiz = resolve(import.meta.dirname, "..");
const iRust = process.argv.indexOf("--rust"), rust = iRust > 0 ? resolve(process.argv[iRust + 1]) : "";
const WS = "lab0001", BP = "bp7g8h9"; // data.exemplo.json: o workspace "Laboratório de ideias" e o único blueprint dele

// o servidor do teste: o stub, ou o binário real com o túnel falso
async function servidor() {
  if (!rust) { const s = await subirStubCompartilhar(); return { url: s.url, tunel: () => ({ publica: s.convidados, local: s.convidados }), fechar: s.fechar }; }
  const tmp = mkdtempSync(join(tmpdir(), "compartilhar-e2e-")), porta = await portaLivre(), info = join(tmp, "tunel.json");
  copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
  writeFileSync(join(tmp, "tunel-falso.cmd"), `@node "${join(raiz, "scripts", "tunel-falso.mjs")}" %*\r\n`);
  const url = `http://127.0.0.1:${porta}`;
  const proc = spawn(rust, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: raiz, BLUEPRINT_SEM_ATUALIZACAO: "1", BLUEPRINT_TUNEL_COMANDO: join(tmp, "tunel-falso.cmd"), TUNEL_FALSO_INFO: info }, stdio: "ignore", windowsHide: true });
  for (let i = 0; ; i++) { try { await fetch(url + "/api/dados"); break; } catch { if (i > 100) throw new Error("o servidor não respondeu"); await dormir(50); } }
  return {
    url,
    tunel: () => (existsSync(info) ? JSON.parse(readFileSync(info, "utf8")) : null),
    fechar: () => { try { proc.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800); },
  };
}

const S = await servidor();
const pessoas = [];
const abrir = async (nome) => { const p = await abrirNavegador(nome); pessoas.push(p); return p; };
const cp = (id) => `(document.getElementById(${JSON.stringify(id)})?.textContent || "")`;
const visivel = (id) => `(document.getElementById(${JSON.stringify(id)})?.offsetParent != null)`;
try {
  // 1. o dono, no app local, abre o workspace e o botão Compartilhar aparece na barra
  const dono = await abrir("dono");
  await dono.ir(`${S.url}/#/c/${WS}`);
  await dono.ate(`typeof carregado !== "undefined" && carregado && workspaceAtual?.id === ${JSON.stringify(WS)} && ${visivel("b-compartilhar")}`, 10000, "Compartilhar visível na barra do workspace (modo local)");
  assert.equal(await dono.js(visivel("b-pessoas-folha")), false, "no app local o botão Pessoas do modo hospedado continua escondido");

  // 2. o diálogo: estado desligado, explicação, foco no Ligar, região anunciada ao leitor de tela; Esc fecha
  await dono.clicar("#b-compartilhar");
  await dono.ate(`document.getElementById("equipe").open && ${cp("cp-titulo")} === "Compartilhamento desligado"`, 8000, "diálogo aberto, desligado");
  assert.equal(await dono.js(cp("equipe-titulo")), "Compartilhar");
  assert.equal(await dono.js(`document.activeElement.id`), "cp-botao", "o foco vai para o Ligar");
  assert.equal(await dono.js(cp("cp-botao")), "Ligar");
  assert.equal(await dono.js(`document.getElementById("cp-titulo").closest("[role=status]")?.getAttribute("aria-live")`), "polite", "estado numa região viva");
  assert.match(await dono.js(`document.querySelector(".cp .eq-dica").textContent`), /precisa ficar aberto.*endereço muda/s, "explica: app aberto e endereço novo a cada vez");
  await dono.cdp("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
  await dono.ate(`!document.getElementById("equipe").open`, 3000, "Esc fecha o diálogo");

  // 3. ligar: progresso (baixando, ligando) até o endereço público
  await dono.clicar("#b-compartilhar");
  await dono.ate(`${cp("cp-botao")} === "Ligar"`, 5000, "reaberto");
  await dono.js(`window.__titulos = []; new MutationObserver(() => __titulos.push(document.getElementById("cp-titulo").textContent)).observe(document.getElementById("cp-titulo"), { childList: true, characterData: true, subtree: true }); true`);
  await dono.clicar("#cp-botao");
  await dono.ate(`${visivel("cp-url")}`, rust ? 20000 : 8000, "ligado: o endereço aparece");
  const titulos = await dono.js(`__titulos`);
  if (!rust) assert.ok(titulos.includes("Baixando o túnel…") && titulos.includes("Ligando o túnel…"), "passou por baixando e ligando: " + titulos);
  assert.equal(await dono.js(cp("cp-titulo")), "Compartilhando «Laboratório de ideias»");
  const t = S.tunel();
  assert.ok(t, "o túnel informou o endereço");
  assert.equal(await dono.js(cp("cp-url")), t.publica, "o diálogo mostra o endereço do túnel");
  assert.equal(await dono.js(cp("cp-botao")), "Desligar");
  const pelaPortaLocal = (link) => link.replace(t.publica, t.local); // o domínio público do túnel falso é a porta local dele

  // 4. com o túnel ligado: quem tem acesso e o link de convite (colaborador, workspace inteiro), mostrado uma vez, no endereço do túnel
  await dono.ate(`!!document.getElementById("eq-conv-form") && /pessoa/.test(${cp("eq-pessoas-status")})`, 8000, "lista de pessoas e formulário de convite no diálogo");
  await dono.escolher(`#eq-conv-form [name=papel]`, "colaborador");
  await dono.js(`document.getElementById("eq-conv-form").requestSubmit()`);
  await dono.ate(`!document.getElementById("eq-link").hidden`, 8000, "o link aparece");
  const link = await dono.js(cp("eq-link-texto"));
  assert.ok(link.startsWith(t.publica + "/convite#"), "o link de convite sai no endereço do túnel: " + link);

  // 5. a pessoa entra pelo túnel e só vê o workspace compartilhado; sem o botão Compartilhar; rotas só locais dão 404
  const bia = await abrir("bia");
  await bia.ir(pelaPortaLocal(link));
  await bia.ate(`!document.getElementById("convite-campos").hidden`, 8000, "a convidada vê a entrada");
  await bia.js(`document.getElementById("convite-nome").value = "Bia"; document.getElementById("convite-form").requestSubmit()`);
  await bia.ate(`sessao.modo === "servidor" && typeof carregado !== "undefined" && carregado && location.hash.startsWith("#/c/")`, 10000, "Bia entrou");
  const bps = await bia.js(`[...(arquivo.blueprints || []), ...(arquivo.workspaces || []).flatMap((w) => w.blueprints || [])].map((b) => b.id)`);
  assert.deepEqual(bps, [BP], "a convidada só vê o blueprint do workspace compartilhado");
  assert.equal(await bia.js(`document.getElementById("b-compartilhar").hidden`), true, "Compartilhar não aparece para quem entra pelo túnel");
  for (const rota of ["/api/compartilhar", "/api/atualizacao", "/api/mcp/sessoes"]) assert.equal((await bia.api(rota)).status, 404, `${rota} não existe pelo túnel`);
  assert.equal((await bia.api("/api/compartilhar/desligar", {})).status, 404, "nem o desligar");

  // 6. o dono vê a Bia na lista (reabrindo o diálogo)
  await dono.js(`document.getElementById("equipe").close()`);
  await dono.clicar("#b-compartilhar");
  await dono.ate(`/Bia/.test(${cp("eq-pessoas")})`, 8000, "Bia aparece em Quem tem acesso");

  // 7. ao vivo entre dono e convidada (só o binário real tem /api/mudancas e /api/eventos)
  if (rust) {
    const nome = (j) => j.js(`opsAchar(arquivo, "blueprint", ${JSON.stringify(BP)})?.no.nome`);
    await bia.js(`opsAchar(arquivo, "blueprint", ${JSON.stringify(BP)}).no.nome = "Nome da Bia"; mudou(); true`);
    await dono.ate(`opsAchar(arquivo, "blueprint", ${JSON.stringify(BP)})?.no.nome === "Nome da Bia"`, 10000, "o dono vê ao vivo a mudança da convidada");
    await dono.js(`opsAchar(arquivo, "blueprint", ${JSON.stringify(BP)}).no.nome = "Nome do dono"; mudou(); true`);
    await bia.ate(`opsAchar(arquivo, "blueprint", ${JSON.stringify(BP)})?.no.nome === "Nome do dono"`, 10000, "a convidada vê ao vivo a mudança do dono");
    assert.equal(await nome(bia), "Nome do dono");
  } else console.log("  (stub: a mudança ao vivo só é conferida com --rust)");

  // 8. desligar (com confirmação): o diálogo volta a desligado, a lista some e o túnel fecha
  await dono.clicar("#cp-botao");
  await dormir(200);
  await dono.confirmar();
  await dono.ate(`${cp("cp-titulo")} === "Compartilhamento desligado" && ${cp("cp-equipe")} === ""`, 8000, "desligado");
  let fechado = false;
  for (let i = 0; i < 30 && !fechado; i++) { try { fechado = (await fetch(t.local + "/api/sessao")).status === 502; } catch { fechado = true; } if (!fechado) await dormir(200); }
  assert.ok(fechado, "o túnel fechou");
  for (const p of pessoas) assert.deepEqual(p.erros, [], `sem erro no console (${p.nome})`);
  console.log(`compartilhar e2e: ok (${rust ? "binário real + túnel falso" : "stub"}: botão só no local, diálogo acessível, ligar com progresso, endereço, convite no túnel, convidada só no workspace, rotas locais 404${rust ? ", ao vivo" : ""}, desligar)`);
} finally {
  for (const p of pessoas) p.fechar();
  S.fechar();
}
