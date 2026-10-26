// ---- o cartão de um agente na visão de acompanhamento (foco-vista.js): figura do Claude na pose atual, papel, estado, onde trabalha, última fala, arquivos, desde quando e os últimos eventos ----
// Só monta o HTML (tudo escapado; os textos do agente nunca entram como HTML). A figura é a mesma do Claude na folha (claude-camada.js, agentes-claude.mjs), desenhada uma vez e guardada.
// O cartão da lista é compacto (3 eventos); o cartão grande é o detalhe do agente fixado (histórico maior, sem recorte da fala). Só indicador: nenhum botão manda ordem ao agente.
const focoPapel = () => ({ explorador: tr("Explorador"), construtor: tr("Construtor"), revisor: tr("Revisor") });

// a figura (parada, a pose do estado) ou, enquanto o desenho do Claude carrega, um espaço do mesmo tamanho; o fim do carregamento refaz a folha e a visão (claudeCarrega → montaRobo → focoDepoisDoRender)
function focoFigura(i, principal) {
  if (!claudeMod || !EMB) { claudeCarrega(); return `<span class="foco-fig vazia" aria-hidden="true"></span>`; }
  const f = claudeFigura(i, principal, false, claudeMod.poseDoEstado);
  return `<span class="foco-fig">${claudeSvg(f.papel, f.pose)}</span>`;
}

// onde ele está, em texto: blueprint, ação (com o número da folha, ex. 2.9) e plano; em repouso, só o blueprint
function focoOnde(i) {
  const p = D.blueprints.find((x) => x.id === i.blueprintId), f = p?.acoes.find((x) => x.id === i.acaoId);
  if (!f) return `<p class="foco-onde">${i.repouso ? tr("Em repouso: sem plano") : tr("Sem plano")}${p ? ` · ${esc(p.nome || tr("sem nome"))}` : ""}</p>`;
  return `<p class="foco-onde">${tr("Ação")} ${esc(numeroDaAcao(p, f))} · ${esc(f.nome || tr("sem nome"))}</p>${i.planoId ? `<p class="foco-plano"><span>${tr("Plano")}</span> ${esc(i.planoTexto || tr("sem texto"))}</p>` : ""}`;
}

// o cartão. `o`: { principal (o objeto do principal), emFoco, grande, eventos (a lista da ação, ou undefined enquanto carrega) }
function focoCartaoHtml(i, o) {
  const papel = focoPapel()[i.papel] || (i === o.principal ? tr("Agente principal") : tr("Subagente")), nome = esc(i.agente || tr("agente"));
  const evs = o.eventos ? focoEventosDoAgente(o.eventos, i, o.grande ? FOCO_HISTORICO : FOCO_EVENTOS) : null;
  const desde = i.planoId ? focoDesde(i.inicioEm, o.eventos, i.agente, i.planoId) : 0, min = focoMinutos(desde, i.em);
  const linha = (rotulo, valor) => (valor ? `<p class="foco-linha"><span>${rotulo}</span> ${esc(valor)}</p>` : "");
  const f = D.blueprints.find((x) => x.id === i.blueprintId)?.acoes.find((x) => x.id === i.acaoId);
  const ir = i.acaoId ? `<button type="button" class="btn mini" data-foco-ir="${esc(i.agente)}" data-bp="${esc(i.blueprintId)}" data-acao="${esc(i.acaoId)}" data-plano="${esc(i.planoId)}">${i.planoId ? tr("Ir ao plano na folha") : tr("Ir à ação na folha")}</button>` : "";
  const abrir = o.grande ? `<b class="foco-nome">${nome}</b>` : `<button type="button" class="foco-abrir" data-foco-abrir="${esc(i.agente)}" aria-label="${esc(tr("Ver o histórico de {agente}", { agente: i.agente || tr("agente") }))}"><b class="foco-nome">${nome}</b></button>`;
  return `<article class="foco-cartao${o.grande ? " grande" : ""}${i === o.principal ? " principal" : ""}${o.emFoco && !o.grande ? " em-foco" : ""} est-${esc(i.estado || "")}" data-agente="${esc(i.agente)}"${o.emFoco && !o.grande ? ' aria-current="true"' : ""}>`
    + `${focoFigura(i, o.principal)}<div class="foco-info"><header>${abrir}<span class="foco-chip">${esc(papel)}</span><span class="foco-chip est">${esc(ESTADO_AG[i.estado] || i.estado || tr("Trabalhando"))}${i.percentual != null ? ` · ${i.percentual}%` : ""}</span></header>`
    + `${focoOnde(i)}${linha(tr("Agora"), i.mensagem ? `${i.mensagem} · ${ldtHora(i.em)}` : tr("sem detalhe"))}${linha(tr("Arquivos"), i.onde.join(" · "))}`
    + `${desde ? `<p class="foco-linha"><span>${tr("No plano")}</span> ${esc(tr("desde {hora}", { hora: ldtHora(desde) }))}${min ? ` · ${esc(tr("{n} min de trabalho até o último aviso", { n: min }))}` : ""}</p>` : ""}`
    + `${i.acaoId ? `<section class="ldt"><h5>${o.grande ? tr("Histórico do agente neste plano") : tr("Últimos eventos")}</h5>${evs ? ldtLista(evs, f) : `<p class="ldt-vazio">${tr("carregando…")}</p>`}</section>` : ""}`
    + `<div class="foco-acoes">${ir}</div></div></article>`;
}
