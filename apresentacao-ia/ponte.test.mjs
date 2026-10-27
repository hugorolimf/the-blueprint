// A ponte texto <-> figura (biblioteca/ponte.js) com um DOM de mentira: quem é chamado, quando, o que é marcado e o que não vaza.
import assert from "node:assert/strict";
import { carregar, exemplo } from "./teste-comum.mjs";

const { L } = carregar();
const { conteudo } = exemplo();
const espera = () => new Promise((r) => setTimeout(r, 8));
const item = (grupo, i) => ({ dataset: { sessao: grupo, item: String(i) }, attrs: {}, setAttribute(k, v) { this.attrs[k] = v; }, removeAttribute(k) { delete this.attrs[k]; }, closest(sel) { return sel.includes("data-item") ? this : null; }, contains(x) { return x === this; } });
const montaRaiz = () => {
  const itens = [0, 1, 2].map((i) => item("recebe", i)).concat([0, 1, 2, 3].map((i) => item("etapas", i)));
  const link = { dataset: { ref: "fichario" }, closest(sel) { return sel.includes("data-ref") ? this : null; } };
  return { itens, link, ouvintes: {}, querySelectorAll: () => itens, addEventListener(n, f) { this.ouvintes[n] = f; }, removeEventListener(n) { delete this.ouvintes[n]; } };
};
const figuraFalsa = () => { const chamadas = []; return { chamadas, selecionar: (i) => chamadas.push(i) }; };
const ev = (alvo, extra = {}) => ({ target: alvo, relatedTarget: null, pointerType: "mouse", preventDefault() {}, ...extra });

// o total vem do conteúdo; grupo desconhecido ou sem itens: ponte que não faz nada (e não quebra)
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo);
  assert.deepEqual([c.criar("recebe").total, c.criar("etapas").total, c.criar("linha").total, c.criar("precisamos").total, c.criar("decisoes").total, c.criar("riscos").total], [3, 4, 4, 2, 2, 2]);
  for (const g of [undefined, "visao", "acoes", "banana"]) {
    const p = c.criar(g);
    assert.equal(p.total, 0);
    assert.equal(typeof p.liga(figuraFalsa()), "function");
    p.mudou(1);
  }
  assert.deepEqual(Object.keys(raiz.itens[0].attrs), [], "ponte sem grupo não marca nada");
  c.desligar();
}

// o mouse entra num item: selecionar(i); sai: selecionar(null) (adiado, para juntar "saiu de um" com "entrou no outro")
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo), f = figuraFalsa();
  c.criar("recebe").liga(f);
  raiz.ouvintes.pointerover(ev(raiz.itens[1]));
  assert.deepEqual(f.chamadas, [1]);
  raiz.ouvintes.pointerover(ev(raiz.itens[1])); // repetido: não chama de novo
  raiz.ouvintes.pointerout(ev(raiz.itens[1]));
  raiz.ouvintes.pointerover(ev(raiz.itens[2])); // de um item direto para o outro: sem null no meio
  await espera();
  assert.deepEqual(f.chamadas, [1, 2]);
  raiz.ouvintes.pointerout(ev(raiz.itens[2]));
  assert.deepEqual(f.chamadas, [1, 2], "o null só vem depois do adiamento");
  await espera();
  assert.deepEqual(f.chamadas, [1, 2, null], "saiu: a figura volta ao passeio");
  raiz.ouvintes.pointerout(ev(raiz.itens[2], { relatedTarget: raiz.itens[2] })); // saída para dentro do próprio item não é saída
  await espera();
  assert.deepEqual(f.chamadas, [1, 2, null]);
  c.desligar();
}

// teclado: o foco que chega seleciona, o que sai solta; toque: o toque seleciona e o toque fora solta; o toque não passa pelo hover
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo), f = figuraFalsa();
  c.criar("recebe").liga(f);
  raiz.ouvintes.focusin(ev(raiz.itens[0]));
  raiz.ouvintes.focusout(ev(raiz.itens[0]));
  await espera();
  assert.deepEqual(f.chamadas, [0, null]);
  raiz.ouvintes.pointerover(ev(raiz.itens[2], { pointerType: "touch" }));
  raiz.ouvintes.pointerdown(ev(raiz.itens[2], { pointerType: "touch" }));
  assert.deepEqual(f.chamadas, [0, null], "toque não seleciona por pointerover");
  raiz.ouvintes.click(ev(raiz.itens[2]));
  assert.deepEqual(f.chamadas, [0, null, 2], "o toque no item seleciona");
  raiz.ouvintes.click(ev({ closest: () => null }));
  assert.deepEqual(f.chamadas, [0, null, 2, null], "o toque fora solta");
  c.desligar();
}

