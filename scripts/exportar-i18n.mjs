// Exporta o catálogo inglês da página (web/i18n/en-*.js) para a interface nativa (ui/assets/i18n-en.json): o texto em português é a chave, como na página.
// Uso: node scripts/exportar-i18n.mjs
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const pasta = join(raiz, "web", "i18n");
const contexto = { I18N: { catalogos: { en: {} } } };
vm.createContext(contexto);
for (const arquivo of readdirSync(pasta).filter((f) => /^en-.*\.js$/.test(f)).sort()) vm.runInContext(readFileSync(join(pasta, arquivo), "utf8"), contexto, { filename: arquivo });
const catalogo = contexto.I18N.catalogos.en;
writeFileSync(join(raiz, "ui", "assets", "i18n-en.json"), JSON.stringify(catalogo));
console.log(`${Object.keys(catalogo).length} textos em ui/assets/i18n-en.json`);
