// O modo hospedado de ponta a ponta: `blueprint serve --hospedar` atrás do proxy de teste exige HTTPS (pelo proxy), recusa Host e Origin de
// fora, põe os cabeçalhos de segurança, imprime o link do dono uma vez e a página abre por ele. Banco temporário; nunca o real.
// Uso: node scripts/servidor-e2e.mjs [caminho do blueprint.exe]   (padrão: target/debug/blueprint.exe; ~5 s, sem gastar tokens)
import assert from "node:assert/strict";
import { rm } from "node:fs/promises";
import { exePadrao, http } from "./lib-e2e-rust.mjs";
import { ambiente, entrar, ligarHospedado, navegador, PUBLICA } from "./lib-hospedado.mjs";

const exe = exePadrao();
const amb = await ambiente(exe, "blueprint-servidor");
let outro;
try {
  const { d, proxy } = amb;
  const impresso = amb.servidor.saida();
  const link = (impresso.match(/https:\/\/equipe\.teste\/convite#[A-Za-z0-9_-]{43}/) || [])[0];
  assert.ok(link, "o link do dono sai na primeira execução:\n" + impresso);

  const direto = await http(d.url + "/", { cabecalhos: { "x-forwarded-proto": "https", host: "equipe.teste" } });
  assert.equal(direto.status, 200, "o proxy de teste está na mesma máquina (loopback): o par é confiável");
  const semProxy = await http(d.url + "/api/sessao");
  assert.equal(semProxy.json?.codigo, "https-obrigatorio", "sem o X-Forwarded-Proto do proxy nada sai");
  const outroHost = await http(d.url + "/", { cabecalhos: { "x-forwarded-proto": "https", host: "evil.exemplo" } });
  assert.equal(outroHost.json?.codigo, "host-invalido");
  console.log("ok: sem HTTPS do proxy (403 https-obrigatorio) e com outro Host (403 host-invalido) nada sai");

  const pagina = await navegador(proxy, "GET", "/convite");
  assert.equal(pagina.status, 200);
  assert.match(pagina.texto, /<html/i);
  const cab = (await fetch(proxy.url + "/")).headers;
  assert.equal(cab.get("referrer-policy"), "no-referrer");
  assert.equal(cab.get("x-frame-options"), "DENY");
  assert.equal(cab.get("x-content-type-options"), "nosniff");
  assert.equal(cab.get("strict-transport-security"), "max-age=31536000");
  console.log("ok: a página sai pelo proxy com Referrer-Policy, X-Frame-Options, nosniff e HSTS");

  const deFora = await navegador(proxy, "POST", "/api/convite/entrar", { corpo: { segredo: link.split("#")[1], nome: "Dona" }, origem: "https://evil.exemplo" });
  assert.equal(deFora.json?.codigo, "origem-invalida", "Origin de fora é recusada antes de conferir o convite");
  const dono = await entrar(proxy, link, "Dona");
  assert.equal(dono.status, 200, JSON.stringify(dono.json));
  assert.equal(dono.json.destino, "/");
  assert.match(dono.setCookie, /HttpOnly/);
  assert.match(dono.setCookie, /SameSite=Lax/);
  assert.match(dono.setCookie, /Secure/);
  const sessao = await navegador(proxy, "GET", "/api/sessao", { cookie: dono.cookie });
  assert.deepEqual(sessao.json.acessos, [{ workspaceId: "*", blueprintId: "", papel: "dono", abre: "folha" }]);
  const dados = await navegador(proxy, "GET", "/api/dados", { cookie: dono.cookie });
  assert.equal(dados.status, 200);
  assert.equal(dados.json.blueprints[0].id, "p1");
  assert.equal((await navegador(proxy, "GET", "/api/dados")).json?.codigo, "sem-sessao");
  const deNovo = await entrar(proxy, link, "Outra").catch((e) => ({ erro: e.message }));
  assert.match(deNovo.erro || "", /404/, "o link do dono é de uso único");
  console.log("ok: Origin de fora recusada; o dono entrou pelo link (cookie HttpOnly, SameSite=Lax, Secure) e o link não vale de novo");

  amb.servidor.proc.kill();
  await new Promise((r) => setTimeout(r, 300));
  outro = await ligarHospedado(exe, d);
  assert.doesNotMatch(outro.saida(), /convite#/, "com dono, a segunda execução não imprime link");
  outro.proc.kill();
  await new Promise((r) => setTimeout(r, 300));
  outro = await ligarHospedado(exe, d, ["--novo-link-dono"]);
  assert.match(outro.saida(), /https:\/\/equipe\.teste\/convite#[A-Za-z0-9_-]{43}/, "--novo-link-dono imprime outro");
  console.log("ok: a segunda execução não imprime o link; --novo-link-dono imprime um novo");
  assert.ok(PUBLICA);
} finally {
  amb.fechar();
  outro?.proc.kill();
  await new Promise((r) => setTimeout(r, 300));
  await rm(amb.d.dir, { recursive: true, force: true }).catch(() => {});
}
