// ---- arrastar com clique e segurar: reordena os blueprints na lista -------------------------
let arr = null;
const ghost = () => document.querySelector(".fantasma");
function iniciarArrasto() {
  if (!arr || arr.ativo) return;
  arr.ativo = true;
  document.body.classList.add("arrastando");
  const p = D.blueprints.find((x) => x.id === arr.id);
  const f = document.createElement("div");
  f.className = "fantasma";
  f.textContent = `↕ ${p?.nome || "Blueprint"}`;
  f.style.left = arr.mx + "px";
  f.style.top = arr.my + "px";
  document.body.appendChild(f);
}
const alvoSob = (x, y) => document.elementFromPoint(x, y)?.closest("[data-goto]")?.dataset.goto ?? null;
function terminarArrasto(cancelado) {
  if (!arr) return;
  clearTimeout(arr.timer);
  const a = arr;
  arr = null;
  host.querySelectorAll(".soltar").forEach((e) => e.classList.remove("soltar"));
  document.body.classList.remove("arrastando");
  ghost()?.remove();
  if (!a.ativo) return;
  semClique = true;
  setTimeout(() => { semClique = false; }, 0);
  if (cancelado || !a.alvo || a.alvo === a.id) return;
  const de = D.blueprints.findIndex((x) => x.id === a.id), para = D.blueprints.findIndex((x) => x.id === a.alvo);
  if (de < 0 || para < 0) return;
  D.blueprints.splice(para, 0, D.blueprints.splice(de, 1)[0]);
  montaEditor();
  mudou();
  aviso(`Blueprint movido para a posição ${para + 1}`);
}
host.addEventListener("pointerdown", (e) => {
  if (e.button !== 0) return;
  const el = e.target.closest("[data-goto]");
  if (!el) return;
  arr = { id: el.dataset.goto, x: e.clientX, y: e.clientY, mx: e.clientX, my: e.clientY, tipo: e.pointerType, ativo: false, alvo: null };
  arr.timer = setTimeout(iniciarArrasto, 220); // segurar ~0,2 s começa o arrasto, no mouse e no toque
});
addEventListener("pointermove", (e) => {
  if (!arr) return;
  arr.mx = e.clientX; arr.my = e.clientY;
  if (!arr.ativo && Math.hypot(e.clientX - arr.x, e.clientY - arr.y) > (arr.tipo === "mouse" ? 6 : 12)) {
    if (arr.tipo === "mouse") iniciarArrasto(); else return terminarArrasto(true);
  }
  if (!arr.ativo) return;
  const f = ghost();
  if (f) { f.style.left = e.clientX + "px"; f.style.top = e.clientY + "px"; }
  const alvo = alvoSob(e.clientX, e.clientY);
  if (alvo !== arr.alvo) {
    host.querySelectorAll(".soltar").forEach((x) => x.classList.remove("soltar"));
    arr.alvo = alvo;
    if (alvo && alvo !== arr.id) host.querySelectorAll(`[data-goto="${CSS.escape(alvo)}"]`).forEach((x) => x.classList.add("soltar"));
  }
  if (e.clientY < 70) scrollBy(0, -16); else if (e.clientY > innerHeight - 70) scrollBy(0, 16);
});
addEventListener("pointerup", () => terminarArrasto(false));
addEventListener("pointercancel", () => terminarArrasto(true));
addEventListener("keydown", (e) => { if (e.key === "Escape" && arr?.ativo) terminarArrasto(true); });

