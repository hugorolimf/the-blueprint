// Exporta os emblemas do kit (emblemas.mjs) como SVG avulsos para a interface nativa (ui/assets/emblemas/), como o SM PRO faz com o anatomy.
// O SVG da página depende do CSS dela (classes e variáveis); aqui as cores entram escritas no próprio arquivo, que é o que o renderizador do egui entende.
// Uso: node scripts/exportar-emblemas.mjs
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as emb from "../emblemas.mjs";

// ---- texto em contornos: o renderizador do egui não tem fontes, então as letras das marcas (o «IA» do chip) viram caminhos da JetBrains Mono Bold ----
const ttf = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "..", "ui", "assets", "fontes", "JetBrainsMono-Bold.ttf"));
const tabelas = Object.fromEntries(Array.from({ length: ttf.readUInt16BE(4) }, (_, i) => [ttf.toString("latin1", 12 + i * 16, 16 + i * 16), ttf.readUInt32BE(20 + i * 16)]));
const upem = ttf.readUInt16BE(tabelas.head + 18), loca32 = ttf.readInt16BE(tabelas.head + 50) === 1;
function glifoDe(codigo) {
  // cmap formato 4 (Windows, Unicode BMP)
  const cm = tabelas.cmap, n = ttf.readUInt16BE(cm + 2);
  let sub = 0;
  for (let i = 0; i < n; i++) if (ttf.readUInt16BE(cm + 4 + i * 8) === 3 && ttf.readUInt16BE(cm + 6 + i * 8) === 1) sub = cm + ttf.readUInt32BE(cm + 8 + i * 8);
  const seg = ttf.readUInt16BE(sub + 6) / 2, fins = sub + 14, inicios = fins + seg * 2 + 2, deltas = inicios + seg * 2, desvios = deltas + seg * 2;
  for (let i = 0; i < seg; i++) {
    if (codigo > ttf.readUInt16BE(fins + i * 2) || codigo < ttf.readUInt16BE(inicios + i * 2)) continue;
    const desvio = ttf.readUInt16BE(desvios + i * 2);
    if (!desvio) return (codigo + ttf.readInt16BE(deltas + i * 2)) & 0xffff;
    const g = ttf.readUInt16BE(desvios + i * 2 + desvio + (codigo - ttf.readUInt16BE(inicios + i * 2)) * 2);
    return g ? (g + ttf.readInt16BE(deltas + i * 2)) & 0xffff : 0;
  }
  return 0;
}
/** O contorno de um glifo simples como caminho SVG (quadráticas), em unidades da fonte com o y para cima; `dx` desloca em x. */
function contorno(g, dx) {
  const a = loca32 ? ttf.readUInt32BE(tabelas.loca + g * 4) : ttf.readUInt16BE(tabelas.loca + g * 2) * 2, b = loca32 ? ttf.readUInt32BE(tabelas.loca + g * 4 + 4) : ttf.readUInt16BE(tabelas.loca + g * 2 + 2) * 2;
  if (a === b) return "";
  let o = tabelas.glyf + a;
  const nc = ttf.readInt16BE(o);
  if (nc < 0) throw new Error(`glifo composto ${g}: não suportado`);
  const fimC = Array.from({ length: nc }, (_, i) => ttf.readUInt16BE(o + 10 + i * 2)), np = fimC[nc - 1] + 1;
  o += 10 + nc * 2;
  o += 2 + ttf.readUInt16BE(o);
  const flags = [];
  while (flags.length < np) { const f = ttf.readUInt8(o++); flags.push(f); if (f & 8) { let r = ttf.readUInt8(o++); while (r--) flags.push(f); } }
  const ler = (curto, mesmo) => { const v = []; let x = 0; for (const f of flags) { if (f & curto) { const d = ttf.readUInt8(o++); x += f & mesmo ? d : -d; } else if (!(f & mesmo)) { x += ttf.readInt16BE(o); o += 2; } v.push(x); } return v; };
  const xs = ler(2, 16), ys = ler(4, 32);
  let d = "", ini = 0;
  for (const fim of fimC) {
    const pts = [];
    for (let i = ini; i <= fim; i++) pts.push({ x: xs[i] + dx, y: ys[i], on: !!(flags[i] & 1) });
    ini = fim + 1;
    // começa num ponto sobre a curva (ou no meio de dois fora dela) e percorre os segmentos
    let k = pts.findIndex((q) => q.on), p0;
    if (k < 0) { p0 = { x: (pts[0].x + pts[pts.length - 1].x) / 2, y: (pts[0].y + pts[pts.length - 1].y) / 2 }; k = -1; } else p0 = pts[k];
    const seq = k < 0 ? pts : [...pts.slice(k + 1), ...pts.slice(0, k + 1)];
    d += `M${p0.x} ${-p0.y}`;
    for (let i = 0; i < seq.length; i++) {
      const q = seq[i];
      if (q.on) { d += `L${q.x} ${-q.y}`; continue; }
      const prox = seq[(i + 1) % seq.length];
      const fim2 = prox.on ? prox : { x: (q.x + prox.x) / 2, y: (q.y + prox.y) / 2 };
      d += `Q${q.x} ${-q.y} ${fim2.x} ${-fim2.y}`;
      if (prox.on) i++;
    }
    d += "Z";
  }
  return d;
}
/** `<text>` (uma linha, `text-anchor: middle`, espaçamento .04em do marca.css) → `<path>` com a mesma matriz. */
function textoEmCaminho(attrs, texto) {
  const tam = Number(/font-size="([^"]+)"/.exec(attrs)[1]), tom = /data-tone="([^"]+)"/.exec(attrs)?.[1], matriz = /transform="([^"]+)"/.exec(attrs)?.[1] ?? "";
  const avanco = 600 + 0.04 * upem, esc = tam / upem;
  const x0 = -(texto.length * avanco) / 2;
  const d = [...texto].map((c, i) => contorno(glifoDe(c.codePointAt(0)), x0 + i * avanco)).join("");
  return `<path class="iso-face-text" ${tom ? `data-tone="${tom}" ` : ""}transform="${matriz} scale(${esc})" d="${d}"/>`;
}

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const destino = join(raiz, "ui", "assets", "emblemas");
mkdirSync(destino, { recursive: true });

