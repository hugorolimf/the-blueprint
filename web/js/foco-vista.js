// ---- a visão de acompanhamento dos agentes (o modo foco): uma tela própria sobre a folha, no tema do app, com um cartão por agente (o principal e os subagentes, trabalhando ou em repouso) ----
// Clicar num cartão fixa o foco nesse agente (cartão grande, com o histórico); Esc ou «Voltar» sobe um degrau. Tudo ao vivo pelos avisos que já existem: a lista de agentes (agentes.js, via
// focoDepoisDoRender) e a linha do tempo da ação (SSE `linha-do-tempo`, via focoLinhaDoTempo); nenhum polling nem timer. Estado em foco.js (focoAberto, focoFixo). Cartões em foco-cartao.js.
let focoVistaEl = null, focoVistaHtml = "";
const focoEventos = new Map(); // "workspace|blueprint|ação" -> eventos da linha do tempo (do mais novo ao mais antigo); só enquanto a visão está aberta
const focoBuscando = new Set();
const FOCO_MAX_BUSCAS = 12; // ações distintas buscadas de uma vez (um pedido por ação)

async function focoCarrega(chave) {
  if (focoBuscando.has(chave)) return;
  focoBuscando.add(chave);
  const [ws, bp, acao] = chave.split("|");
  let lista = [];
  try {
    const r = await fetch(`/api/mcp/linha-do-tempo?workspace=${encodeURIComponent(ws)}&blueprint=${encodeURIComponent(bp)}&acao=${encodeURIComponent(acao)}`, { cache: "no-store" });
    if (r.ok) lista = (await r.json()).eventos || [];
  } catch {}
  focoBuscando.delete(chave);
  if (!focoVistaEl) return; // a visão fechou enquanto buscava
  focoEventos.set(chave, lista); // mesmo com erro (lista vazia): não tenta de novo a cada desenho
  focoVistaRender();
}
const focoChaveAcao = (i) => `${workspaceAtual.id}|${i.blueprintId}|${i.acaoId}`;
// o servidor avisou (SSE) que a linha do tempo de uma ação mudou (chamado por ldtAviso, em linha-do-tempo.js): busca de novo só as que a visão está mostrando
function focoLinhaDoTempo(d) {
  const chave = `${d?.workspaceId}|${d?.blueprintId}|${d?.acaoId}`;
  if (focoVistaEl && focoEventos.has(chave)) focoCarrega(chave);
}

const focoAgentes = () => (workspaceAtual && carregado ? focoCartoes(agentesAtivos(), D.blueprints) : focoCartoes([], []));

function focoVistaHtmlDe(cartoes, emFoco) {
  if (cartoes.vazio) return `<div class="foco-vazio"><p>${tr("Nenhum agente por aqui agora.")}</p><p class="dica">${tr("Quando o seu agente (Claude Code, Codex…) avisar o que faz pelo MCP, ele aparece aqui com o plano em que trabalha, a última fala e os arquivos. Nada é enviado ao agente por esta tela.")}</p></div>`;
  const eventos = (i) => focoEventos.get(focoChaveAcao(i));
  if (focoFixo) { const i = cartoes.itens.find((x) => x.agente === focoFixo); return `<div class="foco-detalhe">${focoCartaoHtml(i, { principal: cartoes.principal, grande: true, eventos: eventos(i) })}</div>`; }
  return `<div class="foco-grade">${cartoes.itens.map((i) => focoCartaoHtml(i, { principal: cartoes.principal, emFoco: i === emFoco, eventos: eventos(i) })).join("")}</div>`;
}

// desenha (ou refaz) a visão; só troca o DOM se o HTML mudou, e devolve o foco ao mesmo botão
function focoVistaRender() {
  if (!focoVistaEl) return;
  const cartoes = focoAgentes();
  if (focoFixoSumiu(cartoes, focoFixo)) focoFixo = ""; // o agente fixado saiu da folha: volta à lista
  for (const chave of [...new Set(cartoes.itens.filter((i) => i.acaoId).map(focoChaveAcao))].slice(0, FOCO_MAX_BUSCAS)) if (!focoEventos.has(chave)) focoCarrega(chave);
  const emFoco = focoEmFoco(cartoes, focoFixo);
  const html = `<header class="ed-topo"><div class="tit"><span>${tr("Modo foco")}</span><b>${esc(focoFixo ? tr("Agente: {nome}", { nome: focoFixo }) : tr("Acompanhar agentes"))}</b></div>`
    + `<span class="foco-conta">${trn(cartoes.itens.length, ["{n} agente", "{n} agentes"])}</span><button class="btn" type="button" data-foco-voltar title="Esc">${focoFixo ? tr("← Voltar aos agentes") : tr("← Voltar à folha")}</button></header>`
    + focoVistaHtmlDe(cartoes, emFoco);
  if (html === focoVistaHtml) return;
  const ativo = document.activeElement, marca = ativo && focoVistaEl.contains(ativo) ? ["data-foco-voltar", "data-foco-abrir", "data-foco-ir"].map((a) => (ativo.hasAttribute(a) ? `[${a}="${CSS.escape(ativo.getAttribute(a))}"]` : "")).join("") : "";
  focoVistaHtml = html;
  focoVistaEl.innerHTML = html;
  if (marca) focoVistaEl.querySelector(marca)?.focus({ preventScroll: true });
}

function focoVistaAbre() {
  if (focoVistaEl) return;
  focoVistaEl = document.createElement("section");
  focoVistaEl.id = "foco-vista";
  focoVistaEl.setAttribute("role", "dialog");
  focoVistaEl.setAttribute("aria-modal", "true");
  focoVistaEl.setAttribute("aria-label", tr("Acompanhar agentes"));
  focoVistaEl.addEventListener("click", (e) => {
    const ir = e.target.closest("[data-foco-ir]"), abrir = e.target.closest("[data-foco-abrir]"), cartao = e.target.closest(".foco-cartao:not(.grande)");
    if (e.target.closest("[data-foco-voltar]")) focoEsc(focoFixo) === "lista" ? focoSobe() : focoFecha();
    else if (ir) focoVaiPara(ir.dataset.bp, ir.dataset.acao, ir.dataset.plano);
    else if (abrir || cartao) { focoFixo = abrir?.dataset.focoAbrir ?? cartao?.dataset.agente ?? ""; focoVistaEl.scrollTop = 0; focoVistaRender(); focoVistaEl.querySelector("[data-foco-voltar]")?.focus({ preventScroll: true }); }
  });
  document.body.append(focoVistaEl);
  $("blueprint-layout").inert = true; // a folha por baixo não recebe foco nem clique
  focoVistaRender();
  focoVistaEl.querySelector("[data-foco-voltar]")?.focus({ preventScroll: true });
}
function focoVistaFecha() {
  focoVistaEl?.remove();
  focoVistaEl = null; focoVistaHtml = "";
  focoEventos.clear();
  $("blueprint-layout").inert = false;
}
