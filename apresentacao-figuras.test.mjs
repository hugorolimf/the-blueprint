// Figuras da apresentação (web/js/apresentacao-figuras.js, script clássico num contexto isolado): a figura sai, é determinística, não deixa texto dos dados passar sem escape e respeita o limite de peças; os GIFs têm limite e assinatura conferida.
//   node apresentacao-figuras.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const raiz = import.meta.dirname;
const EMB = await import("./emblemas.mjs");
const codigo = readFileSync(join(raiz, "web", "js", "apresentacao-figuras.js"), "utf8");
const carrega = (extra = {}) => {
  const ctx = vm.createContext({ location: { href: "http://127.0.0.1:3000/", origin: "http://127.0.0.1:3000" }, atob, btoa, URL, Uint8Array, Promise, ...extra });
  ctx.window = ctx;
  vm.runInContext(codigo, ctx, { filename: "apresentacao-figuras.js" });
  return ctx;
};
const json = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes

const exemplo = JSON.parse(readFileSync(join(raiz, "data.exemplo.json"), "utf8")).blueprints[0];
const acao = (id, status, extra = {}) => ({ id, nome: id, inicio: "2026-11-01", fim: "2026-11-02", status, planos: [], checkpoints: [], ...extra });
const comProjetos = {
  id: "b", nome: "B", projetos: [{ id: "p1", nome: "Interface" }, { id: "p2", nome: "Servidor" }],
  acoes: [acao("a1", "concluido", { projetoId: "p1" }), acao("a2", "andamento", { projetoId: "p1" }), acao("a3", "planejado", { projetoId: "p2" }), acao("a4", "planejado")],
};

// ---- a figura sai e é determinística ----
const A = carrega({ EMB }), B = carrega({ EMB });
const svg = A.figuraDoBlueprint(exemplo, { largura: 480 });
assert.match(svg, /^<svg class="emblema marca-apresentacao" viewBox="0 0 480 \d+"/, "um <svg> com a classe .emblema (os tons do tema vêm dela)");
assert(svg.length > 2000 && !/NaN|undefined|Infinity/.test(svg), "desenho com conteúdo e sem número inválido");
assert.equal(svg, B.figuraDoBlueprint(exemplo, { largura: 480 }), "dois contextos desenham o mesmo SVG");
assert.equal(svg, A.figuraDoBlueprint(exemplo, { largura: 480 }), "a segunda chamada vem do cache");
assert(svg.includes('aria-hidden="true"') && !svg.includes("role="), "sem rótulo, a figura é decorativa");
assert.notEqual(A.figuraDoBlueprint(comProjetos), A.figuraDoBlueprint({ ...comProjetos, acoes: comProjetos.acoes.map((a) => ({ ...a, status: "planejado" })) }), "o progresso muda o desenho");

// ---- torres e andares seguem os dados ----
const d1 = json(A.figuraDoBlueprint.pecas(comProjetos));
assert.equal(d1.torres, 3, "dois projetos + as ações sem projeto (Geral)");
assert.equal(d1.andares, 4);
assert.equal(d1.feitos, 1, "só a ação concluída é andar pronto");
assert.equal(d1.pecas.length, 6 + 4, "mesa (2 lajes + 4 linhas de guia) e 4 andares");
assert(Math.abs(d1.progresso - (6 + 1) / 10) < 1e-9, "o progresso do kit pinta sólido exatamente a mesa e os andares prontos");
assert.equal(d1.pecas[6].tom, "hi", "o andar pronto vem logo depois da mesa");
const semProjetos = json(A.figuraDoBlueprint.pecas({ acoes: [acao("x", "planejado", { planos: [{ feito: true }, { feito: false }] }), acao("y", "concluido")] }));
assert.equal(semProjetos.torres, 2, "sem projetos, uma torre por ação");
assert.equal(semProjetos.andares, 3, "um andar por plano (e um para a ação sem planos)");
assert(A.figuraDoBlueprint({ id: "v", nome: "Vazio", acoes: [] }).startsWith("<svg"), "blueprint sem ações ainda desenha a mesa");

