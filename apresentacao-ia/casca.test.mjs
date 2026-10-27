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

// texto e figura ligados: cada item de texto das seções com itens é focável e tem data-item (0-based, na ordem do conteúdo) e data-sessao
{
  const h = L.casca.html(conteudo, []).html;
  const itens = (grupo) => [...h.matchAll(new RegExp(`data-item="(\\d+)" data-sessao="${grupo}" tabindex="0"`, "g"))].map((m) => +m[1]);
  assert.deepEqual(itens("recebe"), [0, 1, 2], "entregas");
  assert.deepEqual(itens("etapas"), [0, 1, 2, 3], "etapas");
  assert.deepEqual(itens("linha"), [0, 1, 2, 3], "as linhas da linha do tempo valem por etapa");
  assert.deepEqual(itens("precisamos"), [0, 1], "precisamosDeVoce");
  assert.deepEqual(itens("decisoes"), [0, 1], "decisoes");
  assert.deepEqual(itens("riscos"), [0, 1], "riscos");
  assert.ok(!/s-visao[\s\S]*?data-item[\s\S]*?id="s-entregas"/.test(h.slice(0, h.indexOf('id="s-entregas"'))), "a Visão geral não tem itens");
  assert.equal(conta(h, /class="ap-etapa" id="e-e1" data-st="[a-z]+" data-item="0"/g), 1, "o item da etapa é o artigo dela");
  // etapa com data inválida some da linha do tempo mas o índice das outras continua o da etapa
  const torta = { ...conteudo, etapas: conteudo.etapas.map((e, i) => (i === 1 ? { ...e, inicio: "x" } : e)) };
  assert.deepEqual([...L.casca.html(torta, []).html.matchAll(/data-item="(\d+)" data-sessao="linha"/g)].map((m) => +m[1]), [0, 2, 3]);
}

// a linha de referência sob o título: "Fig n · dica", com o link, mais a frase; só nas seções que têm figura
{
  const com = (id, secao, dica) => ({ ...falsa(id), secao, dica });
  const painel = (h, id) => { const i = h.indexOf(`id="s-${id}"`); return h.slice(i, h.indexOf("</section>", i)); };
  const h = L.casca.html(conteudo, comFuncao([com("v", "visao", "dica da visão"), com("rec", "recebe", "Passe o mouse $& aqui"), com("de", "decisoes", "dica de"), com("ri", "riscos", "dica ri")])).html;
  const rec = painel(h, "entregas");
  assert.ok(/<p class="ap-ref"><a class="ap-ref-link" href="#fig-rec" data-ref="rec" aria-label="Ir para a figura 2">Fig 2<\/a> · <span>Passe o mouse \$&amp; aqui<\/span> <span class="ap-ref-dica">Aponte um item do texto para vê-lo na figura\.<\/span><\/p>/.test(rec), "linha de referência de Entregas: " + rec.slice(rec.indexOf("ap-ref") - 5, rec.indexOf("ap-ref") + 380));
  assert.ok(rec.indexOf("ap-ref") < rec.indexOf("ap-com-fig"), "a linha fica sob o título, antes do texto e da figura");
  const vis = painel(h, "visao");
  assert.ok(vis.includes('data-ref="v"') && !vis.includes("ap-ref-dica"), "a Visão geral tem a linha, sem a frase dos itens (ela não tem itens)");
  const dec = painel(h, "decisoes");
  assert.equal(conta(dec, /class="ap-ref"/g), 1);
  assert.ok(dec.includes('data-ref="de"') && dec.includes('data-ref="ri"') && dec.indexOf('data-ref="de"') < dec.indexOf('data-ref="ri"'), "decisões e riscos numa linha só, com as duas figuras");
  assert.equal(conta(painel(h, "etapas"), /ap-ref/g), 0, "seção sem figura: sem linha");
  assert.equal(conta(painel(h, "linha"), /ap-ref/g), 0);
  // o cartão de cada figura tem o id para onde o link rola; em inglês a frase e o rótulo mudam
  assert.ok(h.includes('<div class="ap-fig" id="fig-rec">'));
  const en = L.casca.html({ ...conteudo, idioma: "en" }, comFuncao([com("rec", "recebe", "dica")])).html;
  assert.ok(en.includes("Point at an item of the text to see it in the figure.") && en.includes('aria-label="Go to figure 1"'));
  // as figuras que a etapa cita e as soltas não têm linha de referência (não são de uma seção)
  const solta = L.casca.html(conteudo, comFuncao([falsa("capa"), falsa("solta")])).html;
  assert.ok(solta.includes('data-ref="capa"') && !solta.includes('data-ref="solta"'), "a capa tem linha; a solta (fim de Etapas) não");
  // cada figura viva sabe de que grupo de itens é (para a ponte)
  const { vivos } = L.casca.html(conteudo, comFuncao([{ ...falsa("a", ", vivo() {}"), secao: "recebe" }, { ...falsa("b", ", vivo() {}"), secao: "visao" }, falsa("c", ", vivo() {}")]));
  assert.deepEqual(JSON.parse(JSON.stringify(vivos.map((v) => [v.id, v.grupo ?? null]))), [["a", "recebe"], ["b", null], ["c", null]]); // cópia: o array vem de outro contexto
}

// movimento: só quem devolve `vivo` entra na lista
assert.equal(L.casca.html(conteudo, comFuncao([falsa("a", ", vivo() {}"), falsa("b")])).vivos.length, 1);
// as teclas de uma figura com foco (setas, Home, End) são dela: o atalho global da página as ignora dentro de .iso-stage
{
  const { readFileSync } = await import("node:fs");
  const fonte = readFileSync(new URL("./biblioteca/casca.js", import.meta.url), "utf8");
  assert.ok(fonte.includes('closest(".iso-stage")'), "o atalho global ignora teclas vindas de uma figura");
}
console.log("casca.test.mjs: ok");
