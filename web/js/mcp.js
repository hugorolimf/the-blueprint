// ---- MCP: o agente do usuário (Claude Code, Codex…) fala com o app pelo servidor MCP (blueprint.exe --mcp); o app não abre agentes ----
// A página busca a fila do servidor: as propostas dos agentes chegam para revisar, e o andamento (plano concluído com evidência, bloqueio) aparece aqui.
const mcpFeed = [], mcpVistos = new Set();
let mcpFalhou = false, mcpInfo = null, mcpSse = null, mcpTimer = 0, mcpWsAnterior = null;
// um só relógio: toda consulta cancela a pendente (o aviso do SSE e o laço de reserva não criam laços paralelos)
function mcpAgenda(ms) { clearTimeout(mcpTimer); mcpTimer = setTimeout(mcpConsulta, ms); }
// Servidor Rust: avisa por SSE quando há item novo, e a página só consulta a fila nesse momento (zero trabalho em repouso).
// Servidor Node (sem /api/mcp/eventos) ou conexão caída: volta a consultar a cada 2,5 s.
function mcpAssina() {
  if (typeof EventSource === "undefined") return;
  const es = new EventSource("/api/mcp/eventos");
  es.onopen = () => { mcpSse = es; mcpConsulta(); };
  es.addEventListener("fila", () => mcpConsulta());
  es.addEventListener("agentes", () => agentesBusca()); // a presença dos agentes mudou: o painel atualiza o cartão que mudou
  es.onerror = () => { mcpSse = null; mcpConsulta(); };
}
// a fila só é aplicada ao workspace aberto: ao entrar num workspace (ou quando os dados chegam depois da conexão SSE), consulta de novo,
// senão o que o agente marcou antes de a página abrir ficaria esperando o próximo aviso
function mcpWorkspaceMudou() {
  const id = workspaceAtual?.id || null;
  if (id === mcpWsAnterior) return;
  mcpWsAnterior = id;
  if (id) mcpAgenda(0);
}
function mcpProposta(i) {
  const p = i.blueprintId ? D.blueprints.find((x) => x.id === i.blueprintId) : null;
  if (i.blueprintId && !p) return false;
  const c = conv(p ? p.id : NOVO);
  c.proposta = normalizaProposta(i.proposta.acoes, p || { acoes: [] });
  c.remover = (i.proposta.remover || []).map((r) => ({ ...r, inc: false })); // apagar nunca vem marcado: o usuário aprova item a item
  c.dadosBlueprint = p ? null : { nome: i.proposta.blueprint.nome, descricao: i.proposta.blueprint.descricao };
  c.mensagem = i.proposta.mensagem || "";
  c.emblema = c.lote = null;
  return true;
}
function mcpPlano(i) {
  return D.blueprints.find((x) => x.id === i.blueprintId)?.acoes.find((f) => f.id === i.acaoId)?.planos.find((t) => t.id === i.planoId) || null;
}
async function mcpAplica(itens) {
  const ws = workspaceAtual?.id;
  if (!ws || !carregado) return; // sem workspace aberto, a fila espera: nada é aplicado a outro workspace
  const meus = itens.filter((i) => i.workspaceId === ws);
  let muda = false, viu = false;
  const chave = (x) => `${x.id}|${x.em}`; // a fila é da memória do servidor: ao reiniciar, os números voltam a 1, e só o id faria a página ignorar eventos novos
  for (const i of meus.filter((x) => !mcpVistos.has(chave(x)))) {
    mcpVistos.add(chave(i));
    viu = true;
    agenteAgora ={ ws, blueprintId: i.blueprintId, acaoId: i.acaoId || "", planoId: i.planoId || "", tipo: i.tipo, texto: i.tipo === "proposta" ? "Proposta enviada: revise em ✦ Planejar." : i.texto, em: Date.now() };
    const nomeProj = D.blueprints.find((x) => x.id === i.blueprintId)?.nome;
    if (i.tipo === "progresso" && i.planoId) { // o primeiro aviso de progresso com o plano marca o início do tempo real
      const t0 = mcpPlano(i);
      if (t0 && !t0.feito && !t0.agente?.inicioEm) { t0.agente = { ...normalizaAgente(t0.agente || {}), inicioEm: i.em }; muda = true; }
    }
    if (i.tipo === "proposta") {
      if (!mcpProposta(i)) continue;
      mcpFeed.push({ tipo: "proposta", texto: `Proposta recebida para ${i.blueprintId ? `“${nomeProj}”` : "um blueprint novo"}: revise ao lado e aplique.` });
      aviso("O seu agente enviou uma proposta: abra ✦ Planejar para revisar.");
      muda = true;
    } else if (i.tipo === "prototipo") {
      const f = D.blueprints.find((x) => x.id === i.blueprintId)?.acoes.find((x) => x.id === i.acaoId);
      if (!f) continue;
      f.prototipos ||= [];
      const ex = f.prototipos.find((x) => x.id === i.prototipoId);
      if (ex) { ex.titulo = i.titulo; ex.em = i.em; } else f.prototipos.push({ id: i.prototipoId, titulo: i.titulo, em: i.em });
      mcpFeed.push({ tipo: "prototipo", texto: `Protótipo “${i.titulo}” ${ex ? "atualizado" : "anexado"} na ação “${f.nome}”: veja no ícone de protótipo da ação.` });
      aviso(`O seu agente ${ex ? "atualizou" : "anexou"} o protótipo “${i.titulo}”.`);
      if (protoAlvo?.fid === i.acaoId) mostraProto(); // visor aberto nesta ação: atualiza título, lista e quadro
      muda = true;
    } else if (i.tipo === "checkpoint-feito" || i.tipo === "checkpoint-nao-feito") { // checkpoint da ação: só vira feito com a evidência, que fica no próprio checkpoint
      const m = D.blueprints.find((x) => x.id === i.blueprintId)?.acoes.find((f) => f.id === i.acaoId)?.checkpoints.find((x) => x.id === i.checkpointId);
      if (!m || m.feito) continue;
      const ok = i.tipo === "checkpoint-feito";
      if (ok) { m.feito = true; m.evidencia = i.texto; m.feitoEm = new Date().toISOString(); muda = true; }
      mcpFeed.push({ tipo: i.tipo, texto: `${ok ? "✓" : "✗"} checkpoint “${i.checkpoint}” — ${i.texto}` });
    } else if (i.tipo === "concluir" || i.tipo === "nao-concluida") {
      const t = mcpPlano(i);
      if (!t || t.feito) continue;
      t.agente ||= normalizaAgente({});
      if (i.tipo === "concluir") { t.agente.estado = "revisao"; adicionarComentarioAgente(t, i.texto, "relatorio", `mcp${i.id}`); confirmarConclusaoAgente(t, true); t.agente.feitoEm = i.em; }
      else { t.agente.estado = "pendente"; adicionarComentarioAgente(t, `Não concluído: ${i.texto}`, "comentario", `mcp${i.id}`); }
      mcpFeed.push({ tipo: i.tipo, texto: `${i.tipo === "concluir" ? "✓" : "✗"} ${i.plano} — ${i.texto}` });
      muda = true;
    } else {
      mcpFeed.push({ tipo: i.tipo, texto: `${i.tipo === "bloqueio" ? "⚠ " : ""}${nomeProj ? `${nomeProj}: ` : ""}${i.texto}` });
      if (i.tipo === "bloqueio") aviso(`O seu agente travou: ${umaLinha(i.texto, 120)}`);
    }
  }
  mcpFeed.splice(0, Math.max(0, mcpFeed.length - 50));
  if (!meus.length) return;
  if (muda) mudou();
  else if (viu) render(); // o robô mudou de lugar ou de fala
  if (!hub.hidden) montaHub();
  try { await fetch("/api/mcp/fila/ack", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ ids: meus.map((x) => x.id) }) }); } catch {}
}
async function mcpConsulta() {
  try {
    const r = await fetch("/api/mcp/fila", { cache: "no-store" });
    if (!r.ok) throw new Error(r.status);
    const { itens } = await r.json();
    const voltou = mcpFalhou;
    mcpFalhou = false;
    if (itens.length) await mcpAplica(itens);
    else if (voltou && !hub.hidden) montaMcp();
    if (agenteAgora && Date.now() - agenteAgora.em >= ROBO_VIDA) { agenteAgora = null; render(); }
  } catch { if (!mcpFalhou) { mcpFalhou = true; if (!hub.hidden) montaMcp(); } }
  // com SSE aberto só falta um relógio: esconder o robô quando a fala dele vence
  const proxima = mcpSse ? (agenteAgora ? Math.max(1000, ROBO_VIDA - (Date.now() - agenteAgora.em)) : 0) : 2500;
  if (!mcpSse) agentesBusca(); // sem SSE o painel também precisa do laço de reserva
  if (proxima) mcpAgenda(proxima);
}

