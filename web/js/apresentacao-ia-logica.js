// ---- apresentação com IA: níveis, públicos, o corpo do pedido e as frases do resultado; só lógica, sem DOM (apresentacao-ia.js desenha) ----
// Contrato em docs/interno/apresentacao-ia.md. O nível "epico" não existe aqui de propósito: só vale quando o usuário o pede pelo nome na conversa com o agente.
const AP_IA_NIVEIS = [{ id: "rapido", horas: [0, 1] }, { id: "padrao", horas: [1, 3] }, { id: "completo", horas: [2, 4] }];
const AP_IA_PUBLICOS = ["cliente", "interno"];
const AP_IA_ID = /^[A-Za-z0-9_-]{1,64}$/; // o id vira parte de uma URL e de um atributo: só o que o servidor gera

const apIaRotuloPublico = () => ({ cliente: tr("Cliente"), interno: tr("Interno") });
const apIaRotuloNivel = () => ({ rapido: tr("Rápido"), padrao: tr("Padrão"), completo: tr("Completo") });
const apIaDescPublico = () => ({
  cliente: tr("Plano de projeto: o que o cliente recebe, etapas, datas e o que esperamos dele. Sem termos técnicos."),
  interno: tr("Visão técnica completa, para quem constrói."),
});
const apIaDescNivel = () => ({
  rapido: tr("Uma figura viva na capa e as páginas do plano."),
  padrao: tr("Uma figura por etapa, cada uma um objeto diferente."),
  completo: tr("O Padrão e mais uma máquina densa do sistema inteiro."),
});

// a estimativa que o contrato anuncia antes de começar ("até 1 h", "1 a 3 h")
function apIaEstimativa(nivel) {
  const n = AP_IA_NIVEIS.find((x) => x.id === nivel);
  if (!n) return "";
  return n.horas[0] ? tr("{de} a {ate} h", { de: n.horas[0], ate: n.horas[1] }) : tr("até {ate} h", { ate: n.horas[1] });
}

// o corpo de POST /api/mcp/apresentar; null quando algo não vale (nível fora da lista, como o Épico, ou ids vazios). `sessao` é a escolhida no diálogo (ou nada: terminal novo)
function apIaCorpoPedido({ workspaceId, blueprintId, publico, nivel, sessao } = {}) {
  if (!workspaceId || !blueprintId || !AP_IA_PUBLICOS.includes(publico) || !AP_IA_NIVEIS.some((n) => n.id === nivel)) return null;
  const corpo = { workspaceId, blueprintId, publico, nivel };
  if (sessao) Object.assign(corpo, { sessaoId: sessao.id, ...(sessao.canal ? {} : { semCanal: true }) }); // as mesmas chaves de POST /api/mcp/iniciar
  return corpo;
}

// a resposta do servidor vira uma frase para o usuário (sem inventar: o que o servidor não disse não aparece)
function apIaMensagem(status, j, emSessao) {
  if (status === 200 || j?.ok) return (emSessao ? tr("Pedido enviado à sessão: o Claude vai montar a apresentação.") : tr("Terminal aberto: o Claude vai montar a apresentação.")) + " " + tr("Ela aparece na lista quando ele terminar.");
  if (j?.codigo === "sessao-caiu") return tr("Essa sessão não está mais conectada ao app. Escolha outra ou abra uma nova.");
  if (j?.codigo === "sessao-de-outra-pasta") return tr("Essa sessão não é de uma pasta deste blueprint. Escolha outra ou abra uma nova.");
  if (j?.codigo === "sem-pasta" || status === 409) return tr("Esta pasta não está ligada a um blueprint. Em Editar, preencha “Pasta do projeto” neste blueprint e tente de novo.");
  if (j?.codigo === "sem-terminal" || status === 501) return tr("Não encontrei um terminal para abrir (Windows Terminal ou o prompt do Windows).") + (j?.erro ? " " + tr("Motivo: {erro}", { erro: j.erro }) : "");
  if (status === 404) return tr("O app não achou este blueprint. Recarregue a página e tente de novo.");
  return j?.erro || tr("Não foi possível pedir a apresentação (erro {status}).", { status });
}

// os metadados que o app guarda no blueprint ({id, titulo, publico, nivel, em}), do mais novo ao mais antigo; o que vier torto (id fora do padrão) não entra
function apIaLista(apresentacoes, ocultas = new Set()) {
  return (Array.isArray(apresentacoes) ? apresentacoes : [])
    .filter((a) => a && typeof a.id === "string" && AP_IA_ID.test(a.id) && !ocultas.has(a.id))
    .map((a) => ({
      id: a.id,
      titulo: String(a.titulo || "").slice(0, 120),
      publico: AP_IA_PUBLICOS.includes(a.publico) ? a.publico : "",
      nivel: AP_IA_NIVEIS.some((n) => n.id === a.nivel) ? a.nivel : "", // um "epico" gravado pelo agente aparece sem rótulo de nível, não some
      em: typeof a.em === "string" ? a.em.slice(0, 10) : "",
    }))
    .sort((x, y) => (y.em > x.em ? 1 : y.em < x.em ? -1 : 0));
}
const apIaUrl = (id, baixar = false) => (typeof id === "string" && AP_IA_ID.test(id) ? `/apresentacao/${id}${baixar ? "?baixar=1" : ""}` : "");
