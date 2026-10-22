// ---- painel de agentes: quem está trabalhando agora, cada um com a sua figura do anatomy ----
// Só mostra o que os agentes do usuário reportam pelo MCP (o app não abre, não conversa e não executa agentes: sem botão de iniciar nem de conversar).
// Zero trabalho em repouso: sem agentes o botão nem aparece; com o painel fechado só há um relógio até o próximo agente expirar;
// aberto, o painel redesenha só o cartão que mudou e atualiza o "há quanto tempo" a cada 20 s.
const painelAg = $("agentes-painel"), listaAg = $("ag-cartoes");
let agentesLista = [], agentesVida = 10 * 60 * 1000, agentesFig = null, agentesExpira = 0, agentesRelogio = 0;
const figurasAg = new Map(); // chave da pose -> svg (a mesma pose, em agentes diferentes, é desenhada uma vez só)
const ESTADO_AG = { lendo: "Lendo", escrevendo: "Escrevendo", testando: "Testando", aguardando: "Aguardando", bloqueado: "Bloqueado", concluido: "Concluído", ocioso: "Parado" };
const PAPEL_AG = { explorador: "Explorador", construtor: "Construtor", revisor: "Revisor" };

const haQuanto = (iso) => {
  const s = Math.max(0, Math.round((Date.now() - Date.parse(iso)) / 1000));
  return s < 10 ? "agora" : s < 60 ? `há ${s} s` : s < 3600 ? `há ${Math.round(s / 60)} min` : `há ${Math.round(s / 3600)} h`;
};
// o carro da figura corre na régua: o caminho em que o agente trabalha escolhe a estação (0, ¼, ½, ¾ ou 1), sempre a mesma para o mesmo caminho
const posicaoAg = (onde) => { let h = 0; for (const c of String(onde?.[0] || "")) h = (h * 31 + c.charCodeAt(0)) >>> 0; return (h % 5) / 4; };

async function agentesBusca() {
  try {
    const r = await fetch("/api/mcp/agentes", { cache: "no-store" });
    if (!r.ok) return; // servidor sem painel (o Node antigo): o botão fica escondido
    const j = await r.json();
    agentesVida = j.vidaMs || agentesVida;
    agentesLista = Array.isArray(j.agentes) ? j.agentes : [];
  } catch { return; }
  agentesAtualiza();
}
// só os agentes do workspace aberto e ainda dentro do prazo
const agentesAtivos = () => agentesLista.filter((a) => a.workspaceId === workspaceAtual?.id && Date.now() - Date.parse(a.em) < agentesVida);

function agentesAtualiza() {
  const ativos = agentesAtivos(), botao = $("b-agentes");
  botao.hidden = !ativos.length && !painelAg.open;
  $("b-agentes-n").textContent = ativos.length;
  if (painelAg.open) desenhaCartoes(ativos);
  clearTimeout(agentesExpira); // um relógio só, até o próximo agente vencer
  if (ativos.length) agentesExpira = setTimeout(agentesAtualiza, Math.max(1000, Math.min(...ativos.map((a) => Date.parse(a.em) + agentesVida - Date.now())) + 500));
}

function figuraAg(a) {
  if (!agentesFig || !EMB) return `<p class="vazio">Figura indisponível.</p>`;
  const papel = a.papel || "construtor", estado = a.estado || "escrevendo", pos = posicaoAg(a.onde), prog = Math.round(((a.percentual || 0) / 100) * 10) / 10;
  const chave = `${papel}|${estado}|${pos}|${prog}`;
  if (!figurasAg.has(chave)) {
    const e = EMB.desenhaEmblema(agentesFig.pecasDoAgente({ papel, estado, posicao: pos, progresso: prog }), { largura: 220, altura: 170, progresso: 1, sombra: false, limite: 400 });
    figurasAg.set(chave, `<svg class="emblema ag-fig-svg" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}" role="img" aria-label="${esc(agentesFig.descricaoDoAgente({ papel, estado }))}">${e.svg}</svg>`);
  }
  return figurasAg.get(chave);
}

