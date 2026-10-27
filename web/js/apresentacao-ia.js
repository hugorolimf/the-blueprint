// ---- "Gerar com IA" na tela Apresentação: escolhe público e nível, pede a apresentação ao agente e mostra as que ele já gerou ----
// Contrato em docs/interno/apresentacao-ia.md. A lógica pura está em apresentacao-ia-logica.js; apresentacao.js chama apIaAbrir ao abrir a tela e
// apIaDepoisDePaginas quando volta a mostrar as páginas do app. A apresentação gerada roda num iframe isolado (sandbox sem allow-same-origin): é código de terceiros.
// A lista vem de GET /api/mcp/apresentacoes (o servidor guarda cada apresentação num arquivo, não nos dados do blueprint) e se refaz quando chega o evento SSE
// "apresentacao" (mcp.js). Só por clique do usuário (POST /api/mcp/apresentar, só loopback). Zero trabalho em repouso: nada anima nem consulta sozinho.
const CHAVE_AP_IA = "blueprint.apresentacaoIa";
let apIaRaiz = null, apIaBp = null, apIaSel = "", apIaOcupado = false, apIaDados = [], apIaPronto = false, apIaVolta = 0;
const apIaEscolha = { publico: "cliente", nivel: "padrao" };
try {
  const e = JSON.parse(localStorage.getItem(CHAVE_AP_IA) || "{}");
  if (AP_IA_PUBLICOS.includes(e.publico)) apIaEscolha.publico = e.publico;
  if (AP_IA_NIVEIS.some((n) => n.id === e.nivel)) apIaEscolha.nivel = e.nivel;
} catch {}

function apIaMsg(texto, erro) {
  const el = apIaRaiz?.querySelector(".ap-ia-msg");
  if (!el) return;
  el.textContent = texto || "";
  el.classList.toggle("erro", !!erro);
}

function apIaOpcao(grupo, valor, rotulo, descricao, extra) {
  const marcado = apIaEscolha[grupo] === valor ? " checked" : "";
  return `<label class="ap-ia-op"><input type="radio" name="ap-ia-${grupo}" value="${valor}"${marcado}><span><b>${rotulo}</b>${extra ? `<em>${extra}</em>` : ""}<small>${descricao}</small></span></label>`;
}
function apIaHtmlGrupo() {
  const rp = apIaRotuloPublico(), dp = apIaDescPublico(), rn = apIaRotuloNivel(), dn = apIaDescNivel();
  const publicos = AP_IA_PUBLICOS.map((p) => apIaOpcao("publico", p, rp[p], dp[p])).join("");
  const niveis = AP_IA_NIVEIS.map((n) => apIaOpcao("nivel", n.id, rn[n.id], dn[n.id], apIaEstimativa(n.id))).join("");
  return `<p class="rot">${tr("Gerar com IA")}</p>
<fieldset class="ap-ia-grupo"><legend>${tr("Público")}</legend>${publicos}</fieldset>
<fieldset class="ap-ia-grupo"><legend>${tr("Nível")}</legend>${niveis}</fieldset>
<button class="btn cheio ap-ia-gerar" data-ia="gerar" type="button">${tr("Gerar")}</button>
<p class="ap-ia-msg" role="status"></p>
<div class="ap-ia-lista"></div>`;
}

function apIaDesenhaLista() {
  const alvo = apIaRaiz?.querySelector(".ap-ia-lista");
  if (!alvo) return;
  const lista = apIaDados, rp = apIaRotuloPublico(), rn = apIaRotuloNivel();
  if (!apIaPronto) { alvo.innerHTML = ""; return; } // a primeira resposta do servidor ainda não chegou
  alvo.innerHTML = lista.length
    ? `<p class="rot">${tr("Apresentações geradas")}</p><ul>${lista.map((a) => `<li><button class="ap-ia-item" type="button" data-ia-ver="${a.id}" aria-pressed="${a.id === apIaSel}"><b>${esc(a.titulo || tr("Sem título"))}</b><span>${[rp[a.publico], rn[a.nivel], a.em].filter(Boolean).map(esc).join(" · ")}</span></button></li>`).join("")}</ul>`
    : `<p class="ap-dica">${tr("Nenhuma apresentação gerada ainda. Escolha o público e o nível e clique em Gerar.")}</p>`;
}

