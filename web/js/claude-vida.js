// ---- o Claude respirando, piscando, digitando…: os quadros de cada pose (agentes-claude.mjs: QUADROS, SEQUENCIA, RITMO) alternam num relógio POR FIGURA ----
// Cada quadro já está desenhado (cache), empilhado no mesmo lugar (claude.css); o relógio só troca a classe `on` (visibility, sem refazer o layout) de um <span> para o outro. Zero trabalho fora da hora: só roda com a figura à vista (um único
// IntersectionObserver), a aba aberta e sem prefers-reduced-motion; para ao andar, ao trocar de pose e ao remover a figura. Quem usa: claude-camada.js.
const claudeDoEl = new WeakMap(); // elemento da figura -> registro (r)
const claudeObs = typeof IntersectionObserver === "function" ? new IntersectionObserver((es) => es.forEach((e) => {
  const r = claudeDoEl.get(e.target);
  if (!r) return;
  r.visivel = e.isIntersecting;
  r.visivel ? claudeVidaLiga(r) : claudeVidaPara(r);
})) : null;
const claudeVigia = (r) => { claudeDoEl.set(r.el, r); claudeObs?.observe(r.el); };
const claudeSoltaVigia = (r) => { claudeObs?.unobserve(r.el); claudeDoEl.delete(r.el); };

// o HTML da pose parada: com mais de um quadro (e movimento permitido) um <span> por quadro, só o primeiro à vista; senão o SVG do quadro 0
function claudeHtmlPose(papel, pose) {
  const n = claudeMod.QUADROS[pose] || 1;
  if (n < 2 || reduz()) return claudeSvg(papel, pose, 0);
  return claudeQuadrosUnicos(claudeMod.SEQUENCIA[pose]).map((q) => `<span data-q="${q}"${q ? "" : ' class="on"'}>${claudeSvg(papel, pose, q)}</span>`).join("");
}
function claudeVidaPara(r) { clearInterval(r.vida); r.vida = 0; }
function claudeVidaLiga(r) {
  claudeVidaPara(r);
  const pose = r.dados?.f.pose, spans = r.el.querySelectorAll("span[data-q]");
  if (!pose || spans.length < 2 || r.anim || reduz() || (claudeObs && !r.visivel)) return; // sem IntersectionObserver (shortcut) a figura conta como visível
  const seq = claudeMod.SEQUENCIA[pose];
  let i = 0;
  r.vida = setInterval(() => { const q = String(seq[++i % seq.length]); spans.forEach((s) => s.classList.toggle("on", s.dataset.q === q)); }, claudeMod.RITMO[pose]);
}
// a aba voltou a ficar visível ou a preferência de movimento mudou: refaz o HTML de cada figura parada (com ou sem quadros) e religa o relógio
function claudeVidaRefaz() {
  claudeFiguras.forEach((r) => { if (r.dados && !r.anim) { r.chave = ""; claudeRotula(r); } });
}
document.addEventListener("visibilitychange", () => (document.hidden ? claudeFiguras.forEach(claudeVidaPara) : claudeVidaRefaz()));
matchMedia("(prefers-reduced-motion: reduce)").addEventListener?.("change", claudeVidaRefaz);
