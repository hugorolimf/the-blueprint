function abrirEntrada() {
  if (!nomeUsuario || !desenrolarNaProxima) return;
  desenrolarNaProxima = false;
  desenrolar($("entrada"), $("entrada"));
}

// a folha se abre como um rolo: o papel, já desenhado, se revela de cima para baixo com o rolo descendo à frente
function desenrolar(alvo = host.querySelector("svg.desenho"), cont = host) {
  if (!alvo || reduz()) return;
  if (animPref() === "escrita") return escrever(alvo);
  const h = alvo.getBoundingClientRect().height, dur = Math.min(1500, 650 + h * 0.3), ease = "cubic-bezier(.45,.05,.2,1)";
  const fim = Math.max(0, h - 30);
  alvo.getAnimations().forEach((a) => a.cancel());
  alvo.animate([{ clipPath: "inset(0 0 100% 0)" }, { clipPath: "inset(0 0 0% 0)" }], { duration: dur, easing: ease });
  const rolo = document.createElement("div");
  rolo.className = "rolo";
  cont.appendChild(rolo);
  rolo.animate([{ transform: "translateY(0)", opacity: 1 }, { transform: `translateY(${fim}px)`, opacity: 1, offset: 0.97 }, { transform: `translateY(${fim}px)`, opacity: 0 }], { duration: dur, easing: ease }).finished.then(() => rolo.remove(), () => rolo.remove());
}
const animPref = () => { try { return localStorage.getItem("blueprint.anim") === "escrita" ? "escrita" : "rolo"; } catch { return "rolo"; } };
$("b-anim").value = animPref();
$("b-anim").onchange = () => { try { localStorage.setItem("blueprint.anim", $("b-anim").value); } catch {} desenrolar(); };
// folha dupla: o texto está na de baixo e a água vai deixando a de cima transparente. Cada texto surge em três trechos (ponta, meio, ponta) numa ordem sorteada
function escrever(alvo) {
  const T = 700, PASSO = 55;
  alvo.querySelectorAll("text, h1, h2, h3, h4, p, label, .corpo").forEach((el) => {
    const r = el.getBoundingClientRect();
    if (!r.width || r.bottom < 0 || r.top > innerHeight) return; // o que está fora da janela não precisa esperar
    const svg = el instanceof SVGElement, original = el.textContent, u = Math.max(r.top, 0) / innerHeight * 0.7 + Math.random() * 0.3, ini = T * u * u; // u: vez do texto (de cima para baixo, com sorteio); u² junta os começos no início e espaça os últimos
    const fade = (x, delay) => x.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 200 + 450 * u, delay, easing: "cubic-bezier(.2,.6,.3,1)", fill: "backwards" }); // chega devagar ao opaco
    if (el.children.length) return fade(el, ini); // texto já com partes (tspan etc.): não dá para separar as letras, aparece inteiro
    el.textContent = "";
    const letras = [];
    for (const c of original) {
      if (!c.trim()) { el.append(c); continue; }
      const l = svg ? document.createElementNS("http://www.w3.org/2000/svg", "tspan") : document.createElement("span");
      l.textContent = c;
      el.append(l);
      letras.push(l);
    }
    const ordem = [0, 1, 2];
    for (let i = 2; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [ordem[i], ordem[j]] = [ordem[j], ordem[i]]; }
    letras.forEach((l, i) => fade(l, ini + (ordem[Math.floor((i * 3) / letras.length)] + Math.random()) * PASSO * (1 + u)));
  });
}
// voltar também abre: nada se enrola de volta, a tela de destino se abre de novo
function irHome() {
  if (workspaceAtual) { location.hash = "/c/" + encodeURIComponent(workspaceAtual.id); return; }
  if (!lerHash()) { tela = null; desenrolarNaProxima = true; render(); scrollTo(0, 0); } else location.hash = "";
}
function irBlueprint(id) {
  pedirTelaCheia();
  if (!(id in abertos)) abertos[id] = true; // ao abrir o blueprint, as ações já aparecem
  desenrolarNaProxima = true;
  if (lerHash() === id) { tela = id; render(); scrollTo(0, 0); } else location.hash = "/p/" + encodeURIComponent(id);
}
addEventListener("hashchange", () => {
  fechaTelas();
  consulta = "";
  $("busca").value = "";
  tela = lerHash();
  desenrolarNaProxima = true;
  if (tela && !(tela in abertos)) abertos[tela] = true;
  render();
  scrollTo(0, 0);
});
$("b-voltar").onclick = irHome;
$("b-workspaces").onclick = () => { location.hash = ""; };