function htmlCartao(a) {
  const p = D.blueprints.find((x) => x.id === a.blueprintId), f = p?.acoes.find((x) => x.id === a.acaoId), t = f?.planos.find((x) => x.id === a.planoId);
  const onde = [p?.nome, f?.nome, t?.texto].filter(Boolean).join(" › ");
  const estado = a.estado || "escrevendo", pct = Number.isFinite(a.percentual) ? a.percentual : null;
  return `<div class="ag-fig">${figuraAg(a)}</div><div class="ag-info">
    <div class="ag-topo"><b>${esc(a.agente)}</b><span class="badge">${esc(PAPEL_AG[a.papel] || "Agente")}</span></div>
    <p class="ag-estado e-${esc(estado)}"><i aria-hidden="true"></i>${esc(ESTADO_AG[estado] || estado)}${pct === null ? "" : ` · ${pct}%`}</p>
    ${pct === null ? "" : `<div class="ag-barra" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${pct}"><span style="width:${pct}%"></span></div>`}
    ${t ? `<p class="ag-plano">${esc(onde)} <button class="btn mini" data-ag-ir type="button">Ir até o plano</button></p>` : onde ? `<p class="ag-plano">${esc(onde)}</p>` : ""}
    ${a.onde?.length ? `<p class="ag-onde" title="${esc(a.onde.join("\n"))}">${esc(a.onde.join(" · "))}</p>` : ""}
    <p class="ag-msg">${esc(umaLinha(a.mensagem || "", 160))} <span class="ag-ha" data-em="${esc(a.em)}">${haQuanto(a.em)}</span></p></div>`;
}

// atualiza só o cartão que mudou: cada cartão guarda a "assinatura" do que mostra e é refeito apenas se ela mudar
function desenhaCartoes(ativos) {
  const atuais = new Map([...listaAg.querySelectorAll("[data-ag]")].map((e) => [e.dataset.ag, e]));
  ativos.forEach((a, i) => {
    let el = atuais.get(a.agente);
    if (!el) { el = document.createElement("article"); el.className = "ag-card"; el.dataset.ag = a.agente; }
    const sig = JSON.stringify([a.papel, a.estado, a.onde, a.percentual, a.acaoId, a.planoId, a.mensagem, !!agentesFig, !!EMB]);
    if (el._sig !== sig) { el.innerHTML = htmlCartao(a); el._sig = sig; }
    if (listaAg.children[i] !== el) listaAg.insertBefore(el, listaAg.children[i] || null);
    atuais.delete(a.agente);
  });
  atuais.forEach((el) => el.remove());
  [...listaAg.querySelectorAll(".ag-vazio")].forEach((e) => e.remove());
  if (!ativos.length) listaAg.insertAdjacentHTML("beforeend", `<p class="ag-vazio">Nenhum agente reportou nos últimos ${Math.round(agentesVida / 60000)} minutos.</p>`);
}

function atualizaTempos() {
  listaAg.querySelectorAll("[data-em]").forEach((e) => { e.textContent = haQuanto(e.dataset.em); });
  agentesAtualiza(); // também tira quem expirou enquanto o painel está aberto
}

async function abreAgentes() {
  painelAg.showModal();
  desenhaCartoes(agentesAtivos());
  try { // a figura vem do servidor só na primeira vez que o painel abre (tela carregada sob demanda)
    [agentesFig] = await Promise.all([agentesFig || import("/agentes-figura.mjs"), emblemasProntos]);
  } catch { agentesFig = null; }
  desenhaCartoes(agentesAtivos());
  clearInterval(agentesRelogio);
  agentesRelogio = setInterval(atualizaTempos, 20000);
}
$("b-agentes").onclick = abreAgentes;
$("ag-painel-fechar").onclick = () => painelAg.close();
painelAg.addEventListener("close", () => { clearInterval(agentesRelogio); agentesAtualiza(); });
// o cartão leva até o plano na folha, como o robô da faixa: abre as ações e o robô aparece ao lado dela
listaAg.addEventListener("click", (e) => {
  const nome = e.target.closest("[data-ag-ir]")?.closest("[data-ag]")?.dataset.ag, a = agentesAtivos().find((x) => x.agente === nome);
  if (!a || !D.blueprints.some((x) => x.id === a.blueprintId)) return;
  agenteAgora = { ws: workspaceAtual.id, blueprintId: a.blueprintId, acaoId: a.acaoId || "", planoId: a.planoId || "", tipo: a.tipo === "bloqueio" ? "bloqueio" : "progresso", texto: a.mensagem || "", em: Date.now(), visto: true };
  abertos[a.blueprintId] = true;
  roboRolar = true;
  painelAg.close();
  if (tela === a.blueprintId) render(); else irBlueprint(a.blueprintId);
});
