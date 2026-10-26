// Exportar e imprimir a apresentação. `doc` é o elemento que contém as <section class="ap-pagina"> (ou o HTML delas como texto).
//   exportarApresentacao(doc, {formato: "html"})  baixa UM arquivo autocontido (CSS, SVG e imagens embutidos, sem rede, sem script) e devolve {html, nome}
//   exportarApresentacao(doc, {formato: "pdf"})   abre o diálogo de impressão (no WebView2 e no navegador, "Salvar como PDF")
//   imprimirApresentacao(doc)                     o mesmo que formato "pdf"
// As duas saídas usam o MESMO documento avulso (montarApresentacaoAvulsa): o que se imprime é o que o arquivo exportado imprime, sem depender do CSS de impressão da página do app.
// shortcut: figuras que apontam para <defs> fora das páginas (<use href="#id">) perdem a referência no arquivo avulso; hoje o anatomy desenha cada figura com os seus defs dentro.
const APX_FOLHAS = ["tokens", "marca", "claude", "apresentacao", "impressao"]; // web/css/<nome>.css, nesta ordem (`folhas` troca a lista); a que não existe é pulada
const APX_CSP = "default-src 'none'; style-src 'unsafe-inline'; img-src data:; font-src data:"; // o arquivo exportado não tem script nem rede

function apxPaginas(doc) {
  let raiz = doc;
  if (typeof doc === "string") { raiz = document.createElement("div"); raiz.innerHTML = doc; }
  const paginas = [...raiz.querySelectorAll(".ap-pagina")].map((p) => p.cloneNode(true));
  for (const p of paginas) if (p.querySelector(":scope > .ap-int")) p.classList.add("ap-slide"); // página 16:9 de apresentacao.css: imprime deitada, no tema azul (impressao.css)
  return paginas;
}

async function apxFolhas(folhas) {
  const textos = [];
  for (const nome of folhas) {
    try { const r = await fetch(`/web/css/${nome}.css`); if (r.ok) textos.push(await r.text()); } catch { /* sem a folha, segue sem ela */ }
  }
  return textos.join("\n").replace(/\/\*[\s\S]*?\*\//g, "").replace(/<\/style/gi, "<\\/style"); // sem comentários: o arquivo avulso leva só o que vale
}

const apxDataUrl = (blob) => new Promise((ok, erro) => { const l = new FileReader(); l.onload = () => ok(l.result); l.onerror = erro; l.readAsDataURL(blob); });

// imagens (os GIFs do sistema, por exemplo) viram data: para o arquivo não depender de nada fora dele
async function apxEmbutirImagens(paginas) {
  for (const img of paginas.flatMap((p) => [...p.querySelectorAll("img[src]")])) {
    const src = img.getAttribute("src");
    if (src.startsWith("data:")) continue;
    try { img.setAttribute("src", await apxDataUrl(await (await fetch(src)).blob())); }
    catch { img.removeAttribute("src"); } // melhor sem a imagem do que com um endereço que o arquivo avulso não alcança
  }
}

const apxEsc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const apxNome = (t) => (t.toLowerCase().normalize("NFD").replace(/[\u0300-\u036f]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 60) || "apresentacao") + ".html";

// o documento avulso: { html, nome, paginas }. `estilos` é CSS extra para o arquivo, `folhas` a lista de web/css/<nome>.css (padrão: tokens, marca, claude, apresentacao, impressao)
async function montarApresentacaoAvulsa(doc, { estilos = "", folhas = APX_FOLHAS } = {}) {
  const paginas = apxPaginas(doc);
  if (!paginas.length) throw new Error(tr("A apresentação não tem páginas."));
  for (const p of paginas) p.querySelectorAll("script,.ap-nao-imprimir,[data-nao-imprimir]").forEach((e) => e.remove());
  await apxEmbutirImagens(paginas);
  const titulo = (paginas[0].querySelector(".ap-titulo")?.textContent || "").trim() || tr("Apresentação do blueprint");
  const html = `<!doctype html>\n<html lang="${apxEsc(I18N.ativo)}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">`
    + `<meta http-equiv="Content-Security-Policy" content="${APX_CSP}"><title>${apxEsc(titulo)}</title><style>\n${await apxFolhas(folhas)}\n${estilos.replace(/<\/style/gi, "<\\/style")}\n</style></head>`
    + `<body class="ap-solo"><main class="ap-doc">\n${paginas.map((p) => p.outerHTML).join("\n")}\n</main></body></html>\n`;
  return { html, nome: apxNome(titulo), paginas: paginas.length };
}

// imprime o documento avulso num iframe fora da vista; resolve quando o diálogo fecha (ou em 2 min)
async function imprimirApresentacao(doc, opcoes = {}) {
  const { html } = await montarApresentacaoAvulsa(doc, opcoes);
  return new Promise((pronto) => {
    const f = document.createElement("iframe");
    f.setAttribute("aria-hidden", "true");
    f.style.cssText = "position:fixed;right:0;bottom:0;width:0;height:0;border:0";
    const fim = () => { clearTimeout(limite); f.remove(); pronto(); };
    const limite = setTimeout(fim, 120000);
    f.onload = () => { f.contentWindow.addEventListener("afterprint", fim); f.contentWindow.focus(); f.contentWindow.print(); };
    f.srcdoc = html;
    document.body.appendChild(f);
  });
}

async function exportarApresentacao(doc, { formato = "html", baixar = true, ...opcoes } = {}) {
  if (formato === "pdf") return imprimirApresentacao(doc, opcoes);
  const { html, nome } = await montarApresentacaoAvulsa(doc, opcoes);
  if (baixar) {
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([html], { type: "text/html;charset=utf-8" }));
    a.download = nome;
    document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 10000);
  }
  return { html, nome };
}

