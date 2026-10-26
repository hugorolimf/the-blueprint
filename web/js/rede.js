// ---- rede: carregar, salvar e acompanhar mudanças de outras pessoas ------------------------
const estado = $("estado");
const dizer = (texto, e = "") => { for (const el of [estado, $("ag-estado")]) { el.textContent = texto; el.dataset.e = e; } };
let avisoT = 0;
const aviso = (texto) => { for (const id of ["avisoed", "ag-aviso"]) $(id).textContent = texto; clearTimeout(avisoT); avisoT = setTimeout(() => { for (const id of ["avisoed", "ag-aviso"]) $(id).textContent = ""; }, 7000); };
const ESTADOS_AGENTE = ["pendente", "executando", "revisao", "interrompido", "falhou", "concluido"];
function normalizaAgente(a = {}) {
  const estado = ESTADOS_AGENTE.includes(a.estado) ? a.estado : "pendente";
  return {
    estado,
    alteracoes: (Array.isArray(a.alteracoes) ? a.alteracoes : []).filter((x) => typeof x === "string").slice(-100).map((x) => x.slice(0, 500)),
    sessaoId: typeof a.sessaoId === "string" ? a.sessaoId.slice(0, 80) : "",
    // medição do tempo real do plano: primeiro aviso de progresso e conclusão (só existem depois de medidos)
    ...["inicioEm", "feitoEm"].filter((k) => typeof a[k] === "string" && a[k]).reduce((o, k) => ({ ...o, [k]: a[k].slice(0, 40) }), {}),
    atualizadoEm: typeof a.atualizadoEm === "string" ? a.atualizadoEm.slice(0, 40) : "",
    comentarios: (Array.isArray(a.comentarios) ? a.comentarios : []).filter((c) => c && typeof c.texto === "string").slice(-20).map((c) => ({ texto: c.texto.slice(0, 12000), em: typeof c.em === "string" ? c.em.slice(0, 40) : "", tipo: c.tipo === "relatorio" ? "relatorio" : "comentario", sessaoId: typeof c.sessaoId === "string" ? c.sessaoId.slice(0, 80) : "" })),
  };
}
const limpa = (d) => {
  const o = { empresa: "", titulo: "", rev: "A", exemplo: false, ...d };
  o.blueprints = (Array.isArray(d?.blueprints) ? d.blueprints : []).map((p) => ({
    id: p.id || novoId(), nome: p.nome || "", descricao: p.descricao || "", obs: p.obs || "", emblema: p.emblema && Array.isArray(p.emblema.pecas) ? { pecas: p.emblema.pecas } : null,
    links: (Array.isArray(p.links) ? p.links : []).map((l) => ({ id: l.id || novoId(), rotulo: l.rotulo || "", url: l.url || "" })),
    // projetos (opcionais): só existem os campos quando há projeto, então os dados antigos abrem e voltam iguais
    ...(Array.isArray(p.projetos) && p.projetos.some((x) => x && x.id) ? { projetos: p.projetos.filter((x) => x && x.id).map((x) => ({ id: String(x.id), nome: String(x.nome || "") })) } : {}),
    acoes: (Array.isArray(p.acoes) ? p.acoes : []).map((f) => ({
      id: f.id || novoId(), nome: f.nome || "", inicio: f.inicio || "", fim: f.fim || "", obs: f.obs || "", ...(f.projetoId ? { projetoId: String(f.projetoId) } : {}),
      status: ROTULO[f.status] ? f.status : "planejado", progresso: Number(f.progresso) || 0,
      planos: (Array.isArray(f.planos) ? f.planos : []).map((t) => ({ id: t.id || novoId(), texto: t.texto || "", ...(typeof t.descricao === "string" ? { descricao: t.descricao } : {}), feito: !!t.feito, ...(Number.isFinite(t.horas) && t.horas > 0 ? { horas: t.horas } : {}), ...(t.agente && typeof t.agente === "object" ? { agente: normalizaAgente(t.agente) } : {}) })),
      checkpoints: (Array.isArray(f.checkpoints) ? f.checkpoints : []).map((m) => ({ id: m.id || novoId(), nome: m.nome || "", data: m.data || "", feito: !!m.feito, ...(m.feito && typeof m.evidencia === "string" ? { evidencia: m.evidencia.slice(0, 2000), feitoEm: String(m.feitoEm || "") } : {}) })),
      prototipos: (Array.isArray(f.prototipos) ? f.prototipos : []).filter((x) => x && /^[a-f0-9]{16}$/.test(x.id)).map((x) => ({ id: x.id, titulo: String(x.titulo || "Protótipo").slice(0, 120), em: String(x.em || "").slice(0, 40) })),
    })),
  }));
  return o;
};
const preparaArquivo = (d) => {
  const arquivo = limpa(d);
  arquivo.workspaces = (Array.isArray(arquivo.workspaces) ? arquivo.workspaces : []).map(limpa);
  arquivo.chats = (Array.isArray(arquivo.chats) ? arquivo.chats : []).filter((c) => c && typeof c.id === "string" && typeof c.contexto === "string").map((c) => ({ ...c, msgs: (Array.isArray(c.msgs) ? c.msgs : []).filter((m) => m && typeof m.texto === "string"), pensando: false, consulta: c.consulta || [] }));
  return arquivo;
};
let concluidosAntes = new Set();
const concluidosAgora = () => new Set(D.blueprints.filter(projConcluido).map((p) => p.id));
let ultimo = ""; // o data.json como o servidor o tem (ou como esta janela o gravou por último): a BASE para juntar edições (sincronia.js)
let versaoDados = ""; // a versão desse texto; vai em `If-Match` e o servidor recusa (409) gravar sobre dados que outra janela já mudou
async function carregar() {
  if (arr?.ativo || salvandoPerfil || salvandoWorkspace || timer || arquivo.chats?.some((c) => c.pensando)) return;
  try {
    await emblemasProntos;
    const r = await fetch("/api/dados", { cache: "no-store" });
    if (!r.ok) throw new Error(r.status);
    const texto = await r.text();
    if (salvandoPerfil || salvandoWorkspace || timer || arquivo.chats?.some((c) => c.pensando)) return;
    falhou = false;
    versaoDados = r.headers.get("x-dados-versao") || versaoDados;
    if (texto === ultimo && carregado) { if (hoje !== (hojeSimulado || local(new Date()))) render(); return; }
    ultimo = texto;
    arquivo = preparaArquivo(JSON.parse(texto));
    workspaceAtual = workspaceDaRota(arquivo, location.hash);
    D = workspaceAtual?.dados || arquivo;
    carregado = true;
    $("login-form").querySelector("button").disabled = false;
    if (!editandoNome) {
      nomeUsuario = typeof arquivo.usuario?.nome === "string" ? arquivo.usuario.nome.trim().slice(0, 80) : "";
      mostrarPerfil();
    }
    concluidosAntes = concluidosAgora();
    render();
  } catch {
    $("login-erro").textContent = "Não foi possível carregar os dados. Tentando novamente…";
    if (!falhou) { falhou = true; render(); }
  }
}
const avisosDoMonitor = new Map();
function avisarMonitor(tipo, blueprintId, checkpointId = "") {
  if (avisosDoMonitor.size >= 100) return;
  const evento = { tipo, workspaceId: workspaceAtual.id, blueprintId, checkpointId };
  avisosDoMonitor.set(JSON.stringify(evento), evento);
}
const corpoDosDados = () => JSON.stringify({ ...arquivo, chats: (arquivo.chats || []).map(({ pensando, ...c }) => c) }, null, 2);
// Outra janela (ou um script) gravou antes: traz o que o servidor tem, junta com o que está nesta tela (sincronia.js) e mostra o resultado.
async function reconciliar(corpoMeu) {
  const r = await fetch("/api/dados", { cache: "no-store" });
  if (!r.ok) throw new Error(r.status);
  const texto = await r.text();
  const deles = JSON.parse(texto);
  arquivo = preparaArquivo(sincMesclar(ultimo ? JSON.parse(ultimo) : deles, JSON.parse(corpoMeu), deles)); // sem base, tudo o que difere é meu (como era antes)
  workspaceAtual = workspaceDaRota(arquivo, location.hash);
  D = workspaceAtual?.dados || arquivo;
  ultimo = texto;
  versaoDados = r.headers.get("x-dados-versao") || "";
  concluidosAntes = concluidosAgora();
  render();
}
async function gravarDados() {
  const avisos = [...avisosDoMonitor.entries()];
  let corpo, r;
  for (let tentativa = 0; tentativa < 4; tentativa++) {
    corpo = corpoDosDados();
    r = await fetch("/api/dados", { method: "PUT", headers: { "content-type": "application/json", ...(versaoDados ? { "if-match": versaoDados } : {}) }, body: corpo });
    if (r.status !== 409) break;
    await reconciliar(corpo);
  }
  if (!r.ok) throw new Error(r.status);
  versaoDados = r.headers.get("x-dados-versao") || versaoDados;
  ultimo = corpo;
  // Só avisar depois de persistir: receber a proposta na fila ainda não significa aplicá-la.
  await Promise.all(avisos.map(async ([chave, evento]) => {
    avisosDoMonitor.delete(chave);
    try { await fetch("/api/mcp/aviso", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(evento) }); } catch {}
  }));
}
let timer = 0;
function salvar() {
  dizer("salvando…");
  clearTimeout(timer);
  timer = setTimeout(async () => {
    const gravacao = timer;
    try {
      await gravarDados();
      dizer("salvo ✓");
    } catch { dizer("erro ao salvar", "erro"); }
    finally { if (timer === gravacao) timer = 0; }
  }, 400);
}
const mudou = () => {
  render();
  salvar();
  const agora = concluidosAgora();
  D.blueprints.forEach((p) => {
    if (agora.has(p.id) && !concluidosAntes.has(p.id)) aviso(`“${p.nome || "Sem nome"}” chegou a 100%: está em Concluídos na lista de blueprints.`);
    if (!agora.has(p.id) && concluidosAntes.has(p.id)) aviso(`“${p.nome || "Sem nome"}” voltou para Em andamento.`);
  });
  concluidosAntes = agora;
};
// Tempo real: o servidor avisa (SSE "dados") a cada gravação; se não foi esta janela, ela busca na hora, em vez de ficar com a tela velha e depois
// sobrescrever o que as outras gravaram. Quem está digitando não tem a tela trocada: a atualização espera o foco sair do campo.
let dadosAdiados = false;
const digitando = () => !!document.activeElement?.matches?.("input, textarea, select, [contenteditable]");
function dadosDeOutraJanela(e) {
  let n = "";
  try { n = String(JSON.parse(e.data).versao); } catch {}
  if (n && n === versaoDados) return; // fui eu que gravei
  if (digitando()) { dadosAdiados = true; return; }
  carregar();
}
document.addEventListener("focusout", () => {
  if (dadosAdiados) setTimeout(() => { if (dadosAdiados && !digitando()) { dadosAdiados = false; carregar(); } }, 200);
});
function adicionarComentarioAgente(plano, texto, tipo, sessaoId = "") {
  if (!texto) return;
  plano.agente ||= normalizaAgente({});
  if (sessaoId && plano.agente.comentarios.some((c) => c.tipo === tipo && c.sessaoId === sessaoId)) return;
  plano.agente.comentarios.push({ texto: String(texto).slice(0, 12000), em: new Date().toISOString(), tipo, sessaoId });
  plano.agente.comentarios = plano.agente.comentarios.slice(-20);
}
function confirmarConclusaoAgente(plano, porAgente = false) {
  if (!plano?.agente || plano.agente.estado !== "revisao") return false;
  plano.feito = true;
  plano.agente.estado = "concluido";
  plano.agente.atualizadoEm = new Date().toISOString();
  adicionarComentarioAgente(plano, porAgente ? "Concluído pelo agente, com base no resultado da execução." : "Relatório revisado pelo usuário; plano marcado como concluído.", "comentario", "");
  return true;
}
