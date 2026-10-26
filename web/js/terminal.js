// ---- o painel do terminal do app: uma aba por sessão (cada "Iniciar com o Claude > Terminal do app" abre uma), dividir em dois, fonte, leitor de tela ----
// As sessões vivem no servidor (src/terminal): esconder o painel, recarregar a página ou mandar a janela para a bandeja não para o Claude.
// Zero trabalho em repouso: só as telas visíveis existem (xterm + SSE); aba escondida não tem tela nem conexão, e o painel fechado não tem nenhuma.
// Teclado: Ctrl+` mostra/esconde, F6 sai do terminal para as abas (Esc é do Claude), setas trocam de aba, Delete fecha. Checklist: docs/terminal-acessibilidade.md.
const termGuardado = (chave, padrao) => { try { return localStorage.getItem(chave) ?? padrao; } catch { return padrao; } };
const termGuardar = (chave, v) => { try { localStorage.setItem(chave, String(v)); } catch {} };
const termEstado = { sessoes: [], ativa: "", lado: "", dividir: false, max: false, vistas: new Map(), fonte: termFonteValida(termGuardado("terminal.fonte", TERM_FONTE.padrao)), leitor: termGuardado("terminal.leitor", "0") === "1" };
let termPainel = null;

function termBotaoBarra() {
  let b = document.getElementById("b-terminal");
  if (!b) {
    b = document.createElement("button");
    Object.assign(b, { id: "b-terminal", type: "button", className: "btn" });
    b.setAttribute("aria-keyshortcuts", "Control+`");
    b.setAttribute("aria-controls", "terminal-painel");
    b.addEventListener("click", () => terminalAlternar());
    document.querySelector("#folha .barra")?.append(b);
  }
  const vivas = termEstado.sessoes.filter((s) => s.viva).length;
  b.textContent = vivas ? tr("Terminal · {n}", { n: vivas }) : tr("Terminal");
  b.setAttribute("aria-expanded", String(!!termPainel && !termPainel.hidden));
  return b;
}

function termMontar() {
  if (termPainel) return termPainel;
  const p = (termPainel = document.createElement("section"));
  Object.assign(p, { id: "terminal-painel", className: "term-painel", hidden: true });
  p.setAttribute("aria-label", tr("Terminal do app"));
  const botao = (t, rotulo, extra = "") => `<button class="btn mini" type="button" data-t="${t}" ${extra}>${rotulo}</button>`;
  p.innerHTML = `<div class="term-topo"><div class="term-abas" role="tablist" aria-label="${esc(tr("Sessões do terminal"))}"></div><div class="term-ferramentas">`
    + botao("dividir", tr("Dividir"), `aria-pressed="false" title="${esc(tr("Mostra duas sessões lado a lado"))}"`)
    + botao("menor", "A−", `aria-label="${esc(tr("Diminuir a fonte"))}" title="Ctrl+-"`) + botao("maior", "A+", `aria-label="${esc(tr("Aumentar a fonte"))}" title="Ctrl+="`)
    + botao("leitor", tr("Leitor de tela"), `aria-pressed="false" title="${esc(tr("Deixa o texto do terminal legível por leitores de tela (mais lento)"))}"`)
    + botao("max", tr("Maximizar"), `aria-pressed="false"`) + botao("esconder", tr("Esconder"), `title="Ctrl+\`" aria-keyshortcuts="Control+\`"`)
    + `</div></div><div class="term-corpo"></div><p class="term-dica">${esc(tr("F6 sai do terminal · Ctrl+PageUp/PageDown troca de sessão · Ctrl+` esconde o painel"))}</p>`;
  p.addEventListener("click", (e) => {
    const t = e.target.closest("[data-t]")?.dataset.t, fechar = e.target.closest("[data-fechar]")?.dataset.fechar, aba = e.target.closest("[role=tab]")?.dataset.id;
    if (fechar) return termFecharSessao(fechar);
    if (aba) return termAtivar(aba, true);
    if (t) termAcao(t);
  });
  p.querySelector(".term-abas").addEventListener("keydown", termTeclaAbas);
  document.body.append(p);
  return p;
}

