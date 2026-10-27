// ---- "Gerar com IA" na tela Apresentação: a lista das apresentações geradas, o visor isolado, o editor, a conversa e o pedido de gerar ----
// Contrato em docs/interno/apresentacao-ia.md. A lógica pura está em apresentacao-ia-logica.js, as operações do editor em apresentacao-ia-ops.js e o painel ao lado do visor em
// apresentacao-ia-editor.js (Editar) e apresentacao-ia-conversa.js (Conversar). apresentacao.js chama apIaAbrir ao abrir a tela e apIaDepoisDePaginas quando volta a mostrar
// as páginas do app (a "Visão interna"). Com apresentações geradas a tela abre na mais nova. A apresentação roda num iframe isolado (sandbox sem allow-same-origin): é código
// de terceiros. A lista vem de GET /api/mcp/apresentacoes e se refaz com o evento SSE "apresentacao" (mcp.js). Gerar abre o terminal do app (ou outra sessão), só por clique.
// Zero trabalho em repouso: nada anima nem consulta sozinho.
const CHAVE_AP_IA = "blueprint.apresentacaoIa", CHAVE_AP_IA_ULTIMA = "blueprint.apresentacaoIaUltima.";
let apIaVisivel = ""; // o fragmento da seção que o visor mostra agora (a página avisa por postMessage "ap-estado"): o recarregamento volta para ele
let apIaRaiz = null, apIaBp = null, apIaSel = "", apIaOcupado = false, apIaDados = [], apIaPronto = false, apIaVolta = 0, apIaAba = "", apIaRecarga = 0, apIaContaRecarga = 0, apIaInternaTocada = false;
const apIaEscolha = { publico: "cliente", nivel: "padrao" };
try {
  const e = JSON.parse(localStorage.getItem(CHAVE_AP_IA) || "{}");
  if (AP_IA_PUBLICOS.includes(e.publico)) apIaEscolha.publico = e.publico;
  if (AP_IA_NIVEIS.some((n) => n.id === e.nivel)) apIaEscolha.nivel = e.nivel;
} catch {}

function apIaMsg(texto, erro) {
  const el = apIaRaiz?.querySelector(".ap-ia-gerar-grupo .ap-ia-msg");
  if (!el) return;
  el.textContent = texto || "";
  el.classList.toggle("erro", !!erro);
}
const apIaMeta = (a) => { const rp = apIaRotuloPublico(), rn = apIaRotuloNivel(); return [rp[a.publico], rn[a.nivel], a.em, a.editadoEm ? tr("editada") : ""].filter(Boolean).map(esc).join(" · "); };

function apIaOpcao(grupo, valor, rotulo, descricao, extra) {
  const marcado = apIaEscolha[grupo] === valor ? " checked" : "";
  return `<label class="ap-ia-op"><input type="radio" name="ap-ia-${grupo}" value="${valor}"${marcado}><span><b>${rotulo}</b>${extra ? `<em>${extra}</em>` : ""}<small>${descricao}</small></span></label>`;
}
function apIaHtmlGrupo() {
  const rp = apIaRotuloPublico(), dp = apIaDescPublico(), rn = apIaRotuloNivel(), dn = apIaDescNivel();
  const publicos = AP_IA_PUBLICOS.map((p) => apIaOpcao("publico", p, rp[p], dp[p])).join("");
  const niveis = AP_IA_NIVEIS.map((n) => apIaOpcao("nivel", n.id, rn[n.id], dn[n.id], apIaEstimativa(n.id))).join("");
  return `<div class="ap-ia-lista"></div><div class="ap-ia-gerar-grupo"><p class="rot">${tr("Gerar com IA")}</p>
<fieldset class="ap-ia-grupo"><legend>${tr("Público")}</legend>${publicos}</fieldset>
<fieldset class="ap-ia-grupo"><legend>${tr("Nível")}</legend>${niveis}</fieldset>
<button class="btn cheio ap-ia-gerar" data-ia="gerar" type="button">${tr("Gerar")}</button>
<p class="ap-ia-msg" role="status"></p></div>`;
}

