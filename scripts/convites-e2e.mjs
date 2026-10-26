// Convites, papéis e chaves de ponta a ponta no modo hospedado (pelo proxy de teste): o dono convida, a segunda pessoa entra pelo link, o
// agente dela grava pelo `--mcp` com a chave (curl pela entrada padrão), o supervisor só lê, o visitante vê só o blueprint do link e cai quando
// o link é revogado, e tentativas demais levam 429. Banco temporário; nunca o real.
// Uso: node scripts/convites-e2e.mjs [caminho do blueprint.exe]   (padrão: target/debug/blueprint.exe; ~8 s, sem gastar tokens)
import assert from "node:assert/strict";
import { rm } from "node:fs/promises";
import { exePadrao, mcpStdio } from "./lib-e2e-rust.mjs";
import { ambiente, entrar, navegador, segredoDe } from "./lib-hospedado.mjs";

const exe = exePadrao();
const DADOS = { blueprints: [
  { id: "p1", nome: "Portal", acoes: [{ id: "f1", nome: "Base", inicio: "2026-11-01", fim: "2026-11-05", status: "andamento", planos: [{ id: "t1", texto: "Plano um", descricao: "d", agente: { comentarios: [{ texto: "comentário do agente" }] } }], checkpoints: [] }] },
  { id: "p2", nome: "Aberto", acoes: [] },
] };
const amb = await ambiente(exe, "blueprint-convites", DADOS);
const mcps = [];
try {
  const { proxy } = amb;
  const link = amb.servidor.saida().match(/https:\/\/equipe\.teste\/convite#[A-Za-z0-9_-]{43}/)[0];
  const dono = await entrar(proxy, link, "Dona");
  const convidar = async (corpo) => {
    const r = await navegador(proxy, "POST", "/api/convites", { cookie: dono.cookie, corpo: { workspaceId: "sm", ...corpo } });
    assert.equal(r.status, 201, r.texto);
    return r.json;
  };

  const colabConvite = await convidar({ papel: "colaborador" });
  assert.match(colabConvite.link, /^https:\/\/equipe\.teste\/convite#/);
  const bruno = await entrar(proxy, colabConvite.link, "Bruno");
  assert.equal(bruno.json.destino, "/#/c/sm");
  const pessoas = await navegador(proxy, "GET", "/api/pessoas?workspace=sm", { cookie: dono.cookie });
  assert.deepEqual(pessoas.json.membros.map((m) => [m.nome, m.papel]).sort(), [["Bruno", "colaborador"], ["Dona", "dono"]]);
  assert.equal((await navegador(proxy, "GET", "/api/pessoas?workspace=sm", { cookie: bruno.cookie })).status, 403, "colaborador não lista pessoas");
  console.log("ok: o dono convidou e o Bruno entrou pelo link como colaborador");

  const chave = (await navegador(proxy, "POST", "/api/chave", { cookie: bruno.cookie, corpo: {} })).json.chave;
  assert.match(chave, /^bpk_[A-Za-z0-9_-]{43}$/);
  const agente = (k, curl = "1") => { const m = mcpStdio(exe, { ...process.env, BLUEPRINT_URL: proxy.url, BLUEPRINT_CHAVE: k, BLUEPRINT_MCP_CURL: curl }); mcps.push(m); return m; };
  const semCurl = agente(chave, "0"); // http:// direto pelo socket, com o mesmo Authorization
  await semCurl.iniciar();
  assert.equal((await semCurl.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" })).erro, false);
  const doBruno = agente(chave);
  await doBruno.iniciar();
  const ids = { workspace_id: "sm", blueprint_id: "p1", acao_id: "f1", plano_id: "t1" };
  const leu = await doBruno.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" });
  assert.equal(leu.erro, false, leu.texto);
  const gravou = await doBruno.chamar("registrar_progresso", { ...ids, texto: "Comecei pelo agente do Bruno", agente: "Claude do Bruno" });
  assert.equal(gravou.erro, false, gravou.texto);
  const fila = await navegador(proxy, "GET", "/api/mcp/fila", { cookie: dono.cookie });
  assert.equal(fila.json.itens.at(-1).autor.nome, "Bruno", "o item da fila leva quem mandou");
  console.log("ok: o agente do Bruno leu e gravou pelo --mcp com a chave dele (curl, chave pela entrada padrão)");

  const sup = await entrar(proxy, (await convidar({ papel: "supervisor" })).link, "Sueli");
  const chaveSup = (await navegador(proxy, "POST", "/api/chave", { cookie: sup.cookie, corpo: {} })).json.chave;
  const daSueli = agente(chaveSup);
  await daSueli.iniciar();
  assert.equal((await daSueli.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" })).erro, false);
  const recusado = await daSueli.chamar("registrar_progresso", { ...ids, texto: "não pode" });
  assert.equal(recusado.erro, true);
  assert.match(recusado.texto, /403/);
  assert.equal((await navegador(proxy, "PUT", "/api/dados", { cookie: sup.cookie, corpo: DADOS })).json?.codigo, "use-mudancas");
  console.log("ok: a supervisora lê pelo agente, mas não grava (403) nem troca os dados");

  await navegador(proxy, "POST", "/api/chave/revogar", { cookie: bruno.cookie, corpo: {} });
  const revogada = await doBruno.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" });
  assert.equal(revogada.erro, true);
  assert.match(revogada.texto, /401/);
  console.log("ok: a chave revogada leva 401 no --mcp");

  const pub = await convidar({ papel: "visitante", blueprintId: "p2", abre: "apresentacao" });
  const visitante = await entrar(proxy, pub.link);
  assert.equal(visitante.json.destino, "/#/p/p2");
  const visto = await navegador(proxy, "GET", "/api/dados", { cookie: visitante.cookie });
  assert.deepEqual(visto.json.blueprints.map((b) => b.id), ["p2"]);
  assert.doesNotMatch(visto.texto, /comentário do agente/);
  assert.equal((await navegador(proxy, "GET", "/api/mcp/fila", { cookie: visitante.cookie })).status, 403);
  await navegador(proxy, "POST", "/api/convites/revogar", { cookie: dono.cookie, corpo: { id: pub.convite.id } });
  assert.equal((await navegador(proxy, "GET", "/api/dados", { cookie: visitante.cookie })).status, 401, "revogar o link derruba o visitante");
  assert.equal((await navegador(proxy, "POST", "/api/convite/conferir", { corpo: { segredo: segredoDe(pub.link) } })).json.codigo, "convite-invalido");
  console.log("ok: o visitante vê só o blueprint do link, sem os comentários do agente, e cai quando o link é revogado");

  let ultimo;
  for (let i = 0; i < 12; i++) ultimo = await navegador(proxy, "POST", "/api/convite/conferir", { corpo: { segredo: "x".repeat(43) } });
  assert.equal(ultimo.status, 429);
  assert.ok(ultimo.json.params.segundos > 0);
  const atividade = await navegador(proxy, "GET", "/api/atividade?workspace=sm", { cookie: dono.cookie });
  const tipos = new Set(atividade.json.itens.map((i) => i.tipo));
  for (const t of ["entrou", "convite-criado", "convite-revogado"]) assert.ok(tipos.has(t), t);
  assert.doesNotMatch(atividade.texto, new RegExp(segredoDe(pub.link)), "a atividade nunca guarda segredo");
  console.log("ok: tentativas demais levam 429; a atividade registra entradas e convites sem segredo");
} finally {
  mcps.forEach((m) => m.fechar());
  amb.fechar();
  await new Promise((r) => setTimeout(r, 300));
  await rm(amb.d.dir, { recursive: true, force: true }).catch(() => {});
}
