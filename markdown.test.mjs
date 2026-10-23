// O markdown das instruções da ação (web/js/markdown.js, script clássico: carregado num contexto isolado com o base.js, como o navegador faz).
//   node markdown.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ler = (f) => readFileSync(join(import.meta.dirname, "web", "js", f), "utf8");
const ctx = vm.createContext({ URL, document: {}, matchMedia: () => ({ matches: false }) });
vm.runInContext(ler("base.js") + "\n" + ler("markdown.js") + "\nthis.mdHtml = mdHtml;", ctx);
const md = (t) => ctx.mdHtml(t);

// parágrafos e quebra de linha simples
assert.equal(md("linha um\nlinha dois"), "<p>linha um</p><p>linha dois</p>", "cada linha é um bloco, com espaço entre eles");
assert.equal(md("a\n\nb"), '<p>a</p><p class="md-sep">b</p>', "linha em branco abre um espaço maior");
assert.equal(md(""), "");
assert.equal(md(null), "");
// o rótulo "Objetivo:" vira negrito, uma frase comum com dois-pontos no meio não
assert.equal(md("Objetivo: o usuário instala"), '<p><strong class="md-rot">Objetivo:</strong> o usuário instala</p>');
assert.equal(md("Passo a passo: 1) ler"), '<p><strong class="md-rot">Passo a passo:</strong> 1) ler</p>', "um passo só não vira lista");
assert.equal(md("Passo a passo: 1) ler; 2) criar (rápido); 3) testar."), '<p><strong class="md-rot">Passo a passo:</strong></p><ol class="md-passos"><li>ler</li><li>criar (rápido)</li><li>testar.</li></ol>');
assert.equal(md("Passos: 2) a; 3) b"), '<p><strong class="md-rot">Passos:</strong> 2) a; 3) b</p>', "só vira lista se começar no 1");
assert.equal(md("veja a nota: abaixo"), "<p>veja a nota: abaixo</p>");
// títulos, negrito, itálico, código
assert.equal(md("## Riscos"), '<p class="md-h md-h2">Riscos</p>');
assert.equal(md("um **forte** e *leve* e `x < y`"), "<p>um <strong>forte</strong> e <em>leve</em> e <code>x &lt; y</code></p>");
assert.equal(md("snake_case_nome e 2*3*4"), "<p>snake_case_nome e 2*3*4</p>", "não confunde _ e * dentro de palavras e contas");
// listas, recuo, caixas e citação
assert.equal(md("- a\n- b"), '<ul><li class="md-n0">a</li><li class="md-n0">b</li></ul>');
assert.equal(md("1. um\n2. dois"), '<ol><li class="md-n0">um</li><li class="md-n0">dois</li></ol>');
assert.match(md("- a\n  - filho"), /<li class="md-n1">filho<\/li>/);
assert.match(md("- [x] feito\n- [ ] falta"), /☑<\/span> feito.*☐<\/span> falta/);
assert.equal(md("> nota"), "<blockquote>nota</blockquote>");
assert.equal(md("---"), "<hr>");
// bloco de código: o conteúdo é só texto, mesmo com markdown dentro; sem fecho vai até o fim
assert.equal(md("```\n**x** <b>\n```"), "<pre><code>**x** &lt;b&gt;</code></pre>");
assert.equal(md("```\nsem fecho"), "<pre><code>sem fecho</code></pre>");
// segurança: HTML do agente nunca vira tag; só link http e https
assert.equal(md('<img src=x onerror="alert(1)">'), "<p>&lt;img src=x onerror=&quot;alert(1)&quot;&gt;</p>");
assert.equal(md("<script>alert(1)</script>"), "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
assert.equal(md("[ok](https://exemplo.dev/a?b=1&c=2)"), '<p><a href="https://exemplo.dev/a?b=1&amp;c=2" target="_blank" rel="noopener noreferrer">ok</a></p>');
assert.equal(md("[ruim](javascript:alert(1))"), "<p>[ruim](javascript:alert(1))</p>");
assert.equal(md("[ruim](data:text/html,x)"), "<p>[ruim](data:text/html,x)</p>");
const hostil = md("# <script>x</script>\n- <img onerror=1>\n> <b onclick=1>");
assert.ok(!hostil.replace(/<\/?(p|br|strong|em|code|pre|ul|ol|li|blockquote|hr|a|span)\b[^<>]*>/g, "").includes("<"), "só as tags do próprio renderizador sobram: " + hostil);
// as instruções reais de uma ação: ficha com rótulos, lista e código
const real = "Objetivo: ligar a pasta.\nEntrega: um plugin.\nPasso a passo: 1) ler; 2) criar.\nRiscos: o formato muda.\n\n- primeiro\n- segundo\n\n```\ncargo test\n```";
const h = md(real);
assert.equal((h.match(/class="md-rot"/g) || []).length, 4);
assert.match(h, /<ul>.*<\/ul><pre><code>cargo test<\/code><\/pre>$/);
console.log("ok: markdown.test.mjs");
