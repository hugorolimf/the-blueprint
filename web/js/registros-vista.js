// ---- a tela dos registros no Planejar: o que o agente já fez, com os checks marcados (editáveis) e o que ficou pendente (registros.js guarda o estado) ----
// Todo texto do agente passa por esc(). Os botões são "Adicionar ao blueprint" e "Descartar" (registro não é proposta: nunca "aplicar").
function htmlRegistroItem(item, tipo, j, ex) {
  const titulo = tipo === "cp" ? `◇ ${esc(item.nome)} · ${esc(pvData(item.data))}` : esc(item.texto);
  const jaFeito = ex?.feito;
  const marca = jaFeito ? `<span class="badge">${tr("já estava feito")}</span>` : `<input type="checkbox" data-rc="${tipo}" data-j="${j}" aria-label="${tr("Marcar como feito")}"${item.feito ? " checked" : ""}>`;
  const evidencia = item.evidencia && !jaFeito ? `<p class="rg-evid"><b>${tr("Evidência:")}</b> ${esc(item.evidencia)}</p>` : "";
  const desc = tipo === "plano" && item.descricao && !ex ? `<p class="pv-desc">${esc(item.descricao)}</p>` : "";
  return `<div class="rg-item${item.feito ? " feito" : ""}"><label>${marca}<span class="rg-tit">${titulo}</span>${ex ? "" : `<span class="badge nova">${tr("novo")}</span>`}${item.horas ? etiquetaHoras(item) : ""}</label>${desc}${evidencia}</div>`;
}
function htmlRegistroAcao(a, i, p) {
  const ex = a.id && p.acoes.find((x) => x.id === a.id);
  const lista = (itens, tipo, rot) => itens.length ? `<div class="pv-bloco"><p class="sub2"><span>${rot}</span><span>${tr("{feitos}/{total} feitos", { feitos: itens.filter((x) => x.feito).length, total: itens.length })}</span></p>${itens.map((it, j) => htmlRegistroItem(it, tipo, j, ex && (tipo === "cp" ? ex.checkpoints : ex.planos).find((y) => y.id === it.id))).join("")}</div>` : "";
  return `<div class="pf pv${a.inc ? "" : " off"}" data-ra="${i}"><div class="pv-topo"><input type="checkbox" data-rc="acao" aria-label="${tr("Incluir esta ação")}"${a.inc ? " checked" : ""}><span class="badge${ex ? "" : " nova"}">${ex ? tr("Ação existente") : tr("Ação nova")}</span><b class="pv-nome">${esc(a.nome)}</b></div>
    <p class="pv-meta">${esc(pvData(a.inicio))} → ${esc(pvData(a.fim))}</p>${lista(a.planos, "plano", tr("Planos"))}${lista(a.checkpoints, "cp", tr("Checkpoints"))}</div>`;
}
function htmlRegistro(r, p) {
  const incluidas = r.acoes.filter((a) => a.inc), feitos = incluidas.reduce((n, a) => n + a.planos.filter((t) => t.feito).length + a.checkpoints.filter((m) => m.feito).length, 0);
  const pendentes = incluidas.reduce((n, a) => n + a.planos.filter((t) => !t.feito).length + a.checkpoints.filter((m) => !m.feito).length, 0);
  const resumo = [pvPlural(feitos, "feito", "feitos"), pendentes && pvPlural(pendentes, "pendente", "pendentes")].filter(Boolean).join(" · ");
  const msg = r.mensagem ? `<p class="cl-msg">${esc(r.mensagem)}</p>` : "";
  return `<section class="rg" data-reg="${r.numero}"><div class="cl-prop-topo"><b>${tr("Registro nº {n}", { n: r.numero })} <span class="badge">${tr("Registro")}</span> · ${resumo}</b></div>
    <div class="cl-prop-lista">${msg}<p class="dica">${tr("O seu agente fez isto fora do plano. Nada entra no blueprint até você adicionar; desmarque o que não aceita.")}</p>${r.acoes.map((a, i) => htmlRegistroAcao(a, i, p)).join("")}</div>
    <div class="cl-prop-acoes"><button class="btn cheio" data-ract="adicionar" type="button"${incluidas.length ? "" : " disabled"}>${tr("Adicionar ao blueprint")}${feitos ? ` · ${pvPlural(feitos, "feito", "feitos")}` : ""}</button><button class="btn" data-ract="descartar" type="button">${tr("Descartar")}</button></div></section>`;
}
// o painel só existe quando há registro esperando neste blueprint
function montaRegistros() {
  const box = $("cl-reg"), p = blueprintEd(), lista = p && edProj !== NOVO ? registrosDe(p.id) : [];
  box.hidden = !lista.length;
  if (!lista.length) { box.replaceChildren(); return; }
  const topo = box.scrollTop;
  box.innerHTML = `<div class="cl-prop-topo rg-cab"><b>${tr("Registros do que já foi feito · {n}", { n: lista.length })}</b></div>${lista.map((r) => htmlRegistro(r, p)).join("")}`;
  box.scrollTop = topo;
}
$("cl-reg").addEventListener("change", (e) => {
  const c = e.target.closest("[data-rc]"), sec = e.target.closest("[data-reg]"), p = blueprintEd();
  if (!c || !sec || !p) return;
  const r = registrosDe(p.id).find((x) => x.numero === +sec.dataset.reg), a = r?.acoes[+e.target.closest("[data-ra]")?.dataset.ra];
  if (!a) return;
  if (c.dataset.rc === "acao") a.inc = c.checked;
  else (c.dataset.rc === "cp" ? a.checkpoints : a.planos)[+c.dataset.j].feito = c.checked;
  montaRegistros();
});
$("cl-reg").addEventListener("click", (e) => {
  const b = e.target.closest("[data-ract]"), sec = e.target.closest("[data-reg]"), p = blueprintEd();
  if (!b || !sec || !p) return;
  const r = registrosDe(p.id).find((x) => x.numero === +sec.dataset.reg);
  if (!r) return;
  if (b.dataset.ract === "descartar") { registroRetira(r); montaRegistros(); return aviso(tr("Registro descartado: nada foi adicionado ao blueprint.")); }
  const res = registroAdiciona(r);
  if (!res) return aviso(tr("Este blueprint não existe mais; o registro não foi adicionado."));
  registroRetira(r); // sai da lista e da fila do servidor
  mudou();
  montaHub();
  aviso(tr("Registro adicionado: {feitos} feito(s){nf}{nt}{nc}.", { feitos: res.feitos, nf: res.nf ? tr(", {n} ação(ões) nova(s)", { n: res.nf }) : "", nt: res.nt ? tr(", {n} plano(s) novo(s)", { n: res.nt }) : "", nc: res.nc ? tr(", {n} checkpoint(s) novo(s)", { n: res.nc }) : "" }));
});
