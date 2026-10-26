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
  const visto = notaEl._aoFechar; // quem abriu a folha pode querer saber que ela foi lida e fechada (ex.: o concluído do agente)
  notaEl._aoFechar = null;
  visto?.();
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
// a folha da instrução abre lida; "Editar" mostra o campo, já na altura do texto (até 70% da janela), e "Visualizar" volta ao texto lido com o que foi digitado
function notaModo(modo) {
  const ta = notaEl.querySelector("textarea"), ed = notaEl.querySelector(".obs-edicao"), ver = notaEl.querySelector("[data-nver]");
  if (!ta || !ver) return;
  if (modo === "ver") ver.innerHTML = mdHtml(ta.value) || `<p class="md-vazio">${tr("Sem texto ainda.")}</p>`;
  notaEl.dataset.modo = modo;
  ver.hidden = modo === "editar"; ed.hidden = modo !== "editar";
  notaEl.querySelector(".dica-ed").hidden = modo !== "editar";
  if (modo === "editar") { ta.style.height = "auto"; ta.style.height = Math.min(ta.scrollHeight + 2, innerHeight * 0.7) + "px"; mdPrevia(ta); ta.focus({ preventScroll: true }); }
}
// abrir é o fechar ao contrário: o papel se revela de cima para baixo e o rolo desce na frente (movimento reduzido: aparece pronta)
function revelaNota() {
  if (reduz()) return;
  const h = notaEl.getBoundingClientRect().height, dur = 560, ease = "cubic-bezier(.25,.7,.2,1)";
  const rolo = document.createElement("div");
  rolo.className = "rolo";
  notaEl.appendChild(rolo);
  notaEl.animate([{ clipPath: "inset(0 0 100% 0)" }, { clipPath: "inset(0 0 0% 0)" }], { duration: dur, easing: ease });
  rolo.animate([{ transform: "translateY(0)" }, { transform: "translateY(" + Math.max(0, h - 30) + "px)" }], { duration: dur, easing: ease, fill: "forwards" }).finished.then(() => rolo.remove(), () => rolo.remove());
}
function abreNota({ rotulo, titulo, texto, barra, acoes = [], editar, aoFechar }, ancora) {
  const chave = rotulo + "|" + titulo;
  if (!notaEl.hidden && !notaEl.dataset.fechando && notaEl._chave === chave) return; // já aberta com este conteúdo: clicar de novo não faz nada (e não perde o que estiver sendo editado)
  const substituida = notaEl._aoFechar; // outra folha tomou o lugar desta: a anterior também foi vista
  notaEl._chave = chave;
  notaEl._aoFechar = aoFechar || null;
  notaFechando++; // cancela um fechamento em andamento
  delete notaEl.dataset.fechando;
  notaEl.getAnimations().forEach((a) => a.cancel());
  const r = ancora.getBoundingClientRect();
  notaEl._sujo = false;
  notaEl.dataset.modo = editar ? "ver" : "";
  if (editar) acoes = [{ rotulo: "Editar", modo: "ver", fn: () => notaModo("editar") }, { rotulo: "Salvar", modo: "editar", fn: () => { editar.salvar(notaEl.querySelector("textarea").value); notaEl._sujo = false; fechaNota(); } }, ...acoes];
  notaEl.innerHTML = `<h4>${esc(rotulo)}</h4><h3>${esc(titulo)}</h3>${editar ? `<div class="corpo md" data-nver>${mdHtml(texto) || `<p class="md-vazio">${tr("Sem texto ainda.")}</p>`}</div>${mdEdicao(`aria-label="${esc(rotulo)}"`, texto, { rows: 10, placeholder: editar.dica, oculto: true })}<p class="dica-ed" hidden>${tr("Você edita aqui mesmo. Ctrl+Enter salva; clicar fora não descarta o que você escreveu.")}</p>` : `<div class="corpo${texto ? "" : " vazio"}">${esc(texto || tr("Sem texto ainda."))}</div>`}${barra !== undefined ? `<div class="med" style="margin-top:14px"><i style="width:${barra}%"></i></div>` : ""}<div class="acoes">${acoes.map((a, i) => `<button class="btn mini cheio" type="button" data-nacao="${i}"${a.modo ? ` data-nmodo="${a.modo}"` : ""}>${esc(a.rotulo)}</button>`).join("")}<button class="btn mini" type="button" data-nfecha>${tr("Fechar")}</button></div>`;
  notaEl._acoes = acoes;
  notaEl.hidden = false;
  const larg = notaEl.offsetWidth, alt = notaEl.offsetHeight, janela = document.documentElement.clientWidth;
  let esq = r.right + scrollX + 14;
  if (esq + larg > scrollX + janela - 16) esq = Math.max(scrollX + 16, r.left + scrollX - larg - 14);
  const topo = Math.max(scrollY + 12, Math.min(r.top + scrollY - 10, scrollY + innerHeight - alt - 16));
  notaEl.style.left = esq + "px";
  notaEl.style.top = topo + "px";
  revelaNota(); // a folha pequena abre como o papel se desenrolando, do mesmo jeito que fecha
  notaEl.querySelector("[data-nacao]:not([data-nmodo=editar])")?.focus({ preventScroll: true }) ?? notaEl.querySelector("[data-nfecha]").focus({ preventScroll: true });
  substituida?.();
}
function abreNotaObs(el) { const [pid, fid] = el.dataset.obs.split("|"); abreFolhaDaAcao(pid, fid, el); } // a folha da ação/do blueprint: iniciar-acao.js
function abreNotaMarca(el) {
  const d = el.dataset, pct = Number(d.pct), p = d.proj ? D.blueprints.find((x) => x.id === d.proj) : null;
  abreNota({
    rotulo: p ? tr("Marca do blueprint") : tr("Marca da empresa"), titulo: d.tit, barra: pct,
    texto: `${tr("{feitas} de {total} peças construídas ({pct}%).", { feitas: d.feitas, total: d.total, pct })}\n\n${p ? tr("A marca se completa conforme os planos e os checkpoints do blueprint são concluídos: o que ainda falta aparece só no traço.") : tr("A marca da empresa se completa conforme os blueprints avançam (média de todos os blueprints).")}`,
    acoes: [],
  }, el);
}
notaEl.addEventListener("input", (e) => { notaEl._sujo = true; const ta = e.target.closest("textarea"); if (ta) mdPrevia(ta); });
notaEl.addEventListener("keydown", (e) => { if (e.key === "Enter" && (e.ctrlKey || e.metaKey) && e.target.matches("textarea")) { e.preventDefault(); notaEl.querySelector("[data-nmodo=editar]")?.click(); } });
notaEl.addEventListener("click", (e) => {
  if (e.target.closest("[data-nfecha]")) return fechaNota();
  const b = e.target.closest("[data-nacao]");
  if (b) notaEl._acoes[+b.dataset.nacao]?.fn();
});
document.addEventListener("pointerdown", (e) => { if (!notaEl.hidden && !notaEl._sujo && !notaEl.contains(e.target) && !e.target.closest(".nota, .marca, [data-robo-nome], dialog")) fechaNota(); }); // o diálogo de confirmação (iniciar-acao.js) não fecha a folha por baixo
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

