import assert from "node:assert/strict";
import vm from "node:vm";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { montar, figurasJson, codigosHtml, escScript } from "./montar.mjs";
import { AQUI, exemplo, lerJson } from "./teste-comum.mjs";

const { conteudo, figuras } = exemplo();
const blocos = (html) => [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]);

// caso fixo: a mesma entrada e a mesma saída esperada que o teste do servidor Rust usa (teste-montagem-fixo.json)
const fixo = lerJson("teste-montagem-fixo.json");
assert.equal(montar({ conteudo: fixo.conteudo, figuras: fixo.figuras, modelo: fixo.modelo, css: fixo.css, biblioteca: fixo.biblioteca }), fixo.esperado, "a montagem não bate com o caso fixo");

// a página do exemplo: nenhum marcador sobrando e um <script> por figura, depois do da biblioteca e antes do da página
const html = montar({ conteudo, figuras });
assert.ok(!/\{\{\w+\}\}/.test(html), "nenhum marcador sobrando");
assert.ok(html.includes("<title>Plano do aplicativo de entregas</title>") && html.includes('lang="pt-BR"'));
const b = blocos(html);
assert.equal(b.length, 3 + figuras.length, "kit, biblioteca, uma por figura e a página");
assert.equal(b[0], "", "o kit já vai dentro da biblioteca");
figuras.forEach((f, i) => assert.ok(b[2 + i].startsWith(`(window.AP_CODIGOS=window.AP_CODIGOS||{})["${f.id}"]=(function`), "o bloco de " + f.id));
assert.ok(b[2 + figuras.length].includes("const CONTEUDO") && b[2 + figuras.length].includes("AP_CODIGOS"));
for (const bloco of b) new vm.Script(bloco);

// o modelo tem todos os marcadores do contrato, na ordem: biblioteca, códigos, página
const modelo = readFileSync(join(AQUI, "modelo.html"), "utf8").replace(/\r\n/g, "\n");
for (const m of ["titulo", "idioma", "css", "kit", "biblioteca", "codigos", "conteudo", "figuras"]) assert.ok(modelo.includes(`{{${m}}}`), "modelo sem {{" + m + "}}");
assert.ok(modelo.indexOf("{{biblioteca}}") < modelo.indexOf("{{codigos}}") && modelo.indexOf("{{codigos}}") < modelo.indexOf("{{conteudo}}"));

// {{figuras}} é JSON puro, só com metadados; {{codigos}} tem o formato exato do contrato
const meta = JSON.parse(figurasJson(figuras));
assert.equal(meta.length, figuras.length);
assert.ok(meta.every((f) => !("codigo" in f)) && meta[0].id === "visao" && meta[0].titulo === figuras[0].titulo);
assert.equal(codigosHtml([{ id: "x", codigo: "function () { return '</script>'; }" }]), '<script>(window.AP_CODIGOS=window.AP_CODIGOS||{})["x"]=(function () { return \'<\\/script>\'; });</script>');
assert.equal(codigosHtml([{ id: "a", codigo: "1" }, { id: "b", codigo: "2" }]).split("\n").length, 2, "um bloco por linha");
assert.equal(escScript("</SCRIPT x <!--"), "<\\/SCRIPT x <\\!--");

// um único passo: marcador, "</script" e "<!--" dentro do conteúdo não escapam do <script> nem são reexaminados
const hostil = { ...conteudo, titulo: "a </script><b>x</b> {{css}} <!-- y", resumo: ["{{figuras}}"] };
const h2 = montar({ conteudo: hostil, figuras: [], modelo, css: "/*C*/", biblioteca: "/*B*/" });
const p = blocos(h2).at(-1);
assert.ok(p.includes("{{css}}") && p.includes("{{figuras}}"), "marcadores no conteúdo ficam como texto");
assert.equal(vm.runInNewContext("(" + /const CONTEUDO = (.*);\n/.exec(p)[1] + ")").titulo, hostil.titulo, "o conteúdo volta idêntico");
assert.ok(h2.includes("<title>a &lt;/script&gt;&lt;b&gt;x&lt;/b&gt;"), "título escapado como HTML");

// o código de UMA figura com erro de sintaxe só quebra o bloco dela: os outros blocos compilam
const mista = montar({ conteudo, figuras: [...figuras, { id: "ruim", titulo: "R", codigo: "function (L {" }] });
const compila = blocos(mista).map((bl) => { try { new vm.Script(bl); return true; } catch { return false; } });
assert.equal(compila.filter((x) => !x).length, 1, "só o bloco da figura quebrada não compila");
assert.equal(compila.at(-1), true, "o script da página compila");
console.log("montar.test.mjs: ok");
