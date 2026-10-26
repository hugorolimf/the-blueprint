// ---- aviso do agente: uma folha de blueprint que se abre no meio da tela (folha.js) com a mensagem inteira do agente; regras em notificacao-regras.js ----
// Zero trabalho em repouso: sem aviso na tela não existe timer nem animação. Todo texto do agente entra por textContent, nunca como HTML.
// O aviso que pede atenção (travou, não deu) fica até ser fechado; os outros fecham sozinhos, e com o mouse ou o foco dentro a folha espera.
const NOTIF_FECHA_MS = 16000;
let notifFila = [], notifAtual = null, notifPerdidas = 0, notifTimer = 0, notifEl = null;
const notifPara = (fn, ms) => { clearTimeout(notifTimer); notifTimer = setTimeout(fn, ms); };
function notifCria() {
  const el = document.createElement("aside");
  el.id = "notificacao"; el.hidden = true;
  el.setAttribute("role", "status"); el.setAttribute("aria-live", "polite");
  el.innerHTML = `<header><span class="fa-rot"></span><h3></h3></header><div class="fa-msg" tabindex="0"></div>
<dl class="fa-carimbo"></dl><div class="fa-acoes"><button class="btn mini notif-ir" type="button">${tr("Ir até o blueprint")}</button><button class="btn mini notif-prox" type="button"></button></div>`;
  document.body.appendChild(el);
  el.querySelector(".notif-prox").onclick = notifFecha;
  el.querySelector(".notif-ir").onclick = () => { const a = notifAtual; notifFecha(); if (a) notifIr(a); };
  el.addEventListener("pointerenter", () => clearTimeout(notifTimer));
  el.addEventListener("focusin", () => clearTimeout(notifTimer));
  el.addEventListener("pointerleave", notifRearma);
  el.addEventListener("focusout", (e) => { if (!el.contains(e.relatedTarget)) notifRearma(); });
  addEventListener("keydown", (e) => { if (e.key === "Escape" && notifAtual && !el.hidden) { e.stopPropagation(); notifFecha(); } }, true);
  return el;
}
function notifIr(a) {
  if (a.abre === "plano") { // abre já o plano: a tela Planejar na proposta deste blueprint (ou do blueprint novo), mesmo se o Planejar já estiver aberto em outro
    const id = a.blueprintId || NOVO;
    if (!hub.hidden) { edProj = id; montaHub(); } else agenteTela(true, a.blueprintId ? { proj: id } : { novo: true });
    return;
  }
  if (!D.blueprints.some((x) => x.id === a.blueprintId)) return;
  abertos[a.blueprintId] = true; // abre as ações para o robô e o plano aparecerem
  roboRolar = true;
  if (tela === a.blueprintId) render(); else irBlueprint(a.blueprintId);
}
function notifRearma() { if (notifAtual && notifAtual.prioridade < 3) notifPara(notifFecha, NOTIF_FECHA_MS); }
function notifFecha() {
  clearTimeout(notifTimer);
  if (!notifAtual) return;
  notifAtual = null;
  const segue = notifFila.length > 0;
  folhaFecha(notifEl, () => { if (notifFila.length) notifMostra(); else document.body.classList.remove("aviso-aberto"); }, { fundo: !segue && hub.hidden }); // com outro aviso na fila, ou o Planejar aberto, a cortina fica
}
function notifMostra() {
  const n = notifFila.shift();
  if (!n) return;
  notifAtual = n;
  notifEl ||= notifCria();
  const q = (s) => notifEl.querySelector(s);
  notifEl.className = `folha-aviso${n.nivel ? " " + n.nivel : ""}`;
  q(".fa-rot").textContent = n.nivel === "erro" ? tr("Aviso do agente · atenção") : tr("Aviso do agente");
  q("h3").textContent = n.titulo;
  q(".fa-msg").textContent = n.mensagem;
  q(".fa-msg").scrollTop = 0;
  const celulas = (n.partes || []).filter(([, v]) => v);
  q(".fa-carimbo").replaceChildren(...celulas.flatMap(([k, v]) => { const dt = document.createElement("dt"), dd = document.createElement("dd"); dt.textContent = k; dd.textContent = v; dd.title = v; return [dt, dd]; }));
  q(".fa-carimbo").hidden = !celulas.length;
  const mais = notifFila.length + notifPerdidas;
  q(".notif-prox").textContent = mais ? tr("Próximo ({mais})", { mais }) : tr("Fechar");
  q(".notif-ir").textContent = n.abre === "plano" ? tr("Abrir o plano") : tr("Ir até o blueprint");
  q(".notif-ir").hidden = n.abre !== "plano" && !D.blueprints.some((x) => x.id === n.blueprintId);
  q(".notif-ir").classList.toggle("cheio", n.abre === "plano");
  notifPerdidas = 0;
  notifEl.hidden = false;
  document.body.classList.add("aviso-aberto");
  folhaAbre(notifEl);
  notifRearma();
}
// Chamada pela fila do MCP (mcp.js) para cada evento novo do workspace aberto.
function notificar(item) {
  const p = D.blueprints.find((x) => x.id === item.blueprintId), fi = p?.acoes.findIndex((x) => x.id === item.acaoId) ?? -1, f = fi >= 0 ? p.acoes[fi] : null, t = f?.planos.find((x) => x.id === item.planoId);
  // a folha numera as ações como <nº do blueprint>.<nº da ação> (2.5): é como o usuário as chama
  const acao = f ? `${numeroDaAcao(p, f)} ${f.nome || ""}`.trim() : "";
  const n = notificacaoDe(item, Date.now(), [p?.nome, acao, t?.texto].filter(Boolean).join(" › "));
  if (!n) return;
  n.partes = [[tr("Blueprint"), p?.nome], [tr("Ação"), acao], [tr("Plano"), t?.texto]];
  notifPerdidas += notificacaoEmpilha(notifFila, n);
  if (!notifAtual) notifMostra();
  else if (notifEl) {
    const mais = notifFila.length + notifPerdidas;
    notifEl.querySelector(".notif-prox").textContent = tr("Próximo ({mais})", { mais });
  }
}
