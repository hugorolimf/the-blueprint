import assert from "node:assert/strict";
import { carregar, exemplo } from "./teste-comum.mjs";

const { L, comFuncao } = carregar();
const { conteudo } = exemplo();
const falsa = (id, extra = "") => ({ id, titulo: "Figura " + id, dica: "dica", leitura: "0", chaves: [], legenda: "legenda", codigo: `function (L) { return { svg: '<svg data-fig="${id}"></svg>'${extra} }; }` });
const conta = (texto, re) => (texto.match(re) || []).length;

// estrutura: barra superior, navegação, uma seção por item do menu, rodapé, e as seções fixas do público cliente
const { html } = L.casca.html(conteudo, comFuncao([falsa("capa")]));
for (const parte of ["ap-topo", "ap-migalhas", "ap-titulo-caixa", 'class="ap-nav"', 'id="conteudo"', "ap-rodape", "ap-pular"]) assert.ok(html.includes(parte), "falta " + parte);
assert.equal(conta(html, /<section class="ap-sec"/g), conta(html, /data-nav="/g), "uma seção por link do menu");
for (const sec of ["visao", "entregas", "etapas", "linha", "voce", "decisoes"]) assert.ok(html.includes(`id="s-${sec}"`), "falta a seção " + sec);
assert.ok(!html.includes('id="s-acoes"'), "o público cliente não tem Ações");
assert.ok(html.includes("Plano de projeto") && html.includes("Cliente"), "migalhas em pt-BR");
assert.equal(conta(html, /class="ap-etapa"/g), conteudo.etapas.length);
assert.ok(html.includes('data-st="andamento"') && html.includes('data-st="prevista"'), "situação pelas datas");

// público interno: ganha Ações
const interno = { ...conteudo, publico: "interno", interno: { acoes: [{ numero: "1.1", nome: "Fundação", planos: ["Criar o núcleo"] }] } };
const h2 = L.casca.html(interno, []).html;
assert.ok(h2.includes('id="s-acoes"') && h2.includes("Fundação") && h2.includes("Plano de projeto, visão interna"));

// idioma en troca os textos fixos
assert.ok(L.casca.html({ ...conteudo, idioma: "en" }, []).html.includes("What you receive"));

// escape: o que vem do conteúdo nunca vira marcação
const mau = L.casca.html({ ...conteudo, titulo: "<img src=x onerror=alert(1)>", resumo: ["<script>x</script>"] }, []).html;
assert.ok(!mau.includes("<img src=x") && !mau.includes("<script>x"), "conteúdo escapado");

// capa: figura "capa" na visão geral; figura de etapa dentro da etapa; a da capa não se repete
const comEtapa = { ...conteudo, etapas: conteudo.etapas.map((e, i) => (i === 1 ? { ...e, figura: "f2" } : e)) };
const r = L.casca.html(comEtapa, comFuncao([falsa("capa"), falsa("f2")])).html;
assert.equal(conta(r, /data-fig="capa"/g), 1);
assert.ok(r.indexOf('data-fig="f2"') > r.indexOf('id="e-e2"') && r.indexOf('data-fig="f2"') < r.indexOf('id="e-e3"'), "figura dentro da etapa 2");
assert.equal(L.casca.capaDe(conteudo, [{ id: "x" }, { id: "capa" }]), "capa");

// uma figura que lança erro vira cartão de erro e o resto segue
const quebrada = { ...falsa("ruim"), codigo: "function () { throw new Error('peça fora do lugar'); }" };
const e = L.casca.html(conteudo, comFuncao([quebrada, falsa("capa")]));
assert.ok(e.html.includes('role="alert"') && e.html.includes("peça fora do lugar") && e.html.includes("não pôde ser desenhada"));
assert.ok(e.html.includes('data-fig="capa"'), "a outra figura segue");
const semCodigo = L.casca.html(conteudo, [{ id: "capa", titulo: "z" }, falsa("f9")].map((f, i) => (i ? comFuncao([f])[0] : f))).html;
assert.ok(semCodigo.includes("O código desta figura não carregou.") && semCodigo.includes('role="alert"'), "figura sem `codigo` função (bloco com erro de sintaxe) vira cartão de erro");
assert.ok(L.casca.html({ ...conteudo, idioma: "en" }, [{ id: "capa", titulo: "z" }]).html.includes("code did not load"), "e em inglês");

// uma figura por seção: cada uma no painel da sua seção, ao lado do texto; a da etapa na etapa; a sem seção e sem etapa no fim de Etapas
{
  const com = (id, secao) => ({ ...falsa(id), ...(secao ? { secao } : {}) });
  const painel = (h, id) => { const i = h.indexOf(`id="s-${id}"`); return h.slice(i, h.indexOf("</section>", i)); };
  const figs = [com("v", "visao"), com("rec", "recebe"), com("et", "etapas"), com("li", "linha"), com("pre", "precisamos"), com("de", "decisoes"), com("ri", "riscos"), com("solta")];
  const h = L.casca.html(conteudo, comFuncao(figs)).html;
  for (const [fig, sec] of [["v", "visao"], ["rec", "entregas"], ["et", "etapas"], ["li", "linha"], ["pre", "voce"], ["de", "decisoes"], ["ri", "decisoes"]]) assert.ok(painel(h, sec).includes(`data-fig="${fig}"`), `${fig} no painel ${sec}`);
  for (const sec of ["entregas", "etapas", "linha", "voce"]) assert.ok(painel(h, sec).includes("ap-com-fig-col"), `${sec}: figura ao lado do texto`);
  const dec = painel(h, "decisoes");
  assert.ok(dec.indexOf('data-fig="de"') > dec.indexOf("ap-sub-t") && dec.indexOf('data-fig="de"') < dec.indexOf('data-fig="ri"'), "decisões e riscos: cada figura na sua coluna");
  assert.ok(painel(h, "etapas").includes('data-fig="solta"') && painel(h, "etapas").includes("ap-extras"), "sem seção e sem etapa: fim de Etapas");
  for (const id of ["v", "rec", "et", "li", "pre", "de", "ri", "solta"]) assert.equal(conta(h, new RegExp(`data-fig="${id}"`, "g")), 1, `${id} aparece uma vez`);

  // duas figuras na mesma seção: uma após a outra, na ordem da lista; a "capa" por id fica antes
  const dupla = painel(L.casca.html(conteudo, comFuncao([com("a", "recebe"), com("b", "recebe")])).html, "entregas");
  assert.ok(dupla.indexOf('data-fig="a"') < dupla.indexOf('data-fig="b"') && conta(dupla, /ap-com-fig-col/g) === 1);
  const capa2 = painel(L.casca.html(conteudo, comFuncao([com("outra", "visao"), falsa("capa")])).html, "visao");
  assert.ok(capa2.indexOf('data-fig="capa"') < capa2.indexOf('data-fig="outra"'), "a capa por id vem antes das outras da Visão geral");

  // a figura que uma etapa cita fica na etapa, mesmo com `secao`; seção que não existe (Ações no cliente, Entregas vazia) vai ao fim de Etapas
  const semEntregas = { ...conteudo, entregas: [], etapas: conteudo.etapas.map((e, i) => (i === 0 ? { ...e, figura: "da-etapa" } : e)) };
  const h2 = L.casca.html(semEntregas, comFuncao([com("da-etapa", "linha"), com("rec", "recebe"), com("ac", "acoes")])).html;
  assert.ok(painel(h2, "etapas").indexOf('data-fig="da-etapa"') > painel(h2, "etapas").indexOf('id="e-e1"') && !painel(h2, "linha").includes("da-etapa"), "a da etapa fica na etapa");
  assert.ok(painel(h2, "etapas").includes("ap-extras") && painel(h2, "etapas").includes('data-fig="rec"') && painel(h2, "etapas").includes('data-fig="ac"'), "seção que não existe: fim de Etapas");
  assert.equal(L.casca.capaDe(conteudo, comFuncao([com("x"), com("y", "visao")])), "y", "a figura de secao visao é a capa");
  assert.equal(L.casca.capaDe(conteudo, comFuncao([com("x", "linha")])), undefined, "só figuras de outras seções: sem capa");
  assert.equal(L.casca.secaoDaFigura({ secao: "recebe" }), "entregas");
  assert.equal(L.casca.secaoDaFigura({ secao: "banana" }), undefined, "secao desconhecida vale como sem seção");
}

// movimento: só quem devolve `vivo` entra na lista
assert.equal(L.casca.html(conteudo, comFuncao([falsa("a", ", vivo() {}"), falsa("b")])).vivos.length, 1);
console.log("casca.test.mjs: ok");
