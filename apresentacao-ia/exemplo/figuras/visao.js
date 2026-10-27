function (L, c) {
  // Figura mínima da Visão geral: um carrinho num trilho que vai e volta e a leitura da posição. Serve de ponto de partida: troque o objeto por um que explique o plano.
  const pt = L.idiomaDe(c && c.idioma) === "pt-BR";
  const W = 600, H = 380, plano = { x: -100, y: -50, w: 200, d: 100, r: 8 };
  const P = L.fitProjection([...L.boxCorners(plano, -17, 0), [0, 0, 40]], W, H, { pad: 30, azimuth: 42 });
  const corpo = L.pecas.base(P, plano, { regua: false }) + L.pecas.trilhos(P, { de: -90, ate: 90, ys: [-20, 15] })
    + `<g data-peca="carro">${L.solidSvg(L.slabOf({ x: -20, y: -18, w: 40, d: 36, r: 3 }, 3, 8, P, 5, 0.6), { tone: "hi", lit: true })}</g>`;
  const svg = L.figureSvg({ width: W, height: H, label: pt ? "Um carrinho num trilho, sobre uma mesa" : "A sled on a rail, on a surface plate", body: corpo });
  return { svg, vivo(palco, dizer) {
    const g = palco.querySelector('[data-peca="carro"]'), s = { x: 0, v: 0 };
    let t = 0;
    L.laco(palco, (dt, calmo) => {
      t += dt;
      L.mola(s, Math.sin(t * 0.8) * 60, dt, 120, 14, calmo);
      g.setAttribute("transform", L.translateAlong(P, [s.x, 0, 0]));
      dizer((pt ? "posição " : "position ") + s.x.toFixed(0));
      return true;
    });
  } };
}
