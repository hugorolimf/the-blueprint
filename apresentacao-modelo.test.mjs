// Apresentação (web/js/apresentacao-modelo.js, script clássico carregado num contexto isolado, como o navegador faz): tipos, páginas, escape, paginação e idioma.
//   node apresentacao-modelo.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const contexto = (idioma, catalogos = []) => {
  const ctx = vm.createContext({
    navigator: { languages: [idioma] }, location: { search: "" }, URL, // URL: o contexto isolado não a traz (urlSegura precisa)
    document: { getElementById: () => ({}), hidden: false }, matchMedia: () => ({ matches: false }), CSS: { escape: (s) => s },
    localStorage: { getItem: () => null, setItem() {} },
    host: { addEventListener() {}, querySelector: () => null }, agenteAqui: () => null, roboRolar: false, render() {}, desenharBarras() {},
  });
  const carrega = (pasta, arquivo) => vm.runInContext(readFileSync(join(import.meta.dirname, "web", pasta, arquivo), "utf8"), ctx, { filename: arquivo });
  carrega("js", "i18n.js");
  catalogos.forEach((c) => carrega("i18n", c)); // o navegador carrega os catálogos com document.write; aqui à mão
  ["base.js", "dados.js", "projetos.js", "apresentacao-modelo.js"].forEach((a) => carrega("js", a));
  vm.runInContext("this.api = { montarApresentacao };", ctx);
  return ctx.api.montarApresentacao;
};
const montar = contexto("pt-BR");

const acao = (id, inicio, fim, extra = {}) => ({ id, nome: id.toUpperCase(), inicio, fim, status: "planejado", progresso: 0, obs: "", planos: [], checkpoints: [], ...extra });
const plano = (id, feito, horas = 2) => ({ id, texto: "Plano " + id, feito, horas });
const blueprint = (extra = {}) => ({ id: "b1", nome: "Portal", descricao: "Portal do fornecedor", obs: "", links: [], projetos: [{ id: "pr1", nome: "Interface" }, { id: "pr2", nome: "Servidor" }], acoes: [
  acao("a1", "2026-11-01", "2026-11-05", { projetoId: "pr1", status: "concluido", planos: [plano("p1", true)] }),
  acao("a2", "2026-11-06", "2026-11-10", { projetoId: "pr1", obs: "Objetivo: telas.\nEntrega: protótipo.", planos: [plano("p2", true), plano("p3", false)], checkpoints: [{ id: "c1", nome: "Revisão", data: "2026-11-10", feito: false }] }),
  acao("a3", "2026-11-11", "2026-11-12", { projetoId: "pr2" }),
  acao("a4", "2026-11-13", "2026-11-14")], ...extra });
const tipos = (doc) => [...doc.paginas.map((p) => p.tipo)]; // cópia no realm do teste: o array do contexto isolado tem outro prototype
const HOJE = "2026-11-20";

// cada tipo tem as suas páginas; tipo desconhecido cai no resumo
assert.deepEqual(tipos(montar(blueprint(), { tipo: "resumo", hoje: HOJE })), ["capa", "resumo", "fim"]);
assert.deepEqual(tipos(montar(blueprint(), { tipo: "andamento", hoje: HOJE })), ["capa", "resumo", "projeto", "projeto", "projeto", "fim"], "Interface, Servidor e Geral");
assert.deepEqual(tipos(montar(blueprint(), { tipo: "completo", hoje: HOJE })), ["capa", "resumo", "acao", "acao", "acao", "acao", "fim"]);
assert.equal(montar(blueprint(), { tipo: "nada", hoje: HOJE }).tipo, "resumo");

// contrato com as outras frentes: <section class="ap-pagina" data-tipo>, rodapé "n / total" fora da capa, título na capa
{
  const doc = montar(blueprint(), { tipo: "completo", hoje: HOJE, workspace: "Estúdio" });
  doc.paginas.forEach((p, i) => {
    assert.match(p.html, new RegExp(`^<section class="ap-pagina" data-tipo="${p.tipo}">`));
    assert.equal(p.html.includes("ap-rodape"), p.tipo !== "capa");
    if (p.tipo !== "capa") assert.ok(p.html.includes(`${i + 1} / ${doc.paginas.length}`), "numeração da página");
  });
  assert.match(doc.paginas[0].html, /class="ap-capa"/);
  assert.match(doc.paginas[0].html, /<h1 class="ap-titulo">Portal<\/h1>/);
  assert.ok(doc.paginas[0].html.includes("Estúdio"));
  assert.equal(doc.titulo, "Portal");
}

// a numeração é a da folha (projeto.posição) e as marcas dos planos refletem o que foi feito
{
  const doc = montar(blueprint(), { tipo: "completo", hoje: HOJE });
  const a2 = doc.paginas.find((p) => p.tipo === "acao" && p.html.includes(">A2<")).html;
  assert.ok(a2.includes("1.2 · Interface"), "ação 2 do projeto 1");
  assert.equal((a2.match(/data-feito="1"/g) || []).length, 1);
  assert.equal((a2.match(/data-feito="0"/g) || []).length, 2, "um plano e um checkpoint pendentes");
  assert.ok(a2.includes("Objetivo: telas.") && a2.includes("Revisão"));
}

