// ---- sincronia: juntar a edição desta janela com a que outra janela (ou um script) gravou antes ----
// Desde a gravação por operações (operacoes.js, ao-vivo.js) isto só entra quando a página perde o fio: `recarregar` ou `substituido` no
// /api/eventos (buraco na retomada, app reiniciado, PUT de script). A página então busca os dados atuais e junta três versões: a BASE (o que ela
// leu da última vez), a MINHA (o que está na tela) e a DELES (o que o servidor tem agora). Regra: quem mexeu sozinho em algo, ganha; as listas de
// objetos com `id` (blueprints, ações, planos, checkpoints, chats…) são juntadas por id, então duas pessoas adicionando coisas diferentes ficam
// com as duas; no mesmo campo mexido dos dois lados, vale a minha (a edição mais recente de quem está olhando a tela).
// Limite conhecido: mudar a ORDEM de uma lista não é uma edição que sobreviva ao encontro com a de outra janela (vale a ordem do servidor).
const sincEhObjeto = (x) => x !== null && typeof x === "object" && !Array.isArray(x);
const sincIgual = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const sincListaDeIds = (l) => Array.isArray(l) && l.every((x) => sincEhObjeto(x) && typeof x.id === "string");

function sincMesclarLista(base, meu, deles) {
  const mapa = (l) => new Map((l || []).map((x) => [x.id, x]));
  const B = mapa(base), M = mapa(meu), T = mapa(deles);
  const escolher = (id) => {
    const b = B.get(id), m = M.get(id), t = T.get(id);
    if (m && t) return sincMesclar(b, m, t);
    const sobrou = m || t; // só um lado ainda tem o item
    if (!b) return sobrou; // foi adicionado depois da base
    return sincIgual(sobrou, b) ? null : sobrou; // o outro lado apagou: apaga, a menos que este lado tenha mexido nele
  };
  const visto = new Set(), saida = [];
  for (const x of [...(deles || []), ...(meu || [])]) { // a ordem do servidor primeiro; depois o que só esta janela tem
    if (visto.has(x.id)) continue;
    visto.add(x.id);
    const r = escolher(x.id);
    if (r) saida.push(r);
  }
  return saida;
}

function sincMesclar(base, meu, deles) {
  if (sincIgual(meu, base)) return deles; // esta janela não mexeu: vale o servidor
  if (sincIgual(deles, base) || sincIgual(meu, deles)) return meu; // o servidor não mexeu (ou chegaram ao mesmo): vale o meu
  if (sincEhObjeto(meu) && sincEhObjeto(deles)) {
    const b = sincEhObjeto(base) ? base : {}, saida = {};
    for (const k of new Set([...Object.keys(deles), ...Object.keys(meu)])) {
      const r = sincMesclar(b[k], meu[k], deles[k]);
      if (r !== undefined) saida[k] = r;
    }
    return saida;
  }
  if (sincListaDeIds(meu) && sincListaDeIds(deles) && (base === undefined || sincListaDeIds(base))) return sincMesclarLista(base, meu, deles);
  return meu; // conflito de verdade (mesmo valor mexido dos dois lados): vale a edição de quem está olhando a tela
}
