// ---- dica de tela cheia: aparece uma única vez em cada tipo de tela (lista, blueprint, edição, novo blueprint) ----
const dicaF11 = $("dica-f11");
let dicaTimer = 0;
function sugereF11(chave) {
  if (document.fullscreenElement || (innerWidth >= screen.width - 2 && innerHeight >= screen.height - 2)) return; // já está em tela cheia
  let vistas = {};
  try { vistas = JSON.parse(localStorage.getItem("blueprint.f11") || "{}"); } catch {}
  if (vistas[chave]) return;
  vistas[chave] = 1;
  try { localStorage.setItem("blueprint.f11", JSON.stringify(vistas)); } catch {}
  dicaF11.innerHTML = `<b>F11</b><span>Pressione F11 para uma melhor visualização desta tela.</span><button class="fecha" type="button" aria-label="Fechar dica">×</button>`;
  dicaF11.hidden = false;
  clearTimeout(dicaTimer);
  dicaTimer = setTimeout(() => { dicaF11.hidden = true; }, 9000);
}
dicaF11.addEventListener("click", (e) => { if (e.target.closest(".fecha")) dicaF11.hidden = true; });
// ---- rolagem própria: régua lateral com polegar arrastável (a do navegador fica escondida) ---
const rol = $("rolagem"), polegar = rol.querySelector(".polegar"), rotuloRol = rol.querySelector(".rotulo");
function atualizaRolagem() {
  const total = document.documentElement.scrollHeight, vh = innerHeight;
  rol.hidden = total <= vh + 2;
  if (rol.hidden) return;
  const trilho = rol.clientHeight, alt = Math.max(40, (trilho * vh) / total);
  const frac = scrollY / Math.max(1, total - vh);
  const topo = (trilho - alt) * frac;
  polegar.style.height = alt + "px";
  polegar.style.transform = `translateY(${topo}px)`;
  rotuloRol.textContent = `${Math.round(frac * 100)}%`;
  rotuloRol.style.transform = `translateY(${Math.max(0, topo + alt / 2 - 12)}px)`;
  rol.setAttribute("aria-valuenow", String(Math.round(frac * 100)));
  rol.setAttribute("aria-valuetext", rotuloRol.textContent);
}
addEventListener("scroll", atualizaRolagem, { passive: true });
addEventListener("resize", atualizaRolagem);
rol.addEventListener("pointerdown", (e) => {
  const total = document.documentElement.scrollHeight, vh = innerHeight, trilho = rol.clientHeight;
  const alt = polegar.offsetHeight, topoRol = rol.getBoundingClientRect().top;
  const sobre = e.target === polegar;
  // clicar no trilho leva o polegar até o ponto; arrastar o polegar rola proporcionalmente
  const para = (y) => scrollTo({ top: Math.min(1, Math.max(0, (y - topoRol - alt / 2) / (trilho - alt))) * (total - vh), behavior: "auto" });
  const y0 = e.clientY, s0 = scrollY;
  if (!sobre) para(e.clientY);
  try { rol.setPointerCapture(e.pointerId); } catch {}
  rol.classList.add("arrastando-rol");
  const mover = (ev) => (sobre ? scrollTo({ top: s0 + ((ev.clientY - y0) / (trilho - alt)) * (total - vh), behavior: "auto" }) : para(ev.clientY));
  const soltar = () => { rol.classList.remove("arrastando-rol"); rol.removeEventListener("pointermove", mover); rol.removeEventListener("pointerup", soltar); rol.removeEventListener("pointercancel", soltar); };
  rol.addEventListener("pointermove", mover);
  rol.addEventListener("pointerup", soltar);
  rol.addEventListener("pointercancel", soltar);
});
rol.addEventListener("keydown", (e) => {
  const passo = { ArrowDown: 80, ArrowUp: -80, PageDown: innerHeight * 0.9, PageUp: -innerHeight * 0.9 }[e.key];
  if (passo) { e.preventDefault(); scrollBy({ top: passo, behavior: "auto" }); }
  else if (e.key === "Home") { e.preventDefault(); scrollTo({ top: 0 }); }
  else if (e.key === "End") { e.preventDefault(); scrollTo({ top: document.documentElement.scrollHeight }); }
});

// a folha muda de largura quando a gaveta abre ou a janela redimensiona
let larg = 0, quadro = 0;
new ResizeObserver(() => {
  if (host.clientWidth === larg) return;
  larg = host.clientWidth;
  cancelAnimationFrame(quadro);
  quadro = requestAnimationFrame(render);
}).observe(host);
let altJanela = innerHeight;
addEventListener("resize", () => { if (innerHeight !== altJanela) { altJanela = innerHeight; render(); } }); // a folha acompanha a altura da janela
