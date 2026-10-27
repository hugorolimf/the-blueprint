// ---- aplicar sozinho: a tela ----
// O interruptor no cabeçalho do Planejar, o aviso com Desfazer (por 60 s) que aparece quando algo entra sozinho, a lista "Aplicado sozinho" acima dos registros e a nota
// "precisa da sua confirmação" na proposta que apaga ou move. Regras em auto-aplicar-logica.js, aplicação em auto-aplicar.js. Texto de agente nunca entra como HTML.
// Zero trabalho em repouso: o único timer existe enquanto o aviso está na tela (a janela do Desfazer).
let autoAvisoEl = null, autoAvisoTimer = 0, autoAvisoChave = "";
const autoHora = (ms) => new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
const autoTextoDe = (e) => (e.desfeito ? tr("Desfeito: {texto}", { texto: e.texto }) : e.texto);

function autoAvisoCria() {
  const el = document.createElement("div");
  el.id = "auto-aviso"; el.hidden = true;
  el.setAttribute("role", "status"); el.setAttribute("aria-live", "polite");
  el.innerHTML = `<b class="aa-rot">${esc(tr("Aplicado sozinho"))}</b><span class="aa-texto"></span><button class="btn mini aa-desfazer" type="button">${esc(tr("Desfazer"))}</button><button class="btn mini aa-fechar" type="button" aria-label="${esc(tr("Fechar"))}">×</button>`;
  el.querySelector(".aa-desfazer").onclick = () => autoDesfaz(autoAvisoChave);
  el.querySelector(".aa-fechar").onclick = autoAvisoFecha;
  document.body.append(el);
  return el;
}
function autoAvisoFecha() {
  clearTimeout(autoAvisoTimer);
  if (autoAvisoEl) autoAvisoEl.hidden = true;
}
// `e` acabou de entrar sozinho (ou de ser desfeito): mostra o aviso, que some com a janela do Desfazer, e atualiza a lista do Planejar
function autoMostra(e) {
  autoAvisoEl ||= autoAvisoCria();
  autoAvisoChave = e.chave;
  autoAvisoEl.querySelector(".aa-texto").textContent = autoTextoDe(e);
  autoAvisoEl.querySelector(".aa-desfazer").hidden = !autoPodeDesfazer(e, Date.now());
  autoAvisoEl.hidden = false;
  clearTimeout(autoAvisoTimer);
  autoAvisoTimer = setTimeout(autoAvisoFecha, AUTO_JANELA_MS);
  if (!hub.hidden) montaAutoAplicados();
}

// a nota na proposta aberta que o interruptor não aplica sozinho (apaga ou move): entra no topo dela por htmlTopoProposta (proposta-vista.js)
function autoNotaConfirmar(c, p) {
  if (!p || p.id === NOVO || !autoPrecisaConfirmar(c) || !autoEstadoDe(p.id).ligado) return "";
  return `<p class="dica aa-nota">${esc(tr("A aplicação automática está ligada, mas esta proposta apaga ou move itens: ela precisa da sua confirmação."))}</p>`;
}

// o interruptor e a lista, criados na primeira vez que o Planejar mostra um blueprint (montaMcp, proposta.js)
function montaAutoAplicados() {
  const p = blueprintEd(), novo = edProj === NOVO;
  let chave = $("ag-auto");
  if (!chave) {
    chave = Object.assign(document.createElement("button"), { id: "ag-auto", className: "btn", type: "button" });
    chave.onclick = () => { const b = blueprintEd(); if (b && edProj !== NOVO) { autoDefine(b.id, !autoEstadoDe(b.id).ligado); montaAutoAplicados(); montaProposta(); } };
    $("cu-abrir").before(chave);
  }
  const est = p && !novo ? autoEstadoDe(p.id) : { pode: false, ligado: false };
  chave.hidden = !est.pode; // modo equipe: só o dono vê; quem lê e o convidado nunca aplicam sozinhos
  chave.setAttribute("aria-pressed", String(est.ligado));
  chave.textContent = est.ligado ? tr("Aplicar sozinho: ligado") : tr("Aplicar sozinho: desligado");
  chave.title = tr("O que o seu agente enviar entra no blueprint sem outro clique, com aviso e Desfazer por 60 s. Apagar e mover ações entre blueprints continuam pedindo o seu clique.");
  let caixa = $("cl-auto");
  if (!caixa) {
    caixa = Object.assign(document.createElement("div"), { id: "cl-auto", className: "cl-prop", hidden: true });
    caixa.setAttribute("aria-label", tr("Aplicado sozinho"));
    caixa.addEventListener("click", (e) => { const b = e.target.closest("[data-aa]"); if (b) autoDesfaz(b.dataset.aa); });
    $("cl-reg").before(caixa);
  }
  const ws = workspaceAtual?.id || "sm", lista = p && !novo ? autoLista.filter((e) => e.ws === ws && e.blueprintId === p.id).slice(-5).reverse() : [];
  caixa.hidden = !lista.length;
  if (!lista.length) return void caixa.replaceChildren();
  const agora = Date.now();
  caixa.innerHTML = `<div class="cl-prop-topo rg-cab"><b>${esc(tr("Aplicado sozinho"))} · ${lista.length}</b></div>${lista.map((e) => `<div class="aa-item"><span class="aa-hora">${esc(autoHora(e.em))}</span><span class="aa-texto">${esc(autoTextoDe(e))}</span>${autoPodeDesfazer(e, agora) ? `<button class="btn mini" type="button" data-aa="${esc(e.chave)}">${esc(tr("Desfazer"))}</button>` : e.desfeito ? `<span class="badge">${esc(tr("desfeito"))}</span>` : ""}</div>`).join("")}`;
}