// resumo: porcentagem, contagens e atrasadas (A2 e A3 terminaram antes de HOJE e não estão concluídas, A4 também)
{
  const resumo = montar(blueprint(), { tipo: "resumo", hoje: HOJE }).paginas[1].html;
  assert.ok(resumo.includes("1 de 4") && resumo.includes("ações concluídas"));
  assert.ok(resumo.includes("3 ações atrasadas"));
  assert.ok(!montar(blueprint(), { tipo: "resumo", hoje: "2026-11-01" }).paginas[1].html.includes("atrasada"));
}

// nada vindo dos dados vira HTML; link que não é http(s) fica de fora; sem emoji
{
  const mau = blueprint({ nome: "<img src=x onerror=alert(1)>", descricao: "<b>x</b>", links: [{ id: "l1", rotulo: "Ruim", url: "javascript:alert(1)" }, { id: "l2", rotulo: "Repo", url: "https://example.com/r" }] });
  mau.acoes[0].nome = '"><script>alert(1)</script>';
  const doc = montar(mau, { tipo: "completo", hoje: HOJE });
  const tudo = doc.paginas.map((p) => p.html).join("");
  assert.ok(!/<img|<script/.test(tudo) && !tudo.includes("<b>x</b>") && tudo.includes("&lt;b&gt;x&lt;/b&gt;"), "texto escapado");
  assert.ok(tudo.includes("&lt;img") && tudo.includes("https://example.com/r") && !tudo.includes("javascript:"));
  assert.ok(!tudo.includes("<a "), "nenhum link clicável");
  assert.ok(!/\p{Extended_Pictographic}/u.test(tudo), "sem emojis");
}

// a figura do anatomy só entra na capa, e só quando vem
{
  const com = montar(blueprint(), { tipo: "andamento", hoje: HOJE, figura: '<svg id="fig"></svg>' });
  assert.ok(com.paginas[0].html.includes('<figure class="ap-figura"><svg id="fig"></svg></figure>'));
  assert.ok(!com.paginas.slice(1).some((p) => p.html.includes("ap-figura")));
  assert.ok(!montar(blueprint(), { hoje: HOJE }).paginas[0].html.includes("ap-figura"));
}

// ação com muitos itens continua na página seguinte; blueprint sem projetos usa uma seção só; blueprint vazio não quebra
{
  const grande = blueprint({ projetos: [] });
  grande.acoes = [acao("g1", "2026-11-01", "2026-11-09", { planos: Array.from({ length: 25 }, (_, i) => plano("q" + i, i < 5)) })];
  const doc = montar(grande, { tipo: "completo", hoje: HOJE });
  assert.deepEqual(tipos(doc), ["capa", "resumo", "acao", "acao", "acao", "acao", "fim"], "25 itens = 4 páginas de 7");
  assert.equal(doc.paginas.filter((p) => p.html.includes("(continuação)")).length, 3);
  assert.ok(doc.paginas[2].html.includes("1.1"), "sem projetos: <blueprint>.<posição>");
  const vazio = montar(blueprint({ acoes: [], projetos: [] }), { tipo: "completo", hoje: HOJE });
  assert.deepEqual(tipos(vazio), ["capa", "resumo", "fim"]);
  assert.ok(vazio.paginas[1].html.includes("Nenhuma ação com datas válidas."));
  const lotes = montar(blueprint({ acoes: Array.from({ length: 20 }, (_, i) => acao("x" + i, "2026-11-01", "2026-11-02")), projetos: [] }), { tipo: "andamento", hoje: HOJE });
  assert.equal(lotes.paginas.filter((p) => p.tipo === "projeto").length, 4, "20 ações = 4 páginas de 6");
}

// inglês: o catálogo traduz títulos, rótulos e datas
{
  const en = contexto("en-US", ["en-edicao.js", "en-folha.js", "en-resto.js", "en-apresentacao.js"]) // "{a} de {b}", "Sem nome" e "Geral" vêm dos catálogos da folha;
  const doc = en(blueprint(), { tipo: "andamento", hoje: HOJE });
  const resumo = doc.paginas[1].html;
  assert.equal(doc.idioma, "en");
  assert.ok(doc.paginas[0].html.includes("Progress by project") && resumo.includes("actions completed") && resumo.includes("Next actions"), resumo.slice(0, 300));
  assert.ok(/Nov \d{2}, 2026/.test(resumo), "datas no formato do idioma");
  assert.ok(!/Resumo|Próximas|ações/.test(doc.paginas.map((p) => p.html).join("")), "nada em português");
}
console.log("apresentacao-modelo: ok");
