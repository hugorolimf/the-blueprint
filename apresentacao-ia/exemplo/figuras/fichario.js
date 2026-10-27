function (L, c) {
  // Fichário de cadastro: 14 fichas em pé numa bandeja, um leitor que desliza no trilho da frente e a ficha escolhida que sobe.
  // Cada aba da ficha diz a situação (em dia, pendente, atrasada); o mostrador conta a posição. Dados de exemplo, nenhum real.
  const pt = L.idiomaDe(c && c.idioma) === "pt-BR";
  const W = 660, H = 470;
  const N = 14, PITCH = 17.4, X0 = -118, ESP = 3, ALTO = 50, Z0 = 3, LIFT = 12, FRENTE = 18;
  const PLANO = { x: -150, y: -56, w: 320, d: 112, r: 8 }, BANDEJA = { x: -132, y: -44, w: 264, d: 62, r: 3 };
  const centro = (i) => X0 + i * PITCH + ESP / 2;
  const REACH = [centro(0), centro(N - 1)];
  const situacao = (i) => (i % 5 === 2 ? 2 : i % 4 === 0 ? 1 : 0);
  const ABA_Y = [-33, -15, -1];
  const textoSit = pt ? ["em dia", "pendente", "atrasada"] : ["on time", "pending", "overdue"];
  const abertas = [0, 1, 2];

  const P = L.fitProjection([...L.boxCorners(PLANO, -17, 0), [-132, -44, 82], [170, 56, 82]], W, H, { pad: 14, azimuth: 36 });
  const S = (p, e) => L.solidSvg(p, e), Ln = (d, e) => L.lineSvg(d, e), D = (pts, e) => L.dotsSvg(pts, e), at = (p) => L.iso(p, P);

  // bandeja: fundo, parede de trás e do fundo esquerdo; a da frente e a da direita vêm depois das fichas
  const bandeja = [
    S(L.slabOf(BANDEJA, 0, Z0, P, 4, 0.5), { tone: "mid" }),
    S(L.slabOf({ x: BANDEJA.x, y: BANDEJA.y, w: BANDEJA.w, d: 3, r: 0.6 }, Z0, 18, P, 2, 0.4), { tone: "mid" }),
    S(L.slabOf({ x: BANDEJA.x, y: BANDEJA.y, w: 3, d: BANDEJA.d, r: 0.6 }, Z0, 18, P, 2, 0.4), { tone: "mid" }),
  ].join("");
  const frente = [
    S(L.slabOf({ x: BANDEJA.x, y: BANDEJA.y + BANDEJA.d - 3, w: BANDEJA.w, d: 3, r: 0.6 }, Z0, 18, P, 2, 0.4), { tone: "mid" }),
    S(L.slabOf({ x: BANDEJA.x + BANDEJA.w - 3, y: BANDEJA.y, w: 3, d: BANDEJA.d, r: 0.6 }, Z0, 18, P, 2, 0.4), { tone: "mid" }),
    D([at([BANDEJA.x + 30, BANDEJA.y + BANDEJA.d, 18]), at([BANDEJA.x + BANDEJA.w - 30, BANDEJA.y + BANDEJA.d, 18])], { size: 0.45 }),
    Ln(L.lineOnTop([BANDEJA.x, BANDEJA.y + BANDEJA.d - 1.5], [BANDEJA.x + BANDEJA.w - 3, BANDEJA.y + BANDEJA.d - 1.5], Z0 + 18, P), { tone: "faint" }),
  ].join("");

  const fichas = Array.from({ length: N }, (_, i) =>
    `<g data-i="${i}">${L.pecas.ficha(P, { x: X0 + i * PITCH, y0: -37, y1: 11, z: Z0, alto: ALTO, esp: ESP, aba: { y: ABA_Y[situacao(i)], w: 12, h: 4 }, linhas: 5 })}</g>`).join("");

  // leitor: bloco no trilho da frente, coluna, braço sobre as fichas e um pino de luz sobre a escolhida. Desenhado em x = 0 e deslocado.
  const leitor = [
    S(L.slabOf({ x: -14, y: 20, w: 28, d: 22, r: 3 }, Z0, 8, P, 4, 0.6), { tone: "mid" }),
    D([-8, 8].map((x) => at([x, 40, Z0 + 8])), { size: 0.4, tone: "lo" }),
    Ln(L.lineOnTop([-14, 31], [14, 31], Z0 + 8, P), { tone: "faint" }),
    S(L.cylinder(0, 31, 3.4, Z0 + 8, 63, P, 24), { tone: "mid", crease: "none" }),
    Ln(L.sideArc(0, 31, 3.4, 40, P, 12), { tone: "faint" }),
    S(L.slabOf({ x: -3, y: -4, w: 6, d: 41, r: 1.5 }, 72, 5, P, 3, 0.4), { tone: "mid" }),
    D([at([0, 33, 77]), at([0, -1, 77])], { size: 0.45 }),
    S(L.cylinder(0, 0, 1.8, 67, 5, P, 16), { tone: "hi", crease: "none" }),
    L.pecas.botao(P, { x: 8, y: 46, z: Z0, r: 3, h: 5 }),
  ].join("");

  const mostr = L.pecas.mostrador(P, { x: 152, y: 36, z: 0, r: 14, h: 8 });
  const dial = S(L.cylinder(152, 36, 3, 0, 1, P, 20), { tone: "lo" }) + mostr.svg;

  const corpo = [
    L.pecas.base(P, PLANO),
    L.pecas.placa(P, { x: 139, y: -26, w: 24, d: 12 }),
    bandeja, fichas, frente,
    L.pecas.trilhos(P, { de: -136, ate: 126, ys: [22, 33], d: 5 }),
    dial,
    `<g data-peca="agulha">${Ln(mostr.agulha(0.1), { tone: "hi", free: "start" })}</g>`,
    `<g data-peca="leitor" transform="${L.translateAlong(P, [REACH[0], 0, 0])}">${leitor}</g>`,
  ].join("");

  const svg = L.figureSvg({
    width: W, height: H,
    label: pt ? "Um fichário de 14 fichas numa bandeja. Um leitor desliza no trilho da frente e a ficha escolhida sobe, mostrando a situação do cadastro pela aba."
              : "A card index of 14 cards in a tray. A reader slides on the front rail and the chosen card rises, showing the record status on its tab.",
    body: corpo,
  });

  return {
    svg,
    vivo(palco, dizer, ponte) {
      const raiz = palco.querySelector("svg"), leitorEl = raiz.querySelector('[data-peca="leitor"]'), agulha = raiz.querySelector('[data-peca="agulha"] path');
      const els = [...raiz.querySelectorAll("[data-i]")], corpos = els.map((g) => g.querySelector(".iso-solid"));
      const lift = els.map(() => ({ x: 0, v: 0 }));
      const sx = (i) => L.iso([centro(i), -13, 30], P)[0], sx0 = sx(0), sx1 = sx(N - 1);
      let alvo = 0, atual = 0, x = REACH[0], interagiu = false, relogio = 0, ultimaLeitura = -1, ocioso = !L.calmo(), esperaVolta = 0;
      const PASSOS = [0, 4, 9, 13, 7, 2];
      // a ponte com o texto (só se a figura está numa seção com itens): o item i do texto aponta uma ficha e a ficha atual marca o seu item
      const T = ponte ? ponte.total : 0;
      const cartaoDoItem = (i) => (T > 1 ? Math.round((i * (N - 1)) / (T - 1)) : 0), itemDoCartao = (c) => Math.min(T - 1, Math.floor((c * T) / N));
      const leitura = (i) => `${pt ? "ficha" : "card"} ${String(i + 1).padStart(2, "0")} · ${textoSit[situacao(i)]} · ${abertas[situacao(i)]} ${pt ? "em aberto" : "open"}`;
      const laco = L.laco(palco, (dt, calmo) => {
        if (ocioso && !calmo) { relogio += dt; alvo = PASSOS[Math.floor(relogio / 2.4) % PASSOS.length]; }
        x = L.segue(x, centro(alvo), dt, 0.28, calmo);
        let movendo = Math.abs(x - centro(alvo)) > 0.02;
        if (!movendo) x = centro(alvo);
        leitorEl.setAttribute("transform", L.translateAlong(P, [x, 0, 0]));
        lift.forEach((m, i) => {
          const antes = m.x, mexe = L.mola(m, i === alvo ? LIFT : 0, dt, 360, 24, calmo);
          if (m.x !== antes) els[i].setAttribute("transform", m.x ? L.translateAlong(P, [0, 0, m.x]) : "");
          movendo = movendo || mexe;
        });
        if (alvo !== atual) { corpos[atual].removeAttribute("data-lit"); atual = alvo; corpos[atual].setAttribute("data-lit", ""); if (T) ponte.mudou(itemDoCartao(atual)); }
        agulha.setAttribute("d", mostr.agulha(0.08 + 0.74 * (atual / (N - 1))));
        if (atual !== ultimaLeitura) { ultimaLeitura = atual; dizer(leitura(atual)); }
        return movendo || (ocioso && !calmo);
      });
      const escolhe = (i) => {
        ocioso = false; alvo = Math.max(0, Math.min(N - 1, i));
        clearTimeout(esperaVolta); esperaVolta = setTimeout(() => { ocioso = !L.calmo(); relogio = 0; laco.acordar(); }, 3600);
        laco.acordar();
      };
      if (T) ponte.liga({ selecionar(i) {
        clearTimeout(esperaVolta);
        if (i == null) { ocioso = !L.calmo(); relogio = 0; } else { ocioso = false; alvo = cartaoDoItem(i); } // sem item: volta ao passeio; com item: o passeio espera
        laco.acordar();
      } });
      palco.addEventListener("pointermove", (e) => {
        const caixa = raiz.getBoundingClientRect(), vx = ((e.clientX - caixa.left) / caixa.width) * W;
        escolhe(Math.round(((vx - sx0) / (sx1 - sx0)) * (N - 1)));
      });
      palco.addEventListener("pointerleave", () => escolhe(alvo));
      palco.addEventListener("keydown", (e) => {
        const passo = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
        if (passo) { e.preventDefault(); escolhe((ocioso ? atual : alvo) + passo * (e.shiftKey ? 3 : 1)); }
        else if (e.key === "Home") { e.preventDefault(); escolhe(0); } else if (e.key === "End") { e.preventDefault(); escolhe(N - 1); }
      });
      corpos[0].setAttribute("data-lit", "");
      if (T) ponte.mudou(itemDoCartao(0));
      agulha.setAttribute("d", mostr.agulha(0.08));
      dizer(leitura(0));
    },
  };
}
