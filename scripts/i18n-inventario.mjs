// Inventário dos textos que o usuário pode ler na interface (web/js e blueprint.html): literais de texto e texto entre tags dentro de templates.
// Heurístico: acha candidatos para revisar, não decide sozinho. Texto já passado por tr()/trn() não entra na lista.
//   node scripts/i18n-inventario.mjs                  resumo: candidatos por arquivo
//   node scripts/i18n-inventario.mjs render.js        lista os candidatos de um arquivo (linha: texto)
//   node scripts/i18n-inventario.mjs --pendentes      sai com 1 se restar algum candidato fora da lista de exceções (para o CI, quando a conversão acabar)
import { readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const dir = join(raiz, "web", "js");
const ACENTO = /[áàâãéêíóôõúçÁÀÂÃÉÊÍÓÔÕÚÇ]/;
const PALAVRA = /\b(de|da|do|dos|das|em|no|na|para|com|sem|um|uma|ou|ainda|nenhum|nenhuma|clique|abra|abrir|fechar|salvar|ação|ações|plano|planos|hoje|dia|erro|veja|marque|passe|crie|adicionar|remover|editar|aplicar|descartar|proposta|agente)\b/i;
const CODIGO = /^[\w.#:\-\[\]=,()*$ {}|?!&;<>%\/"'`+@~^\\]+$/; // seletor, caminho, expressão

function candidatos(texto) {
  const achados = [];
  texto.split("\n").forEach((linha, i) => {
    const codigo = linha.replace(/^\s*\/\/.*$/, "").replace(/\s\/\/ .*$/, "");
    if (!codigo.trim()) return;
    const sem = codigo.replace(/\btr\((["'`])(?:\\.|(?!\1)[^\\])*\1/g, "").replace(/\btrn\(\s*[^,]+,\s*\[[^\]]*\]/g, "");
    for (const m of sem.matchAll(/(["'`])((?:\\.|(?!\1)[^\\\n]){2,}?)\1/g)) {
      const s = m[2];
      if (/^\s*$/.test(s) || (CODIGO.test(s) && !ACENTO.test(s) && !(/\s/.test(s) && PALAVRA.test(s)))) continue;
      if (/^[a-z]+(-[a-z]+)*$/.test(s) || /^https?:/.test(s) || /^[\w./-]+\.(js|css|png|svg|json)$/.test(s)) continue;
      if (ACENTO.test(s) || PALAVRA.test(s)) achados.push([i + 1, s]);
    }
    for (const m of sem.matchAll(/>([^<>${}\n]{3,})</g)) if (/[A-Za-zÀ-ú]{3,}/.test(m[1]) && !achados.some(([l, s]) => l === i + 1 && s.includes(m[1].trim()))) achados.push([i + 1, m[1].trim()]);
  });
  return achados;
}

const arquivos = [...readdirSync(dir).filter((f) => f.endsWith(".js")).map((f) => [f, join(dir, f)]), ["blueprint.html", join(raiz, "blueprint.html")]];
const alvo = process.argv.slice(2).find((a) => !a.startsWith("--"));
const resultado = arquivos.map(([nome, caminho]) => [nome, candidatos(readFileSync(caminho, "utf8"))]).filter(([, c]) => c.length);
if (alvo) { for (const [nome, c] of resultado.filter(([n]) => n === alvo)) for (const [l, s] of c) console.log(`${l}: ${s.slice(0, 160)}`); }
else { const total = resultado.reduce((n, [, c]) => n + c.length, 0); for (const [nome, c] of resultado.sort((a, b) => b[1].length - a[1].length)) console.log(String(c.length).padStart(4), nome); console.log(String(total).padStart(4), "candidatos em", resultado.length, "arquivos"); }
if (process.argv.includes("--pendentes")) process.exit(resultado.length ? 1 : 0);
