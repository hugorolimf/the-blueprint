// ---- linha do tempo da execução do agente na folha da ação: o que ele avisou (começou, mudou de arquivo, bloqueou, concluiu) e o plano, do mais novo ao mais antigo ----
// Só leitura. Os eventos vêm de GET /api/mcp/linha-do-tempo (guardados no banco; regra em docs/claude-na-folha.md) e a lista
// só é buscada de novo quando o servidor avisa por SSE (`linha-do-tempo`, com os ids) e a folha daquela ação está aberta. Sem polling.

// o que aconteceu, numa frase; o progresso é a própria frase do agente
function ldtFrase(e) {
  switch (e.tipo) {
    case "concluir": return tr("Plano concluído");
    case "nao-concluida": return tr("Plano não concluído");
    case "checkpoint-feito": return tr("Checkpoint feito");
    case "checkpoint-nao-feito": return tr("Checkpoint desmarcado");
    case "bloqueio": return tr("Bloqueio: {texto}", { texto: e.texto || "" });
    default: return e.texto || tr("Trabalhando");
  }
}

// o nome do plano ou do checkpoint do evento, tirado da ação (o servidor guarda só o id)
function ldtAlvo(e, acao) {
  if (!e.alvo || !acao) return "";
  const lista = String(e.tipo).startsWith("checkpoint") ? acao.checkpoints : acao.planos;
  const item = (lista || []).find((x) => x.id === e.alvo);
  return item ? item.texto || item.nome || "" : "";
}

function ldtLista(eventos, acao) {
  if (!eventos.length) return `<p class="ldt-vazio">${tr("Nenhum evento do agente nesta ação ainda.")}</p>`;
  return `<ol>${eventos.map((e) => {
    const alvo = ldtAlvo(e, acao);
    return `<li class="ldt-${esc(e.tipo)}"><div><span class="ldt-ag">${esc(e.agente || tr("agente"))}</span> ${esc(ldtFrase(e))}${alvo ? `<span class="ldt-alvo">${esc(alvo)}</span>` : ""}</div></li>`;
  }).join("")}</ol>`;
}

// ---- na página: a seção dentro da folha da ação (nota.js, aberta por abreFolhaDaAcao) ----
const ldtChave = (p, f) => `${workspaceAtual?.id || "sm"}|${p.id}|${f.id}`;
function ldtSecao(chave) { return notaEl.hidden ? null : [...notaEl.querySelectorAll("[data-ldt]")].find((s) => s.dataset.ldt === chave) || null; }

async function ldtCarrega(chave, p, f) {
  const [ws, bp, acao] = chave.split("|");
  let html;
  try {
    const r = await fetch(`/api/mcp/linha-do-tempo?workspace=${encodeURIComponent(ws)}&blueprint=${encodeURIComponent(bp)}&acao=${encodeURIComponent(acao)}`, { cache: "no-store" });
    if (!r.ok) throw new Error(r.status);
    html = ldtLista((await r.json()).eventos || [], f);
  } catch { html = `<p class="ldt-vazio">${tr("Não foi possível carregar a linha do tempo.")}</p>`; }
  const s = ldtSecao(chave);
  if (!s) return; // a folha fechou ou mudou de ação enquanto buscava
  s.querySelector(".ldt-corpo").innerHTML = html;
  const r = notaEl.getBoundingClientRect(); // cresceu depois de posicionada: não deixa passar do fim da janela
  if (r.bottom > innerHeight - 16) notaEl.style.top = Math.max(scrollY + 12, scrollY + innerHeight - r.height - 16) + "px";
}

function ldtNaFolha(p, f) {
  const chave = ldtChave(p, f);
  if (!ldtSecao(chave)) {
    const s = document.createElement("section");
    s.className = "ldt";
    s.dataset.ldt = chave;
    s.innerHTML = `<h5>${tr("Linha do tempo do agente")}</h5><div class="ldt-corpo" aria-live="polite"><p class="ldt-vazio">${tr("carregando…")}</p></div>`;
    notaEl.insertBefore(s, notaEl.querySelector(".acoes"));
  }
  ldtCarrega(chave, p, f);
}

// o servidor avisou (SSE) que a linha do tempo de uma ação mudou: só busca se a folha dela estiver aberta
function ldtAviso(d) {
  if (typeof focoLinhaDoTempo === "function") focoLinhaDoTempo(d); // a faixa do plano acompanhado (foco-faixa.js)
  const chave = `${d?.workspaceId}|${d?.blueprintId}|${d?.acaoId}`;
  if (!ldtSecao(chave)) return;
  const p = D.blueprints.find((x) => x.id === d.blueprintId), f = p?.acoes.find((x) => x.id === d.acaoId);
  if (f) ldtCarrega(chave, p, f);
}
