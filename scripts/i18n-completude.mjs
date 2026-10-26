// Teste de completude dos idiomas: toda chave usada no código (tr, trn e pvPlural) tem tradução em cada catálogo, e nenhum catálogo tem chave que o código não usa mais.
//   node scripts/i18n-completude.mjs        sai com 1 se faltar ou sobrar alguma chave (roda no CI)
// Só vê chaves escritas como texto literal; chaves montadas em tempo de execução (ex.: tr(c) em proposta-vista.js) entram na lista DINAMICAS.
import { readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import vm from "node:vm";

const raiz = resolve(import.meta.dirname, "..");
const jsDir = join(raiz, "web", "js"), i18nDir = join(raiz, "web", "i18n");
// chaves que o código monta em tempo de execução (ver os comentários nos arquivos)
const DINAMICAS = ["JAN|FEV|MAR|ABR|MAI|JUN|JUL|AGO|SET|OUT|NOV|DEZ", "nome", "datas", "projeto", "instruções", "concluído", "em desenvolvimento", "planejado"];
const LIT = String.raw`"((?:\\.|[^"\\\n])*)"|'((?:\\.|[^'\\\n])*)'|` + "`((?:\\\\.|[^`\\\\])*)`";
const desescapa = (s) => s.replace(/\\(.)/g, (_, c) => ({ n: "\n", t: "\t" })[c] ?? c);
const literal = (m) => desescapa(m[1] ?? m[2] ?? m[3] ?? "");

const usadas = new Map(); // chave -> arquivo
const marca = (k, arq) => { if (k && k.trim() !== "{n}" && !usadas.has(k)) usadas.set(k, arq); }; // "{n} " sozinho é o prefixo do pvPlural, não uma chave
for (const f of readdirSync(jsDir).filter((x) => x.endsWith(".js") && x !== "i18n.js")) {
  const t = readFileSync(join(jsDir, f), "utf8");
  for (const m of t.matchAll(new RegExp(String.raw`\btr\(\s*(?:${LIT})`, "g"))) marca(literal(m), f);
  for (const m of t.matchAll(new RegExp(String.raw`\btrn\([^,\[]+,\s*\[\s*(?:${LIT})`, "g"))) marca(literal(m), f);
  for (const m of t.matchAll(new RegExp(String.raw`\bpvPlural\([^,]+,\s*(?:${LIT})\s*,\s*(?:${LIT})`, "g"))) { const um = m[1] ?? m[2] ?? m[3]; marca("{n} " + desescapa(um), f); }
}
for (const k of DINAMICAS) marca(k, "(dinâmica)");

// carrega os catálogos de cada idioma como o navegador faz
const idiomas = [...new Set(readdirSync(i18nDir).map((f) => f.split("-")[0]))];
let erro = false;
for (const idioma of idiomas) {
  const ctx = vm.createContext({ I18N: { catalogos: { [idioma]: {} } } });
  for (const f of readdirSync(i18nDir).filter((x) => x.startsWith(idioma + "-"))) vm.runInContext(readFileSync(join(i18nDir, f), "utf8"), ctx, { filename: f });
  const cat = ctx.I18N.catalogos[idioma], chaves = new Set(Object.keys(cat));
  const faltam = [...usadas].filter(([k]) => !chaves.has(k)), sobram = [...chaves].filter((k) => !usadas.has(k));
  console.log(`${idioma}: ${chaves.size} chaves no catálogo, ${usadas.size} usadas no código`);
  if (faltam.length) { erro = true; console.log(`  FALTAM ${faltam.length}:`); for (const [k, a] of faltam.slice(0, 40)) console.log(`   ${a}: ${k.slice(0, 100)}`); }
  // texto da página estática (blueprint.html) é traduzido por correspondência exata e não aparece no código: só avisa do que sobra de verdade
  const html = readFileSync(join(raiz, "blueprint.html"), "utf8");
  const reais = sobram.filter((k) => !html.includes(k));
  if (reais.length) { erro = true; console.log(`  SOBRAM ${reais.length} (não aparecem no código nem no HTML):`); for (const k of reais.slice(0, 40)) console.log(`   ${k.slice(0, 100)}`); }
}
if (!erro) console.log("i18n: catálogos completos e sem chave sobrando");
process.exit(erro ? 1 : 0);
