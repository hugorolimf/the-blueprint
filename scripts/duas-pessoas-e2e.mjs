// Duas pessoas (dois membros de verdade) no mesmo blueprint ao mesmo tempo, gravando por operações (/api/mudancas) e recebendo ao vivo
// (/api/eventos): os cenários do desenho (docs/colaboracao.md, seção 6) e a presença. `blueprint serve --hospedar` com banco temporário atrás
// de um proxy de teste; a Dona entra pelo link do dono impresso pelo servidor e convida o Bruno como colaborador; cada uma num Edge sem janela.
//   node scripts/duas-pessoas-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; ~20 s)
import assert from "node:assert/strict";
import { createServer, request } from "node:http";
import { rmSync } from "node:fs";
import { espera, janela } from "./lib-e2e-edge.mjs";
import { dormir, exePadrao, pastaDeDados, raiz } from "./lib-e2e-rust.mjs";
import { PUBLICA, entrar, ligarHospedado, navegador } from "./lib-hospedado.mjs";

const DADOS = { usuario: { nome: "Equipe" }, blueprints: [{ id: "p1", nome: "Projeto", acoes: [
  { id: "f1", nome: "Ação um", inicio: "2026-11-01", fim: "2026-11-05", status: "andamento", planos: [{ id: "t1", texto: "Plano um", descricao: "d" }, { id: "t2", texto: "Plano dois" }], checkpoints: [] },
  { id: "f9", nome: "Ação nove", inicio: "2026-11-06", fim: "2026-11-09", status: "planejado", planos: [{ id: "t9", texto: "Plano nove" }], checkpoints: [] }] }] };
