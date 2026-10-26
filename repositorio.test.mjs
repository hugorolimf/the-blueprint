// A lógica pura do repositório GitHub do blueprint (web/js/repositorio-logica.js, script clássico carregado num contexto isolado como no navegador):
// a regra da URL (os mesmos casos de src/repositorio/tests.rs, em referencias/repositorio-urls.json), as frases dos estados, quando falta a pasta,
// a pasta de destino, o "Agora não" e que ligar/desligar o repositório vira uma operação de sincronia do blueprint.
//   node repositorio.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const le = (f) => readFileSync(join(import.meta.dirname, "web", "js", f), "utf8");
const guardado = new Map();
const ctx = vm.createContext({ structuredClone, navigator: { languages: ["pt-BR"] }, location: { search: "" }, localStorage: { getItem: (k) => guardado.get(k) ?? null, setItem: (k, v) => guardado.set(k, String(v)) } });
vm.runInContext(le("i18n.js"), ctx, { filename: "i18n.js" });
vm.runInContext(le("operacoes.js"), ctx, { filename: "operacoes.js" });
vm.runInContext(le("repositorio-logica.js") + "\nthis.api = { repositorioValido, repositorioFrase, repositorioSemPasta, repositorioPastaOk, repositorioAgoraNao };", ctx, { filename: "repositorio-logica.js" });
const A = ctx.api;
const js = (x) => JSON.parse(JSON.stringify(x));
const casos = JSON.parse(readFileSync(join(import.meta.dirname, "referencias", "repositorio-urls.json"), "utf8"));

// a regra da URL: os mesmos casos do Rust
for (const { entrada, url } of casos.aceitas) assert.equal(A.repositorioValido(entrada)?.url, url, `devia aceitar ${JSON.stringify(entrada)}`);
for (const ruim of casos.recusadas) assert.equal(A.repositorioValido(ruim), null, `devia recusar ${JSON.stringify(ruim)}`);
assert.deepEqual(js(A.repositorioValido("https://github.com/dono/repo.git")), { url: "https://github.com/dono/repo", dono: "dono", nome: "repo" });
assert.equal(A.repositorioValido(undefined), null);
assert.equal(A.repositorioValido(null), null);

// as frases dos estados (sem estado conhecido, nada)
for (const e of ["url-invalida", "git-ausente", "com-acesso", "sem-acesso", "sem-resposta"]) assert.ok(A.repositorioFrase(e).length > 10, e);
assert.equal(A.repositorioFrase("sem-repositorio"), "");
assert.equal(A.repositorioFrase(undefined), "");
assert.match(A.repositorioFrase("sem-acesso"), /colaborador/);

// falta a pasta: tem repositório válido e nenhuma pasta ligada a ele (neste workspace) nesta máquina
const p = { id: "b1", repositorio: { url: "https://github.com/dono/repo" } };
assert.equal(A.repositorioSemPasta(p, [], "sm"), true);
assert.equal(A.repositorioSemPasta(p, undefined, "sm"), true);
assert.equal(A.repositorioSemPasta(p, [{ cwd: "c:/x", workspaceId: "sm", blueprintId: "b1" }], "sm"), false, "pasta ligada: nada a configurar");
assert.equal(A.repositorioSemPasta(p, [{ cwd: "c:/x", workspaceId: "w2", blueprintId: "b1" }], "sm"), true, "outro workspace com o mesmo id de blueprint não conta");
assert.equal(A.repositorioSemPasta(p, [{ cwd: "c:/x", workspaceId: "sm", blueprintId: "b2" }], "sm"), true);
assert.equal(A.repositorioSemPasta({ id: "b1" }, [], "sm"), false, "sem repositório");
assert.equal(A.repositorioSemPasta({ id: "b1", repositorio: { url: "http://evil.com/a/b" } }, [], "sm"), false, "URL inválida nunca pede configuração");
assert.equal(A.repositorioSemPasta(null, [], "sm"), false);

// a pasta de destino: caminho completo do Windows, sem controle nem ;
for (const ok of ["C:\\Projetos", "c:/projetos/app", "D:\\a b\\c", "\\\\servidor\\pasta"]) assert.equal(A.repositorioPastaOk(ok), true, ok);
for (const ruim of ["", "   ", "projetos", "relativa\\pasta", "C:\\a;b", "C:\\a\nb", "C:\\a\u0000b", "/tmp/x", "C:" , "x".repeat(501)]) assert.equal(A.repositorioPastaOk(ruim), false, JSON.stringify(ruim));

// "Agora não" é por blueprint e sobrevive a um armazenamento que falha
assert.equal(A.repositorioAgoraNao.ler("b1"), false);
A.repositorioAgoraNao.gravar("b1");
assert.equal(A.repositorioAgoraNao.ler("b1"), true);
assert.equal(A.repositorioAgoraNao.ler("b2"), false);
{
  const quebrado = vm.createContext({ localStorage: { getItem() { throw new Error("bloqueado"); }, setItem() { throw new Error("bloqueado"); } } });
  vm.runInContext(le("repositorio-logica.js") + "\nthis.api = { repositorioAgoraNao };", quebrado);
  assert.equal(quebrado.api.repositorioAgoraNao.ler("b1"), false);
  assert.doesNotThrow(() => quebrado.api.repositorioAgoraNao.gravar("b1"));
}

// conectar e desconectar o repositório viram operações do blueprint (sincroniza como qualquer campo) e aplicá-las dá a tela
const base = { empresa: "E", blueprints: [{ id: "b1", nome: "B", acoes: [] }], workspaces: [] };
const com = js(base); com.blueprints[0].repositorio = { url: "https://github.com/dono/repo" };
const aplicar = (arvore, lista) => { const t = js(arvore); for (const op of lista) ctx.aplicarOp(t, op); return js(t); };
const ligar = js(ctx.opsEntre(js(base), js(com)));
assert.deepEqual(ligar, [{ op: "alterar", tipo: "blueprint", id: "b1", campos: { repositorio: { url: "https://github.com/dono/repo" } }, antes: { repositorio: null } }]);
assert.deepEqual(aplicar(base, ligar), com);
const trocar = js(com); trocar.blueprints[0].repositorio = { url: "https://github.com/dono/outro" };
const t = js(ctx.opsEntre(js(com), js(trocar)));
assert.deepEqual(t[0].antes, { repositorio: { url: "https://github.com/dono/repo" } });
assert.deepEqual(aplicar(com, t), trocar);
const desligar = js(ctx.opsEntre(js(com), js(base)));
assert.deepEqual(desligar, [{ op: "alterar", tipo: "blueprint", id: "b1", campos: { repositorio: null }, antes: { repositorio: { url: "https://github.com/dono/repo" } } }]);
assert.deepEqual(aplicar(com, desligar), base);

console.log("repositorio.test.mjs: ok");