function textoSel(sel, texto) {
  sel.textContent = corta(texto, Number(sel.dataset.max) || 160);
}
function marcar(g) {
  host.querySelectorAll(".acao[data-on]").forEach((e) => e.removeAttribute("data-on"));
  g.setAttribute("data-on", "");
  const sel = g.closest("svg.desenho").querySelector(".sel");
  textoSel(sel, "SELECIONADO: " + g.dataset.info);
  sel.style.fill = "#eaf4ff";
  selecionada = g.getAttribute("aria-label");
}
// marca ou desmarca plano ou checkpoint direto no blueprint: o progresso da ação e do blueprint se recalcula
function alternarItem(pid, fid, tipo, id) {
  const f = D.blueprints.find((x) => x.id === pid)?.acoes.find((x) => x.id === fid);
  const it = f && (tipo === "t" ? f.planos : f.checkpoints).find((x) => x.id === id);
  if (!it) return;
  it.feito = !it.feito;
  if (document.body.classList.contains("editando")) montaEditor();
  mudou();
}
// clique numa ação ou checkpoint da folha recolhida: abre as ações, leva até o item e o destaca por um instante
function irPara(pid, fid, mid) {
  abertos[pid] = true;
  render();
  const acao = host.querySelector(`.acao[data-f="${CSS.escape(fid)}"]`);
  const alvo = (mid && host.querySelector(`.chk[data-id="${CSS.escape(mid)}"]`)) || acao;
  if (!alvo) return;
  if (acao) marcar(acao);
  const r = alvo.getBoundingClientRect();
  scrollTo({ top: Math.max(0, scrollY + r.top - innerHeight * (mid ? 0.4 : 0.22)), behavior: reduz() ? "auto" : "smooth" });
  const bb = alvo.getBBox(), pad = 8, ret = document.createElementNS("http://www.w3.org/2000/svg", "rect");
  ret.setAttribute("class", "pisca");
  ret.setAttribute("x", bb.x - pad); ret.setAttribute("y", bb.y - pad);
  ret.setAttribute("width", bb.width + 2 * pad); ret.setAttribute("height", bb.height + 2 * pad);
  alvo.ownerSVGElement.appendChild(ret);
  const fim = () => ret.remove();
  ret.animate([{ opacity: 0 }, { opacity: 1, offset: .12 }, { opacity: .25, offset: .45 }, { opacity: 1, offset: .62 }, { opacity: 0 }], { duration: 2200 }).onfinish = fim;
  setTimeout(fim, 2600);
}
function alternar(id) {
  abertos[id] = abertos[id] !== true;
  render();
}
$("b-abrir").onclick = () => { if (tela) { abertos[tela] = true; render(); } };
$("b-fechar").onclick = () => { if (tela) { abertos[tela] = false; render(); } };

host.addEventListener("click", (e) => { const mes = e.target.closest("[data-mes]"); if (mes) selecionarMes(mes.dataset.mes); });
host.addEventListener("keydown", (e) => { const mes = e.target.closest("[data-mes]"); if (mes && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); selecionarMes(mes.dataset.mes); } });
host.addEventListener("pointerover", (e) => {
  const g = e.target.closest(".acao");
  const m = e.target.closest(".chk, .nota, .marca, .checkpoint, .ag");
  if (g) marcar(g);
  const sel = (g || m)?.closest("svg.desenho")?.querySelector(".sel");
  if (m && sel) { textoSel(sel, m.dataset.info); sel.style.fill = "#eaf4ff"; }
});
host.addEventListener("focusin", (e) => {
  const g = e.target.closest(".acao");
  if (g) marcar(g);
  const n = e.target.closest(".nota, .chk, .ag");
  if (n) textoSel(n.closest("svg.desenho").querySelector(".sel"), n.dataset.info);
});
let semClique = false; // o clique que termina um arrasto não deve abrir nada
host.addEventListener("click", (e) => {
  if (semClique) { semClique = false; return; }
  const ck = e.target.closest(".chk");
  if (ck) { alternarItem(ck.dataset.p, ck.dataset.f, ck.dataset.tipo, ck.dataset.id); return; }
  if (e.target.closest("[data-adicionar-blueprint]")) { $("b-novo-proj").click(); return; }
  const ir = e.target.closest(".acao[data-ir]");
  if (ir) { irPara(ir.dataset.p, ir.dataset.f, e.target.closest(".checkpoint")?.dataset.m); return; }
  const pr = e.target.closest(".nota[data-proto]");
  if (pr) { const [pid, fid] = pr.dataset.proto.split("|"); abreProtos(pid, fid); return; }
  const no = e.target.closest(".nota"), t = e.target.closest("[data-toggle]"), m = e.target.closest("[data-goto]");
  if (no) abreNotaObs(no);
  else if (e.target.closest(".marca")) return; // a marca é só desenho: o clique nela não abre nada (nem o blueprint por trás)
  else if (t) alternar(t.dataset.toggle);
  else if (m) irBlueprint(m.dataset.goto);
});
host.addEventListener("keydown", (e) => {
  if (e.key !== "Enter" && e.key !== " ") return;
  const a = e.target.closest("[data-toggle],[data-goto],[data-adicionar-blueprint],.nota,.chk,.ag,.acao[data-ir]");
  if (!a) return;
  e.preventDefault();
  a.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  // o redesenho troca o elemento: devolve o foco ao equivalente
  const chave = a.dataset.chk ? `[data-chk="${CSS.escape(a.dataset.chk)}"]` : a.dataset.toggle ? `[data-toggle="${CSS.escape(a.dataset.toggle)}"]` : a.dataset.f ? `.acao[data-f="${CSS.escape(a.dataset.f)}"]` : null;
  if (chave) host.querySelector(chave)?.focus({ preventScroll: true });
});

// ---- busca -------------------------------------------------------------------------------
$("busca").addEventListener("input", (e) => { consulta = e.target.value; render(); });
$("busca").addEventListener("keydown", (e) => { if (e.key === "Escape" && consulta) { e.stopPropagation(); consulta = ""; e.target.value = ""; render(); } });

