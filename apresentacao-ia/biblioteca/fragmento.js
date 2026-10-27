// A seção ativa e a URL: o editor do app recarrega o visor depois de cada alteração salva e quer voltar à mesma seção.
// Fragmentos aceitos (o primeiro é o que a página escreve): #visao  #entregas  #etapas  #etapas-3 (a 3ª etapa)  #linha  #voce  #decisoes  #acoes,
// também #s-<seção> (o id real das seções), #e-<id da etapa> e os nomes do conteúdo: #recebe (entregas), #precisamos (voce), #riscos (decisoes).
// Seção ou etapa que deixou de existir cai na mais próxima, sem erro; fragmento que não é nosso não faz nada.
// Do app para a página: window.postMessage({ tipo: "ap-ir", secao: "etapas" [, etapa: 3] }, "*") rola até lá. Da página para o app (quando há um pai):
// postMessage({ tipo: "ap-estado", secao, etapa (1 a n ou null), fragmento, via }) quando a seção visível muda; `via` diz como o fragmento foi gravado
// ("replaceState", "replace" ou "nenhum"; em iframe sandbox sem allow-same-origin o primeiro pode falhar e a página cai no segundo sem quebrar).
(function (L) {
  const ORDEM = ["visao", "entregas", "etapas", "linha", "voce", "decisoes", "acoes"];
  const ALIAS = { recebe: "entregas", precisamos: "voce", riscos: "decisoes" };
  const preso = (n, a, b) => Math.min(b, Math.max(a, n));
  const maisProxima = (nome, secoes) => {
    const i = ORDEM.indexOf(nome);
    return { secao: secoes.reduce((m, s) => (Math.abs(ORDEM.indexOf(s) - i) < Math.abs(ORDEM.indexOf(m) - i) ? s : m), secoes[0]) };
  };

  // fragmento -> { secao, etapa? } (etapa é índice a partir de 0) ou null
  const resolver = (fragmento, secoes, idsEtapas = []) => {
    const f = String(fragmento == null ? "" : fragmento).replace(/^#/, "").trim();
    if (!f || !secoes.length) return null;
    const porId = /^e-(.+)$/.exec(f);
    if (porId) {
      const i = idsEtapas.indexOf(porId[1]);
      return i >= 0 ? { secao: "etapas", etapa: i } : maisProxima("etapas", secoes);
    }
    const m = /^(?:s-)?([a-z]+)(?:-(\d+))?$/.exec(f.toLowerCase());
    const nome = m && (ALIAS[m[1]] || m[1]);
    if (!nome || !ORDEM.includes(nome)) return null;
    const alvo = secoes.includes(nome) ? { secao: nome } : maisProxima(nome, secoes);
    if (alvo.secao === "etapas" && m[2] && idsEtapas.length) alvo.etapa = preso(+m[2], 1, idsEtapas.length) - 1;
    return alvo;
  };
  const texto = (r) => (r.etapa != null ? `etapas-${r.etapa + 1}` : r.secao);

  const ligar = (raiz) => {
    const secoes = [...raiz.querySelectorAll(".ap-sec[data-sec]")], etapas = [...raiz.querySelectorAll(".ap-etapa")];
    const ids = secoes.map((s) => s.dataset.sec), idsEtapas = etapas.map((e) => String(e.id).replace(/^e-/, ""));
    const elDe = (r) => (r.etapa != null && etapas[r.etapa]) || secoes[ids.indexOf(r.secao)];
    let escrito = null, gravado = null, livre = false, mexeu = false, adiar = 0, atual = null, manual = null;
    // `manual`: o alvo pedido (fragmento, ap-ir, menu) vale até o usuário mexer; a última seção, por exemplo, nunca chega ao topo e o observador diria a anterior
    const ir = (r, instantaneo) => {
      const el = r && elDe(r);
      if (!el) return;
      manual = r;
      clearTimeout(adiar);
      adiar = setTimeout(() => publica(r), 200);
      el.scrollIntoView({ block: "start", behavior: instantaneo ? "instant" : "auto" });
    };

    const escreve = (f) => {
      escrito = f;
      try { history.replaceState(null, "", "#" + f); return "replaceState"; } catch (e) { /* sandbox: tenta o outro */ }
      try { location.replace("#" + f); return "replace"; } catch (e) { return "nenhum"; }
    };
    const publica = (r) => {
      const f = texto(r);
      if (!livre || f === gravado) return;
      gravado = f;
      const via = escreve(f);
      try { if (window.parent && window.parent !== window) window.parent.postMessage({ tipo: "ap-estado", secao: r.secao, etapa: r.etapa == null ? null : r.etapa + 1, fragmento: f, via }, "*"); } catch (e) { /* sem pai */ }
    };

    // a seção (e a etapa) à vista: a primeira de cada lista, no mesmo recorte do menu
    if (typeof IntersectionObserver === "function") {
      const sS = new Set(), sE = new Set();
      const io = new IntersectionObserver((es) => {
        for (const e of es) (e.target.classList.contains("ap-sec") ? sS : sE)[e.isIntersecting ? "add" : "delete"](e.target);
        const sec = secoes.find((s) => sS.has(s));
        if (!sec) return;
        atual = { secao: sec.dataset.sec };
        const i = etapas.findIndex((x) => sE.has(x));
        if (atual.secao === "etapas" && i >= 0) atual.etapa = i;
        if (manual) return;
        clearTimeout(adiar);
        adiar = setTimeout(() => publica(atual), 150); // a rolagem suave passa por várias seções: só a que ficou conta
      }, { rootMargin: "-20% 0px -65% 0px" });
      [...secoes, ...etapas].forEach((x) => io.observe(x));
    }

    // abrir já na seção do fragmento (sem rolagem suave) e conferir de novo quando as figuras acabarem de mudar o layout
    const inicial = resolver(location.hash, ids, idsEtapas);
    const solta = () => { livre = true; if (manual || atual) publica(manual || atual); };
    ["wheel", "touchstart", "keydown", "pointerdown"].forEach((ev) => addEventListener(ev, () => { mexeu = true; manual = null; }, { passive: true }));
    if (inicial) {
      ir(inicial, true);
      requestAnimationFrame(() => ir(inicial, true));
      setTimeout(() => { if (!mexeu) ir(inicial, true); solta(); }, 350);
    } else setTimeout(solta, 350);

    addEventListener("hashchange", () => { if (location.hash.slice(1) !== escrito) ir(resolver(location.hash, ids, idsEtapas), false); });
    addEventListener("message", (e) => {
      const d = e.data;
      if (!d || typeof d !== "object" || d.tipo !== "ap-ir" || typeof d.secao !== "string") return;
      if (e.source !== window.parent && e.source !== window) return;
      ir(resolver(d.secao + (Number.isInteger(d.etapa) && !/-\d+$/.test(d.secao) ? "-" + d.etapa : ""), ids, idsEtapas), false);
    });
    // os links do menu não empilham histórico no iframe do app: rolam e gravam o fragmento no lugar
    raiz.addEventListener("click", (e) => {
      const a = e.target.closest("[data-nav]");
      if (!a) return;
      e.preventDefault();
      ir({ secao: a.dataset.nav }, false);
    });
  };

  L.fragmento = { ORDEM, ALIAS, resolver, texto, ligar };
})(L);
