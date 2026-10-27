// ---- tela Apresentação: escolhe o tipo, pré-visualiza as páginas, apresenta em tela cheia, imprime e exporta ----
// O conteúdo vem de montarApresentacao (apresentacao-modelo.js); a figura do anatomy, de window.figuraDoBlueprint (apresentacao-figuras.js), e imprimir/exportar,
// de window.imprimirApresentacao / window.exportarApresentacao (apresentacao-exportar.js). Se alguma faltar, a tela segue sem ela (botão desligado ou capa sem figura).
// Zero trabalho em repouso: nada anima nem mede sozinho; as páginas se ajustam ao tamanho por unidades de contêiner (apresentacao.css).
const CHAVE_AP_TIPO = "blueprint.apresentacaoTipo";
let apRaiz = null, apBlueprint = null, apTipo = "andamento", apDoc = null, apOrigem = null, apPalco = null, apIndice = 0, apAoTecla = null;
try { const t = localStorage.getItem(CHAVE_AP_TIPO); if (TIPOS_APRESENTACAO.includes(t)) apTipo = t; } catch {}

const apDescricoes = () => ({
  andamento: tr("Uma página por projeto, com o progresso e as ações de cada um."),
  completo: tr("Uma página por ação, com planos e checkpoints. Para revisar tudo."),
});

function apFigura(p) {
  try { return window.figuraDoBlueprint?.(p, { largura: 640 }) || ""; } catch { return ""; } // sem figura a capa continua boa
}
const apMonta = () => montarApresentacao(apBlueprint, { tipo: apTipo, workspace: D.empresa, posicao: D.blueprints.indexOf(apBlueprint), hoje: local(new Date()), figura: apFigura(apBlueprint) });

function apMensagem(texto) { if (apRaiz) apRaiz.querySelector(".ap-msg").textContent = texto || ""; }

function apHtmlTela() {
  const rot = apTipoRotulo(), desc = apDescricoes();
  const tipos = TIPOS_APRESENTACAO.map((t) => `<button class="ap-tipo" type="button" data-tipo="${t}" aria-pressed="${t === apTipo}"><b>${rot[t]}</b><span>${desc[t]}</span></button>`).join("");
  const tem = (f) => (typeof window[f] === "function" ? "" : ` disabled title="${esc(tr("Indisponível nesta versão."))}"`);
  return `<header class="ed-topo ap-topo"><button class="btn" data-ap="voltar" type="button">${tr("← Voltar ao blueprint")}</button><div class="tit"><span>${tr("Apresentação")}</span><b class="ap-nome-bp"></b></div>
<div class="ap-acoes"><button class="btn cheio" data-ap="apresentar" type="button">${tr("Apresentar")}</button><button class="btn" data-ap="imprimir" type="button"${tem("imprimirApresentacao")}>${tr("Imprimir")}</button><button class="btn" data-ap="html" type="button"${tem("exportarApresentacao")}>${tr("Exportar HTML")}</button><button class="btn" data-ap="pdf" type="button"${tem("exportarApresentacao")}>${tr("Exportar PDF")}</button></div></header>
<p class="ed-aviso ap-msg" role="status"></p>
<div class="ed-corpo ap-corpo"><nav class="ed-lateral ap-tipos" aria-label="${esc(tr("Tipo de apresentação"))}"><p class="rot">${tr("Tipo de apresentação")}</p>${tipos}<p class="ap-dica">${tr("Apresentar: setas ou espaço passam as páginas, Esc sai.")}</p></nav><main class="ap-previa" tabindex="0" aria-label="${esc(tr("Pré-visualização"))}"></main></div>`;
}

function apDesenha() {
  apDoc = apMonta();
  apRaiz.querySelector(".ap-previa").innerHTML = apDoc.paginas.map((p) => p.html).join("");
  apRaiz.querySelector(".ap-nome-bp").textContent = apDoc.titulo;
  for (const b of apRaiz.querySelectorAll(".ap-tipo")) b.setAttribute("aria-pressed", String(b.dataset.tipo === apTipo));
  if (typeof apIaDepoisDePaginas === "function") apIaDepoisDePaginas(); // apresentacao-ia.js: nenhuma apresentação gerada fica aberta por cima das páginas
}

async function apAcao(nome) {
  try {
    const html = apDoc.paginas.map((p) => p.html).join(""); // o contrato de imprimir/exportar: o elemento ou o HTML das <section class="ap-pagina">
    if (nome === "imprimir") await window.imprimirApresentacao(html);
    else await window.exportarApresentacao(html, { formato: nome });
  } catch { apMensagem(tr("Não foi possível concluir. Tente de novo.")); }
}

