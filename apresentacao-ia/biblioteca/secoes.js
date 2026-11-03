// Seções da apresentação como texto HTML puro (sem DOM): cada função recebe o `conteudo` e devolve uma string. Nada é inventado aqui:
// tudo é o que o agente escreveu em `conteudo`. O que vem de fora passa sempre por esc().
(function (L) {
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const num = (n) => String(n).padStart(2, "0");
  const lista = (v) => (Array.isArray(v) ? v : []);

  // `etapa.situacao` vem do conteúdo (prevista, andamento, concluida): o blueprint é quem sabe o que já começou
  const situacao = (e) => (["prevista", "andamento", "concluida"].includes(e.situacao) ? e.situacao : "prevista");

  const secao = (id, n, titulo, corpo, extra = "") =>
    `<section class="ap-sec" id="s-${id}" aria-labelledby="h-${id}" data-sec="${id}"${extra}><header class="ap-sec-cab"><span class="ap-sec-n" aria-hidden="true">${num(n)}</span><h2 id="h-${id}">${esc(titulo)}</h2><span class="ap-sec-regua" aria-hidden="true"></span></header><div class="ap-sec-corpo">${corpo}</div></section>`;
  // a figura da seção ao lado do texto (abaixo em tela estreita); duas figuras na mesma seção ficam uma após a outra
  const lado = (corpo, fig) => (fig ? `<div class="ap-com-fig"><div class="ap-com-texto">${corpo}</div><div class="ap-com-fig-col">${fig}</div></div>` : corpo);
  const vazio = (idioma) => `<p class="ap-vazio">${esc(L.t("nada", idioma))}</p>`;

  const visao = (c, cartoes, n) => {
    const idioma = c.idioma, t = (k, par) => L.t(k, idioma, par);
    const resumo = lista(c.resumo).map((f) => `<li>${esc(f)}</li>`).join("");
    const kpis = [
      [t("etapas"), String(lista(c.etapas).length)],
      [t("entregas"), String(lista(c.entregas).length)],
    ].map(([r, v]) => `<div class="ap-kpi"><span>${esc(r)}</span><b>${v}</b></div>`).join("");
    return secao("visao", n, t("visao"), `<div class="ap-visao"><div class="ap-visao-texto"><h3 class="ap-titulo">${esc(c.titulo)}</h3>${c.subtitulo ? `<p class="ap-sub">${esc(c.subtitulo)}</p>` : ""}${c.cliente ? `<p class="ap-cliente">${esc(c.cliente)}</p>` : ""}<ul class="ap-resumo">${resumo}</ul></div><div class="ap-visao-fig">${cartoes.capa || ""}</div></div><div class="ap-kpis">${kpis}</div>`);
  };

  const entregas = (c, n, fig) => {
    const l = lista(c.entregas);
    return secao("entregas", n, L.t("entregas", c.idioma), lado(l.length ? `<div class="ap-grade">${l.map((e, i) => `<article class="ap-cartao" data-item="${i}" data-sessao="recebe" tabindex="0"><span class="ap-num">${num(i + 1)}</span><h3>${esc(e.titulo)}</h3><p>${esc(e.texto)}</p></article>`).join("")}</div>` : vazio(c.idioma), fig));
  };

  const etapas = (c, cartoes, n, extras = "", fig = "") => {
    const idioma = c.idioma, l = lista(c.etapas);
    const corpo = l.map((e, i) => {
      const st = situacao(e), marcos = lista(e.marcos);
      const m = marcos.length ? `<ul class="ap-marcos" aria-label="${esc(L.t("marcos", idioma))}">${marcos.map((k) => `<li><i aria-hidden="true"></i><span>${esc(k.nome)}</span></li>`).join("")}</ul>` : "";
      return `<article class="ap-etapa" id="e-${esc(e.id || i + 1)}" data-st="${st}" data-item="${i}" data-sessao="etapas" tabindex="0"><header><span class="ap-num">${num(i + 1)}</span><h3>${esc(e.titulo)}</h3><span class="ap-tag" data-st="${st}">${esc(L.t("st_" + st, idioma))}</span></header><p>${esc(e.texto)}</p>${m}${e.figura && cartoes[e.figura] ? `<div class="ap-etapa-fig">${cartoes[e.figura]}</div>` : ""}</article>`;
    }).join("");
    return secao("etapas", n, L.t("etapas", idioma), lado((l.length ? corpo : vazio(idioma)) + (extras ? `<div class="ap-etapa-fig ap-extras">${extras}</div>` : ""), fig));
  };

  const voce = (c, n, fig = "") => {
    const l = lista(c.precisamosDeVoce), idioma = c.idioma;
    return secao("voce", n, L.t("voce", idioma), lado(l.length ? `<div class="ap-grade ap-grade-voce">${l.map((x, i) => `<article class="ap-cartao ap-pedido" data-item="${i}" data-sessao="precisamos" tabindex="0"><span class="ap-num">${num(i + 1)}</span><h3>${esc(x.titulo)}</h3><p>${esc(x.texto)}</p></article>`).join("")}</div>` : vazio(idioma), fig));
  };

  const decisoes = (c, n, figD = "", figR = "") => {
    const idioma = c.idioma, d = lista(c.decisoes), r = lista(c.riscos);
    const colD = d.length ? d.map((x, i) => `<article class="ap-item" data-item="${i}" data-sessao="decisoes" tabindex="0" data-estado="${x.estado === "decidida" ? "decidida" : "aberta"}"><h4>${esc(x.titulo)}</h4><p>${esc(x.texto)}</p><span class="ap-tag" data-estado="${x.estado === "decidida" ? "decidida" : "aberta"}">${esc(L.t(x.estado === "decidida" ? "decidida" : "aberta", idioma))}</span></article>`).join("") : vazio(idioma);
    const colR = r.length ? r.map((x, i) => `<article class="ap-item" data-item="${i}" data-sessao="riscos" tabindex="0"><h4>${esc(x.titulo)}</h4><p>${esc(x.texto)}</p></article>`).join("") : vazio(idioma);
    return secao("decisoes", n, L.t("decisoes", idioma), `<div class="ap-duas"><div><h3 class="ap-sub-t">${esc(L.t("decisoes_t", idioma))}</h3>${colD}${figD ? `<div class="ap-fig-col">${figD}</div>` : ""}</div><div><h3 class="ap-sub-t">${esc(L.t("riscos", idioma))}</h3>${colR}${figR ? `<div class="ap-fig-col">${figR}</div>` : ""}</div></div>`);
  };

  // só no público interno
  const acoes = (c, n, fig = "") => {
    const l = lista(c.interno && c.interno.acoes);
    return secao("acoes", n, L.t("acoes", c.idioma), lado(l.length ? l.map((a) => `<article class="ap-item"><h4><span class="ap-num">${esc(a.numero)}</span> ${esc(a.nome)}</h4><ul class="ap-planos">${lista(a.planos).map((p) => `<li>${esc(p)}</li>`).join("")}</ul></article>`).join("") : vazio(c.idioma), fig));
  };

  L.secoes = { esc, situacao, visao, entregas, etapas, voce, decisoes, acoes };
})(L);
