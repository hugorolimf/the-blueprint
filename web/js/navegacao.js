function abrirEntrada() {
  if (!nomeUsuario || !desenrolarNaProxima) return;
  desenrolarNaProxima = false;
  desenrolar($("entrada"), $("entrada"));
}

// a abertura da folha é sempre a "escrita" (a de rolo e o seletor saíram); `cont` só existe porque os chamadores ainda o passam
function desenrolar(alvo = host.querySelector("svg.desenho"), cont = host) {
  if (!alvo || reduz()) return;
  escrever(alvo);
  desenharBarras(alvo);
}
// folha dupla: o texto está na de baixo e a água vai deixando a de cima transparente. Cada texto surge em três trechos (ponta, meio, ponta) numa ordem sorteada.
// Leve: primeiro lê todas as posições, depois escreve (sem forçar o layout a cada texto); são 3 animações por texto em vez de uma por letra; e há teto de textos.
const MAX_TEXTOS = 400;
function escrever(alvo) {
  const T = 700, PASSO = 55, lidos = [];
  for (const el of alvo.querySelectorAll("text, h1, h2, h3, h4, p, label, .corpo")) {
    const r = el.getBoundingClientRect();
    if (!r.width || r.bottom < 0 || r.top > innerHeight) continue; // o que está fora da janela não precisa esperar
    lidos.push([el, Math.max(r.top, 0)]);
    if (lidos.length >= MAX_TEXTOS) break; // o excedente aparece sem animação
  }
  for (const [el, topo] of lidos) {
    const svg = el instanceof SVGElement, original = el.textContent, u = topo / innerHeight * 0.7 + Math.random() * 0.3, ini = T * u * u; // u: vez do texto (de cima para baixo, com sorteio); u² junta os começos no início e espaça os últimos
    const quadros = svg ? [{ fillOpacity: 0, strokeOpacity: 0 }, { fillOpacity: 1, strokeOpacity: 1 }] : [{ opacity: 0 }, { opacity: 1 }]; // texto SVG com contorno: fill/stroke-opacity evitam uma camada de composição por texto
    const fade = (x, delay) => x.animate(quadros, { duration: 200 + 450 * u, delay, easing: "cubic-bezier(.2,.6,.3,1)", fill: "backwards" }); // chega devagar ao opaco
    const n = original.length;
    if (el.children.length || n < 6) { fade(el, ini); continue; } // texto já com partes (tspan etc.) ou curto demais para dividir: aparece inteiro
    const ordem = [0, 1, 2];
    for (let i = 2; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [ordem[i], ordem[j]] = [ordem[j], ordem[i]]; }
    const cortes = [0, Math.round(n / 3), Math.round((2 * n) / 3), n];
    el.textContent = "";
    for (let k = 0; k < 3; k++) {
      const parte = svg ? document.createElementNS("http://www.w3.org/2000/svg", "tspan") : document.createElement("span");
      parte.textContent = original.slice(cortes[k], cortes[k + 1]);
      el.append(parte);
      fade(parte, ini + (ordem[k] + Math.random()) * PASSO * (1 + u));
    }
  }
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
  mudarComSaida(pid, () => {
    it.feito = !it.feito;
    if (tipo !== "t" && it.feito) avisarMonitor("checkpoint-marcado", pid, id);
    if (document.body.classList.contains("editando")) montaEditor();
  });
}
// clique numa ação ou checkpoint da folha recolhida: abre as ações, leva até o item e o destaca por um instante
function irPara(pid, fid, mid) {
  abertos[pid] = true;
  const p = D.blueprints.find((x) => x.id === pid);
  if (p) abrirProjetoDaAcao(p, fid); // com projetos, a seção da ação também abre (senão a ação continua compacta); sem projetos não faz nada
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
  desenharBarras();
}
// "Expandir tudo" / "Recolher tudo" (botões do topo e menu do botão direito): as ações do blueprint e, com projetos, todas as seções
function abrirOuFecharTudo(id, aberto) {
  abertos[id] = aberto;
  abrirTodosProjetos(D.blueprints.find((p) => p.id === id), aberto);
  render();
  desenharBarras();
}
$("b-abrir").onclick = () => { if (tela) abrirOuFecharTudo(tela, true); };
$("b-fechar").onclick = () => { if (tela) abrirOuFecharTudo(tela, false); };

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
  if (e.target.closest("[data-adicionar-blueprint]")) { editar(true, { novo: true }); return; }
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

