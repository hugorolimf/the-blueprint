// ---- situação e prazo da ação direto pelo menu do botão direito (a mesma regra do Editar: com planos ou checkpoints a situação vem deles) ----
const SEM_CONCLUIR_COM_ITENS = "Conclui sozinha quando todos os planos e checkpoints estiverem marcados";

function mudarSituacaoAcao(pid, fid, status) {
  const f = acaoDe(pid, fid);
  if (!f) return;
  mudarComSaida(pid, () => {
    f.status = status;
    f.progresso = status === "concluido" ? 1 : status === "planejado" ? 0 : f.progresso;
    if (document.body.classList.contains("editando")) montaEditor();
  });
}
// prorroga o fim; os checkpoints continuam dentro do período porque só o fim se afasta
function prorrogarAcao(pid, fid, dias) {
  const f = acaoDe(pid, fid);
  if (!f || !(dia(f.fim) === dia(f.fim))) return;
  mudarComSaida(pid, () => { f.fim = deDia(dia(f.fim) + dias); });
}

/** Os itens de situação e de prazo de uma ação: `{ grupo, icone, rotulo, fn, desabilitado?, motivo? }`. */
function itensDeSituacao(pid, f) {
  const e = efetiva(f), itens = [];
  const situacao = (status, icone, rotulo) => itens.push({ grupo: "estado", icone, rotulo, fn: () => mudarSituacaoAcao(pid, f.id, status) });
  if (e.status === "concluido" && e.it[1]) return itens; // concluída por planos e checkpoints: só desmarcá-los muda isso
  if (!e.it[1] || !e.it[0]) { // sem itens, ou com itens e nenhum feito: planejada e em andamento são escolha da pessoa (concluída sem itens também reabre aqui)
    if (e.status !== "planejado") situacao("planejado", "circulo", "Marcar como planejada");
    if (e.status !== "andamento") situacao("andamento", "relogio", "Marcar como em andamento");
  }
  if (e.status !== "concluido") {
    if (e.it[1]) itens.push({ grupo: "estado", icone: "check", rotulo: "Marcar como concluída", fn() {}, desabilitado: true, motivo: SEM_CONCLUIR_COM_ITENS });
    else situacao("concluido", "check", "Marcar como concluída");
    itens.push({ grupo: "prazo", icone: "calendario", rotulo: "Prorrogar o fim em 1 dia", fn: () => prorrogarAcao(pid, f.id, 1) });
    itens.push({ grupo: "prazo", icone: "calendario", rotulo: "Prorrogar o fim em 1 semana", fn: () => prorrogarAcao(pid, f.id, 7) });
  }
  return itens;
}