// as regras `.emblema …` de web/css/marca.css, com as variáveis trocadas pelos valores; fora o que o renderizador não conhece (:has, var, transform)
const css = readFileSync(join(raiz, "web", "css", "marca.css"), "utf8");
const variaveis = Object.fromEntries([...css.match(/\.emblema\{([^}]*)\}/)[1].matchAll(/(--anatomy-[\w-]+):([^;]+)/g)].map((m) => [m[1], m[2].trim()]));
const regras = [...css.matchAll(/^(\.emblema [^{]+)\{([^}]*)\}$/gm)]
  .filter((m) => !m[1].includes(":has") && !m[2].includes("non-scaling"))
  .map((m) => `${m[1].replace(/\.emblema /g, "")}{${m[2].replace(/var\((--anatomy-[\w-]+)\)/g, (_, v) => variaveis[v]).replace(/transform:[^;]+;?/g, "")}}`)
  .join("\n");

const marcas = { blueprint: emb.BLUEPRINT, moedas: emb.MOEDAS, livros: emb.LIVROS, chip: emb.CHIP, caminhao: emb.CAMINHAO };
// cartão da entrada (280x120), a folha de boas-vindas (390x280) e a prévia do novo workspace (520x380)
const tamanhos = { cartao: [280, 120], grande: [390, 280], previa: [520, 380] };
for (const [nome, pecas] of Object.entries(marcas)) {
  for (const [sufixo, [largura, altura]] of Object.entries(tamanhos)) {
    const e = emb.desenhaEmblema(pecas, { largura, altura });
    const corpo = e.svg.replace(/<text([^>]*)>([^<]*)<\/text>/g, (_, attrs, t) => textoEmCaminho(attrs, t));
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}"><style>${regras}</style>${corpo}</svg>\n`;
    writeFileSync(join(destino, `${nome}-${sufixo}.svg`), svg);
  }
}
// o emblema «em construção» dos cartões da lista: as peças sólidas seguem o progresso, então vai em 11 passos (0, 10 … 100%), no tamanho máximo do cartão (116x100)
for (let passo = 0; passo <= 10; passo++) {
  const e = emb.desenhaEmblema(emb.CONSTRUCAO, { largura: 116, altura: 100, pad: 2, progresso: passo / 10, sombra: false });
  const corpo = e.svg.replace(/<text([^>]*)>([^<]*)<\/text>/g, (_, attrs, t) => textoEmCaminho(attrs, t));
  writeFileSync(join(destino, `construcao-${String(passo * 10).padStart(3, "0")}.svg`), `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}"><style>${regras}</style>${corpo}</svg>\n`);
}
// a marca-d'água da lista: a marca do workspace «construída» conforme o progresso, em 11 passos (só a padrão; as outras vão inteiras, `-previa`), no formato 1,4:1 da página
for (let passo = 0; passo <= 10; passo++) {
  const e = emb.desenhaEmblema(emb.BLUEPRINT, { largura: 700, altura: 500, progresso: passo / 10, sombra: false });
  const corpo = e.svg.replace(/<text([^>]*)>([^<]*)<\/text>/g, (_, attrs, t) => textoEmCaminho(attrs, t));
  writeFileSync(join(destino, `fundo-${String(passo * 10).padStart(3, "0")}.svg`), `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}"><style>${regras}</style>${corpo}</svg>\n`);
}
console.log(`${Object.keys(marcas).length * Object.keys(tamanhos).length + 22} emblemas em ${destino}`);
