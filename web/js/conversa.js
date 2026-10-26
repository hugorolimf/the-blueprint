// ---- "Conversar com o plano": painel lateral por ação. A mensagem vai à sessão do Claude escolhida (POST /api/mcp/conversa, pelo canal dela) e a resposta volta
// pela ferramenta responder_conversa (evento SSE "conversa" → GET /api/mcp/conversa). Contrato em docs/claude-na-folha.md ("Enviar comentário do plano e conversar
// com o plano"). Nada aqui grava nos dados: a IA só propõe, e a proposta citada abre no Planejar para o usuário aplicar ou descartar como sempre.
// Quem chama: iniciar-acao.js (botão na folha da ação), contexto-itens.js (botão direito na ação), mcp.js (evento "conversa"), comentario-agente.js (conversaPost).
let conversaAlvo = null; // { p, f, sessoes }
const conversaWs = () => workspaceAtual?.id || "sm";
const conversaNomeSessao = (s) => (s.pasta || "").replace(/[\\/]+$/, "").split(/[\\/]/).pop() || s.pasta;

// envia o texto (comentário ou conversa) à sessão escolhida; sem sessão com canal, só copia. Devolve { ok, texto } para a tela mostrar
async function conversaPost(p, f, extra, sessao) {
  const corpo = { workspaceId: conversaWs(), blueprintId: p.id, acaoId: f.id, ...extra, ...(sessao?.canal ? { sessaoId: sessao.id } : { semCanal: true }) };
  try {
    const r = await fetch("/api/mcp/conversa", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
    const j = await r.json().catch(() => ({}));
    if (r.ok && j.entregue) return { ok: true, texto: tr("Enviado à sessão do Claude.") };
    if (r.ok) {
      try { await navigator.clipboard.writeText(j.prompt); return { ok: true, texto: tr("Copiado: cole na sessão do Claude.") }; }
      catch { return { ok: false, texto: tr("Não consegui copiar sozinho. O texto é: {prompt}", { prompt: j.prompt }) }; }
    }
    if (j.codigo === "sessao-caiu") return { ok: false, texto: tr("Essa sessão não está mais conectada ao app. Escolha outra ou abra uma nova.") };
    if (j.codigo === "sessao-de-outra-pasta") return { ok: false, texto: tr("Essa sessão não é de uma pasta deste blueprint. Escolha outra ou abra uma nova.") };
    if (j.codigo === "texto-longo") return { ok: false, texto: tr("Texto longo demais: encurte e envie de novo.") };
    return { ok: false, texto: j.erro || tr("Não foi possível enviar (erro {status}).", { status: r.status }) };
  } catch { return { ok: false, texto: tr("Não consegui falar com o app. Ele está aberto?") }; }
}

// as opções de sessão (<option>): as abertas na pasta e "Só copiar"; a primeira com canal vem marcada
function conversaOpcoes(sessoes) {
  const padrao = sessoes.findIndex((s) => s.canal);
  return sessoes.map((s, i) => `<option value="${i}"${i === padrao ? " selected" : ""}>${esc(s.canal ? tr("Sessão em {nome}", { nome: conversaNomeSessao(s) }) : tr("Sessão em {nome} (sem canal: copia)", { nome: conversaNomeSessao(s) }))}</option>`).join("")
    + `<option value="copiar"${padrao < 0 ? " selected" : ""}>${esc(tr("Só copiar a mensagem"))}</option>`;
}

function conversaPainel() {
  let el = $("conversa");
  if (el) return el;
  el = document.createElement("aside");
  el.id = "conversa";
  el.className = "conversa";
  el.setAttribute("role", "dialog");
  el.hidden = true;
  el.addEventListener("click", (e) => {
    if (e.target.closest("[data-cv-fecha]")) fechaConversa();
    const b = e.target.closest("[data-cv-proposta]");
    if (b) conversaAbreProposta(+b.dataset.cvProposta);
  });
  el.addEventListener("submit", (e) => { e.preventDefault(); conversaEnvia(); });
  el.addEventListener("keydown", (e) => {
    if (e.key === "Escape") fechaConversa();
    if (e.key === "Enter" && e.ctrlKey && e.target.matches("textarea")) { e.preventDefault(); conversaEnvia(); }
  });
  document.body.append(el);
  return el;
}

async function abreConversa(pid, fid) {
  const p = D.blueprints.find((x) => x.id === pid), f = p?.acoes.find((x) => x.id === fid);
  if (!f) return;
  fechaNota?.();
  const sessoes = await sessoesAbertas(p);
  conversaAlvo = { p, f, sessoes };
  const el = conversaPainel(), titulo = `${numeroDaAcao(p, f)} · ${f.nome || tr("sem nome")}`;
  el.setAttribute("aria-label", tr("Conversar com o plano") + " " + titulo);
  el.innerHTML = `<div class="cv-topo"><div><h4>${esc(tr("Conversar com o plano"))}</h4><h3>${esc(titulo)}</h3></div><button class="btn mini" type="button" data-cv-fecha>${esc(tr("Fechar"))}</button></div>
    <p class="cv-dica">${esc(tr("A mensagem vai ao Claude da sua pasta. Ele responde aqui; o que ele mudar no blueprint chega como proposta, que você aplica ou descarta no Planejar."))}</p>
    <ol class="cv-lista" aria-live="polite"></ol>
    <form class="cv-envio">${sessoes.length ? "" : `<p class="cv-dica">${esc(tr("Nenhuma sessão do Claude aberta nesta pasta. Abra uma com “Iniciar com o Claude” ou copie a mensagem."))}</p>`}
      <label class="cv-sessao">${esc(tr("Enviar para"))} <select class="btn mini" data-cv-sessao>${conversaOpcoes(sessoes)}</select></label>
      <textarea data-cv-texto maxlength="800" rows="4" placeholder="${esc(tr("Ex.: divida o plano 2 em dois e mova o checkpoint para sexta."))}" aria-label="${esc(tr("Mensagem"))}"></textarea>
      <div class="cv-acoes"><span class="cv-aviso" role="status"></span><button class="btn mini cheio" type="submit">${esc(tr("Enviar"))}</button></div>
    </form>`;
  el.hidden = false;
  el.querySelector("textarea").focus();
  conversaCarrega();
}

function fechaConversa() {
  const el = $("conversa");
  if (el) el.hidden = true;
  conversaAlvo = null;
}

function conversaAviso(texto, erro) {
  const el = $("conversa")?.querySelector(".cv-aviso");
  if (!el) return;
  el.textContent = texto;
  el.classList.toggle("erro", !!erro);
}

async function conversaCarrega() {
  const a = conversaAlvo;
  if (!a) return;
  try {
    const r = await fetch(`/api/mcp/conversa?workspace=${encodeURIComponent(conversaWs())}&blueprint=${encodeURIComponent(a.p.id)}&acao=${encodeURIComponent(a.f.id)}`);
    if (!r.ok || a !== conversaAlvo) return;
    conversaDesenha((await r.json()).mensagens || []);
  } catch {}
}

function conversaDesenha(mensagens) {
  const lista = $("conversa")?.querySelector(".cv-lista");
  if (!lista) return;
  const hora = (ms) => new Date(ms).toLocaleTimeString(I18N.ativo, { hour: "2-digit", minute: "2-digit" });
  lista.innerHTML = mensagens.length ? mensagens.map((m) => `<li class="cv-msg ${m.de === "agente" ? "agente" : "usuario"}"><span class="cv-quem">${esc(m.de === "agente" ? "Claude" : tr("Você"))} · ${esc(hora(m.em))}</span><p>${esc(m.texto)}</p>${Number.isInteger(m.proposta) ? `<button class="btn mini" type="button" data-cv-proposta="${m.proposta}">${esc(tr("Abrir a proposta nº {numero}", { numero: m.proposta }))}</button>` : ""}</li>`).join("")
    : `<li class="cv-vazio">${esc(tr("Nenhuma mensagem ainda. A conversa fica só na memória do app."))}</li>`;
  lista.scrollTop = lista.scrollHeight;
}

// o clique em Enviar é a confirmação: a sessão escolhida está à vista, ao lado do botão
async function conversaEnvia() {
  const a = conversaAlvo, el = $("conversa");
  if (!a || !el) return;
  const ta = el.querySelector("[data-cv-texto]"), texto = ta.value.trim(), v = el.querySelector("[data-cv-sessao]").value;
  if (!texto) return conversaAviso(tr("Escreva a mensagem."), true);
  const botao = el.querySelector('[type="submit"]');
  botao.disabled = true;
  conversaAviso(tr("Enviando…"));
  const r = await conversaPost(a.p, a.f, { tipo: "conversa", texto }, v === "copiar" ? null : a.sessoes[+v]);
  botao.disabled = false;
  conversaAviso(r.texto, !r.ok);
  if (r.ok) { ta.value = ""; conversaCarrega(); }
}

// evento SSE "conversa" ({workspaceId, blueprintId, acaoId}, sem texto): só o painel daquela ação busca a lista
function conversaRecebe(d) {
  const a = conversaAlvo;
  if (a && d.workspaceId === conversaWs() && d.blueprintId === a.p.id && d.acaoId === a.f.id) conversaCarrega();
}

// a proposta citada pela IA abre no Planejar; aplicar continua sendo o clique de sempre
function conversaAbreProposta(numero) {
  const a = conversaAlvo;
  if (!a) return;
  const c = conv(a.p.id);
  if (!filaTodas(c).some((x) => x.numero === numero)) return conversaAviso(tr("A proposta nº {numero} não está mais na fila.", { numero }), true);
  fechaConversa();
  agenteTela(true, { proj: a.p.id });
  if (filaAbre(c, numero)) { salvar(); montaProposta(); }
}
