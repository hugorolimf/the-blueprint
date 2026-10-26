// Duas páginas no mesmo blueprint ao mesmo tempo, gravando por operações (/api/mudancas) e recebendo ao vivo (/api/eventos): os cenários do
// desenho (docs/colaboracao.md, seção 6) e a presença. Edge sem janela, banco temporário, servidor com BLUEPRINT_HOSPEDAR=1 (liga a presença).
//   node scripts/duas-pessoas-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; ~20 s)
// Antes da frente ACESSO as duas páginas são o mesmo membro ("local"): os avisos saem como "outra janela" (cenário 4). Com a ACESSO, trocar
// para dois convites (dois membros) e conferir o nome da outra pessoa no aviso.
import assert from "node:assert/strict";
import { rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { espera, janela } from "./lib-e2e-edge.mjs";
import { dormir, exePadrao, http, ligarServidor, pastaDeDados, raiz } from "./lib-e2e-rust.mjs";

const DADOS = { usuario: { nome: "Teste" }, blueprints: [{ id: "p1", nome: "Projeto", acoes: [
  { id: "f1", nome: "Ação um", inicio: "2026-11-01", fim: "2026-11-05", status: "andamento", planos: [{ id: "t1", texto: "Plano um", descricao: "d" }, { id: "t2", texto: "Plano dois" }], checkpoints: [] },
  { id: "f9", nome: "Ação nove", inicio: "2026-11-06", fim: "2026-11-09", status: "planejado", planos: [{ id: "t9", texto: "Plano nove" }], checkpoints: [] }] }] };
const pasta = await pastaDeDados("duas-pessoas", { BLUEPRINT_HOSPEDAR: "1" });
writeFileSync(join(pasta.dir, "data.json"), JSON.stringify(DADOS));
pasta.env.BLUEPRINT_ASSETS_DIR = raiz; // serve web/ da pasta: vale o que está no disco
const filhos = [await ligarServidor(exePadrao(), pasta)];
const servidor = async () => (await http(pasta.url + "/api/dados")).json;
const planoNoServidor = async (id) => (await servidor()).blueprints[0].acoes.flatMap((a) => a.planos).find((p) => p.id === id);
const P = (id) => `opsAchar(arquivo, "plano", "${id}").no`;
const E = (id) => `!!opsAchar(arquivo, "plano", "${id}")`;
const aviso = `(() => { const el = document.getElementById("sinc-aviso"); return el && !el.hidden ? el.textContent : ""; })()`;
const pausar = (j) => j.js(`aoVivoES.close(); aoVivoES = { readyState: 2, close() {} }`); // a página fica sem SSE (como uma rede caída)
try {
  const [A, B] = [await janela(pasta, "a", "/#/p/p1", filhos), await janela(pasta, "b", "/#/p/p1", filhos)];
  for (const j of [A, B]) await espera(async () => (await j.js(`typeof aoVivoAberto === "function" && aoVivoAberto() && !!aoVivoBase && ${E("t1")}`)) === true, "a página carregou e abriu o SSE");

  // 1) campos diferentes do mesmo plano: ficam os dois, sem aviso, e cada página vê a do outro
  const t0 = Date.now();
  await Promise.all([A.js(`${P("t1")}.texto = "título da A"; mudou()`), B.js(`${P("t1")}.descricao = "descrição do B"; mudou()`)]);
  const ateB = await espera(async () => (await B.js(`${P("t1")}.texto`)) === "título da A", "B vê o título da A");
  await espera(async () => (await A.js(`${P("t1")}.descricao`)) === "descrição do B", "A vê a descrição do B");
  const p1 = await planoNoServidor("t1");
  assert.deepEqual([p1.texto, p1.descricao], ["título da A", "descrição do B"]);
  assert.equal((await A.js(aviso)) + (await B.js(aviso)), "", "ninguém recebe aviso");
  console.log(`  1) campos diferentes: ok (B viu a mudança da A ${ateB} ms depois do mudou(), com os 400 ms de espera do salvar)`);

  // 2) o mesmo campo: vale a última gravação e quem perdeu é avisado, com Desfazer
  await pausar(B);
  await A.js(`${P("t1")}.texto = "A"; mudou()`);
  await espera(async () => (await planoNoServidor("t1")).texto === "A", "a gravação da A chegou");
  await B.js(`${P("t1")}.texto = "B"; mudou()`);
  await espera(async () => (await planoNoServidor("t1")).texto === "B", "a de B venceu (última gravação do campo)");
  await espera(async () => (await B.js(aviso)).includes("outra janela"), "B é avisado (mesmo membro: outra janela)");
  await espera(async () => (await A.js(`${P("t1")}.texto`)) === "B", "A recebe ao vivo a gravação de B");
  await B.js(`[...document.querySelectorAll("#sinc-aviso button")].find((b) => b.textContent === "Desfazer").click()`);
  await espera(async () => (await planoNoServidor("t1")).texto === "A", "Desfazer devolve o valor que tinha sido sobrescrito");
  await espera(async () => (await A.js(`${P("t1")}.texto`)) === "A", "e A vê o Desfazer");
  await B.js(`aoVivoConectar(true)`);
  console.log("  2) mesmo campo: ok (vale a última gravação, aviso com Desfazer)");

  // 3) apagar contra editar: apagar vence e quem editou é avisado
  await pausar(B);
  await A.js(`D.blueprints[0].acoes = D.blueprints[0].acoes.filter((a) => a.id !== "f9"); mudou()`);
  await espera(async () => !(await servidor()).blueprints[0].acoes.some((a) => a.id === "f9"), "a ação f9 foi apagada");
  await B.js(`${P("t9")}.texto = "editado em B"; mudou()`);
  await espera(async () => (await B.js(aviso)).includes("apagou este item"), "B é avisado de que o item foi apagado");
  assert.equal(await B.js(E("t9")), false, "o item sai da tela de B");
  await B.js(`aoVivoConectar(true)`);
  await espera(async () => (await B.js(`D.blueprints[0].acoes.some((a) => a.id === "f9")`)) === false, "ao reconectar, B recebe o apagar da ação");
  assert.equal((await servidor()).blueprints[0].acoes.some((a) => a.id === "f9"), false, "a ação continua apagada");
  console.log("  3) apagar contra editar: ok");

  // 4) presença: A vê B e o contorno do plano que B está editando; some quando B sai do campo
  await espera(async () => (await A.js(`document.querySelectorAll("#presenca .pres-av").length`)) >= 1, "A vê B no cabeçalho");
  await A.js(`document.getElementById("b-abrir").click()`);
  await B.js(`editar(true, { proj: "p1", sel: "f1" })`);
  await dormir(300);
  await B.js(`document.querySelector('#editor [data-t="t2"] input[data-c=texto]').focus()`);
  await espera(async () => (await A.js(`document.querySelector("#presenca-camada .pres-contorno")?.textContent || ""`)) !== "", "A vê o contorno do plano em edição");
  await B.js(`document.activeElement.blur(); editar(false)`);
  await espera(async () => (await A.js(`document.querySelectorAll("#presenca-camada .pres-contorno").length`)) === 0, "o contorno some quando B sai do campo");
  console.log("  4) presença: ok (avatar no cabeçalho, contorno do item em edição)");

  assert.deepEqual([...A.erros, ...B.erros], [], "sem erro no console");
  console.log("duas pessoas e2e: ok");
} finally {
  for (const f of filhos) { try { f.kill(); } catch {} }
  setTimeout(() => { try { rmSync(pasta.dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800);
}
