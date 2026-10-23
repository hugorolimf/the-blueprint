// ---- agente na folha: estado do que o agente do usuário disse por último e a faixa #robo-agente; vem só da fila do MCP (o app não executa nada).
// O desenho é o Claude (claude-camada.js); o robozinho antigo saiu. ----
const ROBO_VIDA = 10 * 60 * 1000; // sem notícia por 10 min, o robô sai da folha
let agenteAgora = null, roboRolar = false; // { ws, blueprintId, acaoId, planoId, tipo, texto, em }: só na memória da página
const agenteAqui = () => (agenteAgora && !agenteAgora.visto && agenteAgora.ws === workspaceAtual?.id && Date.now() - agenteAgora.em < ROBO_VIDA ? agenteAgora : null); // `visto`: o usuário já clicou no aviso
// uma figura do Claude por agente ativo do blueprint (a lista de agentes.js); o último evento da fila só entra se nenhum agente cobre o mesmo plano
const agenteEvento = (a) => ({ tipo: a.estado === "bloqueado" ? "bloqueio" : a.estado === "concluido" ? "concluir" : "progresso", blueprintId: a.blueprintId, acaoId: a.acaoId || "", planoId: a.planoId || "", agente: a.agente, papel: a.papel, estado: a.estado, onde: a.onde, em: Date.parse(a.em) || 0, texto: a.mensagem || "" });
function agentesDoBlueprint(id) {
  const lista = agentesAtivos().filter((a) => a.blueprintId === id).map(agenteEvento), ev = agenteAqui();
  // o evento da fila só vira figura quando o servidor não conhece NENHUM agente neste blueprint (dispensado ou não) (ex.: a lista ainda não chegou, ou quem concluiu não se identificou); com agente, a figura dele já mostra tudo e o evento seria um boneco a mais
  if (ev?.blueprintId === id && !agentesLista.some((x) => x.blueprintId === id)) lista.push(ev); // nem os dispensados: senão "Concluído" traria o evento de volta como figura sem nome
  return lista; // o limite de figuras (e o contador "+N") é de claude-logica.js
}
const roboClasse = (a) => (a.tipo === "bloqueio" || a.tipo === "nao-concluida" || a.tipo === "checkpoint-nao-feito" ? " erro" : a.tipo === "concluir" || a.tipo === "checkpoint-feito" ? " ok" : "");
const ROBO_ROTULO = { progresso: "trabalhando", concluir: "concluiu", "nao-concluida": "não concluiu", "checkpoint-feito": "atingiu um checkpoint", "checkpoint-nao-feito": "não atingiu um checkpoint", bloqueio: "travou", proposta: "enviou uma proposta", prototipo: "anexou um protótipo" };
function montaRobo() {
  claudeCamadaAtualiza(); // as figuras da folha seguem o estado: toda vez que a folha é desenhada
  const a = agenteAqui(), el = $("robo-agente");
  el.hidden = !a || !!a.visto; // depois de clicar e ir até ele, o aviso some; o robô fica na folha até chegar notícia nova
  if (el.hidden) return;
  const p = D.blueprints.find((x) => x.id === a.blueprintId), f = p?.acoes.find((x) => x.id === a.acaoId), t = f?.planos.find((x) => x.id === a.planoId);
  const onde = [p?.nome, f?.nome, t?.texto].filter(Boolean).join(" › ") || "neste workspace";
  el.className = roboClasse(a).trim();
  el.title = `${onde}\n${a.texto}${p ? "\nClique para ir até ele." : ""}`;
  el.innerHTML = `${claudeMiniatura(a)}<div><b>Agente ${esc(ROBO_ROTULO[a.tipo] || "")} · ${esc(onde)}</b><span>${esc(umaLinha(a.texto, 200))}</span></div>`;
}
// clicar no robô abre a folhinha com o que o agente está fazendo (só leitura: o app não manda trabalho a ninguém)
function abreNotaRobo(el) {
  const nome = el.dataset.roboNome, ag = agentesAtivos().find((x) => x.agente === nome), a = ag || agenteAqui();
  if (!a) return;
  const concluiu = ag?.estado === "concluido"; // concluído e ainda não visto: a folhinha traz a mensagem de conclusão
  const p = D.blueprints.find((x) => x.id === a.blueprintId), f = p?.acoes.find((x) => x.id === a.acaoId), t = f?.planos.find((x) => x.id === a.planoId), pct = Number.isFinite(ag?.percentual) ? ` · ${ag.percentual}%` : "";
  const linhas = [ag && `${ESTADO_AG[ag.estado] || ag.estado || "Trabalhando"}${ag.papel ? ` · ${PAPEL_AG[ag.papel] || ag.papel}` : ""}${pct}`, `${concluiu ? "Mensagem de conclusão" : "Agora"}: ${ag?.mensagem || a.texto || "sem detalhe"}${ag ? ` · ${haQuanto(ag.em)}${Date.now() - Date.parse(ag.em) > 120000 ? " (sem notícia nova)" : ""}` : ""}`, t && `Plano: ${t.texto}`, f && `Ação: ${numeroDaAcao(p, f)} ${f.nome || ""}`.trim(), ag?.onde?.length && `Arquivos: ${ag.onde.join(" · ")}`].filter(Boolean);
  // Concluído = o usuário já viu que acabou: o concluído sai da folha (e a figura de um evento sem agente identificado é dispensada). Fechar só fecha a folhinha: a figura fica.
  const visto = () => { fechaNota(); if (ag) agenteMarcaVisto(ag); else { a.visto = true; render(); } };
  const parado = ["aguardando", "ocioso"].includes(ag?.estado); // aguardando ou em repouso: o mesmo botão o dispensa até ele avisar de novo
  const dispensavel = concluiu || parado || (!ag && a === agenteAgora && a.tipo === "concluir");
  abreNota({ rotulo: concluiu ? "Agente concluiu" : "Agente trabalhando", titulo: nome || "Agente", texto: linhas.join("\n"), acoes: dispensavel ? [{ rotulo: "Concluído", fn: visto }] : [] }, el);
}
$("robo-agente").onclick = () => {
  const a = agenteAqui(), pid = a && D.blueprints.some((x) => x.id === a.blueprintId) ? a.blueprintId : "";
  if (a) { a.visto = true; montaRobo(); }
  if (!pid) return;
  abertos[pid] = true; // abre as ações para a figura aparecer ao lado do plano
  roboRolar = true;
  if (tela === pid) render(); else irBlueprint(pid);
};

