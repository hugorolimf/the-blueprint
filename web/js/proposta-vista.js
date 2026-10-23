// ---- revisão da proposta em modo leitura: mostra o que MUDA em cada ação (novo, alterado, movido, sem mudança), sem campos de edição ----
// A edição (htmlPropAcao, em proposta.js) continua a um clique: "Editar". Aqui só se monta HTML; todo texto do agente passa por esc() ou mdHtml().
const pvNorm = (s) => String(s ?? "").replace(/\s+/g, " ").trim();
const pvData = (iso) => (ISO.test(iso) ? iso.split("-").reverse().slice(0, 2).join("/") : String(iso || "?"));
const pvPlural = (n, um, varios) => `${n} ${n === 1 ? um : varios}`;

// compara a ação da proposta (pf) com a que já existe (ex); sem `ex`, tudo é novo. O servidor completa o que o agente omitiu, então "sem mudança" é de verdade.
function propostaDiff(pf, ex) {
  const planos = pf.planos.filter((t) => t.texto.trim()).map((t) => {
    if (t.de) return { ...t, mud: "movido" };
    const x = t.id && ex?.planos.find((y) => y.id === t.id);
    if (!x) return { ...t, mud: "novo" };
    const titulo = pvNorm(x.texto) !== pvNorm(t.texto), desc = !!t.descricao && pvNorm(x.descricao) !== pvNorm(t.descricao), horas = t.horas != null && t.horas !== x.horas;
    return { ...t, mud: titulo || desc || horas ? "alterado" : "", antes: x, titulo, desc };
  });
  const cps = pf.checkpoints.filter((m) => m.nome.trim()).map((m) => {
    if (m.de) return { ...m, mud: "movido" };
    const x = m.id && ex?.checkpoints.find((y) => y.id === m.id);
    if (!x) return { ...m, mud: "novo" };
    return { ...m, mud: pvNorm(x.nome) !== pvNorm(m.nome) || x.data !== m.data ? "alterado" : "", antes: x };
  });
  const campos = ex ? [pvNorm(pf.nome) !== pvNorm(ex.nome) && "nome", (pf.inicio !== ex.inicio || pf.fim !== ex.fim) && "datas", (ex.projetoId || "") !== (pf.projeto || "") && "projeto", !!pf.obs && pvNorm(pf.obs) !== pvNorm(ex.obs) && "instruções"].filter(Boolean) : [];
  const conta = (l, m) => l.filter((x) => x.mud === m).length;
  return { planos, cps, campos, novos: conta(planos, "novo") + conta(cps, "novo"), alterados: conta(planos, "alterado") + conta(cps, "alterado"), movidos: conta(planos, "movido") + conta(cps, "movido"), mudancas: planos.filter((t) => t.mud).length + cps.filter((m) => m.mud).length + campos.length };
}
const pvFrase = (d) => [d.novos && pvPlural(d.novos, "item novo", "itens novos"), d.alterados && pvPlural(d.alterados, "alterado", "alterados"), d.movidos && pvPlural(d.movidos, "movido", "movidos"), d.campos.length && `${d.campos.join(", ")} da ação`].filter(Boolean);

const PV_MARCA = { novo: "+", alterado: "~", movido: "↦" }, PV_ROTULO = { novo: "novo", alterado: "alterado", movido: "movido" };
function pvLinha(t, p, tipo) {
  const mud = t.mud, origem = mud === "movido" && p.acoes.find((y) => y.id === t.de);
  const velho = mud === "alterado" && pvNorm(t.antes[tipo === "cp" ? "nome" : "texto"]) !== pvNorm(t[tipo === "cp" ? "nome" : "texto"]) ? `<s class="pv-antes">${esc(t.antes[tipo === "cp" ? "nome" : "texto"])}</s>` : "";
  const tit = tipo === "cp" ? `◇ ${esc(t.nome)} · ${esc(pvData(t.data))}${mud === "alterado" && t.antes.data !== t.data ? ` <s class="pv-antes">${esc(pvData(t.antes.data))}</s>` : ""}` : esc(t.texto);
  const desc = tipo === "plano" && t.descricao ? `<p class="pv-desc">${esc(t.descricao)}</p>${t.desc && t.antes.descricao ? `<details class="pv-ant"><summary>Descrição anterior</summary><p class="pv-desc">${esc(t.antes.descricao)}</p></details>` : ""}` : "";
  return `<details class="pv-plano ${mud}"${mud ? " open" : ""}><summary><span class="pv-marca" aria-hidden="true">${PV_MARCA[mud] || "·"}</span><span class="pv-tit">${velho}${tit}</span>${mud ? `<span class="badge ${mud === "movido" ? "mov" : mud}">${PV_ROTULO[mud]}</span>` : ""}${origem ? `<span class="badge mov" title="Movido de “${esc(origem.nome)}”">← ${esc(origem.nome)}</span>` : ""}${tipo === "plano" ? etiquetaHoras(t) : ""}</summary>${desc}</details>`;
}
function pvLista(titulo, itens, p, tipo) {
  if (!itens.length) return "";
  const mudou = itens.filter((x) => x.mud), igual = itens.filter((x) => !x.mud);
  return `<div class="pv-bloco"><p class="sub2"><span>${titulo}</span><span>${itens.length}</span></p>${mudou.map((x) => pvLinha(x, p, tipo)).join("")}${igual.length ? `<details class="pv-igual"><summary>${pvPlural(igual.length, "item sem mudança", "itens sem mudança")}</summary>${igual.map((x) => pvLinha(x, p, tipo)).join("")}</details>` : ""}</div>`;
}

