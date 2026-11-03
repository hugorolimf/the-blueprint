// ---- painel Acompanhar (supervisor e dono; modo hospedado; docs/colaboracao.md, seção 5) -------------------------------
// O progresso de vários blueprints e pessoas num lugar só, sem abrir cada blueprint. Sem rota própria: os blueprints vêm do /api/dados que a
// página já carregou (filtrado pelo papel), as pessoas de /api/pessoas e o que aconteceu de /api/atividade. Módulo ES carregado por import().

// por blueprint: progresso (itens feitos / itens), ações concluídas e em andamento, e o próximo checkpoint em aberto
export function resumoBlueprint(bp) {
  let itens = 0, feitos = 0, concluidas = 0, andamento = 0, proximo = null;
  for (const a of bp.acoes || []) {
    const lista = [...(a.planos || []), ...(a.checkpoints || [])], f = lista.filter((x) => x.feito).length;
    itens += lista.length; feitos += f;
    const concluida = lista.length ? f === lista.length : a.status === "concluido";
    if (concluida) concluidas++;
    else if (f > 0 || a.status === "andamento") andamento++;
    for (const c of a.checkpoints || []) if (!c.feito && !proximo) proximo = { nome: c.nome };
  }
  const acoes = (bp.acoes || []).length;
  return { id: bp.id, nome: bp.nome, acoes, concluidas, andamento, proximo, pct: itens ? Math.round((feitos / itens) * 100) : acoes && concluidas === acoes ? 100 : 0 };
}
// o menos adiantado primeiro (é o que o supervisor precisa ver)
export function resumoSupervisor(workspaces) {
  return workspaces.map((w) => ({ id: w.id, nome: w.nome, blueprints: (w.dados.blueprints || []).map((b) => resumoBlueprint(b)).sort((x, y) => x.pct - y.pct) }));
}
export const textoAtividade = (tipo) => ({
  "entrou": tr("entrou"), "saiu": tr("saiu"), "convite-criado": tr("criou um link de convite"), "convite-revogado": tr("revogou um link de convite"),
  "convite-trocado": tr("trocou um link de convite"), "papel-mudado": tr("mudou o papel de alguém"), "membro-removido": tr("removeu alguém"),
  "chave-criada": tr("gerou uma chave de MCP"), "chave-revogada": tr("revogou uma chave de MCP"), "link-dono-impresso": tr("o servidor imprimiu o link do dono"),
})[tipo] || tipo;

export async function abrirSupervisor() {
  const opcoes = workspacesDaEquipe(["dono", "supervisor"]);
  $("equipe-titulo").textContent = tr("Acompanhar");
  const grupos = resumoSupervisor(opcoes);
  $("equipe-corpo").innerHTML = grupos.map((g) => `<section class="eq-secao eq-sup" data-ws="${esc(g.id)}"><h4>${esc(g.nome)}</h4>
    <div class="eq-grade">${g.blueprints.map((b) => `<article class="eq-bp" data-bp="${esc(b.id)}"><strong>${esc(b.nome || tr("Sem nome"))}</strong>
      <progress max="100" value="${b.pct}" aria-label="${esc(tr("Progresso de {nome}", { nome: b.nome }))}"></progress>
      <small>${b.pct}% · ${esc(tr("{feitas} de {total} ações concluídas", { feitas: b.concluidas, total: b.acoes }))}</small>
      <small>${esc(trn(b.andamento, ["{n} em andamento", "{n} em andamento"]))}</small>
      <small>${b.proximo ? esc(tr("Próximo checkpoint: {nome}", { nome: b.proximo.nome })) : esc(tr("Sem checkpoint em aberto"))}</small>
      <button class="btn mini" type="button" data-abrir="${esc(b.id)}">${tr("Abrir")}</button></article>`).join("") || `<p>${tr("Nenhum blueprint neste workspace.")}</p>`}</div>
    <div class="eq-duas"><div><h5>${tr("Pessoas")}</h5><ul class="eq-lista" data-pessoas><li class="eq-vazio">${tr("carregando…")}</li></ul></div>
    <div><h5>${tr("Atividade recente")}</h5><ul class="eq-lista eq-ativ" data-atividade><li class="eq-vazio">${tr("carregando…")}</li></ul></div></div></section>`).join("") || `<p>${tr("Nenhum workspace para acompanhar.")}</p>`;
  $("equipe-corpo").querySelectorAll("[data-abrir]").forEach((b) => b.addEventListener("click", () => { $("equipe").close(); location.hash = "/p/" + encodeURIComponent(b.dataset.abrir); }));
  if (!$("equipe").open) $("equipe").showModal();
  await Promise.all(grupos.map((g) => pessoasEAtividade($("equipe-corpo").querySelector(`[data-ws="${CSS.escape(g.id)}"]`), g.id)));
}
async function pessoasEAtividade(sec, ws) {
  const q = encodeURIComponent(ws), [p, a] = await Promise.allSettled([equipeApi(`/api/pessoas?workspace=${q}`), equipeApi(`/api/atividade?workspace=${q}&limite=20`)]);
  sec.querySelector("[data-pessoas]").innerHTML = p.status === "fulfilled"
    ? p.value.membros.map((m) => `<li><span class="eq-nome"><strong>${esc(m.nome)}</strong><small>${esc(nomeDoPapel(m.papel))}</small></span></li>`).join("") || `<li class="eq-vazio">${tr("Ninguém ainda.")}</li>`
    : `<li class="eq-vazio">${esc(p.reason.message)}</li>`;
  sec.querySelector("[data-atividade]").innerHTML = a.status === "fulfilled"
    ? a.value.itens.map((i) => `<li><span class="eq-nome"><strong>${esc(i.membro?.nome || tr("Servidor"))}</strong> ${esc(textoAtividade(i.tipo))}</span></li>`).join("") || `<li class="eq-vazio">${tr("Nada ainda.")}</li>`
    : `<li class="eq-vazio">${esc(a.reason.message)}</li>`;
}
