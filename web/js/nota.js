// ---- barra lateral e abas -------------------------------------------------------------------------
$("ed-blueprints").addEventListener("click", (e) => {
  const b = e.target.closest("[data-ep]");
  if (b) { edProj = b.dataset.ep; edSel = "blueprint"; montaEditor(); }
});
ed.addEventListener("input", (e) => {
  const g = e.target.closest("[data-g]");
  if (g) {
    D[g.dataset.g] = g.type === "checkbox" ? g.checked : g.value;
    if (g.dataset.g === "empresa" && workspaceAtual && g.value.trim()) D.nome = g.value.trim(); // num workspace, empresa e nome são a mesma coisa para quem usa
    return mudou();
  }
  const pr = e.target.closest("[data-pref]");
  if (pr) { try { localStorage.setItem("blueprint." + pr.dataset.pref, pr.checked ? "1" : "0"); } catch {} }
});

// ---- folha pequena sobre o blueprint: lê a observação (ou explica a marca) sem sair da tela -------
const notaEl = $("nota-folha");
// fechar a folha pequena é o papel se fechando: o rolo sobe e o papel se recolhe de baixo para cima (rapido = sem animação)
let notaFechando = 0;
function fechaNota(rapido) {
  if (notaEl.hidden || notaEl.dataset.fechando) return;
  const id = ++notaFechando;
  if (rapido === true || reduz()) { notaEl.hidden = true; notaEl._acoes = []; return; }
  notaEl.dataset.fechando = "1";
  const h = notaEl.getBoundingClientRect().height, dur = 520, ease = "cubic-bezier(.5,0,.75,0)";
  const rolo = document.createElement("div");
  rolo.className = "rolo";
  notaEl.appendChild(rolo);
  notaEl.animate([{ clipPath: "inset(0 0 0% 0)" }, { clipPath: "inset(0 0 100% 0)" }], { duration: dur, easing: ease, fill: "forwards" });
  rolo.animate([{ transform: "translateY(" + Math.max(0, h - 30) + "px)" }, { transform: "translateY(0)" }], { duration: dur, easing: ease, fill: "forwards" }).finished.then(() => {
    if (id !== notaFechando) return; // abriu outra folha no meio do fechamento
    notaEl.hidden = true;
    notaEl._acoes = [];
    delete notaEl.dataset.fechando;
    notaEl.getAnimations().forEach((a) => a.cancel());
    rolo.remove();
  }, () => {});
}
function abreNota({ rotulo, titulo, texto, barra, acoes = [], editar }, ancora) {
  notaFechando++; // cancela um fechamento em andamento
  delete notaEl.dataset.fechando;
  notaEl.getAnimations().forEach((a) => a.cancel());
  const r = ancora.getBoundingClientRect();
  notaEl._sujo = false;
  if (editar) acoes = [{ rotulo: "Salvar", fn: () => { editar.salvar(notaEl.querySelector("textarea").value); notaEl._sujo = false; fechaNota(); } }, ...acoes];
  notaEl.innerHTML = `<h4>${esc(rotulo)}</h4><h3>${esc(titulo)}</h3>${editar ? `<textarea aria-label="${esc(rotulo)}" placeholder="${esc(editar.dica)}">${esc(texto)}</textarea><p class="dica-ed">Você edita aqui mesmo. Ctrl+Enter salva; clicar fora não descarta o que você escreveu.</p>` : `<div class="corpo${texto ? "" : " vazio"}">${esc(texto || "Sem texto ainda.")}</div>`}${barra !== undefined ? `<div class="med" style="margin-top:14px"><i style="width:${barra}%"></i></div>` : ""}<div class="acoes">${acoes.map((a, i) => `<button class="btn mini cheio" type="button" data-nacao="${i}">${esc(a.rotulo)}</button>`).join("")}<button class="btn mini" type="button" data-nfecha>Fechar</button></div>`;
  notaEl._acoes = acoes;
  notaEl.hidden = false;
  const larg = notaEl.offsetWidth, alt = notaEl.offsetHeight, janela = document.documentElement.clientWidth;
  let esq = r.right + scrollX + 14;
  if (esq + larg > scrollX + janela - 16) esq = Math.max(scrollX + 16, r.left + scrollX - larg - 14);
  const topo = Math.max(scrollY + 12, Math.min(r.top + scrollY - 10, scrollY + innerHeight - alt - 16));
  notaEl.style.left = esq + "px";
  notaEl.style.top = topo + "px";
  desenrolar(notaEl, notaEl); // a folha pequena também se abre como papel, já escrita
  (notaEl.querySelector("textarea") || notaEl.querySelector("[data-nfecha]")).focus({ preventScroll: true });
}
function abreNotaObs(el) {
  const [pid, fid] = el.dataset.obs.split("|");
  const p = D.blueprints.find((x) => x.id === pid), f = fid && p?.acoes.find((x) => x.id === fid);
  if (!p) return;
  const pi = D.blueprints.indexOf(p) + 1;
  abreNota({
    rotulo: f ? "Instruções da ação" : "Observação do blueprint", titulo: f ? `${pi}.${p.acoes.indexOf(f) + 1} · ${f.nome || "sem nome"}` : p.nome || "Sem nome", texto: (f ? f.obs : p.obs) || "",
    editar: { dica: f ? "O que fazer nesta ação, passo a passo: o roteiro, as decisões, as dependências e os riscos…" : "Anotações do blueprint…", salvar: (v) => { const novo = v.trim() ? v : ""; if (novo === ((f ? f.obs : p.obs) || "")) return; if (f) f.obs = novo; else p.obs = novo; mudou(); } },
    acoes: [],
  }, el);
}
function abreNotaMarca(el) {
  const d = el.dataset, pct = Number(d.pct), p = d.proj ? D.blueprints.find((x) => x.id === d.proj) : null;
  abreNota({
    rotulo: p ? "Marca do blueprint" : "Marca da empresa", titulo: d.tit, barra: pct,
    texto: `${d.feitas} de ${d.total} peças construídas (${pct}%).\n\n${p ? "A marca se completa conforme os planos e os checkpoints do blueprint são concluídos: o que ainda falta aparece só no traço." : "A marca da empresa se completa conforme os blueprints avançam (média de todos os blueprints)."}`,
    acoes: [],
  }, el);
}
notaEl.addEventListener("input", () => { notaEl._sujo = true; });
notaEl.addEventListener("keydown", (e) => { if (e.key === "Enter" && (e.ctrlKey || e.metaKey) && e.target.matches("textarea")) { e.preventDefault(); notaEl.querySelector("[data-nacao]")?.click(); } });
notaEl.addEventListener("click", (e) => {
  if (e.target.closest("[data-nfecha]")) return fechaNota();
  const b = e.target.closest("[data-nacao]");
  if (b) notaEl._acoes[+b.dataset.nacao]?.fn();
});
document.addEventListener("pointerdown", (e) => { if (!notaEl.hidden && !notaEl._sujo && !notaEl.contains(e.target) && !e.target.closest(".nota, .marca")) fechaNota(); });
addEventListener("keydown", (e) => { if (e.key === "Escape" && !notaEl.hidden && menuContexto.hidden) { e.stopImmediatePropagation(); fechaNota(); } }, true);
$("busca").addEventListener("input", fechaNota);

