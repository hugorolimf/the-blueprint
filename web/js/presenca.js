// ---- presença: quem está vendo o workspace (avatares com nome e cor em #presenca) e o contorno do item que outra pessoa está editando -----
// O contorno vive numa camada própria (#presenca-camada, sobre a folha, como a do Claude), então o render() da folha não muda nem o apaga:
// depois de cada desenho a camada relê as caixas no SVG. Só existe no modo hospedado: no local o servidor nunca manda `presenca` e a página
// também não manda nada. Ver docs/colaboracao.md, seção 6 ("Presença").
const PRESENCA_CORES = 6, PRESENCA_MAX = 6;
let presencaLista = [], presencaLigada = false, presencaT = 0, presencaUltima = "";

// ---- lógica pura (presenca.test.mjs) ----
/** A cor do membro: sempre a mesma para o mesmo id, uma das `PRESENCA_CORES` do tema (--pres-0 ... --pres-5). */
function presencaCor(id) {
  let h = 0;
  for (const c of String(id)) h = (h * 31 + c.codePointAt(0)) >>> 0;
  return h % PRESENCA_CORES;
}
function presencaIniciais(nome) {
  const partes = String(nome || "?").trim().split(/\s+/).filter(Boolean);
  return ((partes[0]?.[0] || "?") + (partes.length > 1 ? partes.at(-1)[0] : "")).toUpperCase();
}
/** As pessoas a mostrar: sem esta aba, uma por membro (o item em edição de qualquer aba dela). */
function presencaPessoas(lista, cliente) {
  const porMembro = new Map();
  for (const p of lista || []) {
    if (!p || p.cliente === cliente) continue;
    const ja = porMembro.get(p.id);
    if (!ja) porMembro.set(p.id, { ...p });
    else if (!ja.editando && p.editando) Object.assign(ja, { editando: p.editando, blueprintId: p.blueprintId, acaoId: p.acaoId });
  }
  return [...porMembro.values()];
}
/** Os contornos da tela: quem edita um item do blueprint aberto. */
function presencaContornos(pessoas, blueprintId) {
  return pessoas.filter((p) => p.editando && p.blueprintId === blueprintId).map((p) => ({ ...p.editando, nome: p.nome, cor: presencaCor(p.id) }));
}

// ---- o que esta aba manda: blueprint, ação e item em edição (só quando o servidor tem presença) ----
function presencaOnde() {
  const editor = $("editor"), aberto = editor && !editor.hidden;
  const foco = aberto && document.activeElement?.closest?.("#editor") ? document.activeElement : null;
  const linha = foco?.closest?.("[data-t], [data-m]");
  const acaoId = aberto && typeof edSel === "string" && edSel !== "blueprint" ? edSel : "";
  const editando = linha ? { tipo: linha.dataset.t ? "plano" : "checkpoint", id: linha.dataset.t || linha.dataset.m } : foco && acaoId ? { tipo: "acao", id: acaoId } : null;
  return { workspaceId: workspaceAtual?.id || "sm", blueprintId: (aberto ? edProj : tela) || "", acaoId, editando, cliente: CLIENTE };
}
function presencaEnviar() {
  if (!presencaLigada) return;
  clearTimeout(presencaT);
  presencaT = setTimeout(() => {
    const corpo = JSON.stringify(presencaOnde());
    if (corpo === presencaUltima) return;
    presencaUltima = corpo;
    fetch("/api/presenca", { method: "POST", headers: { "content-type": "application/json" }, body: corpo }).catch(() => {});
  }, 150);
}
for (const ev of ["focusin", "focusout"]) document.addEventListener(ev, presencaEnviar);
window.addEventListener("hashchange", presencaEnviar);

// ---- o que chega: a lista do workspace ----
function presencaRecebida(dados) {
  presencaLigada = true;
  presencaLista = presencaPessoas(dados.pessoas, CLIENTE);
  presencaDesenha();
  presencaEnviar();
}
function presencaDesenha() {
  const caixa = $("presenca");
  if (!caixa) return;
  const pessoas = presencaLista, extra = pessoas.length - PRESENCA_MAX;
  caixa.hidden = !pessoas.length;
  caixa.setAttribute("aria-label", trn(pessoas.length, ["{n} pessoa vendo este workspace", "{n} pessoas vendo este workspace"]));
  caixa.replaceChildren(...pessoas.slice(0, PRESENCA_MAX).map((p) => {
    const el = document.createElement("span");
    el.className = "pres-av";
    el.style.setProperty("--c", `var(--pres-${presencaCor(p.id)})`);
    el.textContent = presencaIniciais(p.nome);
    el.title = p.editando ? tr("{nome} está editando", { nome: p.nome }) : p.nome;
    return el;
  }), ...(extra > 0 ? [Object.assign(document.createElement("span"), { className: "pres-av pres-mais", textContent: "+" + extra })] : []));
  presencaCamada();
}

// ---- o contorno: lê a caixa do item no SVG da folha e põe um retângulo com o nome na camada ----
function presencaCamada() {
  let camada = $("presenca-camada");
  if (!camada) {
    camada = Object.assign(document.createElement("div"), { id: "presenca-camada" });
    camada.setAttribute("aria-hidden", "true");
    $("claude-camada")?.after(camada);
  }
  const svg = host.querySelector("svg.desenho"), contornos = svg && tela ? presencaContornos(presencaLista, tela) : [];
  const c = camada.getBoundingClientRect();
  camada.replaceChildren(...contornos.flatMap((it) => {
    const alvo = it.tipo === "plano" ? host.querySelector(`[data-tipo="t"][data-id="${CSS.escape(it.id)}"]`) : it.tipo === "checkpoint" ? host.querySelector(`[data-tipo="m"][data-id="${CSS.escape(it.id)}"]`) : it.tipo === "acao" ? host.querySelector(`.acao[data-f="${CSS.escape(it.id)}"] .caixa`) : null;
    if (!alvo) return [];
    const r = alvo.getBoundingClientRect(), el = document.createElement("div");
    el.className = "pres-contorno";
    el.style.cssText = `--c:var(--pres-${it.cor});left:${r.left - c.left - 3}px;top:${r.top - c.top - 3}px;width:${r.width + 6}px;height:${r.height + 6}px`;
    el.append(Object.assign(document.createElement("span"), { textContent: it.nome }));
    return [el];
  }));
}
// depois de cada desenho da folha (o render troca o conteúdo de #desenho) e ao mudar o tamanho: só se houver alguém editando
new MutationObserver(() => { if (presencaLista.some((p) => p.editando)) presencaCamada(); }).observe(host, { childList: true });
window.addEventListener("resize", () => { if (presencaLista.some((p) => p.editando)) presencaCamada(); });
