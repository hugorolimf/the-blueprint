// Confere que README.md (inglês) e README.pt-BR.md têm a mesma estrutura: mesmo número de seções, de blocos de código, de tabelas, de imagens e de links para docs/.
//   node scripts/conferir-readmes.mjs        (roda no CI)
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const ler = (f) => readFileSync(join(raiz, f), "utf8");
const perfil = (t) => ({
  secoes: (t.match(/^## /gm) || []).length,
  codigo: (t.match(/^```/gm) || []).length / 2,
  linhasDeTabela: (t.match(/^\|/gm) || []).length,
  imagens: (t.match(/!\[|<img /g) || []).length,
  linksParaDocs: [...t.matchAll(/\]\((docs\/[^)#]+)/g)].map((m) => m[1]).sort().join(","),
});
const [en, pt] = [perfil(ler("README.md")), perfil(ler("README.pt-BR.md"))];
const diferentes = Object.keys(en).filter((k) => en[k] !== pt[k]);
if (diferentes.length) { console.error("README.md e README.pt-BR.md diferem em:", diferentes.map((k) => `${k} (${en[k]} x ${pt[k]})`).join(", ")); process.exit(1); }
const longo = [["README.md", ler("README.md")], ["README.pt-BR.md", ler("README.pt-BR.md")]].filter(([, t]) => t.split("\n").length > 200);
if (longo.length) { console.error("README acima de 200 linhas:", longo.map(([f]) => f).join(", ")); process.exit(1); }
console.log("readmes: mesma estrutura nas duas línguas (" + JSON.stringify({ ...en, linksParaDocs: en.linksParaDocs.split(",").length }) + ")");
