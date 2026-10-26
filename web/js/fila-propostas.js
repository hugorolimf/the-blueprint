// ---- fila de propostas do agente: cada blueprint (e o blueprint novo) pode ter várias propostas numeradas esperando, e o usuário revisa uma de cada vez ----
// A proposta ABERTA fica nos campos de sempre do chat (c.proposta, c.remover, c.mover, c.dadosBlueprint, c.pasta, c.mensagem) e traz c.numero, c.criadaEm
// e c.em (do servidor); as outras ficam em c.fila, com os mesmos campos. A proposta só sai da fila do servidor quando o usuário a aplica ou descarta
// (ack pelo número e pela criadaEm); o servidor guarda a fila só em memória, então ela some quando o app reinicia.
const FILA_CAMPOS = ["proposta", "remover", "mover", "projetosNovos", "decisoesNovas", "dadosBlueprint", "pasta", "mensagem"];
const FILA_IDENT = ["numero", "criadaEm", "em"];

const filaTemAberta = (c) => !!(c.proposta || c.dadosBlueprint || c.remover?.length || c.mover?.length || c.decisoesNovas?.length);
const filaOrdena = (lista) => lista.sort((a, b) => (a.numero ?? 0) - (b.numero ?? 0));
// a proposta aberta como uma entrada de c.fila (null se não há)
function filaAbertaComoEntrada(c) {
  return filaTemAberta(c) ? Object.fromEntries([...FILA_IDENT, ...FILA_CAMPOS].map((k) => [k, c[k] ?? null])) : null;
}
function filaCarrega(c, e) {
  for (const k of [...FILA_IDENT, ...FILA_CAMPOS]) c[k] = e ? e[k] ?? null : null;
  if (!e) c.mensagem = "";
}
// todas as propostas do chat, abertas ou não, da menor para a maior
function filaTodas(c) {
  const aberta = filaAbertaComoEntrada(c);
  return filaOrdena([...(aberta ? [aberta] : []), ...(c.fila || [])]);
}
// troca a proposta aberta pela de número `numero` (a aberta vai para c.fila, com o que o usuário já editou nela)
function filaAbre(c, numero) {
  const e = (c.fila || []).find((x) => x.numero === numero);
  if (!e || numero === c.numero) return false;
  const atual = filaAbertaComoEntrada(c);
  c.fila = filaOrdena([...c.fila.filter((x) => x !== e), ...(atual ? [atual] : [])]);
  filaCarrega(c, e);
  return true;
}
// o alvo da fila no servidor (workspace e blueprint; "" é o blueprint novo) a partir do contexto do chat
function filaAlvo(c) {
  const [workspaceId, ...resto] = String(c.contexto || "").split(":"), blueprintId = resto.join(":");
  return { workspaceId, blueprintId: blueprintId === NOVO ? "" : blueprintId };
}
// a aberta saiu (aplicada ou descartada): o servidor a tira da fila dele e abre a menor das que esperam
function filaConclui(c) {
  if (c.numero != null && c.criadaEm) {
    fetch("/api/mcp/fila/ack", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ ids: [], propostas: [{ ...filaAlvo(c), numero: c.numero, criadaEm: c.criadaEm }] }) }).catch(() => {});
  }
  c.fila = filaOrdena(c.fila || []);
  filaCarrega(c, c.fila.shift() || null);
  c.emblema = null;
}
// tira todas as propostas do chat (o chat do blueprint novo segue para o blueprint criado sem levá-las) e devolve a lista
function filaDesmonta(c) {
  const todas = filaTodas(c);
  c.fila = [];
  filaCarrega(c, null);
  return todas;
}
function filaMonta(c, entradas) {
  const [primeira, ...resto] = filaOrdena([...entradas]);
  c.fila = resto;
  filaCarrega(c, primeira || null);
}
// a proposta que o servidor mandou já está aqui? (mesmo número e mesma criadaEm: a mesma proposta, mesmo ajustada)
const filaTem = (c, i) => filaTodas(c).find((x) => x.numero === i.numero && x.criadaEm === i.criadaEm);
// já recebida e sem ajuste desde então (a página recarregou e o servidor mandou de novo a que ainda não foi aplicada)
function filaJaVista(i) {
  if (i.numero == null || (i.blueprintId && !D.blueprints.some((x) => x.id === i.blueprintId))) return false;
  return filaTem(conv(i.blueprintId || NOVO), i)?.em === i.em;
}
// o item `i` do servidor (com `numero` e `criadaEm`) é uma proposta nova (entra no fim) ou o ajuste/a troca de uma que já está aqui (toma o lugar dela)
function filaRecebe(c, i, dados) {
  const novo = { numero: i.numero ?? null, criadaEm: i.criadaEm ?? null, em: i.em, ...dados };
  if (i.numero == null) return filaCarrega(c, novo); // servidor sem fila numerada: a nova toma o lugar da aberta, como antes
  c.fila ||= [];
  const antiga = filaTem(c, i);
  if (antiga && antiga.numero === c.numero) return filaCarrega(c, novo); // ajuste da que está aberta
  if (antiga) { c.fila = filaOrdena(c.fila.map((x) => (x.numero === antiga.numero ? novo : x))); return; }
  // o app reiniciou e voltou a numerar do 1: a velha, que o servidor já não conhece, ganha outro número para não se confundir com a nova
  const velha = filaTodas(c).find((x) => x.numero === i.numero);
  if (velha) {
    const livre = Math.max(...filaTodas(c).map((x) => x.numero ?? 0), i.numero) + 1;
    if (velha.numero === c.numero) c.numero = livre; else c.fila.find((x) => x.numero === velha.numero).numero = livre;
  }
  if (!filaTemAberta(c)) filaCarrega(c, novo); else c.fila = filaOrdena([...c.fila, novo]);
}

// a faixa de abas: só aparece com duas ou mais propostas
function htmlFila(c) {
  const todas = filaTodas(c);
  if (todas.length < 2) return "";
  const botao = (e) => {
    const n = (e.proposta || []).length, resumo = [n && trn(n, ["{n} ação", "{n} ações"]), (e.remover || []).length && tr("apagar"), (e.mover || []).length && tr("mover")].filter(Boolean).join(" · ");
    return `<button class="btn mini" type="button" data-fila="${e.numero}" aria-pressed="${e.numero === c.numero}" title="${esc(e.mensagem || "")}">Nº ${e.numero ?? "?"}${resumo ? ` · ${esc(resumo)}` : ""}</button>`;
  };
  return `<div class="cl-prop-fila" role="group" aria-label="${tr("Fila de propostas")}">${todas.map(botao).join("")}<span class="dica">${tr("{n} na fila: aplique ou descarte uma de cada vez.", { n: todas.length })}</span></div>`;
}
$("cl-prop").addEventListener("click", (e) => {
  const b = e.target.closest("[data-fila]"), p = blueprintEd();
  if (!b || !p) return;
  if (filaAbre(conv(p.id), +b.dataset.fila)) { salvar(); montaProposta(); }
});