// ---- limite de peças ----
const grande = { projetos: Array.from({ length: 12 }, (_, i) => ({ id: "p" + i })), acoes: Array.from({ length: 240 }, (_, i) => acao("a" + i, i % 3 ? "planejado" : "concluido", { projetoId: "p" + (i % 12) })) };
const dg = json(A.figuraDoBlueprint.pecas(grande));
assert.equal(dg.torres, 8, "no máximo 8 torres");
assert(dg.andares <= 64 && dg.pecas.length <= 160, `dentro do limite de 160 peças (${dg.pecas.length})`);
assert(A.figuraDoBlueprint(grande).startsWith("<svg"));

// ---- nada dos dados entra sem escape ----
const hostil = '<img src=x onerror=alert(1)>"&\'';
const ruim = A.figuraDoBlueprint({ ...comProjetos, nome: hostil, descricao: hostil, projetos: [{ id: "p1", nome: hostil }, { id: "p2", nome: hostil }], acoes: comProjetos.acoes.map((a) => ({ ...a, nome: hostil })) }, { rotulo: hostil });
assert(!ruim.includes("<img") && !ruim.includes("onerror=alert(1)>") && !ruim.includes("<script"), "o rótulo hostil foi escapado");
assert(ruim.includes("&lt;img src=x onerror=alert(1)&gt;&quot;&amp;&#39;") && ruim.includes('role="img"'), "e aparece só como texto alternativo");
const semRotulo = A.figuraDoBlueprint({ ...comProjetos, nome: hostil, acoes: comProjetos.acoes.map((a) => ({ ...a, nome: hostil })) });
assert(!semRotulo.includes("onerror") && !semRotulo.includes("img src"), "o nome do blueprint e das ações nunca entra no SVG");

// ---- sem o kit, ou sem dados, devolve vazio ----
assert.equal(carrega().figuraDoBlueprint(exemplo), "", "kit indisponível: string vazia, sem quebrar");
assert.equal(A.figuraDoBlueprint(null), "");

// ---- GIFs ----
const gif = Buffer.concat([Buffer.from("GIF89a", "latin1"), Buffer.alloc(3000, 1)]);
const uri = (b) => `data:image/gif;base64,${b.toString("base64")}`;
assert.equal(await A.gifParaApresentacao(uri(gif)), uri(gif), "um GIF pequeno passa");
assert.equal(await A.gifParaApresentacao(uri(gif), { maxKb: 2 }), null, "passou do limite");
assert.equal(await A.gifParaApresentacao(uri(Buffer.concat([Buffer.from("<html>"), Buffer.alloc(100)]))), null, "base64 que não é GIF (a assinatura é conferida, não o nome)");
assert.equal(await A.gifParaApresentacao("data:image/svg+xml;base64,PHN2Zz48L3N2Zz4="), null, "só image/gif");
assert.equal(await A.gifParaApresentacao("javascript:alert(1)"), null);
assert.equal(await A.gifParaApresentacao("https://exemplo.invalid/a.gif"), null, "outra origem: recusa sem pedir nada");
const resposta = (corpo, tamanho = corpo.length) => async () => ({ ok: true, headers: { get: () => String(tamanho) }, arrayBuffer: async () => new Uint8Array(corpo).buffer });
const F = carrega({ EMB, fetch: resposta(gif) });
assert.equal(await F.gifParaApresentacao("/videos/a.gif"), uri(gif), "mesma origem: baixa e embute como data URI");
assert.equal(await carrega({ fetch: resposta(gif, 5 * 1024 * 1024) }).gifParaApresentacao("/videos/a.gif"), null, "o content-length acima do limite recusa antes de ler");
assert.equal(await carrega({ fetch: resposta(Buffer.from("<html>nao e gif")) }).gifParaApresentacao("/videos/a.gif"), null, "o conteúdo precisa ser GIF");
assert.equal(await carrega({ fetch: async () => { throw new Error("rede"); } }).gifParaApresentacao("/videos/a.gif"), null, "erro de rede vira null");

console.log("apresentacao-figuras.test.mjs: figura, limite, escape e GIFs conferidos");
