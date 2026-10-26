// Revisão de segurança do modo hospedado, no navegador e na rede (docs/seguranca-servidor.md): cabeçalhos, cookie, link do dono de uso único,
// Origin, rotas só locais ausentes, limite de tentativas e, contra o binário real, nenhum segredo gravado em claro no banco.
//   node scripts/seguranca-servidor-e2e.mjs                         contra o stub (confere o contrato da página)      (~15 s)
//   node scripts/seguranca-servidor-e2e.mjs --rust <blueprint.exe>  contra o binário real (serve --hospedar), com o stub de proxy
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { abrirNavegador, servidorDoTeste } from "./colaboracao-navegador.mjs";

const S = await servidorDoTeste(), real = process.argv.includes("--rust");
const pessoas = [];
const abrir = async (nome) => { const p = await abrirNavegador(nome); pessoas.push(p); return p; };
const segredos = [S.linkDono.split("#")[1]];
try {
  const ana = await abrir("ana");
  await ana.ir(S.linkDono);
  await ana.ate(`!document.getElementById("convite-campos").hidden`, 8000, "entrada do dono");
  await ana.js(`document.getElementById("convite-nome").value = "Ana"; document.getElementById("convite-form").requestSubmit()`);
  await ana.ate(`location.hash.startsWith("#/c/") && sessao.modo === "servidor"`, 10000, "dono entrou");

  // cabeçalhos: Referrer-Policy, nosniff e no-store na API; X-Frame-Options na página
  const cab = (rota) => ana.js(`fetch(${JSON.stringify(rota)}, { cache: "no-store" }).then((r) => Object.fromEntries(["referrer-policy", "x-content-type-options", "cache-control", "x-frame-options"].map((h) => [h, r.headers.get(h)])))`);
  const api = await cab("/api/sessao"), pagina = await cab("/");
  assert.equal(api["referrer-policy"], "no-referrer", "Referrer-Policy na API");
  assert.equal(api["x-content-type-options"], "nosniff");
  assert.match(api["cache-control"] || "", /no-store/, "a API não fica em cache");
  assert.equal(pagina["referrer-policy"], "no-referrer", "Referrer-Policy na página");
  assert.equal(pagina["x-frame-options"], "DENY", "a página não abre em frame de outro site");

  // cookie de sessão: HttpOnly, SameSite=Lax, Path=/ (e Secure no binário real, antes de o proxy de teste tirar)
  const entrada = S.pedidos.find((p) => p.caminho === "/api/convite/entrar" && p.cookie);
  assert.ok(entrada, "a entrada devolve Set-Cookie");
  assert.match(entrada.cookie, /^bp_sessao=[A-Za-z0-9_-]{43};/, "cookie bp_sessao com 43 caracteres");
  for (const f of [/HttpOnly/i, /SameSite=Lax/i, /Path=\//i]) assert.match(entrada.cookie, f, `cookie com ${f}`);
  if (real) assert.match(entrada.cookie, /;\s*Secure/i, "Secure no modo hospedado");
  const { result } = await ana.cdp("Network.getAllCookies");
  const ck = result.cookies.find((c) => c.name === "bp_sessao");
  assert.equal(ck.httpOnly, true, "o JavaScript da página não lê o cookie");
  assert.equal(await ana.js(`document.cookie.includes("bp_sessao")`), false);

  // Origin: com o cookie, POST sem Origin ou de outro site é recusado
  const comCookie = (origem) => fetch(S.url + "/api/convites", { method: "POST", headers: { "content-type": "application/json", cookie: `bp_sessao=${ck.value}`, ...(origem ? { origin: origem } : {}) }, body: JSON.stringify({ workspaceId: "sm", papel: "colaborador" }) });
  for (const origem of ["https://mal.exemplo", ""]) {
    const r = await comCookie(origem);
    assert.equal(r.status, 403, `POST com cookie e Origin "${origem}" é recusado`);
    assert.equal((await r.json()).codigo, "origem-invalida");
  }

  // rotas só do modo local não existem no hospedado
  for (const [rota, corpo] of [["/api/atualizacao"], ["/api/mcp/iniciar", {}], ["/api/terminal/sessoes"], ["/api/mcp/hook/sessao", {}]]) assert.equal((await ana.api(rota, corpo)).status, 404, `${rota} não existe no modo hospedado`);

  // a chave de MCP aparece uma vez; depois só o prefixo
  const chave = await ana.api("/api/chave", {});
  assert.equal(chave.status, 201);
  segredos.push(chave.corpo.chave);
  const depois = await ana.api("/api/chave");
  assert.equal(JSON.stringify(depois.corpo).includes(chave.corpo.chave), false, "GET /api/chave nunca devolve a chave");
  const conv = await ana.api("/api/convites", { workspaceId: "sm", papel: "colaborador" });
  segredos.push(conv.corpo.link.split("#")[1]);
  assert.equal(JSON.stringify((await ana.api("/api/convites?workspace=sm")).corpo).includes(segredos.at(-1)), false, "a lista de convites não devolve o segredo");

  // link do dono: uso único; segredo errado conta tentativa e o limite responde 429 com os segundos
  const outro = await abrir("outro");
  await outro.ir(S.linkDono);
  await outro.ate(`/Convite indisponível/.test(document.getElementById("convite-titulo").textContent)`, 8000, "o link do dono não serve duas vezes");
  let ultimo;
  for (let i = 0; i < 11; i++) ultimo = await outro.api("/api/convite/conferir", { segredo: "x".repeat(43) });
  assert.equal(ultimo.status, 429, "limite de tentativas");
  assert.equal(ultimo.corpo.codigo, "muitas-tentativas");
  assert.ok(ultimo.corpo.params?.segundos > 0, "diz quanto esperar");
  await outro.ir(`${S.url}/convite#${segredos.at(-1)}`);
  await outro.ate(`/Espere \\d+ s/.test(document.getElementById("convite-texto").textContent + document.getElementById("convite-erro").textContent)`, 8000, "a tela mostra a espera em segundos");

  // nenhum segredo viajou na URL ou no Referer; no binário real, nenhum segredo está em claro no banco
  for (const s of segredos) assert.equal(S.pedidos.some((p) => p.caminho.includes(s) || p.referer.includes(s)), false, "segredo fora de URL e Referer");
  if (S.banco) {
    const bytes = readdirSync(S.banco).filter((f) => f.startsWith("b.db")).map((f) => readFileSync(join(S.banco, f)).toString("latin1")).join("");
    for (const s of segredos) assert.equal(bytes.includes(s), false, "só o hash vai para o banco");
  }
  for (const p of pessoas) assert.deepEqual(p.erros, [], `sem erro no console (${p.nome})`);
  console.log(`segurança e2e: ok (${real ? "binário real" : "stub"}: cabeçalhos, cookie HttpOnly/Lax${real ? "/Secure" : ""}, Origin, rotas locais ausentes, chave e convite só uma vez, dono de uso único, limite de tentativas${S.banco ? ", banco sem segredo" : ""})`);
} finally {
  for (const p of pessoas) p.fechar();
  S.fechar();
}
