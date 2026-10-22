// ---- robô do agente: mostra onde o agente do usuário está e o que disse por último; vem só da fila do MCP (o app não executa nada) ----
const ROBO_VIDA = 10 * 60 * 1000; // sem notícia por 10 min, o robô sai da folha
let agenteAgora = null, roboRolar = false; // { ws, blueprintId, acaoId, planoId, tipo, texto, em }: só na memória da página
const agenteAqui = () => (agenteAgora && agenteAgora.ws === workspaceAtual?.id && Date.now() - agenteAgora.em < ROBO_VIDA ? agenteAgora : null);
const roboClasse = (a) => (a.tipo === "bloqueio" || a.tipo === "nao-concluida" || a.tipo === "checkpoint-nao-feito" ? " erro" : a.tipo === "concluir" || a.tipo === "checkpoint-feito" ? " ok" : "");
const roboSvg = (x, y, a) => `<g class="robo${roboClasse(a)}" data-robo aria-hidden="true"><path class="ag-an" d="M${x + 8} ${y + 2}V${y - 1.5}"/><circle class="ag-pt" cx="${x + 8}" cy="${y - 2}" r="1.3"/><rect class="ag-cx" x="${x}" y="${y + 2}" width="16" height="12" rx="3"/><path class="ag-an" d="M${x - 1.6} ${y + 6.5}v4M${x + 17.6} ${y + 6.5}v4"/><circle class="ag-ol" cx="${x + 4.8}" cy="${y + 7.2}" r="1.6"/><circle class="ag-ol" cx="${x + 11.2}" cy="${y + 7.2}" r="1.6"/><path class="ag-vz" d="M${x + 5.2} ${y + 11}h5.6"/></g>`;
const ROBO_ROTULO = { progresso: "trabalhando", concluir: "concluiu", "nao-concluida": "não concluiu", "checkpoint-feito": "atingiu um checkpoint", "checkpoint-nao-feito": "não atingiu um checkpoint", bloqueio: "travou", proposta: "enviou uma proposta", prototipo: "anexou um protótipo" };
function montaRobo() {
  const a = agenteAqui(), el = $("robo-agente");
  el.hidden = !a || !!a.visto; // depois de clicar e ir até ele, o aviso some; o robô fica na folha até chegar notícia nova
  if (el.hidden) return;
  const p = D.blueprints.find((x) => x.id === a.blueprintId), f = p?.acoes.find((x) => x.id === a.acaoId), t = f?.planos.find((x) => x.id === a.planoId);
  const onde = [p?.nome, f?.nome, t?.texto].filter(Boolean).join(" › ") || "neste workspace";
  el.className = roboClasse(a).trim();
  el.title = `${onde}\n${a.texto}${p ? "\nClique para ir até ele." : ""}`;
  el.innerHTML = `<svg width="26" height="24" viewBox="-4 -5 26 22" aria-hidden="true">${roboSvg(0, 0, a)}</svg><div><b>Agente ${esc(ROBO_ROTULO[a.tipo] || "")} · ${esc(onde)}</b><span>${esc(umaLinha(a.texto, 200))}</span></div>`;
}
$("robo-agente").onclick = () => {
  const a = agenteAqui(), pid = a && D.blueprints.some((x) => x.id === a.blueprintId) ? a.blueprintId : "";
  if (a) { a.visto = true; montaRobo(); }
  if (!pid) return;
  abertos[pid] = true; // abre as ações para o robô aparecer ao lado do plano
  roboRolar = true;
  if (tela === pid) render(); else irBlueprint(pid);
};

