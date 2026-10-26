// O campo de pesquisa da barra: digitar (com um pequeno atraso, sem timer em repouso), Esc limpa e sai, Enter/Shift+Enter percorrem os resultados,
// "/" e Ctrl+K focam o campo de qualquer lugar da folha. A consulta é uma só (`consulta`, entrada.js) e vale para a tela que está aberta.
const PESQ_ATRASO = 90; // ms entre a última tecla e o filtro: a folha não é refeita a cada letra
const campoBusca = $("busca");
let pesqTimer = 0;

function pesquisaAplica() {
  clearTimeout(pesqTimer); pesqTimer = 0;
  if (tela) pesquisaFolha(); else render(); // a lista de blueprints refaz os cartões (poucos); o blueprint só marca o que já está desenhado
}
function pesquisaLimpa() {
  consulta = ""; campoBusca.value = "";
  pesquisaAplica();
}
campoBusca.addEventListener("input", () => {
  consulta = campoBusca.value;
  clearTimeout(pesqTimer);
  if (!consulta.trim()) return pesquisaAplica(); // apagar tudo volta na hora
  pesqTimer = setTimeout(pesquisaAplica, PESQ_ATRASO);
});
campoBusca.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (consulta) { e.stopPropagation(); pesquisaLimpa(); } else campoBusca.blur();
  } else if (e.key === "Enter" && tela) {
    e.preventDefault();
    if (pesqTimer) pesquisaAplica(); // Enter logo após digitar: aplica antes de andar
    pesqVaiPara(e.shiftKey ? -1 : 1);
  }
});

// o campo só está ao alcance na folha (lista ou blueprint): fora dela, em diálogos e no menu, "/" e Ctrl+K não fazem nada
const pesqAoAlcance = () =>
  !document.body.classList.contains("carregando") && campoBusca.getClientRects().length > 0 && menuContexto.hidden
  && !document.querySelector("#editor:not([hidden]),#agente-tela:not([hidden]),#roadmap-tela:not([hidden]),#proto-tela:not([hidden]),#workspace-nova:not([hidden]),dialog[open]");
document.addEventListener("keydown", (e) => {
  if (e.defaultPrevented || e.altKey) return;
  const atalho = (e.key === "/" && !e.ctrlKey && !e.metaKey) || ((e.key === "k" || e.key === "K") && (e.ctrlKey || e.metaKey) && !e.shiftKey);
  if (!atalho) return;
  const t = e.target;
  if (t === campoBusca ? e.key === "/" : t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName)) return; // digitando (neste ou em outro campo): a barra é só uma tecla de texto
  if (!pesqAoAlcance()) return;
  e.preventDefault();
  campoBusca.focus(); campoBusca.select();
});