// ---- apresentar: uma página por vez, tela cheia quando o sistema deixa ----
function apMostra(i) {
  apIndice = Math.min(apDoc.paginas.length - 1, Math.max(0, i));
  apPalco.querySelector(".ap-palco-pagina").innerHTML = apDoc.paginas[apIndice].html;
  apPalco.querySelector(".ap-palco-n").textContent = `${apIndice + 1} / ${apDoc.paginas.length}`;
}
function apApresentar() {
  apDoc = apMonta();
  apPalco ||= Object.assign(document.createElement("div"), { className: "ap-palco", tabIndex: -1 });
  apPalco.setAttribute("role", "dialog");
  apPalco.setAttribute("aria-label", tr("Apresentação em tela cheia"));
  apPalco.innerHTML = `<div class="ap-palco-pagina"></div><div class="ap-palco-barra"><button class="btn mini" data-pal="-1" type="button" aria-label="${esc(tr("Página anterior"))}">‹</button><span class="ap-palco-n" role="status"></span><button class="btn mini" data-pal="1" type="button" aria-label="${esc(tr("Próxima página"))}">›</button><button class="btn mini" data-pal="0" type="button">${tr("Sair")}</button></div>`;
  apPalco.hidden = false;
  document.body.append(apPalco);
  apMostra(0);
  apPalco.focus({ preventScroll: true });
  apPalco.requestFullscreen?.().catch(() => {}); // sem tela cheia ela cobre a janela do mesmo jeito
}
function apSair() {
  if (!apPalco || apPalco.hidden) return;
  apPalco.hidden = true;
  delete apPalco.dataset.cheia;
  apPalco.remove();
  if (document.fullscreenElement === apPalco) document.exitFullscreen().catch(() => {});
  apRaiz?.querySelector('[data-ap="apresentar"]')?.focus({ preventScroll: true });
}
function apClique(e) {
  const b = e.target.closest("[data-pal]");
  if (b) return b.dataset.pal === "0" ? apSair() : apMostra(apIndice + Number(b.dataset.pal));
  if (e.target.closest(".ap-palco-pagina")) apMostra(apIndice + (e.clientX < innerWidth / 3 ? -1 : 1)); // o terço da esquerda volta, o resto avança
}

function apTecla(e) {
  if (!apRaiz || apRaiz.hidden || e.ctrlKey || e.metaKey || e.altKey) return;
  const palco = apPalco && !apPalco.hidden;
  if (e.key === "Escape") { e.preventDefault(); palco ? apSair() : apFechar(); return; }
  if (!palco) return;
  const passo = { ArrowRight: 1, ArrowDown: 1, PageDown: 1, " ": 1, Enter: 1, ArrowLeft: -1, ArrowUp: -1, PageUp: -1, Backspace: -1 }[e.key];
  if (passo) { e.preventDefault(); apMostra(apIndice + passo); }
  else if (e.key === "Home") { e.preventDefault(); apMostra(0); }
  else if (e.key === "End") { e.preventDefault(); apMostra(apDoc.paginas.length - 1); }
}

function apFechar() {
  if (!apRaiz || apRaiz.hidden) return;
  apSair();
  if (typeof apIaFechar === "function") apIaFechar();
  apRaiz.hidden = true;
  document.body.classList.remove("ap-aberto");
  apOrigem?.focus?.({ preventScroll: true });
}

function abrirApresentacao(blueprintId, quemAbriu) {
  apBlueprint = D.blueprints.find((b) => b.id === blueprintId);
  if (!apBlueprint) return;
  apOrigem = quemAbriu || null;
  apRaiz ||= Object.assign(document.createElement("section"), { id: "apresentacao-tela", className: "ap" });
  apRaiz.setAttribute("role", "dialog");
  apRaiz.setAttribute("aria-modal", "true");
  apRaiz.setAttribute("aria-label", tr("Apresentação"));
  apRaiz.innerHTML = apHtmlTela();
  document.body.append(apRaiz);
  apRaiz.hidden = false;
  document.body.classList.add("ap-aberto");
  apDesenha();
  if (typeof apIaAbrir === "function") apIaAbrir(apRaiz, apBlueprint); // o grupo "Gerar com IA" e a lista das já geradas (apresentacao-ia.js)
  apRaiz.onclick = (e) => {
    const t = e.target.closest(".ap-tipo");
    if (t) { apTipo = t.dataset.tipo; try { localStorage.setItem(CHAVE_AP_TIPO, apTipo); } catch {} apMensagem(""); apDesenha(); return; }
    const a = e.target.closest("[data-ap]")?.dataset.ap;
    if (a === "voltar") apFechar(); else if (a === "apresentar") apApresentar(); else if (a) apAcao(a);
  };
  if (!apAoTecla) addEventListener("keydown", (apAoTecla = apTecla));
  apRaiz.querySelector('[data-ap="apresentar"]').focus({ preventScroll: true });
}

document.addEventListener("click", (e) => { if (apPalco && !apPalco.hidden && apPalco.contains(e.target)) apClique(e); });
document.addEventListener("fullscreenchange", () => { // Esc na tela cheia sai dela sem passar pelo teclado da página: a apresentação acompanha
  if (!apPalco || apPalco.hidden) return;
  if (document.fullscreenElement === apPalco) apPalco.dataset.cheia = "1";
  else if (apPalco.dataset.cheia) apSair();
});
$("b-apresentar")?.addEventListener("click", (e) => abrirApresentacao(tela, e.currentTarget));
