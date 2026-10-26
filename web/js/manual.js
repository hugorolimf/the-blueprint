// ---- modo manual -------------------------------------------------------------------------------
function htmlItem(t) {
  return `<div class="item-ed${t.feito ? " feito" : ""}" data-t="${esc(t.id)}"><input type="checkbox" data-c="feito" aria-label="${tr("Plano feito")}"${t.feito ? " checked" : ""}><input data-c="texto" placeholder="${tr("Descreva o plano")}" aria-label="${tr("Texto do plano")}" value="${esc(t.texto)}"><input data-c="horas" type="number" min="0.5" max="16" step="0.5" placeholder="h" aria-label="${tr("Horas estimadas do plano")}" title="${tr("Estimativa de trabalho do agente, em horas")}" value="${t.horas ?? ""}"><button class="x" data-act="del-plano" type="button" aria-label="${tr("Excluir plano")}">×</button></div>`;
}
function htmlCheckpoint(m) {
  return `<div class="checkpoint-ed" data-m="${esc(m.id)}"><input type="checkbox" data-c="feito" aria-label="${tr("Checkpoint feito")}"${m.feito ? " checked" : ""}><input data-c="nome" placeholder="${tr("Nome do checkpoint")}" aria-label="${tr("Nome do checkpoint")}" value="${esc(m.nome)}"><input type="date" data-c="data" aria-label="${tr("Data do checkpoint")}" value="${esc(m.data)}"><button class="x" data-act="del-checkpoint" type="button" aria-label="${tr("Excluir checkpoint")}">×</button></div>`;
}
function htmlLinkEd(l) {
  return `<div class="link-ed" data-l="${esc(l.id)}"><input data-c="rotulo" placeholder="${tr("Rótulo (ex.: Site)")}" aria-label="${tr("Rótulo do link")}" value="${esc(l.rotulo)}"><input data-c="url" type="url" placeholder="https://…" aria-label="${tr("Endereço do link")}" value="${esc(l.url)}"><button class="x" data-act="del-link" type="button" aria-label="${tr("Excluir link")}">×</button><p class="erro" hidden>${tr("Endereço inválido: use http:// ou https://")}</p></div>`;
}
function validarLink(linha) {
  const v = linha.querySelector("[data-c=url]").value.trim();
  const ruim = v !== "" && !urlSegura(v);
  linha.classList.toggle("invalida", ruim);
  linha.querySelector(".erro").hidden = !ruim;
}
function htmlProg(f) {
  const e = efetiva(f), pct = Math.round(e.pct * 100), calc = e.it[1] > 0, edita = progEditavel(f);
  const quem = calc ? tr("calculado: {a} de {b} itens feitos", { a: e.it[0], b: e.it[1] }) : f.status === "concluido" ? tr("ação concluída") : edita ? tr("informado por você") : tr("ainda sem progresso");
  return `<div class="big"><span>${pct}%</span><small>${quem}</small></div><div class="med"><i style="width:${pct}%"></i></div>${edita ? `<input type="range" min="0" max="100" step="5" data-c="progresso" value="${pct}" aria-label="${tr("Progresso da ação em porcentagem")}">` : ""}${calc && f.status !== "concluido" ? `<p class="dica">${tr("O progresso vem dos planos e dos checkpoints marcados, aqui ou direto no blueprint.")}</p>` : ""}`;
}
function htmlDetBlueprint(p, pi) {
  return `<div class="det-topo"><span class="rot" style="margin:0">${tr("Blueprint")} ${pad(pi + 1)}${projConcluido(p) ? " · " + tr("✓ concluído") : ""}</span><div class="acoes-topo"><button class="btn mini" data-act="up-blueprint" type="button">${tr("↑ Subir na lista")}</button><button class="btn mini" data-act="down-blueprint" type="button">${tr("↓ Descer na lista")}</button></div></div>
  <label class="f grande">${tr("Nome do blueprint")}<input data-c="nome" value="${esc(p.nome)}" placeholder="${tr("Nome do blueprint")}"></label>
  <div class="card"><h3>${tr("Descrição e observações")}</h3><div class="campos"><label class="f">${tr("Descrição curta")}<input data-c="descricao" value="${esc(p.descricao)}" placeholder="${tr("Para que serve este blueprint")}"></label><label class="f">${tr("Observações do blueprint")}<textarea data-c="obs" placeholder="${tr("Anotações gerais, decisões, riscos…")}">${esc(p.obs)}</textarea></label></div></div>
  <div class="card" data-pasta-blueprint><h3>${tr("Pasta do projeto")}</h3><p role="status">${tr("Carregando vínculo…")}</p></div>
  <div class="card"><h3>${tr("Marca da folha")}<small>${p.emblema ? tr("{n} peças", { n: p.emblema.pecas.length }) : tr("sem marca")}</small></h3><div class="marca-previa">${p.emblema ? previaMarca(p.emblema.pecas, 330, 230, progressoDe(p)) : `<p class="vazio">${tr("Este blueprint ainda não tem marca. A IA desenha uma marca para o blueprint.")}</p>`}</div>${p.emblema ? '<label class="f" style="margin-top:8px">' + tr("Prévia do progresso da marca") + '<input type="range" min="0" max="100" step="5" data-prev-marca value="' + Math.round(progressoDe(p) * 100) + '"></label>' : ""}<div class="acoes">${p.emblema ? '<button class="btn mini perigo" data-act="marca-remover" type="button">' + tr("Remover marca") + '</button>' : ""}</div></div>
  ${htmlProjetosEd(p)}
  <div class="card"><h3>${tr("Links")}<small>${p.links.length || ""}</small></h3>${p.links.length ? p.links.map(htmlLinkEd).join("") : `<p class="vazio">${tr("Nenhum link. Adicione o site, o ambiente de homologação, a documentação…")}</p>`}<div class="acoes"><button class="btn mini" data-act="add-link" type="button">${tr("+ Link")}</button></div></div>
  <div class="card"><h3>${tr("Zona de perigo")}</h3><button class="btn perigo" data-act="del-blueprint" type="button">${tr("Excluir este blueprint…")}</button></div>`;
}
function htmlDetAcao(p, pi, f, fi) {
  const e = efetiva(f);
  return `<div class="det-topo"><span class="rot" style="margin:0">${tr("Ação")} ${numeroDaAcao(p, f, pi)}</span><div class="acoes-topo"><button class="btn mini perigo" data-act="del-acao" type="button">${tr("Excluir ação")}</button></div></div>
  <label class="f grande">${tr("Nome da ação")}<input data-c="nome" value="${esc(f.nome)}" placeholder="${tr("Ex.: Homologação")}"></label>
  <div class="card det-acao"><h3>${tr("Período e situação")}<small class="dur">${durTxt(f)}</small></h3>
    <div class="grade2"><label class="f">${tr("Início")}<input type="date" data-c="inicio" value="${esc(f.inicio)}"></label><label class="f">${tr("Fim")}<input type="date" data-c="fim" value="${esc(f.fim)}"></label></div>
    <p class="erro" hidden>${tr("A data final é anterior à inicial.")}</p>
    ${htmlProjetoDaAcao(p, f)}
    <div class="seg" role="group" aria-label="${tr("Situação")}">${SIT.map(([v, t]) => `<button type="button" data-set="status" data-v="${v}" aria-pressed="${(e.it[1] ? e.status : f.status) === v}"${v === "concluido" && e.it[1] ? ' disabled title="' + tr("Conclui sozinha quando todos os planos e checkpoints estiverem marcados") + '"' : ""}>${t}</button>`).join("")}</div>
    <div class="prog-bloco">${htmlProg(f)}</div></div>
  <div class="card"><h3>${tr("Planos")}<small class="cnt-t">${e.tf[1] ? `${tr("{a} de {b}", { a: e.tf[0], b: e.tf[1] })}` : ""}</small></h3>
    ${f.planos.length ? f.planos.map(htmlItem).join("") : `<p class="vazio">${tr("Nenhum plano ainda. Adicione uma ou cole uma lista.")}</p>`}
    <div class="acoes"><button class="btn mini" data-act="add-plano" type="button">${tr("+ Plano")}</button><button class="btn mini" data-act="toggle-varias" data-alvo="t" type="button">${tr("Colar lista…")}</button></div>
    <div class="varias" hidden><textarea data-varias="t" placeholder="${tr("Um plano por linha")}"></textarea><div class="acoes" style="margin:0"><button class="btn mini cheio" data-act="aplicar-varias" data-alvo="t" type="button">${tr("Adicionar planos")}</button></div></div>
    <p class="dica">${tr("Enter cria o próximo plano; Backspace numa linha vazia a remove.")}</p></div>
  <div class="card"><h3>${tr("Checkpoints")}<small class="cnt-m">${e.mf[1] ? `${tr("{a} de {b}", { a: e.mf[0], b: e.mf[1] })}` : ""}</small></h3>
    ${f.checkpoints.length ? f.checkpoints.map(htmlCheckpoint).join("") : `<p class="vazio">${tr("Nenhum checkpoint. Use-os para checkpoints verificáveis, como “Homologação aprovada”.")}</p>`}
    <div class="acoes"><button class="btn mini" data-act="add-checkpoint" type="button">${tr("+ Checkpoint")}</button><button class="btn mini" data-act="toggle-varias" data-alvo="m" type="button">${tr("Colar lista…")}</button></div>
    <div class="varias" hidden><textarea data-varias="m" placeholder="${tr("Um checkpoint por linha (a data será o fim da ação)")}"></textarea><div class="acoes" style="margin:0"><button class="btn mini cheio" data-act="aplicar-varias" data-alvo="m" type="button">${tr("Adicionar checkpoints")}</button></div></div></div>
  <div class="card"><h3>${tr("Instruções da ação")}<small>${tr("o que fazer: o roteiro desta ação")}</small></h3>${mdEdicao('data-c="obs"', f.obs, { rows: 14, placeholder: tr("O que fazer nesta ação, passo a passo: o plano, as decisões, as dependências e os riscos…") })}</div>${f.prototipos?.length ? `
  <div class="card"><h3>${tr("Protótipos")}<small>${tr("anexados pelo agente (MCP)")}</small></h3>${f.prototipos.map((x) => `<div class="acoes" style="margin:0 0 8px"><button class="btn mini cheio" type="button" data-act="ver-proto" data-id="${esc(x.id)}">▶ ${esc(x.titulo)}</button></div>`).join("")}</div>` : ""}`;
}
function montaListaManual() {
  const p = blueprintEd();
  if (!p) return;
  const pi = D.blueprints.indexOf(p);
  const acoes = p.acoes.map((f, fi) => {
    const e = efetiva(f);
    return `<button class="mn-item" type="button" data-sel="${esc(f.id)}" aria-current="${edSel === f.id}"><span class="n">${numeroDaAcao(p, f, pi)}</span><span class="nome">${esc(f.nome || tr("Ação sem nome"))}</span><span class="sub"><span class="chip">${ROTULO[e.status]}</span><span>${Math.round(e.pct * 100)}%</span>${nomeDoProjeto(p, f.projetoId) ? `<span>${esc(nomeDoProjeto(p, f.projetoId))}</span>` : ""}${e.it[1] ? `<span>${e.it[0]}/${e.it[1]}</span>` : ""}</span></button>`;
  }).join("");
  $("mn-lista").innerHTML = `<button class="mn-item" type="button" data-sel="blueprint" aria-current="${edSel === "blueprint"}"><span class="n">◈</span><span class="nome">${tr("Dados do blueprint")}</span><span class="sub"><span>${tr("nome · links · observações")}</span></span></button><p class="rot">${tr("Ações ({n})", { n: p.acoes.length })}</p>${acoes || `<p class="vazio">${tr("Nenhuma ação ainda.")}</p>`}<button class="btn" type="button" data-act="add-acao">${tr("+ Nova ação")}</button>`;
}
function validarAcao() {
  const caixa = $("mn-detalhe").querySelector(".det-acao");
  if (!caixa) return;
  const a = dia(caixa.querySelector('[data-c="inicio"]').value), b = dia(caixa.querySelector('[data-c="fim"]').value);
  const ruim = a === a && b === b && b < a;
  caixa.classList.toggle("invalida", ruim);
  caixa.querySelector(".erro").hidden = !ruim;
}
function montaManual() {
  const p = blueprintEd();
  if (!p) return;
  if (edSel !== "blueprint" && !acaoEd(p)) edSel = "blueprint";
  const det = $("mn-detalhe"), topo = det.scrollTop, mesma = det.dataset.chave === edSel + p.id;
  montaListaManual();
  const pi = D.blueprints.indexOf(p), f = edSel === "blueprint" ? null : acaoEd(p);
  det.innerHTML = f ? htmlDetAcao(p, pi, f, p.acoes.indexOf(f)) : htmlDetBlueprint(p, pi);
  if (!f) import("/web/js/pastas.js").then((m) => m.editorPasta(det.querySelector("[data-pasta-blueprint]"), workspaceAtual?.id, p.id));
  det.dataset.chave = edSel + p.id; // não pode se chamar data-sel: o clique da lista de ações procura esse atributo
  det.scrollTop = mesma ? topo : 0;
  det.querySelectorAll(".link-ed").forEach(validarLink);
  validarAcao();
}
// atualiza só o que depende das marcações da ação, sem refazer a tela (o foco fica onde está)
function refrescaAcao(f) {
  const det = $("mn-detalhe"), e = efetiva(f);
  const pb = det.querySelector(".prog-bloco");
  if (pb) pb.innerHTML = htmlProg(f);
  const t = det.querySelector(".cnt-t"), m = det.querySelector(".cnt-m"), d = det.querySelector(".dur");
  if (t) t.textContent = e.tf[1] ? `${tr("{a} de {b}", { a: e.tf[0], b: e.tf[1] })}` : "";
  if (m) m.textContent = e.mf[1] ? `${tr("{a} de {b}", { a: e.mf[0], b: e.mf[1] })}` : "";
  if (d) d.textContent = durTxt(f);
  det.querySelectorAll(".seg [data-set]").forEach((b) => { b.setAttribute("aria-pressed", String(b.dataset.v === (e.it[1] ? e.status : f.status))); b.disabled = b.dataset.v === "concluido" && e.it[1] > 0; });
  montaListaManual();
}
const pm = $("painel-manual");
pm.addEventListener("input", (e) => {
  const el = e.target.closest("[data-c]");
  const p = blueprintEd();
  if (!el || !p) return;
  const c = el.dataset.c, val = el.type === "checkbox" ? el.checked : el.value;
  if (c === "obs" && el.matches("textarea")) mdPrevia(el); // prévia em markdown ao lado do campo
  const pj = el.closest(".projeto-ed");
  if (pj) { renomearProjeto(p, pj.dataset.pj, val, c); montaListaManual(); return mudou(); }
  const lr = el.closest(".link-ed");
  if (lr) { p.links.find((x) => x.id === lr.dataset.l)[c] = val; validarLink(lr); return mudou(); }
  if (edSel === "blueprint") {
    p[c] = val;
    if (c === "nome") { $("ed-proj").textContent = val || tr("Sem nome"); montaLateral(); }
    return mudou();
  }
  const f = acaoEd(p);
  if (!f) return;
  const item = el.closest(".item-ed"), checkpoint = el.closest(".checkpoint-ed");
  if (item) {
    const plano = f.planos.find((x) => x.id === item.dataset.t);
    if (c === "horas") { const n = Number(val); if (val === "" || !(n >= 0.5 && n <= 16)) delete plano.horas; else plano.horas = n; return mudou(); }
    plano[c] = val;
    if (c === "feito") { item.classList.toggle("feito", val); refrescaAcao(f); }
    return mudou();
  }
  if (checkpoint) {
    f.checkpoints.find((x) => x.id === checkpoint.dataset.m)[c] = val;
    if (c === "feito" && val) avisarMonitor("checkpoint-marcado", p.id, checkpoint.dataset.m);
    if (c === "feito") refrescaAcao(f);
    return mudou();
  }
  if (c === "progresso") {
    f.progresso = Math.min(100, Math.max(0, Number(val) || 0)) / 100;
    const pct = Math.round(f.progresso * 100), det = $("mn-detalhe");
    det.querySelector(".big span").textContent = pct + "%"; // o controle deslizante não pode ser refeito durante o arrasto
    det.querySelector(".med i").style.width = pct + "%";
    montaListaManual();
    return mudou();
  }
  if (c === "projetoId" && !val) delete f.projetoId; else f[c] = val; // Geral = sem o campo
  if (c === "inicio" || c === "fim") { validarAcao(); const d = $("mn-detalhe").querySelector(".dur"); if (d) d.textContent = durTxt(f); }
  montaListaManual();
  mudou();
});
function novaPlanoApos(f, idx) {
  const t = { id: novoId(), texto: "", feito: false };
  f.planos.splice(idx < 0 ? f.planos.length : idx + 1, 0, t);
  return t;
}
// remove uma ação; pede confirmação se ela tem planos ou checkpoints (ou sempre, com `sempre`). Devolve se removeu.
async function excluirAcao(pid, fid, sempre = false) {
  const p = D.blueprints.find((x) => x.id === pid), f = p?.acoes.find((x) => x.id === fid);
  if (!f) return false;
  const com = f.planos.length || f.checkpoints.length;
  if ((sempre || com) && !(await confirmar({ titulo: tr("Excluir ação?"), texto: tr(`A ação "{nome}" será apagada{extra}. Não dá para desfazer.`, { nome: f.nome || tr("sem nome"), extra: com ? tr(", junto com seus planos e checkpoints") : "" }), rotulo: tr("Excluir ação") }))) return false;
  p.acoes = p.acoes.filter((x) => x.id !== f.id);
  if (edProj === pid) edSel = "blueprint";
  if (!ed.hidden) montaManual();
  mudou();
  return true;
}
async function excluirBlueprint(id) {
  const p = D.blueprints.find((x) => x.id === id);
  if (!p || !(await confirmar({ titulo: tr("Excluir blueprint?"), texto: tr(`O blueprint "{nome}" será apagado, junto com todas as suas ações. Não dá para desfazer.`, { nome: p.nome || tr("Sem nome") }), rotulo: tr("Excluir blueprint") }))) return;
  D.blueprints = D.blueprints.filter((x) => x.id !== id);
  if (tela === id) { tela = null; location.hash = "/c/" + encodeURIComponent(workspaceAtual.id); }
  edProj = D.blueprints[0]?.id ?? null;
  edSel = "blueprint";
  if (ed.hidden) mudou(); else montaEditor();
}
pm.addEventListener("keydown", (e) => {
  const p = blueprintEd(), f = p && acaoEd(p);
  const campo = e.target.closest(".item-ed [data-c=texto], .checkpoint-ed [data-c=nome]");
  if (!f || !campo) return;
  const plano = !!campo.closest(".item-ed");
  const lista = plano ? f.planos : f.checkpoints, id = campo.closest(plano ? ".item-ed" : ".checkpoint-ed").dataset[plano ? "t" : "m"];
  const idx = lista.findIndex((x) => x.id === id);
  if (e.key === "Enter") {
    e.preventDefault();
    const novo = plano ? { id: novoId(), texto: "", feito: false } : { id: novoId(), nome: "", data: f.fim || local(new Date()), feito: false };
    lista.splice(idx + 1, 0, novo);
    montaManual();
    pm.querySelector(`[data-${plano ? "t" : "m"}="${novo.id}"] input[type=text], [data-${plano ? "t" : "m"}="${novo.id}"] input:not([type])`)?.focus();
    mudou();
  } else if (e.key === "Backspace" && campo.value === "" && lista.length) {
    e.preventDefault();
    lista.splice(idx, 1);
    montaManual();
    const alvo = lista[Math.max(0, idx - 1)];
    if (alvo) pm.querySelector(`[data-${plano ? "t" : "m"}="${alvo.id}"] [data-c=${plano ? "texto" : "nome"}]`)?.focus();
    mudou();
  }
});
// plano, checkpoint e link: apagam só depois da confirmação (o `mudou()` do fim do clique roda antes, sem efeito)
const apagar = (titulo, texto, fn) => confirmar({ titulo, texto, rotulo: "Excluir" }).then((ok) => { if (ok) { fn(); montaManual(); mudou(); } });
const nomeDe = (v, vazio) => String(v || vazio).trim() || vazio;
pm.addEventListener("click", (e) => {
  const sel = e.target.closest("[data-sel]");
  if (sel) { edSel = sel.dataset.sel; montaManual(); return; }
  const p = blueprintEd();
  if (!p) return;
  const f = acaoEd(p);
  const st = e.target.closest("[data-set]");
  if (st && f && !st.disabled) {
    f.status = st.dataset.v;
    f.progresso = f.status === "concluido" ? 1 : f.status === "planejado" ? 0 : f.progresso;
    refrescaAcao(f);
    return mudou();
  }
  const b = e.target.closest("[data-act]");
  if (!b) return;
  const a = b.dataset.act;
  const foca = (sel2) => pm.querySelector(sel2)?.focus();
  if (a === "add-acao") {
    const ini = local(new Date()), fim = deDia(dia(ini) + 30);
    const nf = { id: novoId(), nome: "", inicio: ini, fim, status: "andamento", progresso: 0, obs: "", planos: [], checkpoints: [] };
    p.acoes.push(nf);
    edSel = nf.id;
    montaManual();
    foca("#mn-detalhe [data-c=nome]");
  } else if (a === "del-acao" && f) {
    excluirAcao(p.id, f.id);
    return;
  } else if (a === "ver-proto" && f) {
    abreProtos(p.id, f.id, b.dataset.id);
    return;
  } else if (a === "add-plano" && f) {
    const t = novaPlanoApos(f, -1);
    montaManual();
    foca(`[data-t="${t.id}"] [data-c=texto]`);
  } else if (a === "del-plano" && f) {
    const id = b.closest(".item-ed").dataset.t, t = f.planos.find((x) => x.id === id);
    apagar(tr("Excluir plano?"), tr(`O plano "{nome}" será apagado.`, { nome: nomeDe(t?.texto, tr("sem texto")) }), () => { f.planos = f.planos.filter((x) => x.id !== id); });
  } else if (a === "add-checkpoint" && f) {
    const m = { id: novoId(), nome: "", data: f.fim || local(new Date()), feito: false };
    f.checkpoints.push(m);
    montaManual();
    foca(`[data-m="${m.id}"] [data-c=nome]`);
  } else if (a === "del-checkpoint" && f) {
    const id = b.closest(".checkpoint-ed").dataset.m, c = f.checkpoints.find((x) => x.id === id);
    apagar(tr("Excluir checkpoint?"), tr(`O checkpoint "{nome}" será apagado.`, { nome: nomeDe(c?.nome, tr("sem nome")) }), () => { f.checkpoints = f.checkpoints.filter((x) => x.id !== id); });
  } else if (a === "toggle-varias") {
    const v = b.closest(".card").querySelector(".varias");
    v.hidden = !v.hidden;
    if (!v.hidden) v.querySelector("textarea").focus();
    return;
  } else if (a === "aplicar-varias" && f) {
    const area = b.closest(".varias").querySelector("textarea");
    const linhas = area.value.split("\n").map((l) => l.replace(/^\s*(?:[-*•–]|\d+[.)])\s*/, "").trim()).filter(Boolean);
    if (!linhas.length) return;
    if (b.dataset.alvo === "t") linhas.forEach((texto) => f.planos.push({ id: novoId(), texto, feito: false }));
    else linhas.forEach((nome) => f.checkpoints.push({ id: novoId(), nome, data: f.fim || local(new Date()), feito: false }));
    montaManual();
  } else if (a === "add-link") {
    p.links.push({ id: novoId(), rotulo: "", url: "" });
    montaManual();
    foca(".link-ed:last-of-type [data-c=rotulo]");
  } else if (a === "del-link") {
    const id = b.closest(".link-ed").dataset.l, l = p.links.find((x) => x.id === id);
    apagar(tr("Excluir link?"), tr(`O link "{nome}" será apagado.`, { nome: nomeDe(l?.rotulo || l?.url, tr("sem rótulo")) }), () => { p.links = p.links.filter((x) => x.id !== id); });
  } else if (a === "add-projeto") {
    const x = novoProjeto(p);
    montaManual();
    foca(`[data-pj="${x.id}"] [data-c=nome]`);
  } else if (a === "del-projeto") {
    const id = b.closest(".projeto-ed").dataset.pj, n = contaAcoesDoProjeto(p, id);
    apagar(tr("Remover projeto?"), tr(`O projeto "{nome}" será removido{extra}.`, { nome: nomeDe(nomeDoProjeto(p, id), tr("sem nome")), extra: n ? tr("; as {n} ação(ões) dele voltam ao Geral (nenhuma ação é apagada)", { n }) : "" }), () => removerProjeto(p, id));
  } else if (a === "up-blueprint" || a === "down-blueprint") {
    const i = D.blueprints.indexOf(p), j = i + (a === "up-blueprint" ? -1 : 1);
    if (j < 0 || j >= D.blueprints.length) return;
    D.blueprints.splice(j, 0, D.blueprints.splice(i, 1)[0]);
    montaEditor();
  } else if (a === "del-blueprint") {
    excluirBlueprint(p.id);
    return;
  } else if (a === "marca-remover") {
    p.emblema = null;
    montaManual();
  }
  mudou();
});

