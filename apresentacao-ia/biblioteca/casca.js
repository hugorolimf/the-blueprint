// A casca da apresentação: monta a interface (barra superior, navegação por seções, painéis) a partir de `conteudo` e roda as figuras.
// L.casca.html(conteudo, figuras) devolve { html, vivos } sem tocar no DOM (serve aos testes e à prévia); L.casca.montar(raiz, ...) a põe na página.
// Cada figura vai para o painel da sua seção (`figura.secao`: visao, recebe, etapas, linha, precisamos, decisoes, riscos, acoes), ao lado do texto; a que uma etapa
// cita (`etapas[].figura`) fica dentro da etapa; a sem seção e sem etapa vai ao fim de Etapas. Uma figura que lança erro vira um cartão de erro; o resto da página segue.
(function (L) {
  const { esc } = L.secoes, S = L.secoes;

  // `erro` nulo = o bloco do código da figura nem carregou (erro de sintaxe: `codigo` não chegou como função); senão, erro ao desenhar
  const cartaoDeErro = (f, n, idioma, erro) =>
    `<figure class="iso-figure ap-fig-erro" role="alert"><div class="iso-plate"><p class="ap-erro-t"><b>${esc(L.t("fig", idioma))} ${n}</b> ${esc(f.titulo || f.id)}</p>${erro ? `<p>${esc(L.t("figura_erro", idioma))}</p><p class="ap-erro-d">${esc(L.t("figura_erro_det", idioma))}: <code>${esc(erro.message ? erro.message : String(erro))}</code></p>` : `<p>${esc(L.t("codigo_nao_carregou", idioma))}</p>`}</div></figure>`;

  // cada figura vira um cartão do kit (Fig, título, dica, leitura, chaves, legenda); `vivos` guarda o que ganha movimento depois
  // o cartão de cada figura vai num .ap-fig com id: é para onde o link "Fig n" da linha de referência rola
  const embrulha = (f, html) => `<div class="ap-fig" id="fig-${esc(f.id)}">${html}</div>`;
  const rodar = (c, figuras) => {
    const cartoes = {}, vivos = [];
    figuras.forEach((f, i) => {
      const n = i + 1;
      if (typeof f.codigo !== "function") { cartoes[f.id] = embrulha(f, cartaoDeErro(f, n, c.idioma, null)); return; }
      try {
        const r = f.codigo(L, c);
        cartoes[f.id] = embrulha(f, L.plateHtml({
          fig: `${L.t("fig", c.idioma)} ${n}`, title: f.titulo, hint: f.dica, readout: f.leitura || "",
          keys: (f.chaves || []).map((k) => ({ mark: k.marca, label: k.rotulo })), caption: f.legenda,
          body: `<div class="iso-stage" tabindex="0" role="group" aria-label="${esc(f.rotulo || f.titulo)}" data-figura="${esc(f.id)}">${r.svg}</div>`,
        }));
        if (typeof r.vivo === "function") vivos.push({ id: f.id, vivo: r.vivo });
      } catch (e) { cartoes[f.id] = embrulha(f, cartaoDeErro(f, n, c.idioma, e)); }
    });
    return { cartoes, vivos };
  };

  // o nome da figura no conteúdo -> o id da seção da página (riscos mora na seção Decisões e riscos); desconhecido = sem seção
  // a seção da página onde a figura está -> o grupo de itens do texto que ela comanda pela ponte (biblioteca/ponte.js); visao e acoes não têm itens
  const GRUPO_DA_SECAO = { entregas: "recebe", etapas: "etapas", linha: "linha", voce: "precisamos", decisoes: "decisoes", riscos: "riscos" };
  const SECAO_DA_FIGURA = { visao: "visao", recebe: "entregas", entregas: "entregas", etapas: "etapas", linha: "linha", precisamos: "voce", voce: "voce", decisoes: "decisoes", riscos: "riscos", acoes: "acoes" };
  const secaoDaFigura = (f) => SECAO_DA_FIGURA[f && f.secao];
  const usadasPorEtapas = (c) => new Set((c.etapas || []).map((e) => e.figura).filter(Boolean));

  // a figura da capa (Visão geral): a de secao "visao", senão a de id "capa", senão a primeira sem seção; uma que uma etapa cita fica na etapa
  const capaDe = (c, figuras) => {
    const livres = figuras.filter((f) => !usadasPorEtapas(c).has(f.id));
    return (livres.find((f) => secaoDaFigura(f) === "visao") || livres.find((f) => f.id === "capa") || livres.find((f) => !f.secao) || {}).id;
  };

  const secoesDe = (c) => ["visao", c.entregas && c.entregas.length ? "entregas" : null, "etapas", "linha", c.precisamosDeVoce && c.precisamosDeVoce.length ? "voce" : null, "decisoes", c.publico === "interno" ? "acoes" : null].filter(Boolean);

  const html = (c, figuras = []) => {
    c = { ...c, idioma: L.idiomaDe(c.idioma) };
    const idioma = c.idioma, t = (k, p) => L.t(k, idioma, p);
    const { cartoes, vivos } = rodar(c, figuras);
    const ids = secoesDe(c), capa = capaDe(c, figuras), usadas = usadasPorEtapas(c);
    // onde cada figura cai: a da etapa, dentro dela; a da capa e as de seção, no painel da seção; o resto (e o que cairia numa seção que não existe), no fim de Etapas
    // `refs`: a linha "Fig n · dica" de cada seção que tem figura; `grupos`: de que grupo de itens do texto cada figura é (para a ponte)
    const porSecao = {}, refs = {}, grupos = {}, soltas = [];
    for (const f of figuras) {
      if (usadas.has(f.id)) continue;
      const sec = f.id === capa ? "visao" : secaoDaFigura(f), existe = sec && ids.includes(sec === "riscos" ? "decisoes" : sec);
      if (!existe) { soltas.push(cartoes[f.id]); continue; }
      const pos = f.id === capa ? "unshift" : "push";
      (porSecao[sec] = porSecao[sec] || [])[pos](cartoes[f.id]);
      (refs[sec] = refs[sec] || [])[pos]({ id: f.id, n: figuras.indexOf(f) + 1, dica: f.dica });
      grupos[f.id] = GRUPO_DA_SECAO[sec];
    }
    vivos.forEach((v) => { v.grupo = grupos[v.id]; });
    const refLinha = (id) => {
      const lista = [...(refs[id] || []), ...(id === "decisoes" ? refs.riscos || [] : [])];
      if (!lista.length) return "";
      const links = lista.map((r) => `<a class="ap-ref-link" href="#fig-${esc(r.id)}" data-ref="${esc(r.id)}" aria-label="${esc(t("ir_fig", { n: r.n }))}">${esc(t("fig"))} ${r.n}</a>${r.dica ? ` · <span>${esc(r.dica)}</span>` : ""}`).join(" · ");
      return `<p class="ap-ref">${links}${id === "visao" || id === "acoes" ? "" : ` <span class="ap-ref-dica">${esc(t("ref_item"))}</span>`}</p>`;
    };
    const comRef = (id, secaoHtml) => secaoHtml.replace('<div class="ap-sec-corpo">', (m) => m + refLinha(id)); // função: a dica da figura pode ter "$&"
    const j = (k) => (porSecao[k] || []).join(""), extras = soltas.join("");
    const montar = { visao: (n) => S.visao(c, { capa: j("visao") }, n), entregas: (n) => S.entregas(c, n, j("entregas")), etapas: (n) => S.etapas(c, cartoes, n, extras, j("etapas")), linha: (n) => S.linha(c, n, j("linha")), voce: (n) => S.voce(c, n, j("voce")), decisoes: (n) => S.decisoes(c, n, j("decisoes"), j("riscos")), acoes: (n) => S.acoes(c, n, j("acoes")) };
    const corpo = ids.map((id, i) => comRef(id, montar[id](i + 1))).join("");
    const nav = ids.map((id, i) => `<li><a href="#s-${id}" data-nav="${id}"><i>${String(i + 1).padStart(2, "0")}</i>${esc(t(id))}</a></li>`).join("");
    const nivel = c.nivel ? `<span class="ap-etiqueta">${esc(t("nivel"))}: ${esc(t("nivel_" + c.nivel))}</span>` : "";
    const quem = c.publico === "interno" ? "interno" : "cliente";
    const pagina = `<div class="ap-ia iso" data-theme="dark" lang="${esc(idioma)}"><a class="ap-pular" href="#conteudo">${esc(t("pular"))}</a>
<header class="ap-topo"><div class="ap-migalhas"><span>${esc(t(quem))}</span><i aria-hidden="true">/</i><span>${esc(t(quem === "interno" ? "plano_interno" : "plano"))}</span></div><div class="ap-titulo-caixa"><b>${esc(c.titulo)}</b>${c.cliente ? `<span>${esc(c.cliente)}</span>` : ""}</div><div class="ap-topo-acoes">${nivel}<button class="ap-btn" type="button" data-acao="imprimir">${esc(t("imprimir"))}</button></div></header>
<div class="ap-corpo"><nav class="ap-nav" aria-label="${esc(t("menu"))}"><p class="ap-rot">${esc(t("menu"))}</p><ol>${nav}</ol><p class="ap-dica">${esc(t("nav_dica"))}</p></nav><main id="conteudo" class="ap-principal" tabindex="-1">${corpo}</main></div>
<footer class="ap-rodape"><span>${esc(t("sim"))}</span><span>${c.data ? esc(t("gerado", { d: L.dataCurta(c.data, idioma) })) : ""}</span></footer></div>`;
    return { html: pagina, vivos };
  };

  const montar = (raiz, conteudo, figuras) => {
    const { html: pagina, vivos } = html(conteudo, figuras);
    raiz.innerHTML = pagina;
    const pontes = L.ponte.ligar(raiz, conteudo);
    for (const { id, vivo, grupo } of vivos) {
      const palco = raiz.querySelector(`[data-figura="${CSS.escape(id)}"]`), leitura = palco && palco.closest(".iso-plate").querySelector("[data-readout]");
      if (!palco) continue;
      try { vivo(palco, (texto) => { if (leitura && leitura.textContent !== texto) leitura.textContent = texto; }, pontes.criar(grupo)); } catch (e) { console.error(e); }
    }
    ligarNavegacao(raiz);
  };

  // navegação: seção atual no menu, [ e ] trocam de seção, Home e End, e o botão de imprimir
  const ligarNavegacao = (raiz) => {
    const secoes = [...raiz.querySelectorAll(".ap-sec")], links = [...raiz.querySelectorAll("[data-nav]")];
    const marca = (id) => links.forEach((a) => (a.dataset.nav === id ? a.setAttribute("aria-current", "true") : a.removeAttribute("aria-current")));
    if (typeof IntersectionObserver === "function") {
      const visiveis = new Set();
      const io = new IntersectionObserver((es) => {
        for (const e of es) (e.isIntersecting ? visiveis.add(e.target) : visiveis.delete(e.target));
        const primeira = secoes.find((s) => visiveis.has(s));
        if (primeira) marca(primeira.dataset.sec);
      }, { rootMargin: "-20% 0px -65% 0px" });
      secoes.forEach((s) => io.observe(s));
    }
    marca(secoes[0] && secoes[0].dataset.sec);
    const ir = (i) => { const s = secoes[Math.min(secoes.length - 1, Math.max(0, i))]; if (s) { s.scrollIntoView({ block: "start" }); s.focus?.({ preventScroll: true }); } };
    const atual = () => Math.max(0, secoes.findIndex((s) => s.dataset.sec === (links.find((a) => a.getAttribute("aria-current")) || { dataset: {} }).dataset.nav));
    addEventListener("keydown", (e) => {
      if (e.ctrlKey || e.metaKey || e.altKey || /^(INPUT|TEXTAREA|SELECT)$/.test((e.target && e.target.tagName) || "")) return;
      if (e.target && e.target.closest && e.target.closest(".iso-stage")) return; // com o foco numa figura, as teclas (setas, Home, End) são dela, não da página
      if (e.key === "]") { e.preventDefault(); ir(atual() + 1); } else if (e.key === "[") { e.preventDefault(); ir(atual() - 1); }
      else if (e.key === "Home") { e.preventDefault(); ir(0); } else if (e.key === "End") { e.preventDefault(); ir(secoes.length - 1); }
    });
    raiz.addEventListener("click", (e) => { if (e.target.closest("[data-acao=imprimir]")) print(); });
    L.fragmento.ligar(raiz); // abre na seção do fragmento da URL e o mantém em dia (biblioteca/fragmento.js)
  };

  L.casca = { html, montar, capaDe, secoesDe, secaoDaFigura };
})(L);
