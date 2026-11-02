// A diferença entre duas versões de um plano, como um git diff: por ação, por plano (pelos ids) e, dentro de um plano alterado, por palavra.
// É isso que decide o que aparece como novo, o que aparece como alterado e o que continua igual.

/** Diferença por palavra (maior subsequência comum). Devolve pedaços { t: "igual" | "menos" | "mais", s }. */
export function diffPalavras(antes, depois) {
  const a = antes.split(/(\s+)/), b = depois.split(/(\s+)/), m = a.length, n = b.length;
  const d = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = m - 1; i >= 0; i--) for (let j = n - 1; j >= 0; j--) d[i][j] = a[i] === b[j] ? d[i + 1][j + 1] + 1 : Math.max(d[i + 1][j], d[i][j + 1]);
  const out = [], poe = (t, s) => { if (out.at(-1)?.t === t) out.at(-1).s += s; else out.push({ t, s }); };
  let i = 0, j = 0;
  while (i < m || j < n) {
    if (i < m && j < n && a[i] === b[j]) { poe("igual", a[i]); i++; j++; }
    else if (j < n && (i === m || d[i][j + 1] >= d[i + 1][j])) { poe("mais", b[j]); j++; }
    else { poe("menos", a[i]); i++; }
  }
  return out;
}

/** As linhas do diff de uma ação, na ordem da versão nova, com o removido no lugar onde estava. */
export function compararAcao(antiga, nova) {
  const linhas = [], antes = new Map((antiga.planos ?? []).map((q) => [q.id, q])), ids = new Set((nova.planos ?? []).map((q) => q.id));
  const prazo = (a) => (a.de ? `${a.de} → ${a.ate}` : "sem data");
  if (prazo(antiga) !== prazo(nova)) linhas.push({ tipo: "prazo", antes: antiga, depois: nova });
  for (const q of nova.planos ?? []) {
    const v = antes.get(q.id);
    if (!v) linhas.push({ tipo: "novo", plano: q });
    else if (v.texto !== q.texto) linhas.push({ tipo: "alterado", plano: q, antes: v, palavras: diffPalavras(v.texto, q.texto) });
    else linhas.push({ tipo: "igual", plano: q });
  }
  for (const q of antiga.planos ?? []) if (!ids.has(q.id)) linhas.push({ tipo: "removido", plano: q });
  const velhos = new Set((antiga.detalhes ?? []).map((d) => d.id));
  const detalhes = (nova.detalhes ?? []).filter((d) => !velhos.has(d.id));
  const conta = (t) => linhas.filter((l) => l.tipo === t).length;
  const mudou = linhas.some((l) => l.tipo !== "igual") || detalhes.length > 0;
  return { linhas, detalhes, mudou, resumo: { mais: conta("novo"), menos: conta("removido"), alterados: conta("alterado") + conta("prazo"), detalhes: detalhes.length } };
}

/** Compara o plano inteiro: quais ações são novas, quais mudaram (com o diff) e quais continuam iguais. Sem versão anterior, tudo é novo. */
export function compararPlanos(anterior, atual) {
  const acoes = (p) => p?.projetos?.flatMap((pr) => pr.acoes) ?? [], antes = new Map(acoes(anterior).map((a) => [a.id, a]));
  const novas = [], alteradas = {}, iguais = [];
  for (const a of acoes(atual)) {
    const v = antes.get(a.id);
    if (!v) { novas.push(a.id); continue; }
    const c = compararAcao(v, a);
    if (c.mudou) alteradas[a.id] = c; else iguais.push(a.id);
  }
  return { novas, alteradas, iguais };
}

/** Os planos que não podem começar enquanto a mudança da ação não for revisada. */
export const planosTravados = (comp) => Object.values(comp?.alteradas ?? {}).flatMap((c) => c.linhas.filter((l) => ["novo", "alterado"].includes(l.tipo)).map((l) => l.plano.id));
