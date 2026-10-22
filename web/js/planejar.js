// ---- tela Planejar: a proposta que o agente enviou pelo MCP chega aqui para você revisar e aplicar; o editor é só manual ----
const hub = $("agente-tela");
function montaHub() {
  if (!blueprintEd()) edProj = D.blueprints[0]?.id ?? null;
  const p = blueprintEd(), novo = edProj === NOVO;
  hub.querySelector(".ed-topo .tit span").textContent = novo ? "Planejar · blueprint novo" : "Planejar";
  $("ag-proj").textContent = p ? p.nome || (novo ? "Novo blueprint" : "Sem nome") : "Nenhum blueprint";
  montaLateral();
  $("ag-vazio").hidden = !!p;
  $("painel-claude").hidden = !p;
  if (p) montaMcp();
}
function agenteTela(abre, op = {}) {
  if (abre === !hub.hidden) { if (abre) montaHub(); return; }
  if (abre) {
    if (!ed.hidden) editar(false, { semAnim: true });
    if (op.novo) { rascunho = { id: NOVO, nome: "", descricao: "", obs: "", links: [], acoes: [], emblema: null }; edProj = NOVO; }
    else edProj = op.proj ?? tela ?? D.blueprints.find((x) => x.id === edProj)?.id ?? D.blueprints[0]?.id ?? null;
    hub.hidden = false;
    document.body.classList.add("editando");
    document.documentElement.classList.add("editando");
    $("folha").inert = true;
    dizer("pronto");
    montaHub();
    desenrolar(hub, hub);
  } else {
    hub.hidden = true;
    if (edProj === NOVO) edProj = null;
    hub.getAnimations().forEach((a) => a.cancel());
    document.body.classList.remove("editando");
    document.documentElement.classList.remove("editando");
    $("folha").inert = false;
    $("b-ia").focus({ preventScroll: true });
    carregar();
    if (!op.semAnim) desenrolar();
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
  return projConcluido(p) ? "✓ concluído" : !p.acoes.length ? "sem ações" : r.futuro ? "planejado" : Math.round(r.geral * 100) + "%";
};
function montaLateral() {
  $("ed-blueprints").innerHTML = $("ag-blueprints").innerHTML = D.blueprints.map((p, pi) => `<button class="ed-proj" type="button" data-ep="${esc(p.id)}" aria-current="${p.id === edProj}"><span class="n">${pad(pi + 1)}</span><span class="nm">${esc(p.nome || "Sem nome")}</span><span class="chip">${esc(chipBlueprint(p))}</span></button>`).join("") || `<p class="vazio">Nenhum blueprint ainda.</p>`;
}
function montaEditor() {
  if (!blueprintEd()) edProj = D.blueprints[0]?.id ?? null;
  const p = blueprintEd(), novo = edProj === NOVO;
  ed.classList.toggle("modo-novo", novo);
  $("nv").hidden = !novo;
  $("painel-novo-manual").hidden = !novo;
  ed.querySelector(".ed-topo .tit span").textContent = novo ? "Novo blueprint" : "Editar blueprint";
  montaLateral();
  $("ed-proj").textContent = p ? p.nome || (novo ? "Novo blueprint" : "Sem nome") : "Nenhum blueprint";
  $("ed-vazio").hidden = !!p;
  $("painel-manual").hidden = !p || novo;
  ed.querySelectorAll("[data-g]").forEach((el) => { if (el.type === "checkbox") el.checked = !!D[el.dataset.g]; else el.value = D[el.dataset.g] ?? ""; });
  if (p && !novo) montaManual();
}