// atalhos que chegam de dentro do terminal (termTecla) ou dos botões
function termAcao(acao) {
  const ids = termEstado.sessoes.map((s) => s.id), i = ids.indexOf(termEstado.ativa);
  if (acao === "sair") return termPainel.querySelector(`[role=tab][aria-selected=true]`)?.focus();
  if (acao === "sair-voltar") return termPainel.querySelector(`[data-t=esconder]`)?.focus();
  if (acao === "proxima" || acao === "anterior") return ids.length && termAtivar(ids[(i + (acao === "proxima" ? 1 : ids.length - 1)) % ids.length], false, true);
  if (acao === "maior" || acao === "menor" || acao === "normal") {
    termEstado.fonte = acao === "normal" ? TERM_FONTE.padrao : termFonteValida(termEstado.fonte + (acao === "maior" ? 1 : -1));
    termGuardar("terminal.fonte", termEstado.fonte);
    return termEstado.vistas.forEach((v) => v.fonte?.(termEstado.fonte));
  }
  if (acao === "leitor") {
    termEstado.leitor = !termEstado.leitor;
    termGuardar("terminal.leitor", termEstado.leitor ? "1" : "0");
    termEstado.vistas.forEach((v) => v.leitor?.(termEstado.leitor));
    return termFerramentas();
  }
  if (acao === "dividir") { termEstado.dividir = !termEstado.dividir; termFerramentas(); return termMostrar(); }
  if (acao === "max") { termEstado.max = !termEstado.max; return termFerramentas(); }
  if (acao === "esconder" || acao === "painel") terminalEsconder();
}

function termFerramentas() {
  const marca = (t, v) => termPainel.querySelector(`[data-t=${t}]`)?.setAttribute("aria-pressed", String(v));
  marca("dividir", termEstado.dividir); marca("leitor", termEstado.leitor); marca("max", termEstado.max);
  termPainel.classList.toggle("max", termEstado.max);
}

// abas no padrão ARIA: setas e Home/End movem e ativam, Delete fecha, F6 entra no terminal
function termTeclaAbas(e) {
  const ids = termEstado.sessoes.map((s) => s.id), i = ids.indexOf(e.target.closest("[role=tab]")?.dataset.id);
  if (i < 0) return;
  const ir = { ArrowRight: i + 1, ArrowLeft: i - 1 + ids.length, Home: 0, End: ids.length - 1 }[e.key];
  if (ir != null) { e.preventDefault(); return termAtivar(ids[ir % ids.length], true); }
  if (e.key === "Delete") { e.preventDefault(); return termFecharSessao(ids[i]); }
  if (e.key === "F6") { e.preventDefault(); termEstado.vistas.get(termEstado.ativa)?.focar?.(); }
}

function termAbas() {
  if (!termPainel) return termBotaoBarra();
  const lista = termPainel.querySelector(".term-abas"), focada = document.activeElement?.closest?.(".term-abas [role=tab]")?.dataset.id;
  lista.innerHTML = termEstado.sessoes.map((s) => {
    const r = termRotulo(s, typeof agentesLista !== "undefined" ? agentesLista : [], typeof ESTADO_AG !== "undefined" ? ESTADO_AG : {}), sel = s.id === termEstado.ativa;
    return `<div class="term-aba${sel ? " sel" : ""}${r.viva ? "" : " fim"}" role="presentation"><button type="button" role="tab" id="term-aba-${esc(s.id)}" data-id="${esc(s.id)}" aria-selected="${sel}" aria-controls="term-pane-${esc(s.id)}" tabindex="${sel ? 0 : -1}"><span class="t">${esc(r.texto)}</span><small>${esc(r.estado)}</small></button><button type="button" class="term-x" data-fechar="${esc(s.id)}" aria-label="${esc(tr("Fechar a sessão {titulo}", { titulo: r.texto }))}">×</button></div>`;
  }).join("");
  if (focada) lista.querySelector(`[data-id="${CSS.escape(focada)}"]`)?.focus();
  termBotaoBarra();
}

async function termMostrar() {
  if (!termPainel || termPainel.hidden) return;
  const corpo = termPainel.querySelector(".term-corpo"), existe = (id) => termEstado.sessoes.some((s) => s.id === id);
  const visiveis = [termEstado.ativa, termEstado.dividir && termEstado.lado !== termEstado.ativa ? termEstado.lado : ""].filter((id) => id && existe(id));
  for (const [id, v] of termEstado.vistas) if (!visiveis.includes(id)) { v.destruir?.(); v.el.remove(); termEstado.vistas.delete(id); }
  corpo.querySelector(".term-vazio")?.remove();
  corpo.classList.toggle("dividido", visiveis.length > 1);
  if (!visiveis.length) return corpo.insertAdjacentHTML("beforeend", `<p class="term-vazio">${esc(tr("Nenhuma sessão aberta. Na folha de uma ação, use Iniciar com o Claude e escolha Terminal do app."))}</p>`);
  for (const id of visiveis) {
    if (termEstado.vistas.has(id)) continue;
    const el = document.createElement("div"), s = termEstado.sessoes.find((x) => x.id === id), marcador = { el };
    Object.assign(el, { className: "term-tela", id: `term-pane-${id}` });
    el.setAttribute("role", "tabpanel");
    el.setAttribute("aria-labelledby", `term-aba-${id}`);
    corpo.append(el);
    termEstado.vistas.set(id, marcador);
    try {
      const v = await termVista(s, el, { fonte: termEstado.fonte, leitor: termEstado.leitor, aoTecla: termAcao, aoFim: () => termAtualizar() });
      if (termEstado.vistas.get(id) !== marcador) { v.destruir(); continue; } // fechada enquanto o xterm carregava
      Object.assign(marcador, v);
      if (id === termEstado.ativa && termPainel.contains(document.activeElement) === false) v.focar();
    } catch { el.textContent = tr("Não consegui carregar o terminal."); }
  }
  for (const id of visiveis) corpo.append(termEstado.vistas.get(id).el); // a ativa primeiro, à esquerda
}

