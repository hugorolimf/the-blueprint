// Anima a barra da ação (contorno, hachura/progresso, losangos dos checkpoints, selo ✓ e cota de dias) ao abrir a folha e ao expandir/recolher.
// Com `reverso`, toca o inverso (as peças somem na ordem contrária, em metade do tempo) e devolve uma Promise que resolve no fim.
// Só o que está na janela, uma animação curta por peça; não roda nada em repouso.
const MAX_BARRAS = 40, MAX_MOVIDOS = 220; // tetos: acima disso o resto aparece/muda sem animação
const TOTAL_BARRAS = 1550; // fim da última animação direta (ms): o inverso espelha os tempos a partir dele
function desenharBarras(alvo = host.querySelector("svg.desenho"), reverso = false) {
  const fins = [];
  if (!alvo || reduz()) return Promise.resolve();
  const ease = "cubic-bezier(.45,.05,.2,1)";
  const mover = (el, quadros, dur, atraso, ini = 0) => {
    const opc = reverso ? { duration: dur / 2, delay: (TOTAL_BARRAS - ini - atraso - dur) / 2, direction: "reverse", fill: "both", easing: ease } : { duration: dur, delay: ini + atraso, easing: ease, fill: "backwards" };
    fins.push(el.animate(quadros, opc).finished.catch(() => {}));
  };
  const regua = alvo.querySelector(".regua"); // barra de datas: o eixo corre da esquerda, as guias descem, a linha de hoje desce e a etiqueta HOJE aparece por último
  if (regua) {
    const eixo = regua.querySelector(":scope > path.ln"), guias = [...regua.querySelectorAll(":scope > .guia")];
    if (eixo) mover(eixo, [{ transform: "scaleX(0)" }, { transform: "scaleX(1)" }], 500, 0);
    guias.forEach((g, i) => mover(g, [{ opacity: 0, transform: "scaleY(0)" }, { opacity: 1, transform: "scaleY(1)" }], 450, 150 + 350 * (i / Math.max(1, guias.length - 1))));
    const hoje = regua.querySelector(":scope > .eixo"), chip = regua.querySelector(":scope > .hoje-chip");
    if (hoje) mover(hoje, [{ transform: "scaleY(0)" }, { transform: "scaleY(1)" }], 500, 650);
    if (chip) mover(chip, [{ opacity: 0, transform: "scale(.6)" }, { opacity: 1, transform: "scale(1)" }], 280, 1000);
  }
  // duas fases: primeiro só lê as posições (sem animação criada no meio, que forçaria recalcular o layout a cada leitura), depois cria as animações
  const lote = [];
  for (const a of alvo.querySelectorAll(".acao")) {
    if (lote.length >= MAX_BARRAS) break;
    const r = a.getBoundingClientRect();
    if (!r.width || r.bottom < 0 || r.top > innerHeight) continue;
    const caixa = a.querySelector(".caixa"), cr = caixa?.getBoundingClientRect();
    lote.push({ a, caixa, ini: Math.max(r.top, 0) / innerHeight * 450, marcas: [...a.querySelectorAll(".dm")].map((el) => { const m = el.getBoundingClientRect(); return [el, cr?.width ? Math.min(1, Math.max(0, (m.left + m.width / 2 - cr.left) / cr.width)) : 0.5]; }) }); // ini: de cima para baixo; f: onde o losango está na barra
  }
  for (const { a, caixa, ini, marcas } of lote) {
    const anima = (el, quadros, dur, atraso) => mover(el, quadros, dur, atraso, ini);
    if (caixa) anima(caixa, a.dataset.st === "planejado" ? [{ opacity: 0 }, { opacity: 1 }] : [{ strokeDasharray: "1", strokeDashoffset: 1 }, { strokeDasharray: "1", strokeDashoffset: 0 }], 500, 0); // a caixa tem pathLength=1: o traço se desenha
    a.querySelectorAll(".hach, .prog").forEach((el) => anima(el, [{ clipPath: "inset(0 100% 0 0)" }, { clipPath: "inset(0 0 0 0)" }], 600, 250));
    marcas.forEach(([el, f]) => anima(el, [{ opacity: 0, transform: "scale(.2)" }, { opacity: 1, transform: "scale(1)" }], 260, 250 + 600 * f)); // surge quando a hachura passa por ele
    a.querySelectorAll(".selo").forEach((el) => anima(el, [{ opacity: 0, transform: "scale(.2)" }, { opacity: 1, transform: "scale(1)" }], 300, 800));
    a.querySelectorAll(".cota").forEach((el) => anima(el, [{ clipPath: "inset(0 100% 0 0)" }, { clipPath: "inset(0 0 0 0)" }], 500, 450));
  }
  return Promise.all(fins);
}
// a folha se recolhe: barras e textos somem (o inverso da abertura) e a Promise resolve quando terminou
function recolherFolha(alvo = host.querySelector("svg.desenho")) {
  if (!alvo || reduz()) return Promise.resolve();
  const fins = [desenharBarras(alvo, true)];
  const lidos = [];
  for (const el of alvo.querySelectorAll("text")) {
    if (lidos.length >= MAX_TEXTOS) break;
    const r = el.getBoundingClientRect();
    if (r.width && r.bottom >= 0 && r.top <= innerHeight) lidos.push([el, Math.max(r.top, 0)]);
  }
  for (const [el, topo] of lidos) fins.push(el.animate([{ fillOpacity: 1, strokeOpacity: 1 }, { fillOpacity: 0, strokeOpacity: 0 }], { duration: 200, delay: (1 - topo / innerHeight) * 250, fill: "both", easing: "ease-in" }).finished.catch(() => {})); // de baixo para cima; fill/stroke-opacity: o texto tem contorno, e opacity criaria uma camada por texto
  return Promise.all(fins);
}
// reorganização: antes da troca de período/filtro guarda a geometria de cada ação (peça por peça) e de cada plano/checkpoint da lista; depois do redesenho,
// o retângulo da barra muda de x e largura, e as demais peças deslizam da posição antiga para a nova (o que sumiu fica de fora; o novo só se escreve)
const ITENS_FOLHA = ".acao[data-f], .chk[data-id]", chaveFolha = (el) => (el.dataset.id ? "i:" + el.dataset.id : "a:" + el.dataset.f);
const medidaFolha = (el) => { const r = el.getBoundingClientRect(); return { tag: el.tagName, x: r.left, y: r.top, ax: el.getAttribute("x"), aw: el.getAttribute("width"), cls: el.getAttribute("class") || "" }; };
function posicoesDaFolha(alvo = host.querySelector("svg.desenho")) {
  const mapa = new Map();
  if (!alvo || reduz()) return mapa;
  alvo.querySelectorAll(ITENS_FOLHA).forEach((el) => {
    if (el.getBoundingClientRect().width) mapa.set(chaveFolha(el), { ...medidaFolha(el), filhos: el.dataset.id ? [] : [...el.children].map(medidaFolha) });
  });
  return mapa;
}
function organizarFolha(antes, alvo = host.querySelector("svg.desenho"), duracao = 650) {
  if (!alvo || !antes?.size || reduz()) return;
  const ease = "cubic-bezier(.45,.05,.2,1)", plano = []; // fase 1 (só leitura) monta o plano; fase 2 cria as animações
  const desliza = (el, velho) => {
    if (plano.length >= MAX_MOVIDOS) return;
    const r = el.getBoundingClientRect(), dx = velho.x - r.left, dy = velho.y - r.top;
    if (Math.abs(dx) < 1 && Math.abs(dy) < 1) return;
    if (Math.max(r.bottom, velho.y + r.height) < 0 || Math.min(r.top, velho.y) > innerHeight) return; // fora da janela nos dois lugares: não precisa
    plano.push([el, [{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "none" }]]);
  };
  for (const el of alvo.querySelectorAll(ITENS_FOLHA)) {
    const velho = antes.get(chaveFolha(el));
    if (!velho) continue;
    const filhos = [...el.children];
    if (el.dataset.id || filhos.length !== velho.filhos.length) { // plano/checkpoint da lista, ou ação com outra estrutura (ex.: ganhou o selo ✓): o conjunto desliza junto
      desliza(el, velho);
      const fill = !el.dataset.id && el.querySelector(":scope > .prog, :scope > .hach"); // o progresso da barra cresce do que era (ou de zero) até o novo
      if (fill) {
        const de = Number(velho.filhos.find((c) => /\b(prog|hach)\b/.test(c.cls))?.aw) || 0, ate = Number(fill.getAttribute("width"));
        if (de !== ate) plano.push([fill, [{ width: de + "px" }, { width: ate + "px" }]]);
      }
      continue;
    }
    filhos.forEach((f, k) => {
      const v = velho.filhos[k];
      if (plano.length >= MAX_MOVIDOS || v.tag !== f.tagName) return;
      const x = f.getAttribute("x"), w = f.getAttribute("width");
      if (f.tagName === "rect" && v.ax !== null && v.aw !== null && (v.ax !== x || v.aw !== w)) plano.push([f, [{ x: v.ax + "px", width: v.aw + "px" }, { x: x + "px", width: w + "px" }]]); // barra: cresce/encolhe e anda até o novo x
      else desliza(f, v);
    });
  }
  plano.forEach(([el, quadros]) => el.animate(quadros, { duration: duracao, easing: ease, fill: "backwards" }));
}
