const emNovo = () => edProj === NOVO;
let EMB = null; // módulo da marca (peças do anatomy), carregado do servidor
const emblemasProntos = import("/emblemas.mjs").then((m) => { EMB = m; $("login-arte").innerHTML = previaMarca(m.BLUEPRINT, 390, 280); render(); if (!ed.hidden) montaEditor(); if (!hub.hidden) montaHub(); }).catch(() => {});
const progressoDe = (p) => (projConcluido(p) ? 1 : resumo(acoesDe(p)).geral);
const previaMarca = (pecas, w = 330, h = 230, progresso = 1) => {
  const e = EMB && pecas?.length ? EMB.desenhaEmblema(pecas, { largura: w, altura: h, progresso }) : null;
  return e ? `<svg class="emblema previa" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}" role="img" aria-label="Prévia da marca da folha">${e.svg}</svg>` : `<p class="vazio">Prévia indisponível.</p>`;
};

// painel esquerdo da tela Planejar: como ligar o agente ao app e o que ele já avisou (a proposta fica à direita)
function montaMcp() {
  if (!blueprintEd()) return;
  $("ag-mcp").hidden = !mcpFalhou; // só aparece se o servidor não responde
  montaProposta();
}
// como ligar o agente fica numa folha à parte (botão "Como conectar"): a tela do Planejar foca na proposta
function textoComoUsar() {
  const p = blueprintEd() || {}, novo = emNovo(), ws = workspaceAtual?.id || "sm";
  const comando = mcpInfo ? `"${mcpInfo.arquivo}" --mcp` : "<pasta do app>/blueprint.exe --mcp";
  const padrao = !mcpInfo || mcpInfo.url === "http://127.0.0.1:3000";
  return (
    `O app não abre agentes: o seu agente (Claude Code, Codex…) é quem propõe, pelo MCP.\n\n1. Registre o servidor uma vez:\nClaude Code:  claude mcp add blueprint ${padrao ? "" : `-e BLUEPRINT_URL=${mcpInfo.url} `}-- ${comando}\nCodex:  codex mcp add blueprint ${padrao ? "" : `--env BLUEPRINT_URL=${mcpInfo.url} `}-- ${comando}\n\n2. No agente, peça para planejar ${novo ? "um blueprint novo" : `“${p.nome || "este blueprint"}”`}. Ids deste pedido: workspace_id ${ws}${novo ? "" : `, blueprint_id ${p.id}`}.\n\n3. A proposta aparece ao lado: você edita, desmarca o que não quer e só então aplica. Planos concluídos pelo agente (com evidência) são marcados aqui.`)
  ;
}
$("cu-abrir").onclick = () => { $("cu-corpo").textContent = textoComoUsar(); $("como-usar").showModal(); desenrolar($("como-usar"), $("como-usar")); }; // abre como as outras folhas: desenrola (ou escreve), já preenchida
$("cu-fechar").onclick = () => $("como-usar").close();
// estimativa em horas do plano (o servidor já validou de 0,5 a 16); sem número válido, o campo nem existe
const horasDe = (h) => (h != null && h !== "" && Number.isFinite(Number(h)) && Number(h) > 0 ? { horas: Number(h) } : {});
const etiquetaHoras = (t) => (t.horas ? `<span class="badge" title="Estimativa de trabalho do agente">~${t.horas} h</span>` : "");
function normalizaProposta(acoes, p) {
  const iso = (s, alt) => (ISO.test(String(s)) ? s : alt);
  return acoes.map((f) => {
    const ex = p.acoes.find((x) => x.id === f.id);
    const inicio = iso(f.inicio, hoje), fim = iso(f.fim, deDia(dia(inicio) + 30));
    return {
      id: ex ? ex.id : "", nome: String(f.nome || "").trim() || "Nova ação", inicio, fim, status: f.status === "andamento" ? "andamento" : "planejado", obs: String(f.observacao || ""), inc: true,
      planos: (f.planos || []).map((t) => ({ id: t.de || ex?.planos.some((x) => x.id === t.id) ? t.id : "", de: t.de || "", texto: String(t.texto || ""), descricao: String(t.descricao || ""), ...horasDe(t.horas) })),
      checkpoints: (f.checkpoints || []).map((m) => ({ id: m.de || ex?.checkpoints.some((x) => x.id === m.id) ? m.id : "", de: m.de || "", nome: String(m.nome || ""), data: iso(m.data, fim) })),
    };
  });
}
function htmlPropAcao(pf, i, p) {
  const ex = p.acoes.find((x) => x.id === pf.id);
  const semMudar = ex && ex.nome === pf.nome && ex.inicio === pf.inicio && ex.fim === pf.fim && pf.planos.every((t) => t.id && !t.de) && pf.checkpoints.every((m) => m.id && !m.de);
  const origem = (x) => { const o = x.de && p.acoes.find((y) => y.id === x.de); return o ? `<span class="badge mov" title="Movido de “${esc(o.nome)}”">← ${esc(o.nome)}</span>` : ""; }; // o que veio de outra ação
  const badge = !ex ? `<span class="badge nova">Nova</span>` : `<span class="badge">${semMudar ? "Sem mudança" : "Alterada"}</span>`;
  return `<div class="pf${pf.inc ? "" : " off"}" data-i="${i}">
    <div class="pf-topo"><input type="checkbox" data-pc="inc" aria-label="Incluir esta ação"${pf.inc ? " checked" : ""}>${badge}<input data-pc="nome" value="${esc(pf.nome)}" aria-label="Nome da ação"></div>
    <div class="grade3"><label class="f">Início<input type="date" data-pc="inicio" value="${esc(pf.inicio)}"></label><label class="f">Fim<input type="date" data-pc="fim" value="${esc(pf.fim)}"></label><label class="f">Situação<select data-pc="status"><option value="planejado"${pf.status === "planejado" ? " selected" : ""}>Planejada</option><option value="andamento"${pf.status === "andamento" ? " selected" : ""}>Em desenvolvimento</option></select></label></div>
    <label class="f">Instruções da ação<textarea data-pc="obs" rows="6" placeholder="O que fazer nesta ação: o roteiro">${esc(pf.obs)}</textarea></label>
    <div><p class="sub2"><span>Planos</span><span>${pf.planos.length}</span></p>${pf.planos.map((t, j) => `<div class="pi"><span class="pt">${t.de ? "↦" : t.id ? "·" : "+"}</span><input data-pc="plano" data-j="${j}" value="${esc(t.texto)}" aria-label="Plano">${origem(t)}${etiquetaHoras(t)}<button class="x" data-pact="del-t" data-j="${j}" type="button" aria-label="Remover plano">×</button></div>`).join("")}<div class="acoes" style="margin-top:8px"><button class="btn mini" data-pact="add-t" type="button">+ plano</button></div></div>
    <div><p class="sub2"><span>Checkpoints</span><span>${pf.checkpoints.length}</span></p>${pf.checkpoints.map((m, j) => `<div class="pi"><span class="pt">${m.de ? "↦" : m.id ? "·" : "+"}</span><input data-pc="cp-nome" data-j="${j}" value="${esc(m.nome)}" aria-label="Checkpoint">${origem(m)}<input type="date" data-pc="cp-data" data-j="${j}" value="${esc(m.data)}" aria-label="Data do checkpoint"><button class="x" data-pact="del-cp" data-j="${j}" type="button" aria-label="Remover checkpoint">×</button></div>`).join("")}<div class="acoes" style="margin-top:8px"><button class="btn mini" data-pact="add-cp" type="button">+ checkpoint</button></div></div>
  </div>`;
}
// apagar é só um pedido: cada item vem desmarcado e só some se o usuário marcar e aplicar
function htmlRemocoes(c, p) {
  const rem = c.remover || [];
  if (!rem.length) return "";
  const vao = new Set((c.proposta || []).filter((x) => x.inc).flatMap((f) => [...f.planos, ...f.checkpoints]).filter((x) => x.de).map((x) => x.id)); // o que sai da ação por ser movido não é apagado com ela
  const linha = (r, j) => {
    const f = p.acoes.find((x) => x.id === r.acaoId), nT = f ? f.planos.filter((t) => !vao.has(t.id)).length : 0, nM = f ? f.checkpoints.filter((m) => !vao.has(m.id)).length : 0;
    const o = r.planoId ? `Plano “${esc(r.plano)}”` : r.checkpointId ? `Checkpoint “${esc(r.checkpoint)}”` : `Ação inteira “${esc(r.acao)}”`;
    const em = r.planoId || r.checkpointId ? `na ação “${esc(r.acao)}”` : `leva junto ${nT} plano${nT === 1 ? "" : "s"} e ${nM} checkpoint${nM === 1 ? "" : "s"} que ficarem nela`;
    return `<label class="rem-item"><input type="checkbox" data-pc="rem-inc" data-j="${j}"${r.inc ? " checked" : ""}><span>${o}<small>${em}${r.motivo ? ` · ${esc(r.motivo)}` : ""}</small></span></label>`;
  };
  return `<div class="pf rem"><div class="pf-topo"><span class="badge">Apagar</span><b style="letter-spacing:.08em;text-transform:uppercase;font-size:12px">${rem.length} ${rem.length === 1 ? "item" : "itens"} para apagar</b><span style="margin-left:auto"><button class="btn mini" data-pact="rem-todas" type="button">Marcar</button> <button class="btn mini" data-pact="rem-nenhuma" type="button">Nenhum</button></span></div><p class="dica">Nada é apagado sem você marcar aqui e aplicar.</p><div>${rem.map(linha).join("")}</div></div>`;
}
function montaProposta() {
  const p = blueprintEd();
  if (!p) return;
  const c = conv(p.id), box = $("cl-prop"), topoAntes = box.querySelector(".cl-prop-lista")?.scrollTop || 0;
  if (!c.proposta && !c.emblema && !c.dadosBlueprint) {
    box.innerHTML = `<div class="cl-prop-topo"><b>Proposta</b></div><div class="cl-prop-lista"><div class="cl-prop-vazio"><p>Quando o seu agente enviar uma proposta pelo MCP (propor_blueprint), ele aparece aqui, já preenchido.</p><p>Você edita qualquer campo, desmarca o que não quer e só então aplica${emNovo() ? " e cria o blueprint" : " ao blueprint"}.</p></div></div>`;
    return;
  }
  const acoes = c.proposta || [];
  const n = acoes.filter((x) => x.inc).length, marca = c.emblema?.inc, rem = c.remover || [], nr = rem.filter((x) => x.inc).length;
  const nMov = acoes.reduce((a, f) => a + f.planos.filter((t) => t.de).length + f.checkpoints.filter((m) => m.de).length, 0);
  const resumo = [acoes.length && `${acoes.length} ação${acoes.length === 1 ? "" : "s"}`, nMov && `${nMov} movido${nMov === 1 ? "" : "s"}`, rem.length && `${rem.length} para apagar`].filter(Boolean).join(" · ");
  const rotuloBase = emNovo() ? `Criar blueprint${n ? ` com ${n} ação${n === 1 ? "" : "s"}` : ""}${marca ? " e a marca" : ""}` : n ? `Aplicar ${n} ação${n === 1 ? "" : "s"}${marca ? " e a marca" : ""} ao blueprint` : "Aplicar a marca ao blueprint";
  const rotuloAplicar = emNovo() ? rotuloBase : (n || marca || !nr ? rotuloBase : "Aplicar") + (nr ? ` · apagar ${nr}` : "");
  const cardDados = emNovo() && c.dadosBlueprint ? `<div class="pf"><label class="f">Nome proposto<input data-pc="blueprint-nome" value="${esc(c.dadosBlueprint.nome)}"></label><label class="f">Descrição<textarea data-pc="blueprint-descricao">${esc(c.dadosBlueprint.descricao)}</textarea></label></div>` : "";
  const cardMarca = c.emblema ? `<div class="pf${c.emblema.inc ? "" : " off"}" data-marca><div class="pf-topo"><input type="checkbox" data-pc="marca-inc" aria-label="Usar esta marca"${c.emblema.inc ? " checked" : ""}><span class="badge${p.emblema ? "" : " nova"}">${p.emblema ? "Nova marca" : "Marca"}</span><b style="letter-spacing:.08em;text-transform:uppercase;font-size:12px">Marca da folha</b></div><div class="marca-previa">${previaMarca(c.emblema.pecas)}</div><p class="dica">Desenhada com as peças do anatomy (${c.emblema.pecas.length} peças). Fica no centro da folha do blueprint, como a marca dela.</p></div>` : "";
  box.innerHTML = `<div class="cl-prop-topo"><b>Proposta${resumo ? ` · ${resumo}` : ""}</b>${acoes.length ? `<span><button class="btn mini" data-pact="todas" type="button">Marcar todas</button> <button class="btn mini" data-pact="nenhuma" type="button">Nenhuma</button></span>` : ""}</div>
  <div class="cl-prop-lista">${c.mensagem ? `<p class="cl-msg">${esc(c.mensagem)}</p>` : ""}${cardDados}${cardMarca}${acoes.map((pf, i) => htmlPropAcao(pf, i, p)).join("")}${htmlRemocoes(c, p)}</div>
  <div class="cl-prop-acoes"><button class="btn cheio" data-pact="aplicar" type="button"${n || marca || nr || c.dadosBlueprint?.nome.trim() ? "" : " disabled"}>${rotuloAplicar}</button><button class="btn" data-pact="descartar" type="button">Descartar</button></div>`;
  box.querySelector(".cl-prop-lista").scrollTop = topoAntes;
}
// o Claude "move" itens entre ações repetindo o texto na ação nova: tira a cópia pendente da ação de origem para não duplicar
function tiraPendente(p, destino, lista, campo, texto) {
  p.acoes.forEach((o) => {
    if (o === destino) return;
    const i = o[lista].findIndex((y) => !y.feito && norm(y[campo]) === norm(texto));
    if (i >= 0) { o[lista].splice(i, 1); o.status = efetiva(o).status; }
  });
}
// mover = tirar o item da ação de origem e pôr na de destino, com o mesmo id (o feito, o comentário e o histórico vão junto)
function moverItem(p, destino, lista, de, id, mudar) {
  const o = p.acoes.find((x) => x.id === de), i = o ? o[lista].findIndex((y) => y.id === id) : -1;
  if (i < 0) return 0;
  const [x] = o[lista].splice(i, 1);
  Object.assign(x, mudar);
  destino[lista].push(x);
  o.status = efetiva(o).status;
  return 1;
}
function removerItens(p, c) {
  let n = 0;
  (c.remover || []).filter((r) => r.inc).forEach((r) => {
    const o = p.acoes.find((x) => x.id === r.acaoId), lista = r.planoId ? "planos" : r.checkpointId ? "checkpoints" : "", id = r.planoId || r.checkpointId;
    if (!o) return;
    if (!lista) {
      (o.prototipos || []).forEach((x) => fetch("/api/mcp/prototipo/remover", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ prototipoId: x.id }) }).catch(() => {}));
      p.acoes.splice(p.acoes.indexOf(o), 1);
      return void n++;
    }
    const i = o[lista].findIndex((y) => y.id === id);
    if (i < 0) return;
    o[lista].splice(i, 1);
    o.status = efetiva(o).status;
    n++;
  });
  return n;
}
function aplicarAcoes(p, c) {
  let nf = 0, nt = 0, nm = 0, nmv = 0, marca = false;
  (c.proposta || []).filter((x) => x.inc).forEach((pf) => {
    let f = p.acoes.find((x) => x.id === pf.id);
    if (!f) {
      f = { id: novoId(), nome: pf.nome, inicio: pf.inicio, fim: pf.fim, status: pf.status, progresso: 0, obs: pf.obs, planos: [], checkpoints: [] };
      const pos = p.acoes.findIndex((x) => x.inicio > f.inicio); // entra na ordem do tempo, não no fim da lista
      p.acoes.splice(pos < 0 ? p.acoes.length : pos, 0, f);
      nf++;
    }
    else { f.nome = pf.nome; f.inicio = pf.inicio; f.fim = pf.fim; if (f.status !== "concluido") f.status = pf.status; if (pf.obs) f.obs = pf.obs; }
    pf.planos.filter((t) => t.texto.trim()).forEach((t) => {
      const x = f.planos.find((y) => y.id === t.id) || f.planos.find((y) => norm(y.texto) === norm(t.texto));
      if (x) { x.texto = t.texto; if (typeof t.descricao === "string") x.descricao = t.descricao; if (t.horas != null) x.horas = t.horas; } else if (t.de) nmv += moverItem(p, f, "planos", t.de, t.id, { texto: t.texto, descricao: String(t.descricao || ""), ...horasDe(t.horas) }); else { tiraPendente(p, f, "planos", "texto", t.texto); f.planos.push({ id: novoId(), texto: t.texto, descricao: String(t.descricao || ""), feito: false, ...horasDe(t.horas) }); nt++; }
    });
    pf.checkpoints.filter((m) => m.nome.trim()).forEach((m) => {
      const x = f.checkpoints.find((y) => y.id === m.id) || f.checkpoints.find((y) => norm(y.nome) === norm(m.nome));
      if (x) { x.nome = m.nome; x.data = m.data; } else if (m.de) nmv += moverItem(p, f, "checkpoints", m.de, m.id, { nome: m.nome, data: m.data }); else { tiraPendente(p, f, "checkpoints", "nome", m.nome); f.checkpoints.push({ id: novoId(), nome: m.nome, data: m.data, feito: false }); nm++; }
    });
    f.status = efetiva(f).status; // itens novos pendentes tiram a ação de "concluída" também no que fica gravado
    if (f.status === "planejado") f.progresso = 0;
  });
  if (c.emblema?.inc) { p.emblema = { pecas: c.emblema.pecas }; marca = true; }
  return { nf, nt, nm, nmv, nr: removerItens(p, c), marca };
}
function aplicarProposta() {
  const p = blueprintEd(), c = p && conv(p.id);
  if (!c || (!c.proposta && !c.emblema && !c.dadosBlueprint && !c.remover?.length)) return;
  if (emNovo() && c.dadosBlueprint) { p.nome = c.dadosBlueprint.nome.trim(); p.descricao = c.dadosBlueprint.descricao.trim(); }
  const { nf, nt, nm, nmv, nr, marca } = aplicarAcoes(p, c);
  c.proposta = null;
  c.remover = null;
  c.emblema = null;
  c.msgs.push({ papel: "sistema", texto: `Aplicado: ${nf} ação(s) nova(s), ${nt} plano(s) e ${nm} checkpoint(s) novos${nmv ? `, ${nmv} movido(s)` : ""}${nr ? `, ${nr} apagado(s)` : ""}${marca ? ", e a marca da folha" : ""}.` });
  if (emNovo()) montaMcp(); else { mudou(); montaHub(); }
  aviso(`Proposta aplicada: ${nf} ação(s), ${nt} plano(s), ${nm} checkpoint(s)${nmv ? `, ${nmv} movido(s)` : ""}${nr ? `, ${nr} apagado(s)` : ""}${marca ? " e a marca" : ""}.`);
}
function guardarChatDosBlueprints(c, blueprints) {
  blueprints.forEach((p, i) => {
    const chat = i === 0 ? c : { ...c, id: novoId(), msgs: c.msgs.map((m) => ({ ...m })) };
    if (i > 0) arquivo.chats.push(chat);
    chat.contexto = contextoChat(p.id);
    chat.proposta = chat.emblema = chat.dadosBlueprint = chat.lote = null;
    arquivo.chatsAtivos[chat.contexto] = chat.id;
  });
}
// tela "Novo blueprint": o rascunho só vira blueprint quando você cria; o Claude nunca cria blueprints por conta própria
function criarBlueprint(comProposta) {
  const dados = comProposta ? conv(NOVO).dadosBlueprint || rascunho : rascunho;
  const nome = dados.nome.trim();
  if (!nome) { if (!comProposta) $("nv-nome").focus(); return aviso("Defina o nome do blueprint antes de criar."); }
  if (comProposta) aplicarProposta();
  const p = { id: novoId(), nome, descricao: dados.descricao.trim(), obs: "", links: [], acoes: rascunho.acoes, emblema: rascunho.emblema };
  D.blueprints.push(p);
  guardarChatDosBlueprints(conv(NOVO), [p]);
  edProj = p.id;
  mudou();
  fechaTelas();
  irBlueprint(p.id);
}
function abreNovo() {
  rascunho = { id: NOVO, nome: "", descricao: "", obs: "", links: [], acoes: [], emblema: null };
  edProj = NOVO;
  $("nv-nome").value = ""; $("nv-desc").value = "";
  montaEditor();
  $("nv-nome").focus();
}
$("b-novo").onclick = abreNovo;
$("b-novo-proj").onclick = () => { editar(true, { novo: true }); };
$("nv-nome").addEventListener("input", (e) => { rascunho.nome = e.target.value; $("ed-proj").textContent = rascunho.nome || "Novo blueprint"; });
$("nv-desc").addEventListener("input", (e) => { rascunho.descricao = e.target.value; });
$("nv-vazio").onclick = () => criarBlueprint(false);
$("cl-prop").addEventListener("input", (e) => {
  salvar();

  const el = e.target.closest("[data-pc]"), p = blueprintEd();
  if (!el || !p) return;
  const c = conv(p.id), k = el.dataset.pc, j = +el.dataset.j;
  const atualizaBotao = () => { const n = (c.proposta || []).filter((x) => x.inc).length, b = $("cl-prop").querySelector("[data-pact=aplicar]"); if (b) b.disabled = !n && !c.emblema?.inc && !c.dadosBlueprint?.nome.trim() && !(c.remover || []).some((x) => x.inc); };
  if (k === "blueprint-nome" || k === "blueprint-descricao") { c.dadosBlueprint[k === "blueprint-nome" ? "nome" : "descricao"] = el.value; return atualizaBotao(); }
  if (k === "rem-inc") { c.remover[j].inc = el.checked; return montaProposta(); } // o rótulo do botão e a contagem dependem das marcas
  if (k === "marca-inc") { c.emblema.inc = el.checked; el.closest(".pf").classList.toggle("off", !el.checked); return atualizaBotao(); }
  const card = el.closest(".pf");
  if (!c.proposta || !card) return;
  const pf = c.proposta[+card.dataset.i];
  if (k === "inc") { pf.inc = el.checked; card.classList.toggle("off", !pf.inc); atualizaBotao(); }
  else if (k === "plano") pf.planos[j].texto = el.value;
  else if (k === "cp-nome") pf.checkpoints[j].nome = el.value;
  else if (k === "cp-data") pf.checkpoints[j].data = el.value;
  else pf[k] = el.value;
});
$("cl-prop").addEventListener("click", (e) => {
  const b = e.target.closest("[data-pact]"), p = blueprintEd();
  if (!b || !p) return;
  const c = conv(p.id), a = b.dataset.pact, card = b.closest(".pf"), pf = card && c.proposta?.[+card.dataset.i], j = +b.dataset.j;
  salvar();
  if (a === "aplicar") return emNovo() ? criarBlueprint(true) : aplicarProposta();
  if (a === "descartar") { c.lote = null; c.proposta = null; c.remover = null; c.emblema = null; c.dadosBlueprint = null; c.mensagem = ""; salvar(); return montaMcp(); }
  if (a === "rem-todas" || a === "rem-nenhuma") (c.remover || []).forEach((x) => { x.inc = a === "rem-todas"; });
  else if (a === "todas" || a === "nenhuma") c.proposta.forEach((x) => { x.inc = a === "todas"; });
  else if (a === "add-t") pf.planos.push({ id: "", texto: "" });
  else if (a === "del-t") pf.planos.splice(j, 1);
  else if (a === "add-cp") pf.checkpoints.push({ id: "", nome: "", data: pf.fim });
  else if (a === "del-cp") pf.checkpoints.splice(j, 1);
  montaProposta();
});

