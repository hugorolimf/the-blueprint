// A ponte entre o texto e a figura de uma seção: o texto faz referência à figura e as duas se comandam.
// Cada item de texto tem data-item (índice a partir de 0, na ordem do conteúdo), data-sessao e tabindex="0". Grupos (o `secao` da figura): recebe (entregas), etapas e linha
// (etapas), precisamos (precisamosDeVoce), decisoes e riscos. A figura recebe `ponte` como 3º argumento de vivo(palco, dizer, ponte) (pode faltar: sempre `if (ponte)`):
//   ponte.total              quantos itens a seção tem (0 se a figura não é de uma seção com itens)
//   ponte.liga({ selecionar(i) })   registra como a figura seleciona o item i; a página chama selecionar(i) quando o mouse entra ou o foco chega
//                            num item do texto, e selecionar(null) quando sai (a figura volta ao seu passeio); devolve uma função que desfaz o registro
//   ponte.mudou(i | null)    a figura avisa que o item atual mudou (por mouse, teclas ou passeio): a página marca o item (data-ativo, aria-current) e limpa o anterior
// Toque: o toque num item o seleciona (e o foco que ele recebe também); toque fora o solta. Sem figura registrada nada quebra. Os ouvintes são delegados na raiz (um só conjunto,
// desfeito por desligar()) e nada roda em repouso: só os eventos e, ao sair de um item, um único adiamento de 0 ms que junta "saiu de um" com "entrou no outro".
(function (L) {
  const GRUPOS = { recebe: (c) => c.entregas, etapas: (c) => c.etapas, linha: (c) => c.etapas, precisamos: (c) => c.precisamosDeVoce, decisoes: (c) => c.decisoes, riscos: (c) => c.riscos };
  const ITEM = "[data-item][data-sessao]";
  const inteiro = (i) => (Number.isInteger(i) && i >= 0 ? i : null);

  const ligar = (raiz, conteudo) => {
    const itens = {}; // grupo -> elementos pelo índice
    for (const el of raiz.querySelectorAll(ITEM)) (itens[el.dataset.sessao] = itens[el.dataset.sessao] || [])[+el.dataset.item] = el;
    const totalDe = (g) => (GRUPOS[g] && Array.isArray(GRUPOS[g](conteudo || {})) ? GRUPOS[g](conteudo || {}).length : 0);
    const figuras = {}; // grupo -> conjunto de { selecionar }
    const marcado = {}; // grupo -> índice marcado

    const marca = (grupo, i) => {
      i = inteiro(i);
      const antes = marcado[grupo] == null ? null : marcado[grupo];
      if (antes === i) return;
      const lista = itens[grupo] || [], velho = antes == null ? null : lista[antes];
      if (velho) { velho.removeAttribute("data-ativo"); velho.removeAttribute("aria-current"); }
      marcado[grupo] = i;
      const novo = i == null ? null : lista[i];
      if (novo) { novo.setAttribute("data-ativo", "true"); novo.setAttribute("aria-current", "true"); }
    };
    const chama = (grupo, i) => { for (const f of [...(figuras[grupo] || [])]) { try { f.selecionar(i); } catch (e) { console.error(e); } } };

    let ativo = null, pendente = 0, tipo = "mouse";
    const seleciona = (el) => {
      clearTimeout(pendente);
      if (ativo && ativo.el === el) return;
      const grupo = el.dataset.sessao;
      if (ativo && ativo.grupo !== grupo) chama(ativo.grupo, null);
      ativo = { grupo, el };
      chama(grupo, +el.dataset.item);
    };
    const larga = () => { clearTimeout(pendente); if (!ativo) return; const { grupo } = ativo; ativo = null; chama(grupo, null); };
    const solta = () => { clearTimeout(pendente); pendente = setTimeout(larga, 0); }; // sair de um item e entrar noutro vira uma seleção só
    const item = (alvo) => (alvo && alvo.closest ? alvo.closest(ITEM) : null);
    const dentro = (el, alvo) => !!(el && alvo && el.contains && el.contains(alvo));

    const ouvintes = {
      pointerdown: (e) => { tipo = e.pointerType || "mouse"; },
      pointerover: (e) => { const el = e.pointerType === "touch" ? null : item(e.target); if (el) seleciona(el); },
      pointerout: (e) => { const el = e.pointerType === "touch" ? null : item(e.target); if (el && !dentro(el, e.relatedTarget)) solta(); },
      focusin: (e) => { const el = item(e.target); if (el) seleciona(el); },
      focusout: (e) => { const el = item(e.target); if (el && !dentro(el, e.relatedTarget)) solta(); },
      click: (e) => {
        const a = e.target && e.target.closest && e.target.closest("[data-ref]");
        if (a) { e.preventDefault(); irParaFigura(raiz, a.dataset.ref); return; }
        if (tipo === "mouse") return; // o mouse já seleciona ao passar; o toque (e a caneta) seleciona e solta pelo toque
        const el = item(e.target);
        if (el) seleciona(el); else larga();
      },
    };
    for (const [nome, f] of Object.entries(ouvintes)) raiz.addEventListener(nome, f);

    return {
      // a ponte de uma figura da seção `grupo` (qualquer outro valor: ponte sem itens, que não faz nada)
      criar(grupo) {
        const g = GRUPOS[grupo] ? grupo : null;
        let registro = null;
        const desfaz = () => { if (registro && g) (figuras[g] || new Set()).delete(registro); registro = null; };
        return {
          total: g ? totalDe(g) : 0,
          liga(obj) {
            desfaz();
            if (g && obj && typeof obj.selecionar === "function") { registro = obj; (figuras[g] = figuras[g] || new Set()).add(obj); }
            return desfaz;
          },
          mudou(i) { if (g) marca(g, i); },
        };
      },
      desligar() { clearTimeout(pendente); for (const [nome, f] of Object.entries(ouvintes)) raiz.removeEventListener(nome, f); },
    };
  };

  // o link "Fig n" da linha de referência: rola até o cartão da figura e põe o foco no palco dela
  const irParaFigura = (raiz, id) => {
    const palco = raiz.querySelector(`[data-figura="${CSS.escape(String(id))}"]`);
    if (!palco) return;
    (palco.closest(".ap-fig") || palco).scrollIntoView({ block: "center" });
    palco.focus({ preventScroll: true });
  };

  L.ponte = { GRUPOS: Object.keys(GRUPOS), ligar };
})(L);
