// Gera dist/biblioteca.js (kit + lathe do repositório como um só script, mais a biblioteca) e dist/estilo.css (ISO_CSS + estilo/*.css).
// O kit do repositório está dividido em módulos ES; aqui cada um vira um escopo próprio (sem import/export) e o que exportava sai num objeto.
// Uso: node apresentacao-ia/empacotar.mjs             (grava dist/)
//      node apresentacao-ia/empacotar.mjs --conferir  (não grava; erro se dist/ estiver diferente do que seria gerado)
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const AQUI = dirname(fileURLToPath(import.meta.url));
const KIT = join(AQUI, "..", "kit");
const BIBLIOTECA = ["textos", "laco", "auditoria", "pecas", "secoes", "fragmento", "casca"];
const ESTILOS = ["base", "secoes", "impressao"];

const ler = (caminho) => readFileSync(caminho, "utf8").replace(/\r\n/g, "\n");
const IMPORTA = /^import\s*\{[^}]*\}\s*from\s*"([^"]+)";?\s*$/gm;
const REEXPORTA = /^export \* from "([^"]+)";?\s*$/gm;
const NL = "\n";
// Motivo escrito para passar do limite de linhas (regra do projeto: teto 500 só com motivo no topo do arquivo): estes dois arquivos são GERADOS.
const CABECALHO_JS = [
  "// ARQUIVO GERADO por apresentacao-ia/empacotar.mjs. Não edite: mude kit/ ou biblioteca/*.js e rode `node apresentacao-ia/empacotar.mjs`.",
  "// Por ser gerado, fica fora do limite de 350 linhas (teto 500): o limite vale para as fontes, que são kit/**/*.mjs e biblioteca/*.js (cada uma até 350).",
  "// `node apresentacao-ia/empacotar.mjs --conferir` e empacotar.test.mjs falham se este arquivo estiver diferente do que o empacotador geraria agora.",
  "",
].join(NL);
const CABECALHO_CSS = [
  "/* ARQUIVO GERADO por apresentacao-ia/empacotar.mjs. Não edite: mude kit/ ou estilo/*.css e rode `node apresentacao-ia/empacotar.mjs`.",
  "   Por ser gerado, fica fora do limite de 350 linhas (teto 500): o limite vale para as fontes (ISO_CSS do kit e estilo/*.css). --conferir e empacotar.test.mjs o vigiam. */",
  "",
].join(NL);

// módulos do kit em ordem de dependência (quem é importado vem antes); os índices (`export * from`) entram depois dos filhos
function modulosDoKit() {
  const ordem = [], visto = new Set();
  const visita = (arquivo) => {
    arquivo = resolve(arquivo);
    if (visto.has(arquivo)) return;
    visto.add(arquivo);
    const fonte = ler(arquivo);
    for (const m of fonte.matchAll(REEXPORTA)) visita(join(dirname(arquivo), m[1]));
    for (const m of fonte.matchAll(IMPORTA)) visita(join(dirname(arquivo), m[1]));
    ordem.push(arquivo);
  };
  visita(join(KIT, "iso-kit.mjs"));
  visita(join(KIT, "lathe.mjs"));
  return ordem;
}

// Cada módulo vira um escopo próprio (os dois `interno.mjs` repetem nomes de propósito): o que ele importa vem de const M<n> dos módulos
// anteriores e o que exporta sai num objeto; os índices só juntam os objetos dos filhos. L recebe o conjunto dos dois índices.
export function gerarBiblioteca() {
  const arquivos = modulosDoKit(), idDe = (arquivo) => "M" + arquivos.indexOf(resolve(arquivo)), partes = [];
  for (const arquivo of arquivos) {
    const fonte = ler(arquivo), rotulo = arquivo.slice(KIT.length + 1).split("\\").join("/"), filhos = [...fonte.matchAll(REEXPORTA)];
    if (filhos.length) {
      partes.push(`const ${idDe(arquivo)} = Object.assign({}, ${filhos.map((f) => idDe(join(dirname(arquivo), f[1]))).join(", ")}); // ${rotulo}`);
      continue;
    }
    const usa = [...fonte.matchAll(IMPORTA)].map((m) => `const { ${m[0].match(/\{([^}]*)\}/)[1].trim()} } = ${idDe(join(dirname(arquivo), m[1]))};`);
    const nomes = [...fonte.matchAll(/^export\s+(?:const|let|var|function|class)\s+([\w$]+)/gm)].map((m) => m[1]);
    const corpo = fonte.replace(IMPORTA, "").replace(/^export\s+(?=(?:const|let|var|function|class|async)\b)/gm, "");
    partes.push([`const ${idDe(arquivo)} = (() => { // ${rotulo}`, ...usa, corpo, `return { ${nomes.join(", ")} };`, "})();"].join(NL));
  }
  const termos = JSON.parse(ler(join(AQUI, "termos-tecnicos.json"))).termos;
  const kit = [...partes, `const L = Object.assign({}, ${idDe(join(KIT, "iso-kit.mjs"))}, ${idDe(join(KIT, "lathe.mjs"))});`, `L.TERMOS = ${JSON.stringify(termos)};`, ""].join(NL);
  const lib = BIBLIOTECA.map((n) => `// ---- biblioteca/${n}.js ----${NL}` + ler(join(AQUI, "biblioteca", n + ".js"))).join(NL);
  return CABECALHO_JS + kit + lib;
}

export async function gerarEstilo() {
  const { ISO_CSS } = await import(pathToFileURL(join(KIT, "iso-kit.mjs")).href);
  return CABECALHO_CSS + `${ISO_CSS}${NL}` + ESTILOS.map((n) => `/* ---- estilo/${n}.css ---- */${NL}` + ler(join(AQUI, "estilo", n + ".css"))).join(NL);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const js = gerarBiblioteca(), css = await gerarEstilo();
  const dist = (nome) => join(AQUI, "dist", nome);
  if (process.argv.includes("--conferir")) {
    // como gerar-roadmap.mjs --conferir: não grava nada; sai com erro se dist/ não for o que o empacotador geraria agora
    const velho = [["biblioteca.js", js], ["estilo.css", css]].filter(([nome, novo]) => !existsSync(dist(nome)) || ler(dist(nome)) !== novo).map(([nome]) => "dist/" + nome);
    if (velho.length) { console.error(`desatualizado: ${velho.join(", ")}. Rode: node apresentacao-ia/empacotar.mjs`); process.exit(1); }
    console.log("dist/ confere com as fontes");
  } else {
    mkdirSync(join(AQUI, "dist"), { recursive: true });
    writeFileSync(dist("biblioteca.js"), js);
    writeFileSync(dist("estilo.css"), css);
    console.log(`dist/biblioteca.js ${js.length} bytes, dist/estilo.css ${css.length} bytes`);
  }
}
