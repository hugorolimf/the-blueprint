// ---- atualização do app: o app baixa a versão nova sozinho, em segundo plano, e só instala quando o usuário clica em Atualizar (src/atualizador, /api/atualizacao) ----
// Zero trabalho em repouso: o estado chega por SSE ("atualizacao", mcp.js) e uma consulta ao abrir; sem timer. O texto das notas vem do GitHub: entra só por textContent.
const ATUALIZACAO_DEPOIS = "blueprint.atualizacao.depois", ATUALIZACAO_ADIA_MS = 24 * 60 * 60 * 1000;
let atualizacao = null, atualizacaoEl = null, atualizacaoErro = "";
function atualizacaoAdiada(versao) { // "Depois" vale por 24 h para aquela versão (só conveniência: localStorage pode faltar)
  try { const d = JSON.parse(localStorage.getItem(ATUALIZACAO_DEPOIS) || "null"); return !!d && d.versao === versao && d.ate > Date.now(); } catch { return false; }
}
function atualizacaoCria() {
  const el = document.createElement("aside");
  el.id = "atualizacao"; el.hidden = true;
  el.setAttribute("role", "status"); el.setAttribute("aria-live", "polite");
  el.innerHTML = `<b>${tr("Atualização")}</b><p class="at-t"></p><div class="at-barra" hidden><i></i></div><p class="at-notas" hidden></p><p class="at-motivo" hidden></p>
<div class="at-acoes" hidden><button class="btn mini at-ok" type="button">${tr("Atualizar e reiniciar")}</button><button class="btn mini at-depois" type="button">${tr("Depois")}</button></div>`;
  document.body.appendChild(el);
  el.querySelector(".at-ok").onclick = atualizacaoInstala;
  el.querySelector(".at-depois").onclick = () => {
    try { localStorage.setItem(ATUALIZACAO_DEPOIS, JSON.stringify({ versao: atualizacao?.versao, ate: Date.now() + ATUALIZACAO_ADIA_MS })); } catch {}
    atualizacaoErro = ""; atualizacaoDesenha();
  };
  return el;
}
async function atualizacaoBusca() {
  try { const r = await fetch("/api/atualizacao", { cache: "no-store" }); if (r.ok) atualizacaoRecebe(await r.json()); } catch {}
}
function atualizacaoRecebe(e) {
  if (!e || typeof e !== "object") return;
  const antes = atualizacao?.fase;
  if (e.fase === "erro" && (antes === "pronta" || antes === "instalando")) atualizacaoErro = e.erro || ""; // o clique falhou no servidor (arquivo trocado, por exemplo): o usuário vê o porquê
  else if (e.fase !== "pronta" || e.motivo) atualizacaoErro = ""; // o servidor mandou o estado novo (ou o motivo dele): vale mais que o erro do clique anterior
  atualizacao = e;
  atualizacaoDesenha();
}
// o clique do usuário é a autorização: só aqui o servidor roda o instalador (e antes confere a fila do MCP, o hash e copia o banco)
async function atualizacaoInstala() {
  atualizacaoEl ||= atualizacaoCria();
  const botao = atualizacaoEl.querySelector(".at-ok");
  botao.disabled = true; atualizacaoErro = "";
  try {
    const r = await fetch("/api/atualizacao/instalar", { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
    if (!r.ok) { const j = await r.json().catch(() => ({})); atualizacaoErro = j.erro || tr("Não consegui atualizar ({status}).", { status: r.status }); }
  } catch { atualizacaoErro = tr("Não consegui falar com o app."); }
  botao.disabled = false;
  atualizacaoDesenha();
}
function atualizacaoDesenha() {
  const e = atualizacao;
  const baixando = e?.fase === "baixando", pronta = e?.fase === "pronta" && !atualizacaoAdiada(e.versao), instalando = e?.fase === "instalando", erro = e?.fase === "erro" && !!atualizacaoErro;
  const topo = document.getElementById("perfil-atualizar"); // o botão ao lado do nome (tela Workspaces); "Depois" esconde só o aviso, não ele
  if (topo) {
    topo.hidden = !(baixando || e?.fase === "pronta");
    topo.disabled = baixando;
    topo.textContent = baixando ? tr("Baixando… {pct}%", { pct: e.percentual ?? 0 }) : tr("Atualizar para {versao}", { versao: e?.versao || "" });
    topo.onclick = atualizacaoInstala;
  }
  if (!baixando && !pronta && !instalando && !erro && !atualizacaoErro) { if (atualizacaoEl) atualizacaoEl.hidden = true; return; }
  atualizacaoEl ||= atualizacaoCria();
  const q = (s) => atualizacaoEl.querySelector(s), v = e.versao ? ` ${e.versao}` : "";
  atualizacaoEl.hidden = false;
  atualizacaoEl.classList.toggle("erro", !!atualizacaoErro);
  q(".at-t").textContent = baixando ? tr("Baixando a versão{v}… {pct}%", { v, pct: e.percentual ?? 0 }) : instalando ? tr("Atualizando: o app fecha e abre de novo sozinho.") : tr("A versão{v} está pronta (você tem a {atual}).", { v, atual: e.atual });
  q(".at-barra").hidden = !baixando;
  if (baixando) q(".at-barra i").style.transform = `scaleX(${(e.percentual ?? 0) / 100})`; // transform, não width: a barra não força layout
  const notas = (e.notas || "").trim().split("\n").find((l) => l.trim()) || "";
  q(".at-notas").hidden = !(pronta && notas); q(".at-notas").textContent = notas.slice(0, 240);
  q(".at-motivo").textContent = atualizacaoErro || e.motivo || ""; q(".at-motivo").hidden = !q(".at-motivo").textContent; // o motivo também vem do servidor (recusa pela bandeja)
  q(".at-acoes").hidden = !pronta;
}
