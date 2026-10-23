// ---- o menu do botão direito na tela: cabeçalho, itens em grupos com ícone, posição e fechamento (itens em contexto-itens.js, eventos em contexto.js) ----
const menuContexto = $("menu-contexto");
let alvoContexto = null;
function fecharMenuContexto(restaurar = false) {
  if (menuContexto.hidden) return;
  menuContexto.hidden = true;
  if (restaurar && alvoContexto?.isConnected) alvoContexto.focus?.({ preventScroll: true });
  alvoContexto = null;
}
function botaoDoContexto({ rotulo, fn, icone, perigo, desabilitado, motivo }) {
  const botao = document.createElement("button"), texto = document.createElement("span"), ic = document.createElement("span");
  botao.type = "button";
  botao.setAttribute("role", "menuitem");
  botao.tabIndex = -1;
  ic.className = "mc-ic";
  ic.append(iconeContexto(icone));
  texto.className = "mc-tx";
  texto.textContent = rotulo;
  botao.append(ic, texto);
  if (perigo) botao.classList.add("perigo");
  if (desabilitado) { botao.setAttribute("aria-disabled", "true"); if (motivo) botao.title = motivo; }
  botao.addEventListener("click", () => { if (desabilitado) return; fecharMenuContexto(); fn(); });
  return botao;
}
function abrirMenuContexto(e, alvo, { titulo, itens }) {
  alvoContexto = alvo;
  const topo = document.createElement("div");
  topo.className = "mc-topo";
  topo.setAttribute("aria-hidden", "true");
  topo.textContent = corta(titulo, 48);
  const nos = [topo];
  itens.forEach((item, i) => {
    if (i && item.grupo !== itens[i - 1].grupo) { const sep = document.createElement("div"); sep.className = "mc-sep"; sep.setAttribute("role", "separator"); nos.push(sep); }
    nos.push(botaoDoContexto(item));
  });
  menuContexto.setAttribute("aria-label", titulo);
  menuContexto.replaceChildren(...nos);
  menuContexto.hidden = false;
  const caixa = alvo?.getBoundingClientRect();
  const x = e.clientX || caixa?.left || 8, y = e.clientY || caixa?.bottom || 8;
  menuContexto.style.left = Math.max(8, Math.min(x, innerWidth - menuContexto.offsetWidth - 8)) + "px";
  menuContexto.style.top = Math.max(8, Math.min(y, innerHeight - menuContexto.offsetHeight - 8)) + "px";
  menuContexto.querySelector("button")?.focus({ preventScroll: true });
}
