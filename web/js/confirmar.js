// Confirmação no meio da tela (no lugar do confirm() do navegador). Devolve uma Promise<boolean>; Esc, fundo e "Cancelar" respondem não.
function confirmar({ titulo, texto, rotulo = "Excluir" }) {
  return new Promise((resolve) => {
    const d = document.createElement("dialog");
    d.className = "confirmar";
    d.setAttribute("role", "alertdialog");
    d.setAttribute("aria-label", titulo);
    d.innerHTML = `<div class="cu-topo"><h3>${esc(titulo)}</h3></div><p class="cu-corpo">${esc(texto)}</p><div class="confirmar-acoes"><button class="btn" type="button" data-r="0">${tr("Cancelar")}</button><button class="btn perigo" type="button" data-r="1">${esc(rotulo)}</button></div>`;
    d.addEventListener("click", (e) => { if (e.target === d) d.close("0"); }); // clique no fundo
    d.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => d.close(b.dataset.r)));
    d.addEventListener("close", () => { d.remove(); resolve(d.returnValue === "1"); });
    document.body.append(d);
    d.showModal();
    d.querySelector('[data-r="0"]').focus(); // o foco começa em Cancelar: Enter sem querer não apaga
  });
}