function termAtivar(id, focarAba = false, focarTela = false) {
  if (id !== termEstado.ativa) [termEstado.lado, termEstado.ativa] = [termEstado.ativa, id];
  termAbas();
  termMostrar().then(() => { if (focarTela) termEstado.vistas.get(id)?.focar?.(); });
  if (focarAba) termPainel.querySelector(`[role=tab][data-id="${CSS.escape(id)}"]`)?.focus();
}

// a lista vem do servidor; sem ele (ou erro), fica a que já havia
async function termAtualizar() {
  try {
    const r = await fetch("/api/terminal/sessoes");
    if (r.ok) termEstado.sessoes = (await r.json()).sessoes || [];
  } catch {}
  if (!termEstado.sessoes.some((s) => s.id === termEstado.ativa)) termEstado.ativa = termEstado.sessoes.at(-1)?.id || "";
  termAbas();
  return termMostrar();
}

async function termFecharSessao(id) {
  const s = termEstado.sessoes.find((x) => x.id === id);
  if (!s) return;
  if (s.viva && !(await confirmar({ titulo: tr("Fechar a sessão?"), texto: tr("O Claude desta sessão será encerrado. O que ele já gravou nos arquivos fica."), rotulo: tr("Fechar a sessão") }))) return;
  await termPost("fechar", { id }).catch(() => {});
  if (termEstado.ativa === id) termEstado.ativa = "";
  await termAtualizar();
  (termPainel.querySelector("[role=tab][aria-selected=true]") || termPainel.querySelector("[data-t=esconder]"))?.focus();
}

async function terminalMostrar(id) {
  const p = termMontar();
  const estava = !p.hidden;
  p.hidden = false;
  document.body.classList.add("com-terminal");
  if (id) termEstado.ativa = id;
  termFerramentas();
  await termAtualizar();
  if (!estava && !id) (termPainel.querySelector("[role=tab][aria-selected=true]") || termPainel.querySelector("[data-t=esconder]"))?.focus();
  if (id) termEstado.vistas.get(id)?.focar?.();
}

function terminalEsconder() {
  if (!termPainel || termPainel.hidden) return;
  termEstado.vistas.forEach((v) => { v.destruir?.(); v.el.remove(); }); // painel fechado não tem tela nem conexão
  termEstado.vistas.clear();
  termPainel.hidden = true;
  document.body.classList.remove("com-terminal");
  termBotaoBarra().focus();
}

const terminalAlternar = () => (termPainel && !termPainel.hidden ? terminalEsconder() : terminalMostrar());
// uma sessão que o "Iniciar com o Claude" acabou de abrir (iniciar-acao.js)
function terminalAbrirSessao(sessao) {
  if (sessao && !termEstado.sessoes.some((s) => s.id === sessao.id)) termEstado.sessoes.push(sessao);
  return terminalMostrar(sessao?.id);
}
// o estado das abas acompanha os avisos dos agentes (agentes.js chama a cada atualização)
function terminalAtualizaAbas() { if (termPainel && !termPainel.hidden) termAbas(); }

document.addEventListener("keydown", (e) => {
  if (e.ctrlKey && !e.altKey && !e.metaKey && e.key === "`" && !e.defaultPrevented) { e.preventDefault(); terminalAlternar(); }
});
termBotaoBarra();
fetch("/api/terminal/sessoes").then((r) => (r.ok ? r.json() : null)).then((j) => { if (j) { termEstado.sessoes = j.sessoes || []; termBotaoBarra(); } }).catch(() => {}); // uma vez: o botão mostra quantas seguem rodando
