// Confere os links relativos de todos os .md do repositório: o arquivo existe e, quando há #âncora, o título existe no arquivo de destino. Sem dependências.
// Também confere a regra de idiomas da documentação (docs/development.md#documentation-languages): cada docs/pt-BR/<x>.md tem o inglês
// em docs/<x>.md (CONTRIBUTING, SECURITY e AGENTS na raiz), os dois se apontam e têm os mesmos níveis de título, na mesma ordem.
//   node scripts/conferir-links.mjs        sai com 1 se algum link estiver quebrado (roda no CI)
import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const IGNORAR = new Set(["node_modules", "target", ".git", ".dev", "privado", "videos", "graphify-out", ".impeccable", ".claude"]);
const mds = [];
(function anda(d) { for (const e of readdirSync(d)) { if (IGNORAR.has(e)) continue; const p = join(d, e); statSync(p).isDirectory() ? anda(p) : e.endsWith(".md") && mds.push(p); } })(raiz);
const slug = (t) => t.toLowerCase().replace(/<[^>]+>/g, "").replace(/[`*_~]/g, "").replace(/[^\p{L}\p{N}\s-]/gu, "").trim().replace(/\s+/g, "-");
const ancoras = (arq) => { const s = new Set(); let dentro = false; for (const l of readFileSync(arq, "utf8").split("\n")) { if (l.startsWith("```")) dentro = !dentro; const m = !dentro && /^#{1,6}\s+(.*)$/.exec(l); if (m) s.add(slug(m[1])); } return s; };
const cache = new Map();
const erros = [];
const apontaPara = new Map();
for (const arq of mds) {
  const texto = readFileSync(arq, "utf8").replace(/```[\s\S]*?```/g, "").replace(/`[^`\n]*`/g, "");
  const links = [...texto.matchAll(/\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)].map((m) => m[1]).concat([...texto.matchAll(/(?:href|src)="([^"]+)"/g)].map((m) => m[1]));
  apontaPara.set(arq, new Set());
  for (const l of links) {
    if (/^(https?:|mailto:|data:|#$)/.test(l) || l.includes("<") || l.includes("{")) continue;
    const [caminho, ancora] = l.split("#");
    const alvo = caminho ? resolve(dirname(arq), decodeURIComponent(caminho)) : arq;
    apontaPara.get(arq).add(alvo);
    if (!existsSync(alvo)) { erros.push(`${arq.slice(raiz.length + 1)}: ${l} (arquivo não existe)`); continue; }
    if (ancora && alvo.endsWith(".md") && statSync(alvo).isFile()) {
      if (!cache.has(alvo)) cache.set(alvo, ancoras(alvo));
      if (!cache.get(alvo).has(decodeURIComponent(ancora).toLowerCase())) erros.push(`${arq.slice(raiz.length + 1)}: ${l} (título não existe)`);
    }
  }
}
const niveis = (arq) => { let dentro = false; return readFileSync(arq, "utf8").split("\n").filter((l) => { if (l.startsWith("```")) dentro = !dentro; return !dentro && /^#{1,6}\s/.test(l); }).map((l) => l.indexOf(" ")).join(""); };
const ptBR = join(raiz, "docs", "pt-BR");
const pares = mds.filter((a) => dirname(a).startsWith(ptBR));
for (const pt of pares) {
  const rel = pt.slice(ptBR.length + 1);
  const en = ["CONTRIBUTING.md", "SECURITY.md", "AGENTS.md"].includes(rel) ? join(raiz, rel) : join(raiz, "docs", rel);
  const nome = pt.slice(raiz.length + 1), nomeEn = en.slice(raiz.length + 1);
  if (!existsSync(en)) { erros.push(`${nome}: falta a versão em inglês (${nomeEn})`); continue; }
  if (!apontaPara.get(pt).has(en)) erros.push(`${nome}: não aponta para a versão em inglês (${nomeEn})`);
  if (!apontaPara.get(en).has(pt)) erros.push(`${nomeEn}: não aponta para a versão em português (${nome})`);
  if (niveis(pt) !== niveis(en)) erros.push(`${nome}: títulos diferentes do inglês (níveis ${niveis(en)} no inglês, ${niveis(pt)} aqui)`);
}
if (erros.length) { console.error(`${erros.length} erro(s) nos links ou nos idiomas:\n - ` + erros.join("\n - ")); process.exit(1); }
console.log(`links: ${mds.length} arquivos .md conferidos, nenhum link relativo quebrado; ${pares.length} pares inglês/pt-BR com os mesmos títulos`);