// vários grupos e várias figuras: cada figura só ouve o seu grupo; mudar de grupo solta o anterior
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo), a = figuraFalsa(), b = figuraFalsa(), outra = figuraFalsa();
  c.criar("recebe").liga(a); c.criar("recebe").liga(b); c.criar("etapas").liga(outra);
  raiz.ouvintes.focusin(ev(raiz.itens[1]));
  raiz.ouvintes.focusin(ev(raiz.itens[3])); // item 0 das etapas
  assert.deepEqual([a.chamadas, b.chamadas, outra.chamadas], [[1, null], [1, null], [0]]);
  c.desligar();
}

// a figura marca o item (data-ativo e aria-current), limpa o anterior, ignora repetição, valor fora da faixa e null
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo), p = c.criar("recebe"), q = c.criar("etapas");
  const marcado = (g) => raiz.itens.filter((e) => e.dataset.sessao === g).map((e) => (e.attrs["data-ativo"] === "true" && e.attrs["aria-current"] === "true" ? 1 : 0)).join("");
  p.mudou(1); assert.equal(marcado("recebe"), "010");
  p.mudou(1); assert.equal(marcado("recebe"), "010");
  p.mudou(2); assert.equal(marcado("recebe"), "001", "limpou o anterior");
  q.mudou(0); assert.equal(marcado("etapas"), "1000"); assert.equal(marcado("recebe"), "001", "cada grupo tem o seu marcado");
  p.mudou(null); assert.equal(marcado("recebe"), "000");
  p.mudou(2); p.mudou(99); assert.equal(marcado("recebe"), "000", "índice fora da faixa não marca");
  p.mudou(-1); p.mudou("x"); assert.equal(marcado("recebe"), "000");
  c.desligar();
}

// liga devolve o desfazer; ligar de novo substitui; uma figura que lança erro não derruba as outras; sem ninguém ligado nada acontece
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo), p = c.criar("recebe"), f = figuraFalsa(), g = figuraFalsa();
  const desfaz = p.liga(f);
  raiz.ouvintes.focusin(ev(raiz.itens[0])); raiz.ouvintes.focusout(ev(raiz.itens[0])); await espera();
  desfaz();
  raiz.ouvintes.focusin(ev(raiz.itens[1])); raiz.ouvintes.focusout(ev(raiz.itens[1])); await espera();
  assert.deepEqual(f.chamadas, [0, null], "depois de desfazer, a figura não é mais chamada");
  p.liga(f); p.liga(g);
  raiz.ouvintes.focusin(ev(raiz.itens[2])); await espera();
  assert.deepEqual([f.chamadas.length, g.chamadas], [2, [2]], "ligar de novo troca o registro da mesma ponte");
  c.criar("recebe").liga({ selecionar() { throw new Error("boom"); } });
  const calado = console.error; console.error = () => {};
  raiz.ouvintes.focusout(ev(raiz.itens[2])); await espera();
  raiz.ouvintes.focusin(ev(raiz.itens[0]));
  console.error = calado;
  assert.deepEqual(g.chamadas, [2, null, 0], "o erro de uma figura não impede as outras");
  p.liga({}); p.liga(null); // sem selecionar: ignorado
  c.desligar();
  const vazia = montaRaiz(), c2 = L.ponte.ligar(vazia, conteudo);
  vazia.ouvintes.focusin(ev(vazia.itens[0])); vazia.ouvintes.pointerover(ev(vazia.itens[1])); await espera(); // sem figura registrada
  c2.desligar();
}

// sem vazamento: desligar tira todos os ouvintes
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo);
  assert.equal(Object.keys(raiz.ouvintes).length, 6);
  c.desligar();
  assert.equal(Object.keys(raiz.ouvintes).length, 0);
}

// o link "Fig n" é interceptado (não muda o fragmento da URL)
{
  const raiz = montaRaiz(), c = L.ponte.ligar(raiz, conteudo);
  let impediu = false, procurou = null;
  raiz.querySelector = (sel) => { procurou = sel; return null; };
  raiz.ouvintes.click({ target: raiz.link, preventDefault() { impediu = true; } });
  assert.ok(impediu && procurou.includes('data-figura="fichario"'));
  c.desligar();
}
console.log("ponte.test.mjs: ok");
