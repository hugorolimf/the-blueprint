// ---- dados normalizados -----------------------------------------------------------------
// A situação da ação vem dos planos e dos checkpoints marcados: só está concluída quando TODOS estão feitos.
// O status guardado só vale para ações sem nenhum item (aí o progresso é o número digitado).
function efetiva(f) {
  const t = f.planos || [], m = f.checkpoints || [];
  const ft = t.filter((x) => x.feito).length, fm = m.filter((x) => x.feito).length;
  const total = t.length + m.length, feitos = ft + fm;
  let status = f.status, pct = status === "concluido" ? 1 : status === "andamento" ? Math.min(1, Math.max(0, Number(f.progresso) || 0)) : 0;
  if (total) {
    pct = feitos / total;
    status = pct >= 1 ? "concluido" : pct > 0 ? "andamento" : f.status === "andamento" ? "andamento" : "planejado";
  } else if (status === "andamento" && pct >= 1) status = "concluido";
  if (status === "concluido") pct = 1;
  return { status, pct, tf: [ft, t.length], mf: [fm, m.length], it: [feitos, total] };
}
const acoesDe = (p) =>
  p.acoes.map((f, idx) => ({ ...f, ...efetiva(f), idx }));
const soPendentes = {}; // por blueprint: filtro "Pendentes" ligado
const soAndamento = {}; // por blueprint: filtro "Em andamento" ligado: só as ações em que há robô trabalhando
// (soMeus, "Meus planos": responsavel-logica.js)
// o que a folha desenha: com o filtro ligado, só as ações não concluídas e, nelas, só planos e checkpoints pendentes (o progresso continua calculado sobre tudo)
const acoesVisiveis = (p) => {
  const comRobo = soAndamento[p.id] ? new Set(agentesDoBlueprint(p.id).map((a) => a.acaoId)) : null, doRobo = comRobo ? acoesDe(p).filter((f) => comRobo.has(f.id)) : acoesDe(p);
  const base = soMeus[p.id] ? doRobo.filter((f) => planosFiltrados(f).length).map((f) => ({ ...f, planos: planosFiltrados(f) })) : doRobo; // "Meus planos": só as ações em que há plano meu, e nelas só os meus
  return soPendentes[p.id] ? base.filter((f) => f.status !== "concluido").map((f) => ({ ...f, planos: f.planos.filter((t) => !t.feito), checkpoints: f.checkpoints.filter((m) => !m.feito) })) : base;
};
const projConcluido = (p) => p.acoes.length > 0 && p.acoes.every((f) => efetiva(f).status === "concluido");
const resumo = (fs) => {
  const peso = (f) => Math.max(1, f.it[1]), itens = fs.reduce((t, f) => t + peso(f), 0); // cada ação pesa os seus itens
  const todas = fs.length > 0 && fs.every((f) => f.status === "concluido");
  const media = itens ? fs.reduce((t, f) => t + peso(f) * f.pct, 0) / itens : 0;
  return {
    geral: todas ? 1 : Math.min(media, 0.99), // só chega a 100% quando toda etapa está concluída
    concl: fs.filter((f) => f.status === "concluido").length,
    futuro: fs.length > 0 && fs.every((f) => f.status === "planejado"),
  };
};
// trocar o filtro: a folha se recolhe (inverso da abertura) e, só depois, o novo filtro é aplicado; a folha se escreve de novo e o que ficou desliza para o novo lugar
let trocandoFiltro = false;
async function trocarFiltro(muda) {
  if (trocandoFiltro || !tela) return;
  trocandoFiltro = true;
  const antes = posicoesDaFolha();
  try { await recolherFolha(); } finally { muda(); render(); trocandoFiltro = false; desenrolar(); organizarFolha(antes); }
}
$("periodo-pendentes").onclick = () => trocarFiltro(() => { soPendentes[tela] = !soPendentes[tela]; }); // liga/desliga: mostra só o que falta fazer
$("periodo-andamento").onclick = () => trocarFiltro(() => { soAndamento[tela] = !soAndamento[tela]; }); // liga/desliga: só o que está em andamento, com os robôs
$("periodo-meus").onclick = () => trocarFiltro(() => { soMeus[tela] = !soMeus[tela]; }); // responsavel-logica.js
$("periodo-auto").onclick = () => trocarFiltro(() => { delete soPendentes[tela]; delete soAndamento[tela]; delete soMeus[tela]; }); // "Todo o blueprint" também tira os filtros

// ícone de protótipo (tela com apoio): abre o protótipo que o agente anexou à ação
const protoSvg = (x, y, info, alvo) =>
  `<g class="nota" tabindex="0" role="button" data-proto="${esc(alvo)}" data-info="${esc(info)}" aria-label="${esc(info + ". " + tr("Ver protótipo"))}"><rect fill="transparent" x="${x - 4}" y="${y - 4}" width="24" height="26"/><path class="ic" d="M${x - 1} ${y + 1}h16v10h-16z"/><path class="ic2" d="M${x + 3} ${y + 15}h8M${x + 7} ${y + 11}v4M${x + 6} ${y + 4}l4 3l-4 3z"/></g>`;
// ícone de nota (folha com a ponta dobrada) para as observações
const notaSvg = (x, y, info, alvo) =>
  `<g class="nota" tabindex="0" role="button" data-obs="${esc(alvo)}" data-info="${esc(info)}" aria-label="${esc(info + ". " + (alvo.split("|")[1] ? tr("Abrir instruções") : tr("Abrir observações")))}"><rect fill="transparent" x="${x - 4}" y="${y - 4}" width="24" height="26"/><path class="ic" d="M${x} ${y}h9l5 5v11h-14z"/><path class="ic2" d="M${x + 9} ${y}v5h5M${x + 3} ${y + 9}h8M${x + 3} ${y + 13}h6"/></g>`;

