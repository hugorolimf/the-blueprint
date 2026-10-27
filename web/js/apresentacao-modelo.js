// ---- apresentação: o modelo (função pura, sem DOM) ----
// montarApresentacao(blueprint, opcoes) -> { titulo, tipo, idioma, paginas: [{ tipo, html }] }; cada `html` é um <section class="ap-pagina" data-tipo="..."> inteiro.
// Tipos: "andamento" (capa, uma página por projeto, fim) e "completo" (capa, uma página por ação com planos e checkpoints, fim). Não há página de resumo (decisão do usuário, 2026-10-26).
// opcoes: { tipo, workspace, hoje (ISO), posicao (índice do blueprint no workspace, para numerar as ações), figura (SVG pronto da figura do anatomy; confiável, entra sem escape) }.
// Usa os globais de base.js (esc, dia, corta, ROTULO), dados.js (acoesDe, resumo), projetos.js (projetoDaAcao, projetosOrdenados, numeroDaAcao) e i18n.js (tr).
// Todo texto vindo dos dados passa por esc(); links não viram <a> (clicar numa página da apresentação nunca leva a janela embora).
const TIPOS_APRESENTACAO = ["andamento", "completo"];
const AP_POR_PAGINA = { andamento: 6, completo: 7 }; // linhas por página antes de continuar na seguinte (a página é 16:9 e não rola: o que passa vai para a próxima)

const apPct = (x) => Math.round(x * 100);
const apBarra = (x) => `<div class="ap-barra" role="img" aria-label="${apPct(x)}%"><i style="--p:${apPct(x)}"></i></div>`;
const apData = (iso) => {
  const d = dia(iso);
  return d === d ? new Intl.DateTimeFormat(I18N.ativo, { day: "2-digit", month: "short", year: "numeric", timeZone: "UTC" }).format(d * 864e5) : "";
};
const apPeriodo = (f) => `${apData(f.inicio)} – ${apData(f.fim)}`;
const apPartes = (lista, n) => Array.from({ length: Math.max(1, Math.ceil(lista.length / n)) }, (_, i) => lista.slice(i * n, (i + 1) * n));
const apMarca = (feito) => `<span class="ap-marca" data-feito="${feito ? 1 : 0}" aria-hidden="true"></span><span class="ap-so-leitor">${feito ? tr("Feito") : tr("Pendente")}</span>`;
const apTipoRotulo = () => ({ andamento: tr("Andamento por projeto"), completo: tr("Plano completo") });

// as ações em grupos (os projetos na ordem da folha e, por último, o Geral); sem projetos, um grupo só com o blueprint inteiro
function apGrupos(p, fs) {
  if (!p.projetos?.length) return [{ id: "", nome: p.nome || tr("Sem nome"), descricao: "", acoes: fs, unico: true }];
  const por = (id) => fs.filter((f) => projetoDaAcao(p, f) === id);
  const ids = [...projetosOrdenados(p).map((x) => ({ id: x.id, nome: x.nome || tr("Sem nome"), descricao: x.descricao || "" })), { id: "", nome: tr("Geral"), descricao: "" }];
  return ids.map((g) => ({ ...g, acoes: por(g.id) })).filter((g) => g.acoes.length);
}

const apLinhaAcao = (f, num) => `<li><span class="ap-num">${esc(num)}</span><span class="ap-nome" title="${esc(f.nome)}">${esc(f.nome)}</span><span class="ap-datas">${esc(apPeriodo(f))}</span><span class="ap-pct" data-status="${f.status}">${apPct(f.pct)}%</span></li>`;

function apCapa(p, o, titulo) {
  const figura = o.figura ? `<figure class="ap-figura">${o.figura}</figure>` : "";
  return { tipo: "capa", corpo: `<div class="ap-capa"><div class="ap-capa-texto"><p class="ap-rot">${esc(o.workspace || "The Blueprint")}</p><h1 class="ap-titulo">${esc(p.nome)}</h1>${p.descricao ? `<p class="ap-sub">${esc(p.descricao)}</p>` : ""}<p class="ap-data">${esc(titulo)} · ${esc(apData(o.hoje))}</p></div>${figura}</div>` };
}

