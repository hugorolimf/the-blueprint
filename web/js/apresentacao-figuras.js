// Figuras da apresentação: o blueprint vira um objeto isométrico do anatomy (uma torre por projeto, um andar por ação) e os GIFs do sistema entram como data URI.
// Tudo dentro de uma função: os scripts clássicos dividem o escopo global e aqui só `window.figuraDoBlueprint`, `window.gifParaApresentacao` e `window.figurasProntas` saem.
(() => {
  const MAX_TORRES = 8, MAX_ANDARES = 8, LIMITE_PECAS = 160, MAX_CACHE = 24, GIF_KB_PADRAO = 1600;
  let emb = null;
  // o módulo do kit é o mesmo que a folha carrega (o navegador guarda um só); `EMB` é o da folha, se já existir
  let pronto = Promise.resolve(null);
  try { pronto = import("/emblemas.mjs").then((m) => (emb = m)).catch(() => null); } catch { /* sem módulos: a figura sai vazia */ }
  window.figurasProntas = pronto;
  const modulo = () => (typeof EMB !== "undefined" && EMB) || emb;
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

  const lista = (x) => (Array.isArray(x) ? x : []);
  const concluida = (a) => a?.status === "concluido" || (lista(a?.planos).length > 0 && lista(a.planos).every((p) => p?.feito) && lista(a.checkpoints).every((c) => c?.feito));
  const emAndamento = (a) => a?.status === "andamento" || lista(a?.planos).some((p) => p?.feito);
  const estadoDaAcao = (a) => (concluida(a) ? "feito" : emAndamento(a) ? "andamento" : "planejado");
  const estadoDoGrupo = (estados) => (estados.length && estados.every((e) => e === "feito") ? "feito" : estados.some((e) => e !== "planejado") ? "andamento" : "planejado");

  // Os andares de uma torre, na ordem da folha: ações de um projeto, ou os planos de uma ação. Passando de MAX_ANDARES, vários itens dividem um andar.
  function andares(estados) {
    const n = estados.length, m = Math.min(Math.max(n, 1), MAX_ANDARES);
    if (!n) return ["planejado"];
    return Array.from({ length: m }, (_, j) => estadoDoGrupo(estados.slice(Math.floor((j * n) / m), Math.floor(((j + 1) * n) / m))));
  }

  // Uma torre por projeto (as ações sem projeto formam a torre "Geral", por último); sem projetos, uma torre por ação, com um andar por plano.
  function torres(bp) {
    const acoes = lista(bp?.acoes), projetos = lista(bp?.projetos);
    let grupos;
    if (projetos.length) {
      const ids = new Set(projetos.map((p) => p?.id));
      grupos = projetos.map((p) => acoes.filter((a) => a?.projetoId === p?.id).map(estadoDaAcao));
      const soltas = acoes.filter((a) => !ids.has(a?.projetoId)).map(estadoDaAcao);
      if (soltas.length) grupos.push(soltas);
    } else {
      grupos = acoes.map((a) => (lista(a?.planos).length ? lista(a.planos).map((p) => (p?.feito ? "feito" : estadoDaAcao(a) === "andamento" ? "andamento" : "planejado")) : [estadoDaAcao(a)]));
    }
    if (grupos.length > MAX_TORRES) grupos.splice(MAX_TORRES - 1, grupos.length, grupos.slice(MAX_TORRES - 1).flat()); // as que passam do limite se juntam na última
    const ordem = { feito: 0, andamento: 1, planejado: 2 };
    return grupos.map((g) => andares(g).sort((a, b) => ordem[a] - ordem[b])); // o que está pronto fica embaixo: a torre é construída de baixo para cima
  }

  // A lista de peças, em ordem de construção: a mesa e os andares prontos primeiro, para o `progresso` pintar sólido exatamente o que está feito.
  function pecasDe(bp) {
    const ts = torres(bp), n = Math.max(ts.length, 1), LARG = 34, VAO = 14, tudo = n * LARG + (n - 1) * VAO, esq = -tudo / 2;
    const mesa = [
      { tipo: "laje", x: -tudo / 2 - 16, y: -32, w: tudo + 32, d: 64, z: 0, h: 5, r: 4, tom: "mid" },
      { tipo: "laje", x: -tudo / 2 - 12, y: -28, w: tudo + 24, d: 56, z: 5, h: 0.6, r: 3, tom: "lo" },
      ...Array.from({ length: n + 1 }, (_, i) => ({ tipo: "linha", x: esq - 7 + i * (LARG + VAO), y: -26, x2: esq - 7 + i * (LARG + VAO), y2: 26, z: 5.6, tom: "faint" })),
    ];
    const andaresDe = ts.flatMap((estados, i) => estados.map((e, j) => ({ e, peca: { tipo: "laje", x: esq + i * (LARG + VAO), y: -LARG / 2, w: LARG, d: LARG, z: 5.6 + j * 9, h: 7.4, r: 1, tom: e === "feito" ? "hi" : e === "andamento" ? "mid" : "lo" } })));
    const feitos = andaresDe.filter((a) => a.e === "feito"), resto = andaresDe.filter((a) => a.e !== "feito");
    const pecas = [...mesa, ...feitos.map((a) => a.peca), ...resto.map((a) => a.peca)];
    return { pecas, progresso: (mesa.length + feitos.length) / pecas.length, torres: ts.length, andares: andaresDe.length, feitos: feitos.length };
  }

  const cache = new Map();
  // Devolve o <svg> da figura (ou '' se o kit ainda não carregou). `rotulo`, já traduzido, vira o texto alternativo; sem ele a figura é decorativa.
  window.figuraDoBlueprint = (bp, { largura = 520, rotulo = "" } = {}) => {
    const m = modulo();
    if (!m || !bp) return "";
    const L = Math.min(1400, Math.max(160, Math.round(Number(largura) || 520)));
    const d = pecasDe(bp), chave = `${L}|${rotulo}|${JSON.stringify(d.pecas)}|${d.progresso}`;
    if (cache.has(chave)) return cache.get(chave);
    const e = m.desenhaEmblema(d.pecas, { largura: L, altura: Math.round(L * 0.58), progresso: d.progresso, limite: LIMITE_PECAS });
    const svg = e ? `<svg class="emblema marca-apresentacao" viewBox="0 0 ${e.largura} ${e.altura}" width="${e.largura}" height="${e.altura}" ${rotulo ? `role="img" aria-label="${esc(rotulo)}"` : 'aria-hidden="true"'}>${e.svg}</svg>` : "";
    if (cache.size >= MAX_CACHE) cache.delete(cache.keys().next().value);
    cache.set(chave, svg);
    return svg;
  };
  window.figuraDoBlueprint.pecas = pecasDe; // para os testes e para quem queira contar torres e andares

  // ---- GIFs do sistema -------------------------------------------------------------------------
  const bytesDoBase64 = (b64) => Math.floor((b64.length * 3) / 4) - (b64.endsWith("==") ? 2 : b64.endsWith("=") ? 1 : 0);
  const ehGif = (b64) => { try { return /^GIF8[79]a$/.test(atob(b64.slice(0, 8))); } catch { return false; } };
  const paraBase64 = (bytes) => { let s = ""; for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000)); return btoa(s); };
  // Devolve (numa Promise) um data URI de GIF ou null. Recusa o que não é GIF de verdade (confere a assinatura, não o nome), o que passa de maxKb e endereços de outra origem.
  window.gifParaApresentacao = async (origem, { maxKb = GIF_KB_PADRAO } = {}) => {
    const limite = Math.max(1, Number(maxKb) || GIF_KB_PADRAO) * 1024, texto = String(origem ?? "").trim();
    const dados = /^data:image\/gif;base64,([A-Za-z0-9+/]+={0,2})$/.exec(texto);
    if (dados) return bytesDoBase64(dados[1]) <= limite && ehGif(dados[1]) ? texto : null;
    try {
      const url = new URL(texto, location.href);
      if (url.protocol !== "blob:" && url.origin !== location.origin) return null; // nada de rede de fora: o app roda isolado
      const r = await fetch(url.href);
      if (!r.ok || Number(r.headers.get("content-length")) > limite) return null;
      const bytes = new Uint8Array(await r.arrayBuffer());
      if (bytes.length > limite) return null;
      const b64 = paraBase64(bytes);
      return ehGif(b64) ? `data:image/gif;base64,${b64}` : null;
    } catch { return null; }
  };
})();
