// ---- os eventos do menu do botão direito (itens em contexto-itens.js, desenho em contexto-menu.js) ----
document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  if (menuContexto.contains(e.target)) return;
  fecharMenuContexto();
  const alvo = e.target instanceof Element ? e.target : e.target.parentElement;
  abrirMenuContexto(e, alvo, itensDoContexto(alvo));
}, true);
document.addEventListener("pointerdown", (e) => { if (!menuContexto.hidden && !menuContexto.contains(e.target)) fecharMenuContexto(); }, true);
document.addEventListener("keydown", (e) => {
  if (menuContexto.hidden) return;
  const itens = [...menuContexto.querySelectorAll('[role="menuitem"]')];
  const atual = itens.indexOf(document.activeElement);
  if (e.key === "Escape") { e.preventDefault(); e.stopImmediatePropagation(); fecharMenuContexto(true); }
  else if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
    e.preventDefault();
    const proximo = e.key === "Home" ? 0 : e.key === "End" ? itens.length - 1 : (atual + (e.key === "ArrowDown" ? 1 : -1) + itens.length) % itens.length;
    itens[proximo]?.focus();
  } else if (e.key === "Tab") fecharMenuContexto(true);
}, true);
addEventListener("scroll", () => fecharMenuContexto(), true);
addEventListener("resize", () => fecharMenuContexto());