function apProjetos(p, fs, num) {
  return apGrupos(p, fs).flatMap((g) => {
    const r = resumo(g.acoes);
    return apPartes(g.acoes, AP_POR_PAGINA.andamento).map((parte, i) => ({ tipo: "projeto", corpo: `<p class="ap-rot">${g.unico ? tr("Todas as ações") : tr("Projeto")}</p><h2 class="ap-titulo">${esc(g.nome)}${i ? ` <small>${tr("(continuação)")}</small>` : ""}</h2>${g.descricao && !i ? `<p class="ap-sub">${esc(g.descricao)}</p>` : ""}
<div class="ap-geral"><span class="ap-grande">${apPct(r.geral)}%</span>${apBarra(r.geral)}</div><p class="ap-nota">${tr("{a} de {b} ações concluídas", { a: r.concl, b: g.acoes.length })}</p>
<ol class="ap-lista">${parte.map((f) => apLinhaAcao(f, numeroDaAcao(p, f, num.posicao))).join("")}</ol>` }));
  });
}

function apAcoes(p, fs, num) {
  return apGrupos(p, fs).flatMap((g) => g.acoes.flatMap((f) => {
    const itens = [...(f.planos || []).map((x) => ({ texto: x.texto, feito: x.feito, meta: x.horas ? `${x.horas} h` : "" })), ...(f.checkpoints || []).map((x) => ({ texto: x.nome, feito: x.feito, meta: x.data ? apData(x.data) : "", cp: true }))];
    const obs = String(f.obs || "").trim() ? String(f.obs).trim().split(/\n+/).slice(0, 4).map((l) => `<p>${esc(corta(l, 260))}</p>`).join("") : "";
    return apPartes(itens, AP_POR_PAGINA.completo).map((parte, i) => ({ tipo: "acao", corpo: `<p class="ap-rot">${esc(numeroDaAcao(p, f, num.posicao))}${g.unico ? "" : ` · ${esc(g.nome)}`}</p><h2 class="ap-titulo">${esc(f.nome)}${i ? ` <small>${tr("(continuação)")}</small>` : ""}</h2>
<p class="ap-meta"><span class="ap-status" data-status="${f.status}">${esc(ROTULO[f.status] || f.status)}</span> ${esc(apPeriodo(f))} · ${apPct(f.pct)}%</p>${apBarra(f.pct)}${obs && !i ? `<div class="ap-obs">${obs}</div>` : ""}
${parte.length ? `<ul class="ap-itens">${parte.map((x) => `<li${x.cp ? ' class="ap-cp"' : ""}>${apMarca(x.feito)}<span class="ap-texto" title="${esc(x.texto)}">${esc(x.texto)}</span><span class="ap-meta-item">${esc(x.meta)}</span></li>`).join("")}</ul>` : `<p class="ap-vazio">${tr("Sem planos nem checkpoints.")}</p>`}` }));
  }));
}

function apFim(p) {
  const links = (p.links || []).filter((l) => urlSegura(l.url));
  const corpo = links.length ? `<h2 class="ap-titulo">${tr("Referências")}</h2><ul class="ap-links">${links.map((l) => `<li><b>${esc(rotuloDe(l))}</b><span class="ap-url">${esc(urlSegura(l.url))}</span></li>`).join("")}</ul>` : `<h2 class="ap-titulo">${tr("Fim")}</h2>`;
  return { tipo: "fim", corpo: `${corpo}<p class="ap-nota">${tr("Gerado pelo The Blueprint")}</p>` };
}

function montarApresentacao(p, opcoes = {}) {
  const tipo = TIPOS_APRESENTACAO.includes(opcoes.tipo) ? opcoes.tipo : "andamento";
  const o = { ...opcoes, tipo, hoje: opcoes.hoje || local(new Date()) }, num = { hoje: o.hoje, posicao: opcoes.posicao ?? 0 };
  const fs = acoesDe(p), titulo = apTipoRotulo()[tipo];
  const paginas = [apCapa(p, o, titulo), ...(tipo === "andamento" ? apProjetos(p, fs, num) : []), ...(tipo === "completo" ? apAcoes(p, fs, num) : []), apFim(p)];
  return {
    titulo: p.nome || tr("Sem nome"), tipo, idioma: I18N.ativo,
    paginas: paginas.map((pg, i) => ({ tipo: pg.tipo, html: `<section class="ap-pagina" data-tipo="${pg.tipo}"><div class="ap-int">${pg.corpo}${pg.tipo === "capa" ? "" : `<footer class="ap-rodape"><span>${esc(p.nome)}</span><span>${i + 1} / ${paginas.length}</span></footer>`}</div></section>` })),
  };
}