const pasta = await pastaDeDados("duas-pessoas", { BLUEPRINT_ASSETS_DIR: raiz }); // serve web/ da pasta: vale o que está no disco
await (await import("node:fs/promises")).writeFile(pasta.env.DADOS_ARQ, JSON.stringify(DADOS));
const servidor = await ligarHospedado(exePadrao(), pasta);
// o proxy de teste, mais a Origin: o navegador fala com http://127.0.0.1:<porta>, e o servidor só aceita a origem pública (como atrás do Caddy)
const proxy = await new Promise((ok) => {
  const alvo = new URL(pasta.url);
  const srv = createServer((req, res) => {
    const cab = { ...req.headers, host: "equipe.teste", "x-forwarded-proto": "https", "x-forwarded-for": "127.0.0.1", ...(req.headers.origin ? { origin: PUBLICA } : {}) };
    const ida = request({ hostname: alvo.hostname, port: alvo.port, path: req.url, method: req.method, headers: cab }, (volta) => { res.writeHead(volta.statusCode, volta.headers); volta.pipe(res); });
    ida.on("error", () => { if (!res.headersSent) res.writeHead(502); res.end(); }); // o servidor caiu no meio (fim do teste)
    req.pipe(ida);
  });
  srv.listen(0, "127.0.0.1", () => ok({ url: `http://127.0.0.1:${srv.address().port}`, fechar: () => srv.close() }));
});
const filhos = [servidor.proc];
const link = servidor.saida().match(/https:\/\/equipe\.teste\/convite#[A-Za-z0-9_-]{43}/)[0];
const dona = await entrar(proxy, link, "Dona");
const convite = (await navegador(proxy, "POST", "/api/convites", { cookie: dona.cookie, corpo: { workspaceId: "sm", papel: "colaborador" } })).json;
const bruno = await entrar(proxy, convite.link, "Bruno");
assert.ok(dona.cookie && bruno.cookie, "as duas pessoas entraram pelos links");
const servidorTem = async () => (await navegador(proxy, "GET", "/api/dados", { cookie: dona.cookie })).json;
const planoNoServidor = async (id) => (await servidorTem()).blueprints[0].acoes.flatMap((a) => a.planos).find((p) => p.id === id);
const P = (id) => `opsAchar(arquivo, "plano", "${id}").no`;
const E = (id) => `!!opsAchar(arquivo, "plano", "${id}")`;
const aviso = `(() => { const el = document.getElementById("sinc-aviso"); return el && !el.hidden ? el.textContent : ""; })()`;
const pausar = (j) => j.js(`aoVivoES.close(); aoVivoES = { readyState: 2, close() {} }`); // a página fica sem SSE (como uma rede caída)
const site = { dir: pasta.dir, url: proxy.url };
try {
  const [A, B] = [await janela(site, "a", "/#/p/p1", filhos, { cookie: dona.cookie }), await janela(site, "b", "/#/p/p1", filhos, { cookie: bruno.cookie })];
  for (const j of [A, B]) await espera(async () => (await j.js(`typeof aoVivoAberto === "function" && aoVivoAberto() && !!aoVivoBase && ${E("t1")}`)) === true, "a página carregou e abriu o SSE", 10000);

  // 1) campos diferentes do mesmo plano: ficam os dois, sem aviso, e cada página vê a do outro
  const t0 = Date.now();
  await Promise.all([A.js(`${P("t1")}.texto = "título da A"; mudou()`), B.js(`${P("t1")}.descricao = "descrição do B"; mudou()`)]);
  const ateB = await espera(async () => (await B.js(`${P("t1")}.texto`)) === "título da A", "B vê o título da A");
  await espera(async () => (await A.js(`${P("t1")}.descricao`)) === "descrição do B", "A vê a descrição do B");
  const p1 = await planoNoServidor("t1");
  assert.deepEqual([p1.texto, p1.descricao], ["título da A", "descrição do B"]);
  assert.equal((await A.js(aviso)) + (await B.js(aviso)), "", "ninguém recebe aviso");
  console.log(`  1) campos diferentes: ok (o Bruno viu a mudança da Dona ${ateB} ms depois do mudou(), com os 400 ms de espera do salvar)`);

  // 2) o mesmo campo: vale a última gravação e quem perdeu é avisado, com Desfazer
  await pausar(B);
  await A.js(`${P("t1")}.texto = "A"; mudou()`);
  await espera(async () => (await planoNoServidor("t1")).texto === "A", "a gravação da A chegou");
  await B.js(`${P("t1")}.texto = "B"; mudou()`);
  await espera(async () => (await planoNoServidor("t1")).texto === "B", "a de B venceu (última gravação do campo)");
  await espera(async () => (await B.js(aviso)).includes("Dona tinha mudado"), "Bruno é avisado com o nome da Dona");
  await espera(async () => (await A.js(aviso)).includes("Bruno mudou"), "e a Dona, com o nome do Bruno");
  await espera(async () => (await A.js(`${P("t1")}.texto`)) === "B", "A recebe ao vivo a gravação de B");
  await B.js(`[...document.querySelectorAll("#sinc-aviso button")].find((b) => b.textContent === "Desfazer").click()`);
  await espera(async () => (await planoNoServidor("t1")).texto === "A", "Desfazer devolve o valor que tinha sido sobrescrito");
  await espera(async () => (await A.js(`${P("t1")}.texto`)) === "A", "e A vê o Desfazer");
  await B.js(`aoVivoConectar(true)`);
  console.log("  2) mesmo campo: ok (vale a última gravação, aviso com Desfazer)");

  // 3) apagar contra editar: apagar vence e quem editou é avisado
  await pausar(B);
  await A.js(`D.blueprints[0].acoes = D.blueprints[0].acoes.filter((a) => a.id !== "f9"); mudou()`);
  await espera(async () => !(await servidorTem()).blueprints[0].acoes.some((a) => a.id === "f9"), "a ação f9 foi apagada");
  await B.js(`${P("t9")}.texto = "editado em B"; mudou()`);
  await espera(async () => (await B.js(aviso)).includes("Dona apagou este item"), "Bruno é avisado de que a Dona apagou o item");
  assert.equal(await B.js(E("t9")), false, "o item sai da tela de B");
  await B.js(`aoVivoConectar(true)`);
  await espera(async () => (await B.js(`D.blueprints[0].acoes.some((a) => a.id === "f9")`)) === false, "ao reconectar, B recebe o apagar da ação");
  assert.equal((await servidorTem()).blueprints[0].acoes.some((a) => a.id === "f9"), false, "a ação continua apagada");
  console.log("  3) apagar contra editar: ok");

  // 4) presença: A vê B e o contorno do plano que B está editando; some quando B sai do campo
  await espera(async () => (await A.js(`[...document.querySelectorAll("#presenca .pres-av")].map((e) => e.title).join()`)) === "Bruno", "a Dona vê o Bruno no cabeçalho");
  await A.js(`document.getElementById("b-abrir").click()`);
  await B.js(`editar(true, { proj: "p1", sel: "f1" })`);
  await dormir(300);
  await B.js(`document.querySelector('#editor [data-t="t2"] input[data-c=texto]').focus()`);
  await espera(async () => (await A.js(`document.querySelector("#presenca-camada .pres-contorno")?.textContent || ""`)) === "Bruno", "a Dona vê o contorno do plano que o Bruno edita");
  await B.js(`document.activeElement.blur(); editar(false)`);
  await espera(async () => (await A.js(`document.querySelectorAll("#presenca-camada .pres-contorno").length`)) === 0, "o contorno some quando B sai do campo");
  console.log("  4) presença: ok (avatar no cabeçalho, contorno do item em edição)");

  assert.deepEqual([...A.erros, ...B.erros], [], "sem erro no console");
  console.log("duas pessoas e2e: ok");
} finally {
  for (const f of filhos) { try { f.kill(); } catch {} } // só os processos que este script abriu
  proxy.fechar();
  setTimeout(() => { try { rmSync(pasta.dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800);
}
