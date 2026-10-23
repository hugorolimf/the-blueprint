// Marcar plano, checkpoint ou ação como concluído (pelo clique ou pelo MCP) redesenha a folha; aqui a troca é animada em vez de abrupta:
// a barra de progresso cresce até o novo valor (organizarFolha, em barras.js) e, com o filtro "Pendentes" ligado, o que deixou de ser pendente
// esmaece e desliza para a direita antes do redesenho, e o que ficou sobe para o lugar dele.
// `mudaDados` (opcional) altera os dados antes de medir quem sai; o DOM só é refeito pelo `mudou()` no fim. Outra tela ou movimento reduzido: só `mudou()`.
async function mudarComSaida(pid, mudaDados) {
  const ativo = pid === tela && !reduz();
  const antes = ativo ? posicoesDaFolha() : null;
  mudaDados?.();
  const p = ativo && soPendentes[pid] && D.blueprints.find((x) => x.id === pid);
  if (p) {
    const fica = new Set(acoesVisiveis(p).flatMap((f) => [f.id, ...f.planos.map((t) => t.id), ...f.checkpoints.map((m) => m.id)]));
    const saem = [...host.querySelectorAll(ITENS_FOLHA)].filter((el) => !fica.has(el.dataset.id || el.dataset.f));
    await Promise.all(saem.map((el) => el.animate([{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateX(28px)" }], { duration: 320, easing: "cubic-bezier(.45,.05,.2,1)", fill: "forwards" }).finished.catch(() => {})));
  }
  mudou();
  if (antes) organizarFolha(antes);
}
