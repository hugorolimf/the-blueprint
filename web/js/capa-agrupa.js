// ---- o cartão do blueprint na lista (a "capa", em render.js): o que não cabe é agrupado em vez de virar um borrão. PURO, só dados e texto de SVG;
// testado em capa-agrupa.test.mjs. Um blueprint com ~100 ações punha uma célula de 1 a 5 px por ação e um losango por checkpoint, uns sobre os outros.
const CAPA_CELULA = 9; // px por ação (contorno + vão): com menos que isso as células viram um borrão
const CAPA_LOSANGO = 8; // largura do losango de checkpoint: os que se tocariam viram um só

// cel: as ações já na ordem de situação ({status, pct, it:[feitos, total]}); largura: px da barra. Se cabem, volta igual. Se não, um bloco por situação
// (concluídas | em andamento | a fazer) com largura proporcional aos itens e o andamento médio, marcado com `grupo` (quantas ações juntou).
function capaCelulas(cel, largura) {
  if (cel.length * CAPA_CELULA <= largura) return cel;
  const blocos = [];
  for (const f of cel) {
    let b = blocos[blocos.length - 1];
    if (!b || b.status !== f.status) blocos.push((b = { status: f.status, grupo: 0, it: [0, 0], soma: 0 }));
    const peso = Math.max(1, f.it[1]); // o mesmo peso que a barra usa para a largura de uma ação
    b.grupo++; b.it[0] += f.it[0]; b.it[1] += peso; b.soma += f.pct * peso;
  }
  return blocos.map(({ soma, ...b }) => ({ ...b, pct: soma / b.it[1] }));
}

// o texto de dica de uma célula: a ação (com o número da folha, que `numero()` calcula só quando preciso) ou o grupo
function capaTitulo(f, numero) {
  if (f.grupo) {
    if (f.status === "concluido") return trn(f.grupo, ["{n} ação concluída", "{n} ações concluídas"]);
    if (f.status === "andamento") return trn(f.grupo, ["{n} ação em andamento ({pct}% feito)", "{n} ações em andamento ({pct}% feito)"], { pct: Math.round(f.pct * 100) });
    return trn(f.grupo, ["{n} ação a fazer", "{n} ações a fazer"]);
  }
  return `${numero()} ${f.nome || tr("SEM NOME")} · ${f.status === "concluido" ? tr("concluída") : f.status === "andamento" ? tr("em andamento, {pct}% feito", { pct: Math.round(f.pct * 100) }) : tr("a fazer")}`;
}

// marcos: [{x (px), feito}]. Os que se tocariam (menos de CAPA_LOSANGO de distância do primeiro do grupo) viram um losango só, no meio deles.
function capaMarcos(marcos) {
  const grupos = [];
  for (const m of [...marcos].sort((a, b) => a.x - b.x)) {
    const g = grupos[grupos.length - 1];
    if (g && m.x - g.ini < CAPA_LOSANGO) { g.n++; g.feitos += m.feito ? 1 : 0; g.soma += m.x; }
    else grupos.push({ ini: m.x, n: 1, feitos: m.feito ? 1 : 0, soma: m.x });
  }
  return grupos.map((g) => ({ x: g.soma / g.n, n: g.n, feitos: g.feitos }));
}

// o losango de um grupo: cheio só se todos estão feitos; com mais de um, a dica diz quantos
const capaMarcoSvg = (g, yc) => `<path class="dm${g.feitos === g.n ? " ok" : ""}" d="M${g.x} ${yc - 5}l4 5l-4 5l-4 -5z">${g.n > 1 ? `<title>${tr("{feitos} de {n} checkpoints feitos", { feitos: g.feitos, n: g.n })}</title>` : ""}</path>`;