// ---- ver, exportar, remover ----
function apIaMostra(id) {
  const a = apIaDados.find((x) => x.id === id);
  const previa = apIaRaiz?.querySelector(".ap-previa");
  if (!a || !previa) return;
  apIaSel = id;
  const rp = apIaRotuloPublico(), rn = apIaRotuloNivel();
  previa.innerHTML = `<div class="ap-ia-visor"><div class="ap-ia-barra"><div class="ap-ia-nome"><b>${esc(a.titulo || tr("Sem título"))}</b><span>${[rp[a.publico], rn[a.nivel], a.em].filter(Boolean).map(esc).join(" · ")}</span></div>
<div class="ap-acoes"><button class="btn" data-ia="voltar" type="button">${tr("Ver as páginas")}</button><button class="btn" data-ia="refazer" type="button">${tr("Gerar de novo")}</button><a class="btn" href="${apIaUrl(id, true)}" download>${tr("Exportar HTML")}</a><button class="btn perigo" data-ia="remover" type="button">${tr("Remover")}</button></div></div>
<iframe class="ap-ia-quadro" sandbox="allow-scripts" allowfullscreen src="${apIaUrl(id)}" title="${esc(tr("Apresentação gerada: {titulo}", { titulo: a.titulo || "" }))}"></iframe></div>`;
  for (const b of apIaRaiz.querySelectorAll(".ap-tipo")) b.setAttribute("aria-pressed", "false");
  apIaDesenhaLista();
}
async function apIaRemover() {
  const a = apIaDados.find((x) => x.id === apIaSel);
  if (!a) return;
  if (!(await confirmar({ titulo: tr("Remover a apresentação?"), texto: tr("«{titulo}» deixa de existir no app. Gerar de novo cria outra.", { titulo: a.titulo || tr("Sem título") }), rotulo: tr("Remover") }))) return;
  try {
    const r = await fetch("/api/mcp/apresentacao/remover", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: workspaceAtual?.id || "sm", blueprintId: apIaBp.id, apresentacaoId: a.id }) });
    if (!r.ok) throw new Error(r.status);
    await apIaCarrega(); // a aberta sumiu da lista: volta às páginas
    apIaMsg(tr("Apresentação removida."));
  } catch { apIaMsg(tr("Não foi possível remover. Tente de novo."), true); }
}

