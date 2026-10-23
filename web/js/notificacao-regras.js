// ---- notificações do agente: QUAIS eventos da fila do MCP viram aviso na tela, com que título e mensagem (puro, sem DOM; testado em notificacao.test.mjs) ----
// O progresso de edição (registrar_progresso) não avisa de propósito: chega a cada arquivo e já tem o robô da folha. Só o que pede atenção do usuário.
const NOTIF_IDADE_MAX = 2 * 60 * 1000; // evento mais velho que isso (a fila de antes de a página abrir) só é aplicado, sem avisar
const NOTIF_FILA_MAX = 5; // passou disso, o aviso de menor prioridade mais antigo some (a contagem "+N" avisa que houve mais)
const NOTIF_MSG_MAX = 4000;
// prioridade: 3 = travou ou não deu, 2 = pede revisão ou entrega, 1 = informativo; nivel pinta o cartão
const NOTIF_REGRAS = {
  bloqueio: { titulo: "O agente travou", prioridade: 3, nivel: "erro" },
  "nao-concluida": { titulo: "Tarefa não concluída", prioridade: 3, nivel: "erro" },
  "checkpoint-nao-feito": { titulo: "Checkpoint não atingido", prioridade: 3, nivel: "erro" },
  proposta: { titulo: "Nova proposta do agente", prioridade: 2, nivel: "" },
  concluir: { titulo: "Tarefa concluída", prioridade: 2, nivel: "ok" },
  "checkpoint-feito": { titulo: "Checkpoint atingido", prioridade: 2, nivel: "ok" },
  prototipo: { titulo: "Protótipo anexado", prioridade: 1, nivel: "" },
};
// `item` é o da fila do servidor; `onde` é "blueprint › ação › plano" já montado pela página. Devolve null para o que não avisa.
function notificacaoDe(item, agora, onde = "") {
  const regra = NOTIF_REGRAS[item?.tipo];
  const quando = Date.parse(item?.em);
  if (!regra || !(agora - quando < NOTIF_IDADE_MAX)) return null;
  const fala = String(item.tipo === "proposta" ? item.proposta?.mensagem || "Revise a proposta em ✦ Planejar e aplique o que quiser." : item.texto ?? "");
  const mensagem = (item.plano && item.tipo !== "proposta" ? `${item.plano}\n\n` : "") + fala;
  return {
    chave: `${item.id}|${item.em}`,
    tipo: item.tipo,
    titulo: regra.titulo,
    mensagem: mensagem.length > NOTIF_MSG_MAX ? mensagem.slice(0, NOTIF_MSG_MAX - 1) + "…" : mensagem,
    onde,
    prioridade: regra.prioridade,
    nivel: regra.nivel,
    blueprintId: item.blueprintId || "",
    acaoId: item.acaoId || "",
    planoId: item.planoId || "",
    abre: item.tipo === "proposta" ? "plano" : "blueprint", // a proposta abre a tela Planejar direto nela; o resto leva ao blueprint
  };
}
// Põe `n` no fim da fila e respeita o limite. Devolve quantos avisos ficaram de fora (para a contagem "+N").
function notificacaoEmpilha(fila, n) {
  fila.push(n);
  let perdidas = 0;
  while (fila.length > NOTIF_FILA_MAX) {
    const menor = Math.min(...fila.map((x) => x.prioridade));
    fila.splice(fila.findIndex((x) => x.prioridade === menor), 1);
    perdidas++;
  }
  return perdidas;
}