function apIaDesenhaLista() {
  const alvo = apIaRaiz?.querySelector(".ap-ia-lista");
  if (!alvo) return;
  if (!apIaPronto) { alvo.innerHTML = ""; return; } // a primeira resposta do servidor ainda não chegou
  alvo.innerHTML = apIaDados.length
    ? `<p class="rot">${tr("Apresentações geradas")}</p><ul>${apIaDados.map((a) => `<li><button class="ap-ia-item" type="button" data-ia-ver="${a.id}" aria-pressed="${a.id === apIaSel}"><b>${esc(a.titulo || tr("Sem título"))}</b><span>${apIaMeta(a)}</span></button></li>`).join("")}</ul>`
    : `<p class="ap-dica">${tr("Nenhuma apresentação gerada ainda. Escolha o público e o nível e clique em Gerar.")}</p>`;
}

// ---- o visor: a barra de ações, o iframe isolado e o painel ao lado (Editar | Conversar) ----
const apIaLembra = (id) => { try { localStorage.setItem(CHAVE_AP_IA_ULTIMA + apIaBp.id, id); } catch {} };
const apIaLembrada = () => { try { return localStorage.getItem(CHAVE_AP_IA_ULTIMA + apIaBp.id) || ""; } catch { return ""; } };
function apIaTopoVisivel(visivel) { const t = apIaRaiz?.querySelector(".ap-topo > .ap-acoes"); if (t) t.hidden = !visivel; } // Apresentar/Imprimir/Exportar do topo são das páginas do app
function apIaMostra(id) {
  const a = apIaDados.find((x) => x.id === id), previa = apIaRaiz?.querySelector(".ap-previa");
  if (!a || !previa) return;
  apEdFechar(); apCvFechar();
  apIaSel = id; apIaAba = ""; apIaVisivel = "";
  apIaLembra(id);
  previa.innerHTML = `<div class="ap-ia-visor"><div class="ap-ia-barra"><div class="ap-ia-nome"><b>${esc(a.titulo || tr("Sem título"))}</b><span>${apIaMeta(a)}</span></div>
<div class="ap-acoes"><button class="btn" data-ia="aba-editar" type="button" aria-pressed="false">${tr("Editar")}</button><button class="btn" data-ia="aba-conversa" type="button" aria-pressed="false">${tr("Conversar")}</button><button class="btn cheio" data-ia="tela-cheia" type="button">${tr("Apresentar")}</button><button class="btn" data-ia="imprimir" type="button">${tr("Imprimir")}</button><a class="btn" href="${apIaUrl(id, true)}" download>${tr("Exportar HTML")}</a><button class="btn" data-ia="refazer" type="button">${tr("Gerar de novo")}</button><button class="btn perigo" data-ia="remover" type="button">${tr("Remover")}</button></div></div>
<div class="ap-ia-corpo"><iframe class="ap-ia-quadro" sandbox="allow-scripts" allowfullscreen src="${apIaUrl(id)}" title="${esc(tr("Apresentação gerada: {titulo}", { titulo: a.titulo || "" }))}"></iframe>
<aside class="ap-ia-lado" hidden aria-label="${esc(tr("Editar e conversar"))}"><div class="ap-ia-painel" data-painel="editar" hidden></div><div class="ap-ia-painel" data-painel="conversa" hidden></div></aside></div></div>`;
  apIaTopoVisivel(false);
  for (const b of apIaRaiz.querySelectorAll(".ap-tipo")) b.setAttribute("aria-pressed", "false");
  apIaDesenhaLista();
  apIaMsg("");
}
function apIaAlternaAba(aba) { // abre ou fecha o painel ao lado do visor (Editar ou Conversar); clicar de novo na mesma aba fecha
  const lado = apIaRaiz.querySelector(".ap-ia-lado");
  if (!lado) return;
  const nova = apIaAba === aba ? "" : aba;
  apEdFechar(); apCvFechar();
  apIaAba = nova;
  lado.hidden = !nova;
  for (const p of lado.querySelectorAll(".ap-ia-painel")) p.hidden = p.dataset.painel !== nova;
  for (const b of apIaRaiz.querySelectorAll('[data-ia^="aba-"]')) b.setAttribute("aria-pressed", String(b.dataset.ia === "aba-" + nova));
  apIaRaiz.querySelector(".ap-ia-corpo").dataset.lado = nova;
  const ids = { workspaceId: workspaceAtual?.id || "sm", blueprintId: apIaBp.id, apresentacaoId: apIaSel };
  if (nova === "editar") apEdAbrir(lado.querySelector('[data-painel="editar"]'), { ids, aoSalvo: apIaRecarregaVisor, aoFoco: apIaVai, aoOutro: () => { apCvAtualizou(); apIaRecarregaVisor(""); } });
  else if (nova === "conversa") apCvAbrir(lado.querySelector('[data-painel="conversa"]'), { bp: apIaBp, id: apIaSel });
}
// recarrega o iframe depois de uma pausa (várias mudanças seguidas viram um só recarregamento), na seção que foi editada
function apIaRecarregaVisor(fragmento) {
  clearTimeout(apIaRecarga);
  apIaRecarga = setTimeout(() => {
    const q = apIaRaiz?.querySelector(".ap-ia-quadro");
    const f = fragmento || apIaVisivel; // sem uma seção pedida, volta para a que o usuário estava vendo
    if (q && apIaSel) q.src = `${apIaUrl(apIaSel)}?v=${++apIaContaRecarga}${f ? "#" + f : ""}`;
  }, 900);
}
// o editor pede ao visor para rolar até a seção do campo em foco; a origem do iframe é opaca, então o destino é "*" (a mensagem só leva o nome de uma seção)
function apIaVai(destino) {
  const q = apIaRaiz?.querySelector(".ap-ia-quadro");
  if (q?.contentWindow && destino?.secao) q.contentWindow.postMessage({ tipo: "ap-ir", secao: destino.secao, ...(destino.etapa ? { etapa: destino.etapa } : {}) }, "*");
}
window.addEventListener("message", (e) => { // a página gerada avisa qual seção está à vista; só vale vindo do iframe do visor e com um fragmento de aparência segura
  const q = apIaRaiz?.querySelector(".ap-ia-quadro"), d = e.data;
  if (!q || e.source !== q.contentWindow || d?.tipo !== "ap-estado" || typeof d.fragmento !== "string" || !/^[\w-]{1,60}$/.test(d.fragmento)) return;
  apIaVisivel = d.fragmento;
});
function apIaTelaCheia() { const q = apIaRaiz?.querySelector(".ap-ia-quadro"); q?.requestFullscreen?.().catch(() => apIaMsg(tr("O sistema não deixou abrir em tela cheia."), true)); }
// o visor é isolado e não imprime sozinho (o sandbox não tem allow-modals): exporta o arquivo, que se imprime no navegador
function apIaImprimir() {
  const a = document.createElement("a");
  a.href = apIaUrl(apIaSel, true); a.download = ""; a.click();
  apIaMsg(tr("O visor é isolado e não imprime sozinho. Exportei o arquivo: abra-o no navegador e use Imprimir."));
}
async function apIaRemover() {
  const a = apIaDados.find((x) => x.id === apIaSel);
  if (!a) return;
  if (!(await confirmar({ titulo: tr("Remover a apresentação?"), texto: tr("«{titulo}» deixa de existir no app. Gerar de novo cria outra.", { titulo: a.titulo || tr("Sem título") }), rotulo: tr("Remover") }))) return;
  try {
    const r = await fetch("/api/mcp/apresentacao/remover", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: workspaceAtual?.id || "sm", blueprintId: apIaBp.id, apresentacaoId: a.id }) });
    if (!r.ok) throw new Error(r.status);
    apEdFechar(); apCvFechar(); apIaSel = "";
    if (!(await apIaCarrega(true))) apDesenha(); // a aberta sumiu da lista: abre a seguinte, ou volta às páginas
    apIaMsg(tr("Apresentação removida."));
  } catch { apIaMsg(tr("Não foi possível remover. Tente de novo."), true); }
}

