// ---- registros do agente: o que ele JÁ fez fora do plano (registrar_trabalho). NÃO é proposta: nada muda nos dados antes de o usuário clicar em "Adicionar ao blueprint". ----
// Cada registro espera aqui (e na fila do servidor, só em memória, até a confirmação por número e criadaEm) com os planos e checkpoints já marcados como feitos;
// o usuário pode desmarcar o que não aceita. Quem desenha é registros-vista.js; aqui ficam o estado e a aplicação aos dados.
const registrosFila = []; // { blueprintId, numero, criadaEm, em, mensagem, acoes: [{ id, nome, inicio, fim, status, observacao, projeto, inc, planos, checkpoints }] }

const registroChave = (r) => `${r.blueprintId}|${r.numero}|${r.criadaEm}`;
const registrosDe = (blueprintId) => registrosFila.filter((r) => r.blueprintId === blueprintId).sort((a, b) => a.numero - b.numero);

// o item `i` da fila do servidor vira um registro da página. Falso se já está aqui (a página recarregou e o servidor mandou de novo) ou se o blueprint não existe mais.
function registroRecebe(i) {
  const r = { blueprintId: i.blueprintId, numero: i.numero, criadaEm: i.criadaEm, em: i.em, mensagem: String(i.registro?.mensagem || "") };
  if (registrosFila.some((x) => registroChave(x) === registroChave(r))) return false;
  if (!D.blueprints.some((x) => x.id === i.blueprintId)) { registroConfirma(r); return false; }
  r.acoes = (i.registro?.acoes || []).map((a) => ({
    ...a, inc: true,
    planos: (a.planos || []).map((t) => ({ ...t, feito: t.feito === true })),
    checkpoints: (a.checkpoints || []).map((m) => ({ ...m, feito: m.feito === true })),
  }));
  registrosFila.push(r);
  return true;
}
// o registro saiu (adicionado ou descartado): o servidor o tira da fila dele (pela chave inteira, como nas propostas)
function registroConfirma(r) {
  fetch("/api/mcp/fila/ack", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ ids: [], registros: [{ workspaceId: workspaceAtual?.id || "sm", blueprintId: r.blueprintId, numero: r.numero, criadaEm: r.criadaEm }] }) }).catch(() => {});
}
function registroRetira(r) {
  registrosFila.splice(registrosFila.indexOf(r), 1);
  registroConfirma(r);
}
// plano feito pelo registro: a evidência vira o comentário do plano, como em marcar_concluido
function registroMarcaPlano(t, evidencia, r) {
  t.agente ||= normalizaAgente({});
  t.agente.estado = "revisao";
  adicionarComentarioAgente(t, evidencia, "relatorio", `registro${r.criadaEm}`); // a chave é a criadaEm: o número volta a 1 quando o app reinicia
  confirmarConclusaoAgente(t, true);
  t.agente.feitoEm = r.em;
}
// o que se conta ao usuário depois de adicionar um registro (o clique em "Adicionar ao blueprint" e o aplicar sozinho usam o mesmo texto)
const textoRegistroAdicionado = (res) => tr("Registro adicionado: {feitos} feito(s){nf}{nt}{nc}.", { feitos: res.feitos, nf: res.nf ? tr(", {n} ação(ões) nova(s)", { n: res.nf }) : "", nt: res.nt ? tr(", {n} plano(s) novo(s)", { n: res.nt }) : "", nc: res.nc ? tr(", {n} checkpoint(s) novo(s)", { n: res.nc }) : "" });
// grava o registro no blueprint: ação nova entra na ordem do tempo; plano e checkpoint novos entram pendentes e só ficam feitos se o registro (e o usuário) os marcaram.
// Devolve { nf, nt, nc, feitos } ou null se o blueprint sumiu.
function registroAdiciona(r) {
  const p = D.blueprints.find((x) => x.id === r.blueprintId);
  if (!p) return null;
  let nf = 0, nt = 0, nc = 0, feitos = 0;
  for (const pf of r.acoes.filter((x) => x.inc)) {
    let f = pf.id && p.acoes.find((x) => x.id === pf.id);
    if (!f) {
      f = { id: novoId(), nome: pf.nome, inicio: pf.inicio, fim: pf.fim, status: pf.status || "planejado", progresso: 0, obs: pf.observacao || "", planos: [], checkpoints: [] };
      const pos = p.acoes.findIndex((x) => x.inicio > f.inicio);
      p.acoes.splice(pos < 0 ? p.acoes.length : pos, 0, f);
      const projetoId = idDoProjetoEscolhido(p, typeof pf.projeto === "string" ? pf.projeto : "");
      if (projetoId) f.projetoId = projetoId;
      nf++;
    }
    for (const t of pf.planos) {
      let x = t.id && f.planos.find((y) => y.id === t.id);
      if (!x) { x = { id: novoId(), texto: t.texto, descricao: String(t.descricao || ""), feito: false, ...horasDe(t.horas) }; f.planos.push(x); nt++; }
      if (t.feito && !x.feito) { registroMarcaPlano(x, t.evidencia, r); feitos++; }
    }
    for (const m of pf.checkpoints) {
      let x = m.id && f.checkpoints.find((y) => y.id === m.id);
      if (!x) { x = { id: novoId(), nome: m.nome, data: m.data, feito: false }; f.checkpoints.push(x); nc++; }
      if (m.feito && !x.feito) { x.feito = true; x.evidencia = m.evidencia || ""; x.feitoEm = new Date().toISOString(); feitos++; }
    }
    f.status = efetiva(f).status; // o que ficou pendente mantém a ação aberta; tudo feito a conclui
    if (f.status === "planejado") f.progresso = 0;
  }
  return { nf, nt, nc, feitos };
}
