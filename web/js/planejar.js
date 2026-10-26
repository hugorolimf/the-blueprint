// ---- tela Planejar: a proposta que o agente enviou pelo MCP chega aqui para você revisar e aplicar; o editor é só manual ----
const hub = $("agente-tela");
function montaHub() {
  if (!blueprintEd()) edProj = D.blueprints[0]?.id ?? null;
  const p = blueprintEd(), novo = edProj === NOVO;
  hub.querySelector(".ed-topo .tit span").textContent = novo ? "Planejar · blueprint novo" : "Planejar";
  $("ag-proj").textContent = p ? p.nome || (novo ? tr("Novo blueprint") : tr("Sem nome")) : tr("Nenhum blueprint");
  montaLateral();
  montaOutras();
  $("ag-vazio").hidden = !!p;
  $("painel-claude").hidden = !p;
  if (p) montaMcp();
}
// A tela abre já na proposta do blueprint em que o usuário está. A lista de blueprints fica escondida: só aparece, no topo, o que tem proposta esperando em OUTRO blueprint.
function montaOutras() {
  let el = $("ag-outras");
  if (!el) { el = document.createElement("span"); el.id = "ag-outras"; hub.querySelector(".ed-topo .tit").after(el); el.onclick = (e) => { const b = e.target.closest("[data-ep]"); if (b) { edProj = b.dataset.ep; montaHub(); } }; }
  const pref = (workspaceAtual?.id || "sm") + ":", aqui = contextoChat(edProj);
  const ids = (arquivo.chats || []).filter((c) => c.contexto?.startsWith(pref) && c.contexto !== aqui && (c.proposta?.length || c.fila?.length)).map((c) => c.contexto.slice(pref.length));
  el.replaceChildren(...ids.map((id) => { const b = document.createElement("button"); b.className = "btn mini"; b.type = "button"; b.dataset.ep = id; b.textContent = id === NOVO ? tr("Proposta de blueprint novo") : tr("Proposta em «{nome}»", { nome: D.blueprints.find((x) => x.id === id)?.nome || tr("outro blueprint") }); return b; }));
}
function agenteTela(abre, op = {}) {
  if (abre === !hub.hidden) { if (abre) montaHub(); return; }
  if (abre) {
    if (!ed.hidden) editar(false, { semAnim: true });
    if (op.novo) { rascunho = { id: NOVO, nome: "", descricao: "", obs: "", links: [], acoes: [], emblema: null }; edProj = NOVO; }
    else edProj = op.proj ?? tela ?? D.blueprints.find((x) => x.id === edProj)?.id ?? D.blueprints[0]?.id ?? null;
    hub.classList.add("fa-planejar");
    hub.hidden = false;
    document.body.classList.add("editando");
    document.documentElement.classList.add("editando");
    $("folha").inert = true;
    dizer("pronto");
    montaHub();
    folhaAbre(hub); // abre como o aviso do agente: o papel se revela de cima para baixo e a marca em construção monta atrás
  } else {
    if (edProj === NOVO) edProj = null;
    hub.getAnimations().forEach((a) => a.cancel());
    document.body.classList.remove("editando");
    document.documentElement.classList.remove("editando");
    $("folha").inert = false;
    $("b-ia").focus({ preventScroll: true });
    carregar();
    const fim = () => { hub.hidden = true; }; // a folha de baixo continua desenhada: fechar o Planejar não a anima de novo
    if (op.semAnim) { folhaFundo(false); hub.hidden = true; } else folhaFecha(hub, fim);
  }
}
const fechaTelas = () => { if (!hub.hidden) agenteTela(false, { semAnim: true }); if (!ed.hidden) editar(false, { semAnim: true }); };
$("b-ia").onclick = () => agenteTela(true, { proj: tela });
$("ag-fechar").onclick = () => agenteTela(false);
$("ag-blueprints").addEventListener("click", (e) => { const b = e.target.closest("[data-ep]"); if (b) { edProj = b.dataset.ep; montaHub(); } });
$("ag-novo").onclick = () => { rascunho = { id: NOVO, nome: "", descricao: "", obs: "", links: [], acoes: [], emblema: null }; edProj = NOVO; montaHub(); };

// ---- montagem ----------------------------------------------------------------------------------
const chipBlueprint = (p) => {
  const r = resumo(acoesDe(p));
  return projConcluido(p) ? tr("✓ concluído") : !p.acoes.length ? tr("sem ações") : r.futuro ? tr("planejado") : Math.round(r.geral * 100) + "%";
};
function montaLateral() {
  $("ed-blueprints").innerHTML = $("ag-blueprints").innerHTML = D.blueprints.map((p, pi) => `<button class="ed-proj" type="button" data-ep="${esc(p.id)}" aria-current="${p.id === edProj}"><span class="n">${pad(pi + 1)}</span><span class="nm">${esc(p.nome || tr("Sem nome"))}</span><span class="chip">${esc(chipBlueprint(p))}</span></button>`).join("") || `<p class="vazio">${tr("Nenhum blueprint ainda.")}</p>`;
}
function montaEditor() {
  if (!blueprintEd()) edProj = D.blueprints[0]?.id ?? null;
  const p = blueprintEd(), novo = edProj === NOVO;
  ed.classList.toggle("modo-novo", novo);
  $("nv").hidden = !novo;
  $("painel-novo-manual").hidden = !novo;
  ed.querySelector(".ed-topo .tit span").textContent = novo ? tr("Novo blueprint") : tr("Editar blueprint");
  montaLateral();
  $("ed-proj").textContent = p ? p.nome || (novo ? tr("Novo blueprint") : tr("Sem nome")) : tr("Nenhum blueprint");
  $("ed-vazio").hidden = !!p;
  $("painel-manual").hidden = !p || novo;
  ed.querySelectorAll("[data-g]").forEach((el) => { if (el.type === "checkbox") el.checked = !!D[el.dataset.g]; else el.value = D[el.dataset.g] ?? ""; });
  if (p && !novo) montaManual();
}

