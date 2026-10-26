// ---- modo foco: a visão de acompanhamento dos agentes (cartões em foco-vista.js e foco-cartao.js; lógica pura em foco-logica.js) ----
// Ligado (botão Foco ou Alt+F), uma tela própria cobre a folha com um cartão por agente; clicar num cartão fixa o foco nele (histórico); Esc sobe um degrau (detalhe -> lista -> folha).
// O foco SEGUE o agente em foco (o fixado ou o principal): quando ele troca de plano ou de ação o cartão acompanha (a visão se refaz com os dados novos) e, se saiu da faixa à vista, rola até ele.
// Só indicador: não manda nada ao agente. Zero trabalho em repouso: nenhum timer nem polling; tudo corre nos avisos que já existem (render() e agentesAtualiza() chamam focoDepoisDoRender)
// e em UM quadro (rAF) por aviso, e só com a visão aberta. «Ir ao plano na folha» fecha a visão e leva a folha até lá.
let focoAberto = false, focoFixo = "", focoVista = "", focoPendente = null, focoFrame = 0, focoFrameRola = 0; // focoFixo: o nome do agente fixado; focoVista: o lugar do agente em foco; focoPendente: o plano a mostrar na folha
const focoBotao = $("b-foco");

// chamado depois de cada render() (robo.js, montaRobo) e a cada aviso de presença (agentes.js, agentesAtualiza)
function focoDepoisDoRender() {
  if (focoPendente) { cancelAnimationFrame(focoFrameRola); focoFrameRola = requestAnimationFrame(focoRola); } // continua o «Ir ao plano na folha» depois de a folha ser refeita
  if (!focoAberto) return;
  cancelAnimationFrame(focoFrame);
  focoFrame = requestAnimationFrame(focoQuadro);
}
function focoQuadro() {
  if (!focoAberto) return;
  focoVistaRender();
  const emFoco = focoEmFoco(focoAgentes(), focoFixo), prox = focoProximo(focoVista, emFoco);
  focoVista = prox.chave;
  if (!prox.mudou || focoFixo) return; // o detalhe é um cartão só: já está à vista
  const el = focoVistaEl.querySelector(`[data-agente="${CSS.escape(emFoco.agente)}"]`); // o cartão do agente em foco mudou de plano: rola até ele se saiu da faixa à vista
  if (el && focoDeveRolar(el.getBoundingClientRect(), innerHeight)) el.scrollIntoView({ block: "center", behavior: reduz() ? "auto" : "smooth" });
}

function focoAbre(agente = "") {
  if (!workspaceAtual || !carregado || !ed.hidden || !hub.hidden) return; // com o Editar ou o Planejar aberto, não abre
  focoFixo = agente; focoVista = "";
  if (focoAberto) { focoVistaRender(); return; }
  focoAberto = true;
  focoBotao.setAttribute("aria-pressed", "true");
  focoVistaAbre();
  focoVista = focoLugar(focoEmFoco(focoAgentes(), focoFixo));
}
function focoFecha(devolveFoco = true) {
  if (!focoAberto) return;
  focoAberto = false; focoFixo = ""; focoVista = "";
  cancelAnimationFrame(focoFrame);
  focoVistaFecha();
  focoBotao.setAttribute("aria-pressed", "false");
  if (devolveFoco) focoBotao.focus({ preventScroll: true });
}
const focoSobe = () => { focoFixo = ""; focoVista = ""; focoVistaRender(); }; // do detalhe de volta à lista
const focoAlterna = () => (focoAberto ? focoFecha() : focoAbre());

// «Ir ao plano na folha»: fecha a visão e leva a folha até o plano (abre o blueprint, a seção e a ação se preciso; cada passo refaz a folha, que chama o foco de novo) e rola só se saiu da faixa à vista
function focoVaiPara(blueprintId, acaoId, planoId) {
  focoFecha(false);
  focoPendente = { blueprintId, acaoId, planoId, n: 0 };
  focoRola();
}
function focoRola() {
  const alvo = focoPendente;
  if (!alvo) return;
  const p = D.blueprints.find((x) => x.id === alvo.blueprintId);
  if (!p || ++alvo.n > 4) { focoPendente = null; return; }
  if (tela !== p.id) { irBlueprint(p.id); return; } // a folha nova chama focoRola de novo (depois do render)
  const busca = () => host.querySelector(alvo.planoId ? `.chk[data-id="${CSS.escape(alvo.planoId)}"]` : `.acao[data-f="${CSS.escape(alvo.acaoId)}"]`);
  const el = busca();
  if (!el && !alvo.abriu) { alvo.abriu = true; abertos[p.id] = true; abrirProjetoDaAcao(p, alvo.acaoId); render(); return; }
  focoPendente = null;
  if (el && focoDeveRolar(el.getBoundingClientRect(), innerHeight)) el.scrollIntoView({ block: "center", behavior: reduz() ? "auto" : "smooth" });
}

focoBotao.onclick = focoAlterna;
document.addEventListener("keydown", (e) => {
  if (e.altKey && !e.ctrlKey && !e.metaKey && e.key.toLowerCase() === "f") { e.preventDefault(); focoAlterna(); return; } // Alt+F liga e desliga
  if (e.key === "Escape" && focoAberto && !e.defaultPrevented) { e.preventDefault(); focoEsc(focoFixo) === "lista" ? focoSobe() : focoFecha(); } // um degrau por vez
});