// ---- gerar: no terminal do app (padrão), num terminal novo ou numa sessão aberta; só por clique confirmado ----
async function apIaGerar() {
  if (apIaOcupado || !apIaBp) return;
  if (typeof repositorioConvidadoNoApp === "function" && repositorioConvidadoNoApp()) return apIaMsg(tr("Gerar com IA só funciona no app local, não na janela da equipe."), true);
  if (typeof poderesDaSessao === "function" && poderesDaSessao(sessao).soLeitura) return apIaMsg(tr("O seu papel só lê este workspace: não dá para pedir uma apresentação daqui."), true);
  const { publico, nivel } = apIaEscolha, ws = workspaceAtual?.id || "sm";
  const sessoes = await sessoesAbertas(apIaBp);
  const texto = `${tr("Gasta tokens.")} ${tr("Público: {publico}. Nível: {nivel} ({tempo}).", { publico: apIaRotuloPublico()[publico], nivel: apIaRotuloNivel()[nivel], tempo: apIaEstimativa(nivel) })}`;
  const escolha = await escolherSessao({ titulo: tr("Gerar a apresentação com o Claude?"), texto, sessoes, padrao: "app", rotulo: tr("Gerar") });
  if (!escolha) return;
  apIaOcupado = true;
  try {
    if (escolha.app) { // o terminal do app: a conversa fica aqui dentro, numa aba do painel, e o usuário acompanha e responde
      apIaMsg(tr("Abrindo o terminal do app…"));
      const corpo = apIaCorpoTerminal({ workspaceId: ws, blueprintId: apIaBp.id, publico, nivel, titulo: tr("Apresentação: {nome}", { nome: apIaBp.nome || "" }) });
      const r = await fetch("/api/terminal/iniciar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
      const j = await r.json().catch(() => ({}));
      if (!r.ok || !j.sessao) return apIaMsg(j?.codigo === "sem-terminal" ? tr("Não consegui abrir o terminal do app. Motivo: {erro}", { erro: j.erro || r.status }) : apIaMensagem(r.status, j, false), true);
      apIaMsg(tr("Terminal do app aberto: o Claude vai montar a apresentação. Responda ali o que ele perguntar."));
      return terminalAbrirSessao(j.sessao);
    }
    const sessaoEscolhida = escolha.sessao;
    const corpo = apIaCorpoPedido({ workspaceId: ws, blueprintId: apIaBp.id, publico, nivel, sessao: sessaoEscolhida });
    apIaMsg(sessaoEscolhida ? (sessaoEscolhida.canal ? tr("Enviando o pedido à sessão…") : tr("Copiando o pedido…")) : tr("Abrindo o terminal…"));
    const r = await fetch("/api/mcp/apresentar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
    const j = await r.json().catch(() => ({}));
    if (sessaoEscolhida && !sessaoEscolhida.canal && r.ok && j.prompt) { // sem canal não há entrega: o pedido vai para a área de transferência
      try { await navigator.clipboard.writeText(j.prompt); apIaMsg(tr("Pedido copiado: cole na sessão para o Claude montar a apresentação.")); }
      catch { apIaMsg(tr("Não consegui copiar sozinho. O pedido é: {prompt}", { prompt: j.prompt }), true); }
    } else apIaMsg(apIaMensagem(r.status, j, !!sessaoEscolhida), !r.ok);
  } catch { apIaMsg(tr("Não consegui falar com o app para abrir o terminal. Ele está aberto?"), true); }
  finally { apIaOcupado = false; }
}
async function apIaRefazer() { // o mesmo público e nível da apresentação aberta, para o usuário só confirmar; se há edições à mão, avisa que serão substituídas
  const a = apIaDados.find((x) => x.id === apIaSel);
  if (!a) return;
  if (a.editadoEm && !(await confirmar({ titulo: tr("Gerar de novo?"), texto: tr("Isto substitui as suas edições."), rotulo: tr("Gerar de novo") }))) return;
  if (a.publico) apIaEscolha.publico = a.publico;
  if (a.nivel) apIaEscolha.nivel = a.nivel;
  apIaSalva(); apIaMarca(); apIaGerar();
}

function apIaClique(e) {
  if (e.target.closest("details.ap-interna > summary")) apIaInternaTocada = true; // o usuário abriu ou fechou a Visão interna: a escolha dele vale mais que o padrão
  const ver = e.target.closest("[data-ia-ver]");
  if (ver) return apIaMostra(ver.dataset.iaVer);
  const acao = e.target.closest("[data-ia]")?.dataset.ia;
  if (acao === "gerar") apIaGerar();
  else if (acao === "remover") apIaRemover();
  else if (acao === "refazer") apIaRefazer();
  else if (acao === "tela-cheia") apIaTelaCheia();
  else if (acao === "imprimir") apIaImprimir();
  else if (acao === "aba-editar" || acao === "aba-conversa") apIaAlternaAba(acao.slice(4));
}
function apIaMarca() {
  for (const [grupo, valor] of Object.entries(apIaEscolha)) { const el = apIaRaiz?.querySelector(`input[name="ap-ia-${grupo}"][value="${valor}"]`); if (el) el.checked = true; }
}
function apIaSalva() { try { localStorage.setItem(CHAVE_AP_IA, JSON.stringify(apIaEscolha)); } catch {} }

// ---- ligações com apresentacao.js e com o recarregamento dos dados ----
// Devolve uma promessa de true quando abriu uma apresentação gerada no visor (a tela não precisa desenhar as páginas do app) e de false quando não há ou o servidor não respondeu.
function apIaAbrir(raiz, blueprint) {
  apIaRaiz = raiz; apIaBp = blueprint; apIaSel = ""; apIaDados = []; apIaPronto = false; apIaAba = ""; apIaInternaTocada = false;
  const lateral = raiz.querySelector(".ap-tipos");
  if (!lateral) return Promise.resolve(false);
  lateral.querySelector(".ap-ia")?.remove();
  lateral.insertAdjacentHTML("afterbegin", `<div class="ap-ia">${apIaHtmlGrupo()}</div>`);
  if (!raiz.dataset.apIa) { // os ouvintes ficam na seção, que sobrevive às trocas do conteúdo
    raiz.dataset.apIa = "1";
    raiz.addEventListener("click", apIaClique);
    raiz.addEventListener("change", (e) => {
      const m = /^ap-ia-(publico|nivel)$/.exec(e.target.name || "");
      if (m) { apIaEscolha[m[1]] = e.target.value; apIaSalva(); }
    });
  }
  apIaDesenhaLista();
  return apIaCarrega(true);
}
function apIaDepoisDePaginas() { // apDesenha mostrou as páginas do app (Visão interna): nenhuma apresentação gerada fica aberta
  apEdFechar(); apCvFechar();
  apIaSel = ""; apIaAba = "";
  apIaTopoVisivel(true);
  apIaDesenhaLista();
}
function apIaFechar() { apEdFechar(); apCvFechar(); clearTimeout(apIaRecarga); apIaRaiz?.querySelector(".ap-ia-quadro")?.setAttribute("src", "about:blank"); apIaSel = ""; } // o iframe não fica rodando por trás
// o servidor guardou, editou ou removeu uma apresentação (evento SSE "apresentacao", mcp.js): busca a lista de novo se for deste blueprint e, se a aberta mudou por mão de
// outro (o agente), recarrega o visor
async function apIaRecebe(d) {
  if (!apIaRaiz || apIaRaiz.hidden || !apIaBp || (d?.blueprintId && d.blueprintId !== apIaBp.id)) return;
  const aberta = apIaSel && (!d?.apresentacaoId || d.apresentacaoId === apIaSel);
  await apIaCarrega();
  if (!aberta || !apIaSel) return;
  if (apIaAba === "editar") await apEdOutroAutor(); // o editor decide se foi outro autor (as edições do próprio usuário não recarregam duas vezes) e chama aoOutro
  else { apCvAtualizou(); apIaRecarregaVisor(""); }
}
// GET /api/mcp/apresentacoes: os metadados {id, titulo, publico, nivel, em, editadoEm?}, da mais nova para a mais antiga; sem resposta, a lista que já havia fica
async function apIaCarrega(primeira = false) {
  const bp = apIaBp, volta = ++apIaVolta;
  try {
    const r = await fetch(`/api/mcp/apresentacoes?workspace=${encodeURIComponent(workspaceAtual?.id || "sm")}&blueprint=${encodeURIComponent(bp.id)}`, { cache: "no-store" });
    if (!r.ok) return false;
    const j = await r.json();
    if (volta !== apIaVolta || bp !== apIaBp) return false; // uma resposta velha não sobrescreve a nova
    apIaDados = apIaLista(j.apresentacoes);
    apIaPronto = true;
  } catch { return false; }
  const interna = apIaRaiz.querySelector("details.ap-interna");
  if (interna && !apIaInternaTocada) interna.open = apIaDados.length === 0; // com apresentações geradas, a Visão interna fica recolhida
  if (apIaSel && !apIaDados.some((a) => a.id === apIaSel)) { apIaSel = ""; if (apIaDados.length) apIaMostra(apIaDados[0].id); else return (apDesenha(), false); } // a que estava aberta sumiu
  else if (primeira && apIaDados.length) { apIaMostra(apIaDados.some((a) => a.id === apIaLembrada()) ? apIaLembrada() : apIaDados[0].id); return true; } // a tela abre na última que o usuário viu (ou na mais nova)
  apIaDesenhaLista();
  return !!apIaSel;
}
