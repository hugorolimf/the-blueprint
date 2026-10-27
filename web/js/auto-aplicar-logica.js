// ---- aplicar sozinho, parte PURA (sem DOM; testada em auto-aplicar-logica.test.mjs) ----
// O usuário já pediu ou aprovou na conversa com o agente: com o interruptor ligado, a proposta ou o registro que o agente envia entra no blueprint sem outro clique.
// A regra de segurança não muda: quem grava é a PÁGINA (nada do agente grava por fora), a autorização é do usuário (o interruptor, por blueprint, revogável) e
// tudo que entra sozinho avisa e pode ser desfeito. Remover e mover ação entre blueprints nunca entram sozinhos: esperam o clique, como sempre.
// Usa, de operacoes.js (carregado antes): opsEntre, opsIndice, opsAchar, opsChave, opsIgual, opsClone e aplicarOp.
const AUTO_JANELA_MS = 60000; // quanto tempo o Desfazer fica à mão
const AUTO_RASTRO_MAX = 30; // quantos itens aplicados sozinhos a página lembra (localStorage)

const autoChaveBlueprint = (ws, bp) => `blueprint.autoAplicar.${ws}:${bp}`;
const autoChaveItem = (i) => [i.workspaceId, i.blueprintId, i.tipo, i.numero, i.criadaEm].join("|"); // o número volta a 1 quando o app reinicia: a criadaEm desempata

// { pode, ligado }: no app local o padrão é LIGADO; no modo hospedado (equipe) é DESLIGADO, só o dono pode ligar, e quem lê ou convidado nunca aplica sozinho.
// `guardado` é o que está no localStorage: "1", "0" ou null (nunca escolheu).
function autoEstado({ hospedado = false, papel = "", soLeitura = false, guardado = null } = {}) {
  const pode = !soLeitura && (!hospedado || papel === "dono");
  const ligado = pode && (guardado === "1" || (guardado == null && !hospedado));
  return { pode, ligado };
}

// O que fazer com um item que chegou da fila do servidor (`item` é o dele): { aplica } ou { aplica: false, motivo } para a nota de "precisa da sua confirmação".
function autoDecide(item, { ligado, existe }) {
  if (!ligado) return { aplica: false, motivo: "desligado" };
  if (!item?.blueprintId || !existe) return { aplica: false, motivo: "sem-blueprint" }; // proposta de blueprint NOVO espera o clique
  if (item.tipo === "registro") return (item.registro?.acoes || []).length ? { aplica: true } : { aplica: false, motivo: "vazio" };
  if (item.tipo !== "proposta") return { aplica: false, motivo: "tipo" };
  if ((item.proposta?.remover || []).length) return { aplica: false, motivo: "remover" };
  if ((item.proposta?.mover || []).length) return { aplica: false, motivo: "mover" };
  return { aplica: true };
}

// a proposta ABERTA de um chat (c.remover, c.mover) precisa do clique mesmo com o interruptor ligado?
const autoPrecisaConfirmar = (c) => !!(c && (c.remover?.length || c.mover?.length));

// ---- Desfazer: as operações que a aplicação fez (opsEntre antes/depois do blueprint) são invertidas contra o estado ATUAL ----
// Só desfaz o que ainda está como a aplicação deixou: o que alguém mexeu depois (outra janela, o próprio agente) fica e é contado como conflito.
const autoEnvolve = (bp) => ({ blueprints: [opsClone(bp)] }); // opsEntre trabalha numa raiz com a lista `blueprints`
const autoOpsDe = (antesEnvolvido, depoisBlueprint) => opsEntre(antesEnvolvido, autoEnvolve(depoisBlueprint));
// volta o blueprint `p` (no lugar, para as referências da tela continuarem valendo) ao que o retrato `antesEnvolvido` guardou: só para quando a aplicação falha no meio
function autoRestaura(p, antesEnvolvido) {
  const b = antesEnvolvido.blueprints[0];
  for (const k of Object.keys(p)) if (!(k in b)) delete p[k];
  Object.assign(p, opsClone(b));
}

// `ops` e `antes` (a raiz envolvida do blueprint ANTES) vêm de quando se aplicou; `raiz` é o que está na tela (D). Devolve { feitas, conflitos }.
function autoDesfazOps(ops, antes, raiz) {
  const B = opsIndice(antes);
  let feitas = 0, conflitos = 0;
  for (const op of [...ops].reverse()) {
    const atual = opsAchar(raiz, op.tipo, op.id), b = B.get(opsChave(op.tipo, op.id));
    if (op.op === "criar") {
      if (!atual) continue; // já foi embora
      if (!opsIgual(atual.no, { ...opsClone(op.valor), id: op.id })) { conflitos++; continue; } // mexeram no que a aplicação criou: fica
      feitas += aplicarOp(raiz, { op: "apagar", tipo: op.tipo, id: op.id }) ? 1 : 0;
    } else if (op.op === "alterar") {
      if (!atual) { conflitos++; continue; }
      const volta = {};
      for (const [k, v] of Object.entries(op.campos || {})) {
        if (opsIgual(atual.no[k], v === null ? undefined : v)) volta[k] = op.antes?.[k] ?? null; else conflitos++; // o valor já não é o da aplicação
      }
      if (Object.keys(volta).length) feitas += aplicarOp(raiz, { ...op, campos: volta }) ? 1 : 0;
    } else if (op.op === "mover") {
      if (!atual || !b || atual.paiNo?.id !== op.pai) { conflitos++; continue; }
      const antigo = b.paiNo[b.lista].indexOf(b.no);
      feitas += aplicarOp(raiz, { op: "mover", tipo: op.tipo, id: op.id, pai: b.paiNo.id, posicao: antigo }) ? 1 : 0;
    } else if (op.op === "apagar") {
      if (atual || !b) continue;
      feitas += aplicarOp(raiz, { op: "criar", tipo: op.tipo, id: op.id, pai: b.paiNo.id, posicao: b.paiNo[b.lista].indexOf(b.no), valor: opsClone(b.no) }) ? 1 : 0;
    }
  }
  return { feitas, conflitos };
}

// o Desfazer ainda está à mão? (dentro da janela, não desfeito, e a aplicação mudou alguma coisa)
const autoPodeDesfazer = (e, agora) => !!e && !e.desfeito && !!e.ops?.length && agora - e.em < AUTO_JANELA_MS;

// o rastro: a lista inteira vive na memória (com as operações do Desfazer); entre recarregamentos só o que serve para listar e para não aplicar duas vezes
const autoRastroAdiciona = (lista, e) => [...lista.filter((x) => x.chave !== e.chave), e].slice(-AUTO_RASTRO_MAX);
const autoRastroGuardavel = (lista) => lista.map(({ chave, ws, blueprintId, em, texto, tipo }) => ({ chave, ws, blueprintId, em, texto, tipo }));
const autoRastroTem = (lista, chave) => lista.some((x) => x.chave === chave);