function htmlVistaAcao(pf, i, p) {
  const ex = p.acoes.find((x) => x.id === pf.id), d = propostaDiff(pf, ex);
  const badge = !ex ? `<span class="badge nova">Nova</span>` : d.mudancas ? `<span class="badge alt">Alterada</span>` : `<span class="badge">Sem mudança</span>`;
  const dias = dia(pf.fim) - dia(pf.inicio) + 1, velhas = d.campos.includes("datas") ? ` <s class="pv-antes">${pvData(ex.inicio)} → ${pvData(ex.fim)}</s>` : "";
  const meta = [`${pvData(pf.inicio)} → ${pvData(pf.fim)}${velhas}`, Number.isFinite(dias) && pvPlural(dias, "dia", "dias"), pvPlural(d.planos.length, "plano", "planos"), pvPlural(d.cps.length, "checkpoint", "checkpoints")].filter(Boolean).join(" · ");
  const frase = pvFrase(d), obsRot = !pf.obs ? "vazias" : !ex || d.campos.includes("instruções") ? (ex ? "alteradas" : "novas") : "sem mudança";
  return `<div class="pf pv${pf.inc ? "" : " off"}" data-i="${i}">
    <div class="pv-topo"><input type="checkbox" data-pc="inc" aria-label="Incluir esta ação"${pf.inc ? " checked" : ""}>${badge}<b class="pv-nome">${esc(pf.nome)}</b><button class="btn mini" data-pact="editar-acao" type="button">Editar</button></div>
    <p class="pv-meta">${meta}</p>${frase.length ? `<p class="pv-chips">${frase.map((f) => `<span class="chip">${esc(f)}</span>`).join("")}</p>` : ""}
    ${pvLista("Planos", d.planos, p, "plano")}${pvLista("Checkpoints", d.cps, p, "cp")}
    <details class="pv-bloco pv-obs"><summary>Instruções da ação · ${obsRot}</summary><div class="md obs-ver">${mdHtml(pf.obs) || '<p class="md-vazio">Sem instruções.</p>'}</div></details>
  </div>`;
}

// topo da lista: a mensagem do agente (curta aberta, longa recolhida) e o resumo do que a proposta inteira muda
function htmlTopoProposta(c, p) {
  const diffs = (c.proposta || []).filter((x) => x.inc).map((pf) => propostaDiff(pf, p.acoes.find((x) => x.id === pf.id)));
  const soma = (k) => diffs.reduce((t, d) => t + d[k], 0), semMudar = diffs.filter((d) => !d.mudancas).length;
  const resumo = [soma("novos") && pvPlural(soma("novos"), "item novo", "itens novos"), soma("alterados") && pvPlural(soma("alterados"), "alterado", "alterados"), soma("movidos") && pvPlural(soma("movidos"), "movido", "movidos"), semMudar && pvPlural(semMudar, "ação sem mudança", "ações sem mudança")].filter(Boolean);
  const msg = c.mensagem ? (c.mensagem.length > 280 ? `<details class="pv-msg"><summary>Mensagem do agente</summary><p class="cl-msg">${esc(c.mensagem)}</p></details>` : `<p class="cl-msg">${esc(c.mensagem)}</p>`) : "";
  return `${msg}${resumo.length ? `<p class="pv-resumo">${resumo.map((f) => `<span class="chip">${esc(f)}</span>`).join("")}</p>` : ""}`;
}
