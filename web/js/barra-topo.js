// ---- a barra do topo da folha nunca cobre o cabeçalho (nome e subtítulo, marcados com data-titulo; estilo em barra.css) ----
// Com espaço: botões com texto e a pesquisa na segunda fileira, as duas à direita do subtítulo.
// Sem espaço: "compacta", uma fileira só de ícones (a pesquisa vira lupa e abre por cima dela), na altura do nome, que é a única coisa a evitar.
const BARRA_CHEIA = 760; // largura que os botões com texto pedem (medida com Compartilhar e Terminal à vista)
function barraAjusta() {
  const barra = document.querySelector("#folha .barra"), folha = $("folha"), base = folha.getBoundingClientRect().left - folha.scrollLeft;
  const fim = (sel) => Math.max(0, ...[...host.querySelectorAll(sel)].map((t) => t.getBoundingClientRect().right - base + 24));
  const cabecalho = fim("[data-titulo]"), compacta = !!cabecalho && barra.getBoundingClientRect().right - base - cabecalho < BARRA_CHEIA;
  barra.classList.toggle("compacta", compacta);
  barra.style.setProperty("--barra-esq", cabecalho ? `${Math.round(compacta ? fim('[data-titulo="nome"]') : cabecalho)}px` : "");
  // botão só com ícone: o nome aparece ao passar o mouse (quem já tem uma dica própria no HTML a mantém)
  for (const b of barra.querySelectorAll(".btn")) if (!b.title || b.dataset.dicaAuto) { b.title = b.textContent.replace(/^[^\p{L}]+/u, "").trim(); b.dataset.dicaAuto = "1"; }
}
// render.js chama no quadro seguinte a cada desenho; a largura da janela decide se cabe o texto
addEventListener("resize", () => requestAnimationFrame(barraAjusta));
