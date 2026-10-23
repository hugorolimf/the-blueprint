// ---- markdown → HTML seguro, para ler as instruções da ação com legibilidade (puro, sem DOM; testado em markdown.test.mjs) ----
// Tudo é escapado antes de virar tag, então texto do agente nunca vira HTML por conta própria; link só http e https (urlSegura). Cobre o que as instruções usam:
// títulos (#), **negrito**, *itálico*, `código`, bloco ```, listas (-, *, 1.) com recuo, [ ] e [x], citação (>), linha (---) e links. Quebra de linha simples
// vira um bloco próprio com espaço entre eles (o texto do agente quase nunca deixa linha em branco entre os itens), e "Rótulo: 1) … 2) …" vira o rótulo com os passos numa lista, e "Rótulo: texto" no começo da linha ganha negrito para o roteiro ler como ficha.
// shortcut: sem tabelas nem listas aninhadas de verdade (o recuo só desloca o item); se as instruções passarem a precisar disso, trocar por uma biblioteca.
function mdInline(bruto) {
  const codigos = [];
  let s = esc(bruto).replace(/`([^`\n]+)`/g, (_, c) => `\u0000${codigos.push(c) - 1}\u0000`);
  s = s.replace(/\*\*([^*\n]+?)\*\*|__([^_\n]+?)__/g, (_, a, b) => `<strong>${a ?? b}</strong>`);
  s = s.replace(/(^|[^*\w])\*([^*\s][^*\n]*?)\*(?![*\w])/g, "$1<em>$2</em>");
  s = s.replace(/(^|[^_\w])_([^_\s][^_\n]*?)_(?![_\w])/g, "$1<em>$2</em>");
  s = s.replace(/\[([^\]\n]+)\]\(([^)\s]+)\)/g, (m, texto, u) => {
    const url = urlSegura(u.replace(/&amp;/g, "&"));
    return url ? `<a href="${esc(url)}" target="_blank" rel="noopener noreferrer">${texto}</a>` : m;
  });
  return s.replace(/\u0000(\d+)\u0000/g, (_, i) => `<code>${codigos[+i]}</code>`);
}
// "Objetivo: …", "Passo a passo: …": o rótulo curto no começo da linha vira negrito (só rótulos que começam com maiúscula e não têm pontuação no meio)
const MD_ROTULO = /^([A-ZÀ-Ú][\p{L}\d /()-]{1,38}):(?=\s|$)/u;
function mdLinhaTexto(l) {
  const m = MD_ROTULO.exec(l);
  return m ? `<strong class="md-rot">${mdInline(m[1])}:</strong>${mdInline(l.slice(m[0].length))}` : mdInline(l);
}
// "1) ler; 2) criar." depois de um rótulo vira uma lista numerada (só com pelo menos dois passos, começando em 1); senão devolve "" e o texto fica como está
function mdPassos(resto) {
  const marcas = [...resto.matchAll(/(?:^|[\s;])(\d{1,2})\)\s+/g)];
  if (marcas.length < 2 || marcas[0][1] !== "1" || resto.slice(0, marcas[0].index + marcas[0][0].length).trim() !== "1)") return "";
  const itens = marcas.map((m, i) => resto.slice(m.index + m[0].length, marcas[i + 1]?.index ?? resto.length).replace(/[;\s]+$/, "").trim());
  return itens.every(Boolean) ? itens.map((x) => `<li>${mdInline(x)}</li>`).join("") : "";
}
function mdBloco(l, sep) {
  const cl = sep ? ' class="md-sep"' : "", m = MD_ROTULO.exec(l), passos = m ? mdPassos(l.slice(m[0].length)) : "";
  return passos ? `<p${cl}><strong class="md-rot">${mdInline(m[1])}:</strong></p><ol class="md-passos">${passos}</ol>` : `<p${cl}>${mdLinhaTexto(l)}</p>`;
}
function mdHtml(texto) {
  const linhas = String(texto ?? "").replace(/\r\n?/g, "\n").split("\n");
  const out = [];
  let par = [], lista = null;
  let sep = false; // havia linha em branco antes: o bloco seguinte ganha um espaço maior
  const fechaPar = () => { for (const l of par) { out.push(mdBloco(l, sep)); sep = false; } par = []; };
  const fechaLista = () => { if (lista) out.push(`<${lista.tag}>${lista.itens.join("")}</${lista.tag}>`); lista = null; };
  for (let i = 0; i < linhas.length; i++) {
    const l = linhas[i];
    if (/^\s*```/.test(l)) { // bloco de código até o fecho (ou até o fim do texto)
      fechaPar(); fechaLista();
      const cod = [];
      for (i++; i < linhas.length && !/^\s*```/.test(linhas[i]); i++) cod.push(linhas[i]);
      out.push(`<pre><code>${esc(cod.join("\n"))}</code></pre>`);
      continue;
    }
    if (!l.trim()) { fechaPar(); fechaLista(); sep = out.length > 0; continue; }
    let m;
    if ((m = /^\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$/.exec(l))) { fechaPar(); fechaLista(); out.push(`<p class="md-h md-h${m[1].length}">${mdInline(m[2])}</p>`); continue; }
    if (/^\s{0,3}([-*_])(\s*\1){2,}\s*$/.test(l)) { fechaPar(); fechaLista(); out.push("<hr>"); continue; }
    if ((m = /^\s{0,3}>\s?(.*)$/.exec(l))) { fechaPar(); fechaLista(); out.push(`<blockquote>${mdInline(m[1])}</blockquote>`); continue; }
    if ((m = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/.exec(l))) {
      fechaPar();
      const tag = /\d/.test(m[2]) ? "ol" : "ul";
      if (lista && lista.tag !== tag) fechaLista();
      lista ||= { tag, itens: [] };
      const nivel = Math.min(3, Math.floor(m[1].replace(/\t/g, "  ").length / 2));
      const caixa = /^\[( |x|X)\]\s+/.exec(m[3]);
      const corpo = caixa ? `<span class="md-cx" aria-hidden="true">${caixa[1] === " " ? "☐" : "☑"}</span> ${mdInline(m[3].slice(caixa[0].length))}` : mdInline(m[3]);
      lista.itens.push(`<li class="md-n${nivel}">${corpo}</li>`);
      continue;
    }
    fechaLista();
    par.push(l.trim());
  }
  fechaPar(); fechaLista();
  return out.join("");
}
// Campo de texto com a prévia ao lado: o que se digita aparece renderizado em tempo real (um quadro por vez, para não pesar com texto grande).
const MD_VAZIO = '<p class="md-vazio">A prévia aparece aqui enquanto você escreve.</p>';
function mdEdicao(attrs, texto, { rows = 10, placeholder = "", oculto = false } = {}) {
  return `<div class="obs-edicao"${oculto ? " hidden" : ""}><textarea ${attrs} rows="${rows}" placeholder="${esc(placeholder)}">${esc(texto)}</textarea><div class="obs-previa-bloco"><span class="obs-rot">Prévia</span><div class="md obs-previa">${mdHtml(texto) || MD_VAZIO}</div></div></div>`;
}
let mdQuadro = 0;
function mdPrevia(ta) {
  const alvo = ta.closest(".obs-edicao")?.querySelector(".obs-previa");
  if (!alvo) return;
  cancelAnimationFrame(mdQuadro);
  mdQuadro = requestAnimationFrame(() => { alvo.innerHTML = mdHtml(ta.value) || MD_VAZIO; });
}