pm.addEventListener("input", (e) => {
  const sl = e.target.closest("[data-prev-marca]"), p = blueprintEd();
  if (sl && p?.emblema) sl.closest(".card").querySelector(".marca-previa").innerHTML = previaMarca(p.emblema.pecas, 330, 230, sl.value / 100);
});

// a marca fica no centro da janela, não da folha: acompanha a rolagem (sem sair da folha)
function posicionaMarca() {
  const m = host.querySelector(".emblema.marca"), s = m?.ownerSVGElement;
  if (!m || !s) return;
  const r = s.getBoundingClientRect(), H = s.height.baseVal.value, W = s.width.baseVal.value;
  const larg = +m.getAttribute("width"), lado = +m.getAttribute("height");
  const cx = Math.min(W - larg / 2, Math.max(larg / 2, document.documentElement.clientWidth / 2 - r.left));
  const cy = Math.min(H - lado / 2, Math.max(lado / 2, innerHeight / 2 - r.top));
  m.setAttribute("x", cx - larg / 2);
  m.setAttribute("y", cy - lado / 2);
}
let quadroMarca = 0;
addEventListener("scroll", () => { if (!quadroMarca) quadroMarca = requestAnimationFrame(() => { quadroMarca = 0; posicionaMarca(); }); }, { passive: true });
addEventListener("resize", posicionaMarca);

