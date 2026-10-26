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
  p.acoes
    .map((f, idx) => ({ ...f, ...efetiva(f), idx, a: dia(f.inicio), b: dia(f.fim) }))
    .filter((f) => f.a === f.a && f.b === f.b)
    .map((f) => ({ ...f, b: Math.max(f.a, f.b) }));
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
  const dur = fs.reduce((t, f) => t + (f.b - f.a + 1), 0);
  const todas = fs.length > 0 && fs.every((f) => f.status === "concluido");
  const media = dur ? fs.reduce((t, f) => t + (f.b - f.a + 1) * f.pct, 0) / dur : 0;
  return {
    geral: todas ? 1 : Math.min(media, 0.99), // só chega a 100% quando toda etapa está concluída
    concl: fs.filter((f) => f.status === "concluido").length,
    horas: fs.reduce((t, f) => t + f.planos.reduce((s, x) => s + (Number(x.horas) || 0), 0), 0), // tempo de desenvolvimento do blueprint: soma das horas dos planos
    futuro: fs.length > 0 && fs.every((f) => f.status === "planejado"),
  };
};
const periodos = {}, janelasRenderizadas = {};
const indiceMes = (iso) => Number(iso.slice(0, 4)) * 12 + Number(iso.slice(5, 7)) - 1;
const mesIso = (n) => `${Math.floor(n / 12)}-${String(n % 12 + 1).padStart(2, "0")}`;
const mesValido = (s) => /^\d{4}-(0[1-9]|1[0-2])$/.test(s || "");
function janelaPeriodo(acoes, dataHoje, periodo) {
  const atual = dia(dataHoje), datas = acoes.flatMap((f) => [f.a, f.b]);
  const a = datas.length ? Math.min(...datas) - 7 : atual - 7;
  const b = datas.length ? Math.max(...datas) + 7 : atual + 7;
  let inicio = indiceMes(deDia(a)), fim = indiceMes(deDia(b));
  const escolhido = periodo?.aguardando ? periodo.anterior : periodo;
  if (mesValido(escolhido?.inicio) && mesValido(escolhido?.fim)) {
    inicio = Math.min(indiceMes(escolhido.inicio), indiceMes(escolhido.fim));
    fim = Math.max(indiceMes(escolhido.inicio), indiceMes(escolhido.fim));
  }
  return { inicio, fim, t0: dia(mesIso(inicio) + "-01"), t1: dia(mesIso(fim + 1) + "-01") };
}
function selecionarMes(mes) {
  if (!tela || !mesValido(mes)) return;
  const atual = periodos[tela];
  const muda = () => {
    periodos[tela] = atual?.aguardando
      ? { inicio: [atual.inicio, mes].sort()[0], fim: [atual.inicio, mes].sort()[1] }
      : { inicio: mes, fim: mes, aguardando: true, anterior: janelasRenderizadas[tela] };
  };
  if (atual?.aguardando) trocarFiltro(muda); // o período mudou de fato (2º clique): as barras mudam de posição e largura, então a folha se recolhe e se escreve de novo
  else { muda(); render(); } // 1º clique só marca o mês inicial: a janela não muda, sem animação
}
// trocar o filtro: a folha se recolhe (inverso da abertura) e, só depois, o novo filtro é aplicado; a folha se escreve de novo e o que ficou desliza para o novo lugar
let trocandoFiltro = false;
async function trocarFiltro(muda) {
  if (trocandoFiltro || !tela) return;
  trocandoFiltro = true;
  const antes = posicoesDaFolha();
  try { await recolherFolha(); } finally { muda(); render(); trocandoFiltro = false; desenrolar(); organizarFolha(antes); }
}
$("periodo-pendentes").onclick = () => trocarFiltro(() => { soPendentes[tela] = !soPendentes[tela]; delete periodos[tela]; }); // liga/desliga: mostra só o que falta fazer
$("periodo-andamento").onclick = () => trocarFiltro(() => { soAndamento[tela] = !soAndamento[tela]; delete periodos[tela]; }); // liga/desliga: só o que está em andamento, com os robôs
$("periodo-meus").onclick = () => trocarFiltro(() => { soMeus[tela] = !soMeus[tela]; delete periodos[tela]; }); // responsavel-logica.js
$("periodo-auto").onclick = () => trocarFiltro(() => { delete periodos[tela]; delete soPendentes[tela]; delete soAndamento[tela]; delete soMeus[tela]; }); // "Todo o blueprint" também tira os filtros
$("periodo-hoje").onclick = () => trocarFiltro(() => { const mes = local(new Date()).slice(0, 7); periodos[tela] = { inicio: mes, fim: mes, meses: [mes] }; });

// distribui itens em linhas sem sobreposição (largura de cada um dada por "ate")
function linhas(itens, de, ate) {
  const fim = [];
  [...itens].sort((x, y) => de(x) - de(y)).forEach((it) => {
    let r = fim.findIndex((e) => e < de(it));
    if (r < 0) r = fim.length;
    fim[r] = ate(it);
    it.linha = r;
  });
  return fim.length;
}
// datas sob a cota: em ação curta, início e fim cabem num rótulo só, sem se sobrepor
const datasCota = (f, cy) => (f.x1 - f.x0 < 104
  ? `<text class="mid" x="${(f.x0 + f.x1) / 2}" y="${cy + 18}" font-size="10" text-anchor="middle">${fmt(f.inicio)} → ${fmt(f.fim)}</text></g>`
  : `<text class="mid" x="${f.x0}" y="${cy + 18}" font-size="10">${fmt(f.inicio)}</text><text class="mid" x="${f.x1}" y="${cy + 18}" font-size="10" text-anchor="end">${fmt(f.fim)}</text></g>`);
// ícone de protótipo (tela com apoio): abre o protótipo que o agente anexou à ação
const protoSvg = (x, y, info, alvo) =>
  `<g class="nota" tabindex="0" role="button" data-proto="${esc(alvo)}" data-info="${esc(info)}" aria-label="${esc(info + ". " + tr("Ver protótipo"))}"><rect fill="transparent" x="${x - 4}" y="${y - 4}" width="24" height="26"/><path class="ic" d="M${x - 1} ${y + 1}h16v10h-16z"/><path class="ic2" d="M${x + 3} ${y + 15}h8M${x + 7} ${y + 11}v4M${x + 6} ${y + 4}l4 3l-4 3z"/></g>`;
// ícone de nota (folha com a ponta dobrada) para as observações
const notaSvg = (x, y, info, alvo) =>
  `<g class="nota" tabindex="0" role="button" data-obs="${esc(alvo)}" data-info="${esc(info)}" aria-label="${esc(info + ". " + (alvo.split("|")[1] ? tr("Abrir instruções") : tr("Abrir observações")))}"><rect fill="transparent" x="${x - 4}" y="${y - 4}" width="24" height="26"/><path class="ic" d="M${x} ${y}h9l5 5v11h-14z"/><path class="ic2" d="M${x + 9} ${y}v5h5M${x + 3} ${y + 9}h8M${x + 3} ${y + 13}h6"/></g>`;

