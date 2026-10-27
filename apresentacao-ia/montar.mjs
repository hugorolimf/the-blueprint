// Monta o HTML autocontido da apresentação. A substituição de texto é a MESMA que o servidor Rust faz (para testes e prévias):
// um só passo sobre modelo.html, com estes marcadores, e o que entra nunca é reexaminado (um conteúdo com "{{css}}" dentro não vira CSS).
//   {{titulo}} {{idioma}}  texto, com escape de HTML
//   {{css}}                dist/estilo.css, com "</style" neutralizado
//   {{kit}}                vazio: o kit já vai dentro de dist/biblioteca.js (o marcador fica no modelo por compatibilidade com o contrato)
//   {{biblioteca}}         dist/biblioteca.js, com "</script" neutralizado
//   {{conteudo}}           o conteúdo como JSON (literal JS), com "</script" e "<!--" neutralizados
//   {{figuras}}            JSON puro (matriz) só com os metadados de cada figura: todos os campos MENOS `codigo`
//   {{codigos}}            um <script> por figura, na ordem das figuras, separados por uma quebra de linha:
//                          <script>(window.AP_CODIGOS=window.AP_CODIGOS||{})[<id como JSON>]=(<codigo cru>);</script>
//                          O CSP não tem unsafe-eval, então `codigo` precisa ser código da página; em <script> próprio, um erro de sintaxe numa figura
//                          só derruba o bloco dela (a casca mostra o cartão de erro) e o resto da página segue. "</script" e "<!--" são neutralizados
//                          no código e no id, e "</style" no css, do mesmo jeito no servidor Rust.
// Uso: node apresentacao-ia/montar.mjs <conteudo.json> <figuras.json> <saida.html>
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const AQUI = dirname(fileURLToPath(import.meta.url));
// CRLF vira LF ao ler: o .gitattributes manda *.html em CRLF no Windows e a saída tem de ser a mesma em qualquer máquina (o Rust faz igual)
const lerArquivo = (...partes) => readFileSync(join(AQUI, ...partes), "utf8").replace(/\r\n/g, "\n");

export const escHtml = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
// "</script" fecharia o <script> da página e "<!--" abriria um comentário dentro dele; ambos viram o mesmo texto com uma barra, válido em JS
export const escScript = (s) => s.replace(/<\/(script)/gi, "<\\/$1").replace(/<!--/g, "<\\!--");
const escStyle = (s) => s.replace(/<\/(style)/gi, "<\\/$1");
export const figurasJson = (figuras) => escScript(JSON.stringify(figuras.map(({ codigo, ...metadados }) => metadados)));
export const codigosHtml = (figuras) => figuras.map((f) => `<script>(window.AP_CODIGOS=window.AP_CODIGOS||{})[${escScript(JSON.stringify(f.id))}]=(${escScript(String(f.codigo ?? ""))});</script>`).join("\n");

export function montar({ conteudo, figuras = [], modelo, css, biblioteca }) {
  modelo ??= lerArquivo("modelo.html");
  css ??= lerArquivo("dist", "estilo.css");
  biblioteca ??= lerArquivo("dist", "biblioteca.js");
  const valores = {
    titulo: escHtml(conteudo.titulo),
    idioma: escHtml(conteudo.idioma || "en"),
    css: escStyle(css),
    kit: "",
    biblioteca: escScript(biblioteca),
    conteudo: escScript(JSON.stringify(conteudo)),
    figuras: figurasJson(figuras),
    codigos: codigosHtml(figuras),
  };
  return modelo.replace(/\{\{(\w+)\}\}/g, (inteiro, nome) => (nome in valores ? valores[nome] : inteiro));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [conteudo, figuras, saida] = process.argv.slice(2);
  if (!conteudo || !figuras || !saida) { console.error("uso: node apresentacao-ia/montar.mjs <conteudo.json> <figuras.json> <saida.html>"); process.exit(2); }
  const html = montar({ conteudo: JSON.parse(readFileSync(conteudo, "utf8")), figuras: JSON.parse(readFileSync(figuras, "utf8")) });
  writeFileSync(saida, html);
  console.log(`${saida} ${html.length} bytes`);
}
