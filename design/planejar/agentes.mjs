// Marcar a caixinha de um plano chama um agente, e o próprio agente marca o plano como pronto ao concluir. Esta regra decide quem pode rodar
// agora, sem dois agentes pisarem um no outro:
// - revisão: plano alterado não começa antes de você revisar a mudança;
// - dependência: o plano espera os planos de `depende_de` ficarem prontos; se um deles travou, fica parado e avisa;
// - área: dois planos da mesma `area` (ou de uma dentro da outra) nunca rodam juntos; o segundo espera na fila;
// - limite: no máximo `limite` agentes ao mesmo tempo. Cada agente trabalha na própria cópia (ramo agente/<plano>).
// Genérico: `area` é um texto livre (pasta, módulo, cômodo da obra); plano sem área não bloqueia ninguém.

export const SITUACOES = {
  nao: ["Não iniciado", "#9cc4ee", "parado"], revisao: ["Revise a mudança", "#ffcf6b", "esperando"], trabalhando: ["Trabalhando", "#bfe0ff", "trabalhando"], fila: ["Na fila", "#9cc4ee", "esperando"],
  dependencia: ["Esperando plano", "#ffcf6b", "esperando"], area: ["Esperando área", "#ffcf6b", "esperando"], parado: ["Parado", "#ff8f80", "bloqueado"],
  bloqueado: ["Pediu ajuda", "#ff8f80", "bloqueado"], pronto: ["Pronto", "#8be3b5", "concluido"],
};

export const planosDe = (p) => p.projetos.flatMap((pr) => pr.acoes).flatMap((a) => a.planos ?? []);

/** Duas áreas conflitam se são iguais ou se uma está dentro da outra (`app` e `app/barra`). */
export function areasConflitam(a, b) {
  if (!a || !b) return false;
  const [x, y] = [a, b].map((s) => s.replace(/\/+$/, ""));
  return x === y || x.startsWith(`${y}/`) || y.startsWith(`${x}/`);
}

/** O ciclo de dependências, se houver: a lista de ids, ou null. */
export function acharCiclo(planos) {
  const por = new Map(planos.map((q) => [q.id, q])), feito = new Set(), caminho = [];
  const visita = (id) => {
    if (caminho.includes(id)) return [...caminho.slice(caminho.indexOf(id)), id];
    if (feito.has(id) || !por.has(id)) return null;
    caminho.push(id);
    for (const d of por.get(id).depende_de ?? []) { const c = visita(d); if (c) return c; }
    caminho.pop();
    feito.add(id);
    return null;
  };
  for (const q of planos) { const c = visita(q.id); if (c) return c; }
  return null;
}

/** A situação de cada plano. `iniciados`: caixinha marcada. `prontos`: o agente concluiu. `bloqueados`: o agente pediu ajuda. `travados`: mudança sem revisão. */
export function escalonar(planos, { iniciados = [], prontos = [], bloqueados = [], travados = [], rodando = [], limite = 4 }) {
  const [ap, ok, no, jaRoda, trava] = [iniciados, prontos, bloqueados, rodando, travados].map((l) => new Set(l));
  const saida = {}, ativos = planos.filter((q) => jaRoda.has(q.id));
  let agentes = ativos.length;
  for (const q of planos) saida[q.id] = { estado: ok.has(q.id) ? "pronto" : no.has(q.id) ? "bloqueado" : jaRoda.has(q.id) ? "trabalhando" : trava.has(q.id) ? "revisao" : ap.has(q.id) ? null : "nao" };
  for (const q of planos) {
    if (saida[q.id].estado) continue;
    const faltam = (q.depende_de ?? []).filter((d) => !ok.has(d));
    const preso = faltam.find((d) => no.has(d));
    if (preso) { saida[q.id] = { estado: "parado", por: preso }; continue; }
    if (faltam.length) { saida[q.id] = { estado: "dependencia", por: faltam[0], semIniciar: faltam.filter((d) => !ap.has(d)) }; continue; }
    const ocupa = ativos.find((o) => areasConflitam(q.area, o.area));
    if (ocupa) { saida[q.id] = { estado: "area", por: ocupa.id, area: q.area }; continue; }
    if (agentes >= limite) { saida[q.id] = { estado: "fila" }; continue; }
    ativos.push(q);
    agentes++;
    saida[q.id] = { estado: "trabalhando" };
  }
  return saida;
}

/** O que o orquestrador está fazendo agora, escrito: primeiro o que pede atenção, depois quem ele chamou, segurou e fechou. */
export function diario(planos, exec) {
  const de = (...e) => planos.filter((q) => e.includes(exec[q.id]?.estado));
  return [
    ...de("bloqueado").map((q) => `${q.id} pediu ajuda: vendo com o agente`),
    ...de("trabalhando").map((q) => `Chamou um agente para ${q.id}`),
    ...de("area").map((q) => `Segurou ${q.id}: ${exec[q.id].por} já mexe em ${q.area}`),
    ...de("pronto").map((q) => `Conferiu ${q.id}: concluído, com evidência`),
  ];
}

/** Os avisos que a pessoa precisa ver, escritos. */
export function avisos(planos, exec) {
  const out = [];
  for (const q of planos) {
    const e = exec[q.id];
    if (e?.estado === "parado") out.push(`${q.id} está parado: ${e.por} pediu ajuda.`);
    if (e?.estado === "dependencia" && e.semIniciar?.length) out.push(`${q.id} depende de ${e.semIniciar.join(", ")}, que ainda não foi iniciado.`);
    else if (e?.estado === "dependencia") out.push(`${q.id} espera ${e.por} ficar pronto.`);
    if (e?.estado === "revisao") out.push(`${q.id} mudou: revise antes de iniciar.`);
    if (e?.estado === "area") out.push(`${q.id} espera ${e.por}: os dois mexem em ${e.area}.`);
  }
  return out;
}
