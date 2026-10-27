// Peças reutilizáveis do anatomy: cada uma devolve um trecho de SVG (ou { svg, ... }), desenhado uma vez no ponto em que a peça está parada.
// Todas recebem a projeção P (L.fitProjection) e medidas no mundo do kit: x e y no chão, z para cima, 1 unidade ~ 1 px.
// O que se move sai em um <g data-peca="nome"> e a figura o desloca com L.translateAlong; nada aqui anima.
(function (L) {
  const { slabOf, cylinder, solidSvg, lineSvg, dotsSvg, iso, planOutline, insetPlan, corners, ring, haloOf, topTicks, knurl, radialTicks, lineOnTop, segment, depthOf } = L;
  const sol = (paths, estilo) => solidSvg(paths, estilo);
  const porProfundidade = (P, pontos) => [...pontos].sort((a, b) => depthOf([a[0], a[1], 0], P) - depthOf([b[0], b[1], 0], P));

  // Base: placa com quatro pés, sulco, parafusos nos cantos e uma régua com marcas. O topo da placa fica em z = 0.
  const base = (P, plano, { h = 12, pe = 5, regua = true } = {}) => {
    const out = [`<path class="iso-halo" d="${haloOf(plano, -h - pe, h, P)}"/>`];
    for (const [x, y] of porProfundidade(P, corners(plano, 22))) {
      out.push(sol(cylinder(x, y, 7, -h - pe, 1.6, P, 28), { tone: "lo" }), sol(cylinder(x, y, 5.4, -h - pe + 1.6, pe - 1.6, P, 28), { tone: "mid" }));
    }
    out.push(sol(slabOf(plano, -h, h, P, 8, 1.6), { tone: "mid" }), lineSvg(planOutline(insetPlan(plano, 5), 0, P), { tone: "lo" }));
    const parafusos = corners(plano, 10);
    out.push(lineSvg(parafusos.map(([x, y]) => ring(x, y, 2.2, 0, P, 16)).join(""), { tone: "lo" }), dotsSvg(parafusos.map(([x, y]) => iso([x, y, 0], P)), { size: 0.5 }));
    if (regua) {
      const faixa = { x: plano.x + 18, y: plano.y + plano.d - 15, w: plano.w - 36, d: 7, r: 1 };
      out.push(sol(slabOf(faixa, 0, 0.6, P, 3), { tone: "lo", crease: "none" }));
      const marcas = topTicks(faixa.x + 4, faixa.x + faixa.w - 4, 4, 5, faixa.y + 0.8, 0.6, [2, 3.6], P);
      out.push(lineSvg(marcas.minor, { tone: "lo" }), lineSvg(marcas.major, { tone: "mid" }));
      out.push(dotsSvg([iso([faixa.x + 2, faixa.y + 3.5, 0.6], P), iso([faixa.x + faixa.w - 2, faixa.y + 3.5, 0.6], P)], { size: 0.45 }));
    }
    return out.join("");
  };

  // Dois trilhos ao longo de x (em cada y da lista) com batentes nas pontas, sobre z = 0.
  const trilhos = (P, { de, ate, ys, d = 5, h = 3, topes = true }) => {
    const out = [];
    for (const y of ys) {
      out.push(sol(slabOf({ x: de, y, w: ate - de, d, r: 1 }, 0, h, P, 3, 0.4), { tone: "mid" }));
      out.push(dotsSvg([de + 6, ate - 6].map((x) => iso([x, y + d / 2, h], P)), { size: 0.4 }));
    }
    if (topes) {
      const y0 = Math.min(...ys) - 3, y1 = Math.max(...ys) + d + 3;
      for (const x of [de - 5, ate]) {
        out.push(sol(slabOf({ x, y: y0, w: 5, d: y1 - y0, r: 1.2 }, 0, 7, P, 3, 0.5), { tone: "mid" }));
        out.push(dotsSvg([iso([x + 2.5, y0 + 5, 7], P), iso([x + 2.5, y1 - 5, 7], P)], { size: 0.45 }));
      }
    }
    return out.join("");
  };

  // Ficha em pé: lâmina fina em x (a face larga olha para +x), com linhas de texto e uma aba no topo. Devolve o <g> da ficha.
  // A peça "lit" é ligada de fora: el.querySelector(".iso-solid").toggleAttribute("data-lit").
  const ficha = (P, { x, y0, y1, z = 0, alto, esp = 3, aba = null, linhas = 5 }) => {
    const out = [sol(slabOf({ x, y: y0, w: esp, d: y1 - y0, r: 0.6 }, z, alto, P, 2, 0.3), { tone: "hi" })];
    const xf = x + esp, passo = (alto - 16) / Math.max(linhas, 1);
    for (let i = 0; i < linhas; i++) {
      const za = z + alto - 9 - i * passo, fim = i === 0 ? y1 - 6 : y1 - 6 - ((i * 7) % 9);
      out.push(lineSvg(segment([xf, y0 + 5, za], [xf, fim, za], P), { tone: i === 0 ? "mid" : "faint", free: true }));
    }
    if (aba) {
      out.push(sol(slabOf({ x, y: aba.y, w: esp, d: aba.w, r: 0.4 }, z + alto, aba.h, P, 2), { tone: "mid" }));
      out.push(dotsSvg([iso([xf, aba.y + aba.w / 2, z + alto + aba.h / 2], P)], { size: 0.45, tone: "hi" }));
    }
    return `<g data-peca="ficha">${out.join("")}</g>`;
  };

  // Botão serrilhado (cilindro com ranhuras e um risco no topo).
  const botao = (P, { x, y, z = 0, r = 6, h = 4 }) => [
    sol(cylinder(x, y, r, z, h, P, 32), { tone: "mid" }),
    lineSvg(knurl(x, y, r, z, z + h, 36, P), { tone: "lo" }),
    lineSvg(lineOnTop([x, y], [x + r, y], z + h, P), { tone: "hi" }),
    dotsSvg([iso([x, y, z + h], P)], { size: 0.5, tone: "hi" }),
  ].join("");

  // Mostrador: caixa redonda com anel, marcas radiais e uma agulha. agulha(volta) devolve o `d` da agulha para volta em [0, 1) e a figura o grava.
  const mostrador = (P, { x, y, z = 0, r = 20, h = 10 }) => {
    const topo = z + h, out = [sol(cylinder(x, y, r, z, h, P, 64, 0.8), { tone: "hi", crease: "lo" })];
    out.push(lineSvg(knurl(x, y, r, topo - 3.4, topo, 84, P, 0.3), { tone: "lo" }), lineSvg(ring(x, y, r - 2.2, topo, P, 64), { tone: "lo" }));
    const marcas = radialTicks(x, y, r - 3, 50, 5, topo, [1.3, 2.8], P);
    out.push(lineSvg(marcas.minor, { tone: "lo" }), lineSvg(marcas.major, { tone: "mid" }), lineSvg(ring(x, y, 1.6, topo, P, 16), { tone: "mid" }));
    const agulha = (volta) => {
      const a = volta * Math.PI * 2;
      return lineOnTop([x - Math.sin(a) * 3.4, y + Math.cos(a) * 3.4], [x + Math.sin(a) * (r - 4), y - Math.cos(a) * (r - 4)], topo + 0.2, P);
    };
    return { svg: out.join(""), agulha, topo };
  };

  // Plaquinha de identificação: lâmina baixa com três linhas de regra e dois pontos.
  const placa = (P, { x, y, w = 24, d = 10, z = 0 }) => [
    sol(slabOf({ x, y, w, d, r: 1 }, z, 1.2, P, 3), { tone: "lo", crease: "none" }),
    ...[0.28, 0.5, 0.72].map((t, i) => lineSvg(lineOnTop([x + 3, y + d * t], [x + w - 3 - i * 4, y + d * t], z + 1.2, P), { tone: "faint", free: true })),
    dotsSvg([iso([x + 1.6, y + 1.6, z + 1.2], P), iso([x + w - 1.6, y + d - 1.6, z + 1.2], P)], { size: 0.4 }),
  ].join("");

  L.pecas = { base, trilhos, ficha, botao, mostrador, placa };
})(L);
