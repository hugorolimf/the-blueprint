// ---- o modo foco, parte PURA (sem DOM; testada em foco-logica.test.mjs): os cartões da visão de acompanhamento (quem aparece e em que ordem), o agente em foco, quando seguir e rolar, o que
// o Esc faz, desde quando o agente está no plano e quais eventos da linha do tempo são dele. O DOM fica em foco.js, foco-vista.js e foco-cartao.js.
// Entra: os agentes de agentesAtivos() ({ agente, papel?, estado, blueprintId, acaoId, planoId, em (ISO), mensagem?, onde?, percentual?, repouso? }) e os blueprints (D.blueprints).
// Usa claudePrincipal (claude-logica.js, carregado antes) para achar o agente principal.
const FOCO_ENTRE = [0.1, 0.9]; // faixa da janela (fração da altura) em que um alvo já está bem à vista: dentro dela nada rola
const FOCO_EVENTOS = 3; // eventos da linha do tempo no cartão da lista
const FOCO_HISTORICO = 12; // ... e no cartão grande (o detalhe do agente)
const FOCO_ORDEM = { bloqueado: 0, escrevendo: 1, testando: 1, lendo: 1, aguardando: 2, concluido: 3, ocioso: 4 }; // quem precisa de atenção ou trabalha vem antes de quem espera, concluiu ou repousa

// Os cartões: um por agente (trabalhando ou em repouso), com o lugar (blueprint, ação, plano) resolvido nos dados. O que sumiu dos dados vira lugar vazio (o cartão continua).
// Ordem: o principal primeiro e depois por urgência (FOCO_ORDEM), quem avisou há menos tempo antes e, por fim, o nome (a ordem não muda a cada aviso).
function focoCartoes(ags, blueprints) {
  const itens = (ags || []).map((a) => {
    const repouso = !!a.repouso || a.estado === "ocioso";
    const p = repouso ? null : (blueprints || []).find((x) => x.id === a.blueprintId), f = p?.acoes?.find((x) => x.id === a.acaoId) || null, t = f && a.planoId ? (f.planos || []).find((x) => x.id === a.planoId) || null : null;
    return { agente: a.agente || "", papel: a.papel || "", estado: repouso ? "ocioso" : a.estado, repouso, blueprintId: p ? p.id : a.blueprintId || "", acaoId: f ? f.id : "", planoId: t ? t.id : "", acaoNome: f?.nome || "", planoTexto: t?.texto || "", feito: !!t?.feito,
      em: Date.parse(a.em) || 0, mensagem: a.mensagem || "", onde: Array.isArray(a.onde) ? a.onde : [], percentual: Number.isFinite(a.percentual) ? a.percentual : null, inicioEm: Date.parse(t?.agente?.inicioEm) || Number(t?.agente?.inicioEm) || 0 };
  });
  const principal = claudePrincipal(itens), rank = (i) => FOCO_ORDEM[i.estado] ?? 2;
  itens.sort((a, b) => (b === principal) - (a === principal) || rank(a) - rank(b) || b.em - a.em || a.agente.localeCompare(b.agente));
  return { itens, principal: principal || null, vazio: !itens.length };
}

// O agente em foco: o que o usuário fixou (se ainda existe) ou, senão, o principal.
const focoEmFoco = (cartoes, fixo) => cartoes.itens.find((i) => fixo && i.agente === fixo) || cartoes.principal;

// a chave do lugar do agente; vazia sem plano nem ação
const focoLugar = (i) => (i && i.acaoId ? `${i.blueprintId}|${i.acaoId}|${i.planoId}` : "");

// Seguir: o cartão em foco só "se mexe" quando o LUGAR dele muda (outro plano ou outra ação), não a cada aviso.
function focoProximo(lugarAnterior, emFoco) {
  const chave = focoLugar(emFoco);
  return { chave, mudou: !!chave && chave !== lugarAnterior };
}

// Rolar só se o alvo saiu da faixa confortável da janela (a tela não sacode a cada troca). `caixa`: { top, bottom } em px da janela.
const focoDeveRolar = (caixa, alto) => !caixa || caixa.top < alto * FOCO_ENTRE[0] || caixa.bottom > alto * FOCO_ENTRE[1];

// O Esc, um degrau por vez: do detalhe de um agente volta à lista; da lista fecha a visão.
const focoEsc = (fixo) => (fixo ? "lista" : "fechar");
// O agente fixado sumiu (concluiu e o usuário fechou a mensagem, ou o servidor o esqueceu): volta à lista.
const focoFixoSumiu = (cartoes, fixo) => !!fixo && !cartoes.itens.some((i) => i.agente === fixo);

// Desde quando o agente trabalha neste plano: o início medido do plano (`inicioEm`) ou, sem ele, o evento mais antigo da sequência dele neste plano (os eventos vêm do mais novo ao mais antigo).
// Devolve ms, ou 0 se não dá para saber.
function focoDesde(inicioEm, eventos, agente, planoId) {
  if (inicioEm) return inicioEm;
  let desde = 0;
  for (const e of eventos || []) {
    if (e.alvo !== planoId || (e.agente && e.agente !== agente)) { if (desde) break; continue; }
    desde = Date.parse(e.em) || desde;
  }
  return desde;
}
// Quanto trabalhou: do começo no plano até o ÚLTIMO AVISO (não até agora, para o texto não envelhecer sem aviso novo: nada corre em repouso). Em minutos, arredondado; 0 se não dá.
const focoMinutos = (desde, ate) => (desde && ate > desde ? Math.max(1, Math.round((ate - desde) / 60000)) : 0);

// Os eventos da linha do tempo que são deste agente neste plano (o evento geral da ação, sem plano, também conta), do mais novo ao mais antigo.
const focoEventosDoAgente = (eventos, i, n = FOCO_EVENTOS) => (eventos || []).filter((e) => (!e.agente || e.agente === i.agente) && (!e.alvo || !i.planoId || e.alvo === i.planoId)).slice(0, n);
