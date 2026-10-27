// Seções da apresentação como texto HTML puro (sem DOM): cada função recebe o `conteudo` e devolve uma string. Nada é inventado aqui:
// os números vêm das datas de `conteudo`, o resto é o que o agente escreveu. O que vem de fora passa sempre por esc().
(function (L) {
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const dia = (iso) => (/^\d{4}-\d{2}-\d{2}$/.test(String(iso || "")) ? Date.UTC(+iso.slice(0, 4), +iso.slice(5, 7) - 1, +iso.slice(8, 10)) / 86400000 : NaN);
  const num = (n) => String(n).padStart(2, "0");
  const lista = (v) => (Array.isArray(v) ? v : []);

  // período do plano e onde a data do documento cai nele
  const periodo = (c) => {
    const ini = Math.min(...lista(c.etapas).map((e) => dia(e.inicio)).filter(Number.isFinite)), fim = Math.max(...lista(c.etapas).map((e) => dia(e.fim)).filter(Number.isFinite));
    if (!Number.isFinite(ini) || !Number.isFinite(fim)) return null;
    const hoje = dia(c.data), total = fim - ini + 1;
    const n = Number.isFinite(hoje) ? Math.min(total, Math.max(0, hoje - ini + 1)) : 0;
    return { ini, fim, total, n, pct: Math.round((n / total) * 100), hoje: Number.isFinite(hoje) ? hoje : null };
  };
  // `etapa.situacao` (prevista, andamento, concluida), quando vem no conteúdo, vale mais que as datas: o blueprint é quem sabe o que já começou
  const situacao = (e, c) => {
    if (["prevista", "andamento", "concluida"].includes(e.situacao)) return e.situacao;
    const hoje = dia(c.data);
    if (!Number.isFinite(hoje)) return "prevista";
    return hoje > dia(e.fim) ? "concluida" : hoje >= dia(e.inicio) ? "andamento" : "prevista";
  };
  const doDia = (n, idioma) => L.dataCurta(new Date(n * 86400000).toISOString().slice(0, 10), idioma);

  const secao = (id, n, titulo, corpo, extra = "") =>
    `<section class="ap-sec" id="s-${id}" aria-labelledby="h-${id}" data-sec="${id}"${extra}><header class="ap-sec-cab"><span class="ap-sec-n" aria-hidden="true">${num(n)}</span><h2 id="h-${id}">${esc(titulo)}</h2><span class="ap-sec-regua" aria-hidden="true"></span></header><div class="ap-sec-corpo">${corpo}</div></section>`;
  // a figura da seção ao lado do texto (abaixo em tela estreita); duas figuras na mesma seção ficam uma após a outra
  const lado = (corpo, fig) => (fig ? `<div class="ap-com-fig"><div class="ap-com-texto">${corpo}</div><div class="ap-com-fig-col">${fig}</div></div>` : corpo);
  const vazio = (idioma) => `<p class="ap-vazio">${esc(L.t("nada", idioma))}</p>`;
  const datas = (a, b, idioma) => (a && b ? `${esc(L.dataCurta(a, idioma))} &rarr; ${esc(L.dataCurta(b, idioma))}` : "");

  const visao = (c, cartoes, n) => {
    const p = periodo(c), idioma = c.idioma, t = (k, par) => L.t(k, idioma, par);
    const resumo = lista(c.resumo).map((f) => `<li>${esc(f)}</li>`).join("");
    const kpis = p ? [
      [t("periodo"), `${doDia(p.ini, idioma)} &rarr; ${doDia(p.fim, idioma)}`],
      [t("etapas"), String(lista(c.etapas).length)],
      [t("entregas"), String(lista(c.entregas).length)],
      [t("progresso"), p.hoje === null || p.hoje < p.ini ? t("comeca", { d: doDia(p.ini, idioma) }) : t("dia_de", { n: p.n, total: p.total })],
    ].map(([r, v]) => `<div class="ap-kpi"><span>${esc(r)}</span><b>${v}</b></div>`).join("") : "";
    const barra = p ? `<div class="ap-barra" role="img" aria-label="${esc(t("progresso"))}: ${p.pct}%"><i style="width:${p.pct}%"></i></div>` : "";
    return secao("visao", n, t("visao"), `<div class="ap-visao"><div class="ap-visao-texto"><h3 class="ap-titulo">${esc(c.titulo)}</h3>${c.subtitulo ? `<p class="ap-sub">${esc(c.subtitulo)}</p>` : ""}${c.cliente ? `<p class="ap-cliente">${esc(c.cliente)}</p>` : ""}<ul class="ap-resumo">${resumo}</ul></div><div class="ap-visao-fig">${cartoes.capa || ""}</div></div><div class="ap-kpis">${kpis}</div>${barra}`);
  };

  const entregas = (c, n, fig) => {
    const l = lista(c.entregas);
    return secao("entregas", n, L.t("entregas", c.idioma), lado(l.length ? `<div class="ap-grade">${l.map((e, i) => `<article class="ap-cartao"><span class="ap-num">${num(i + 1)}</span><h3>${esc(e.titulo)}</h3><p>${esc(e.texto)}</p></article>`).join("")}</div>` : vazio(c.idioma), fig));
  };

  const etapas = (c, cartoes, n, extras = "", fig = "") => {
    const idioma = c.idioma, l = lista(c.etapas);
    const corpo = l.map((e, i) => {
      const st = situacao(e, c), marcos = lista(e.marcos);
      const m = marcos.length ? `<ul class="ap-marcos" aria-label="${esc(L.t("marcos", idioma))}">${marcos.map((k) => `<li><i aria-hidden="true"></i><b>${esc(L.dataCurta(k.data, idioma))}</b><span>${esc(k.nome)}</span></li>`).join("")}</ul>` : "";
      return `<article class="ap-etapa" id="e-${esc(e.id || i + 1)}" data-st="${st}"><header><span class="ap-num">${num(i + 1)}</span><h3>${esc(e.titulo)}</h3><span class="ap-datas">${datas(e.inicio, e.fim, idioma)}</span><span class="ap-tag" data-st="${st}">${esc(L.t("st_" + st, idioma))}</span></header><p>${esc(e.texto)}</p>${m}${e.figura && cartoes[e.figura] ? `<div class="ap-etapa-fig">${cartoes[e.figura]}</div>` : ""}</article>`;
    }).join("");
    return secao("etapas", n, L.t("etapas", idioma), lado((l.length ? corpo : vazio(idioma)) + (extras ? `<div class="ap-etapa-fig ap-extras">${extras}</div>` : ""), fig));
  };

  const linha = (c, n, fig = "") => {
    const p = periodo(c), idioma = c.idioma;
    if (!p) return secao("linha", n, L.t("linha", idioma), lado(vazio(idioma), fig));
    const pos = (d) => (((d - p.ini) / p.total) * 100).toFixed(2);
    const eixo = [];
    const passo = Math.max(1, Math.ceil(p.total / 6));
    for (let d = p.ini; d <= p.fim; d += passo) eixo.push(`<span style="left:${pos(d)}%">${esc(doDia(d, idioma))}</span>`);
    const linhas = lista(c.etapas).map((e, i) => {
      const a = dia(e.inicio), b = dia(e.fim);
      if (!Number.isFinite(a) || !Number.isFinite(b)) return "";
      const marcos = lista(e.marcos).map((k) => (Number.isFinite(dia(k.data)) ? `<b class="ap-g-marco" style="left:${pos(dia(k.data) + 0.5)}%" title="${esc(k.nome)}"></b>` : "")).join("");
      return `<li class="ap-g-linha"><span class="ap-g-nome"><i>${num(i + 1)}</i>${esc(e.titulo)}</span><div class="ap-g-faixa"><i class="ap-g-barra" data-st="${situacao(e, c)}" style="left:${pos(a)}%;width:${(((b - a + 1) / p.total) * 100).toFixed(2)}%"></i>${marcos}</div></li>`;
    }).join("");
    const hoje = p.hoje !== null && p.hoje >= p.ini && p.hoje <= p.fim ? `<i class="ap-g-hoje" style="--p:${((p.hoje + 0.5 - p.ini) / p.total).toFixed(4)}" title="${esc(L.t("hoje", idioma))}"></i>` : "";
    return secao("linha", n, L.t("linha", idioma), lado(`<div class="ap-gantt" style="--passo:${((passo / p.total) * 100).toFixed(3)}%"><div class="ap-g-eixo" aria-hidden="true"><span></span><div>${eixo.join("")}</div></div><ol class="ap-g-lista">${linhas}</ol>${hoje}</div>`, fig));
  };

  const voce = (c, n, fig = "") => {
    const l = lista(c.precisamosDeVoce), idioma = c.idioma;
    return secao("voce", n, L.t("voce", idioma), lado(l.length ? `<div class="ap-grade ap-grade-voce">${l.map((x, i) => `<article class="ap-cartao ap-pedido"><span class="ap-num">${num(i + 1)}</span><h3>${esc(x.titulo)}</h3><p>${esc(x.texto)}</p>${x.ate ? `<span class="ap-ate">${esc(L.t("ate", idioma))} ${esc(L.dataCurta(x.ate, idioma))}</span>` : ""}</article>`).join("")}</div>` : vazio(idioma), fig));
  };

  const decisoes = (c, n, figD = "", figR = "") => {
    const idioma = c.idioma, d = lista(c.decisoes), r = lista(c.riscos);
    const colD = d.length ? d.map((x) => `<article class="ap-item" data-estado="${x.estado === "decidida" ? "decidida" : "aberta"}"><h4>${esc(x.titulo)}</h4><p>${esc(x.texto)}</p><span class="ap-tag" data-estado="${x.estado === "decidida" ? "decidida" : "aberta"}">${esc(L.t(x.estado === "decidida" ? "decidida" : "aberta", idioma))}</span></article>`).join("") : vazio(idioma);
    const colR = r.length ? r.map((x) => `<article class="ap-item"><h4>${esc(x.titulo)}</h4><p>${esc(x.texto)}</p></article>`).join("") : vazio(idioma);
    return secao("decisoes", n, L.t("decisoes", idioma), `<div class="ap-duas"><div><h3 class="ap-sub-t">${esc(L.t("decisoes_t", idioma))}</h3>${colD}${figD ? `<div class="ap-fig-col">${figD}</div>` : ""}</div><div><h3 class="ap-sub-t">${esc(L.t("riscos", idioma))}</h3>${colR}${figR ? `<div class="ap-fig-col">${figR}</div>` : ""}</div></div>`);
  };

  // só no público interno
  const acoes = (c, n, fig = "") => {
    const l = lista(c.interno && c.interno.acoes);
    return secao("acoes", n, L.t("acoes", c.idioma), lado(l.length ? l.map((a) => `<article class="ap-item"><h4><span class="ap-num">${esc(a.numero)}</span> ${esc(a.nome)}</h4><ul class="ap-planos">${lista(a.planos).map((p) => `<li>${esc(p)}</li>`).join("")}</ul></article>`).join("") : vazio(c.idioma), fig));
  };

  L.secoes = { esc, dia, periodo, situacao, visao, entregas, etapas, linha, voce, decisoes, acoes };
})(L);