// ---- gerar: o pedido vai ao agente pelo caminho do "Iniciar com o Claude" (sessão já aberta ou terminal novo), só por clique confirmado ----
async function apIaGerar() {
  if (apIaOcupado || !apIaBp) return;
  if (typeof repositorioConvidadoNoApp === "function" && repositorioConvidadoNoApp()) return apIaMsg(tr("Gerar com IA só funciona no app local, não na janela da equipe."), true);
  if (typeof poderesDaSessao === "function" && poderesDaSessao(sessao).soLeitura) return apIaMsg(tr("O seu papel só lê este workspace: não dá para pedir uma apresentação daqui."), true);
  const { publico, nivel } = apIaEscolha;
  const sessoes = await sessoesAbertas(apIaBp);
  const texto = `${tr("Gasta tokens.")} ${tr("Público: {publico}. Nível: {nivel} ({tempo}).", { publico: apIaRotuloPublico()[publico], nivel: apIaRotuloNivel()[nivel], tempo: apIaEstimativa(nivel) })}`;
  const escolha = await escolherSessao({ titulo: tr("Gerar a apresentação com o Claude?"), texto, sessoes, semApp: true });
  if (!escolha) return;
  const corpo = apIaCorpoPedido({ workspaceId: workspaceAtual?.id || "sm", blueprintId: apIaBp.id, publico, nivel, sessao: escolha.sessao });
  if (!corpo) return apIaMsg(tr("Não foi possível pedir a apresentação (erro {status}).", { status: 400 }), true);
  apIaOcupado = true;
  const sessaoEscolhida = escolha.sessao;
  apIaMsg(sessaoEscolhida ? (sessaoEscolhida.canal ? tr("Enviando o pedido à sessão…") : tr("Copiando o pedido…")) : tr("Abrindo o terminal…"));
  try {
    const r = await fetch("/api/mcp/apresentar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
    const j = await r.json().catch(() => ({}));
    if (sessaoEscolhida && !sessaoEscolhida.canal && r.ok && j.prompt) { // sem canal não há entrega: o pedido vai para a área de transferência
      try { await navigator.clipboard.writeText(j.prompt); apIaMsg(tr("Pedido copiado: cole na sessão para o Claude montar a apresentação.")); }
      catch { apIaMsg(tr("Não consegui copiar sozinho. O pedido é: {prompt}", { prompt: j.prompt }), true); }
    } else apIaMsg(apIaMensagem(r.status, j, !!sessaoEscolhida), !r.ok);
  } catch { apIaMsg(tr("Não consegui falar com o app para abrir o terminal. Ele está aberto?"), true); }
  finally { apIaOcupado = false; }
}

function apIaClique(e) {
  const ver = e.target.closest("[data-ia-ver]");
  if (ver) return apIaMostra(ver.dataset.iaVer);
  const acao = e.target.closest("[data-ia]")?.dataset.ia;
  if (acao === "gerar") apIaGerar();
  else if (acao === "voltar") apDesenha();
  else if (acao === "remover") apIaRemover();
  else if (acao === "refazer") { // o mesmo público e nível da apresentação aberta, para o usuário só confirmar
    const a = apIaDados.find((x) => x.id === apIaSel);
    if (a?.publico) apIaEscolha.publico = a.publico;
    if (a?.nivel) apIaEscolha.nivel = a.nivel;
    apIaSalva();
    apIaMarca();
    apIaGerar();
  }
}
function apIaMarca() {
  for (const [grupo, valor] of Object.entries(apIaEscolha)) { const el = apIaRaiz?.querySelector(`input[name="ap-ia-${grupo}"][value="${valor}"]`); if (el) el.checked = true; }
}
function apIaSalva() { try { localStorage.setItem(CHAVE_AP_IA, JSON.stringify(apIaEscolha)); } catch {} }

// ---- ligações com apresentacao.js e com o recarregamento dos dados ----
function apIaAbrir(raiz, blueprint) {
  apIaRaiz = raiz;
  apIaBp = blueprint;
  apIaSel = "";
  apIaDados = [];
  apIaPronto = false;
  const lateral = raiz.querySelector(".ap-tipos");
  if (!lateral) return;
  lateral.querySelector(".ap-ia")?.remove();
  lateral.insertAdjacentHTML("beforeend", `<div class="ap-ia">${apIaHtmlGrupo()}</div>`);
  if (!raiz.dataset.apIa) { // os ouvintes ficam na seção, que sobrevive às trocas do conteúdo
    raiz.dataset.apIa = "1";
    raiz.addEventListener("click", apIaClique);
    raiz.addEventListener("change", (e) => {
      const m = /^ap-ia-(publico|nivel)$/.exec(e.target.name || "");
      if (m) { apIaEscolha[m[1]] = e.target.value; apIaSalva(); }
    });
  }
  apIaDesenhaLista();
  apIaCarrega();
}
function apIaDepoisDePaginas() { // apDesenha mostrou as páginas do app: nenhuma apresentação gerada está aberta
  apIaSel = "";
  apIaDesenhaLista();
}
function apIaFechar() { apIaRaiz?.querySelector(".ap-ia-quadro")?.setAttribute("src", "about:blank"); apIaSel = ""; } // o iframe não fica rodando por trás
// o servidor guardou ou removeu uma apresentação (evento SSE "apresentacao", mcp.js): busca a lista de novo se for deste blueprint
function apIaRecebe(d) {
  if (!apIaRaiz || apIaRaiz.hidden || !apIaBp || (d?.blueprintId && d.blueprintId !== apIaBp.id)) return;
  apIaCarrega();
}
// GET /api/mcp/apresentacoes: os metadados {id, titulo, publico, nivel, em}, da mais nova para a mais antiga; sem resposta, a lista que já havia fica
async function apIaCarrega() {
  const bp = apIaBp, volta = ++apIaVolta;
  try {
    const r = await fetch(`/api/mcp/apresentacoes?workspace=${encodeURIComponent(workspaceAtual?.id || "sm")}&blueprint=${encodeURIComponent(bp.id)}`, { cache: "no-store" });
    if (!r.ok) return;
    const j = await r.json();
    if (volta !== apIaVolta || bp !== apIaBp) return; // uma resposta velha não sobrescreve a nova
    apIaDados = apIaLista(j.apresentacoes);
    apIaPronto = true;
  } catch { return; }
  if (apIaSel && !apIaDados.some((a) => a.id === apIaSel)) return apDesenha(); // a que estava aberta sumiu
  apIaDesenhaLista();
}
