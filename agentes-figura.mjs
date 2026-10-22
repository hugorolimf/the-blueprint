// Figura de um agente: uma ESTAÇÃO DE TRABALHO isométrica, só dados (lista de peças para desenhaEmblema), pura e determinística.
// O que move e o que significa: o carro corre no trilho ao longo da régua (posicao = qual arquivo/área); a lança sobe e desce
// (estado); a ferramenta na ponta muda com o papel; a barra de segmentos e o mostrador mostram o progresso; a lâmpada e a bandeja contam o resto.
// A base é igual em tudo e o enquadramento não muda (as extremidades são sempre as mesmas peças).
// Cada peça leva `grupo`: quem monta o painel desenha grupos que não mudam uma vez só e desliza o que depende de posicao.
export const ESTADOS = ["ocioso", "lendo", "escrevendo", "testando", "aguardando", "bloqueado", "concluido"];
export const PAPEIS = ["explorador", "construtor", "revisor"];
// ordem de pintura dos grupos; MOVEIS acompanham a posicao do carro (deslizam juntos), os demais ficam parados
export const GRUPOS = ["base", "etiqueta", "torre", "mostrador", "agulha", "enchimento", "cartas", "frente", "alvo", "trava", "carro", "cabo", "roda", "raios", "cubo", "braco"];
export const MOVEIS = new Set(["alvo", "trava", "carro", "roda", "raios", "cubo", "braco"]);
// o cabo do carro até o painel muda de comprimento com a posição: não desliza como os MOVEIS, é redesenhado a cada posição
export const REDESENHADOS = new Set(["cabo"]);
export const FASES_RAIOS = 8; // quadros de rotação dos raios da roda (3 raios: o desenho se repete a cada 120 graus)

// folga da ponta sobre o cursor, tom da lâmpada, intensidade das lentes e a peça única "lit" de cada estado
const POSE = {
  ocioso: { folga: 22, lampada: "lo", lit: "cursor", texto: "Parado, sem plano" },
  lendo: { folga: 3, lampada: "hi", lit: "ponta", texto: "Lendo o arquivo sob o cursor" },
  escrevendo: { folga: 0.6, lampada: "hi", lit: "ponta", texto: "Escrevendo no arquivo" },
  testando: { folga: 0.4, lampada: "hi", lit: "agulha", texto: "Rodando testes" },
  aguardando: { folga: 12, lampada: "mid", lit: "cursor", texto: "Aguardando resposta" },
  bloqueado: { folga: 10, lampada: "lit", lit: "lampada", texto: "Bloqueado: trava abaixada" },
  concluido: { folga: 18, lampada: "hi", lit: "bandeira", texto: "Concluído: bandeja cheia" },
};
const ANEIS = { lo: 0, mid: 1, hi: 2, lit: 3 }; // anéis da lente: quanto mais, mais acesa
const TEXTO_PAPEL = { explorador: "Explorador lê", construtor: "Construtor escreve", revisor: "Revisor testa" };
const SIGLA = { explorador: "EXP", construtor: "CON", revisor: "REV" };
const COMPRIMENTO = { explorador: 27.5, construtor: 20.6, revisor: 18.75 }; // da base da luva à ponta da ferramenta

const lim = (v, a = 0, b = 1) => (Number.isFinite(+v) ? Math.min(b, Math.max(a, +v)) : a);
const L = (x, y, w, d, z, h, tom = "mid", o = {}) => ({ tipo: "laje", x, y, w, d, z, h, tom, ...o });
const C = (cx, cy, r, z, h, tom = "mid", o = {}) => ({ tipo: "cilindro", cx, cy, r, z, h, tom, ...o });
const S = (cx, cy, r, z, h, n, tom = "lo") => ({ tipo: "serrilha", cx, cy, r, z, h, n, tom, inset: 0 }); // as nervuras vão de borda a borda
const A = (cx, cy, r, z, tom = "lo") => ({ tipo: "anel", cx, cy, r, z, tom });
const N = (x, y, x2, y2, z, tom = "lo", libre) => ({ tipo: "linha", x, y, x2, y2, z, tom, ...(libre ? { libre } : {}) }); // `libre`: ponta feita para terminar no ar (ponteiro, traço de texto, marca de régua)
const K = (x, y, z, x2, y2, z2, tom = "lo", libre) => ({ tipo: "risco", x, y, z, x2, y2, z2, tom, ...(libre ? { libre } : {}) });
const Q = (plano, a, _, c, d, z, z2, tom = "faint") => ({ tipo: "quadro", plano, ...(plano === "y" ? { x: c, y: a, x2: d, y2: a } : { x: a, y: c, x2: a, y2: d }), z, z2, tom }); // contorno fechado (placa, veneziana) numa face em pé: plano y = face em y=a, de x=c a x=d; plano x = face em x=a, de y=c a y=d
const D = (pts, tam = 0.5, tom = "mid") => ({ tipo: "pontos", pts, tam, tom });
const fundo = (z) => 13.5 + (z - 13) * 0.002; // peças do fundo (torre, barra, mostrador, bandeja): sempre atrás da maquinaria
const com = (p, ordem) => ({ ...p, ordem });

// ferramenta na ponta da lança: zs = fundo da luva; ponta = tom da peça que toca o arquivo
function ferramenta(papel, cx, ty, zs, ponta) {
  if (papel === "explorador") { // lupa: haste, disco de vidro e aro (o vidro, de raio 6, passa ao lado do cabo do carro)
    return [C(cx, ty, 2, zs - 15.5, 15.5, "mid"), { tipo: "roda", cx, cy: ty, z: zs - 21.5, r: 6, w: 3, eixo: "x", tom: "hi" },
      { tipo: "aro", cx: cx + 1.5, cy: ty, z: zs - 21.5, r: 4.6, eixo: "x", tom: ponta }, { tipo: "aro", cx: cx + 1.5, cy: ty, z: zs - 21.5, r: 2.4, eixo: "x", tom: "faint" }];
  }
  if (papel === "construtor") { // caneta: mandril serrilhado, haste, cone e ponta
    return [C(cx, ty, 4.2, zs - 6.25, 6.25, "mid"), S(cx, ty, 4.2, zs - 6.25, 6.25, 20), C(cx, ty, 2.5, zs - 15, 8.75, "hi"), C(cx, ty, 1.9, zs - 18.1, 3.1, "hi"), C(cx, ty, 1, zs - 20.6, 2.5, ponta)];
  }
  // revisor: compasso de duas pernas a cavalo do cursor, com êmbolo central
  return [L(cx - 11, ty - 3.75, 22, 7.5, zs - 3.75, 3.75, "mid", { r: 1 }), D([[cx - 8, ty, zs], [cx + 8, ty, zs]], 0.55, "hi"),
    C(cx - 7.5, ty, 1.75, zs - 18.75, 15, ponta), C(cx + 7.5, ty, 1.75, zs - 18.75, 15, "hi"), C(cx, ty, 1.25, zs - 11.25, 7.5, "lo")];
}

// cabo: trechos retos nos eixos (deitados na roda) ligados por luvas em pé nos cantos (pontos seguidos no mesmo x,y viram uma luva só, mais alta).
// Os trechos encostam nas luvas e nas tomadas sem se atravessar (a auditoria de profundidade não aceita peças que se interpenetram); `ordem` o põe na pilha de pintura.
function cabo(rota, ordem, r = 1.1, tom = "hi") {
  const nos = [];
  for (const [x, y, z] of rota) {
    const u = nos.at(-1);
    if (u && u.x === x && u.y === y) u.z1 = z;
    else nos.push({ x, y, z0: z, z1: z });
  }
  const luva = (n, i) => (i > 0 && i < nos.length - 1) || n.z0 !== n.z1; // as pontas ficam coladas na tomada
  const P = [];
  nos.forEach((n, i) => {
    if (luva(n, i)) P.push(C(n.x, n.y, r, Math.min(n.z0, n.z1) - r, Math.abs(n.z1 - n.z0) + 2 * r, "mid"));
    const b = nos[i + 1];
    if (!b) return;
    const eixo = n.x !== b.x ? "x" : "y", a0 = eixo === "x" ? n.x : n.y, a1 = eixo === "x" ? b.x : b.y, sinal = Math.sign(a1 - a0);
    const de = a0 + (luva(n, i) ? sinal * r : 0), ate = a1 - (luva(b, i + 1) ? sinal * r : 0);
    if (Math.abs(ate - de) < 0.3) return;
    const c = (de + ate) / 2, w = Math.abs(ate - de);
    P.push(eixo === "x" ? { tipo: "roda", cx: c, cy: n.y, z: n.z1, r, w, eixo, tom } : { tipo: "roda", cx: n.x, cy: c, z: n.z1, r, w, eixo, tom });
  });
  return P.map((p) => ({ ...p, ordem }));
}

export function pecasDoAgente({ papel, estado, posicao, progresso } = {}) {
  papel = PAPEIS.includes(papel) ? papel : "construtor";
  estado = ESTADOS.includes(estado) ? estado : "ocioso";
  const pose = POSE[estado];
  const pos = estado === "ocioso" ? 0 : lim(posicao); // parado, o carro fica recuado na origem da régua
  const prog = lim(progresso);
  const cx = -72 + 144 * pos; // as 5 estações (0, .25, .5, .75, 1) caem nas marcas grandes da régua
  const ty = 46; // linha do cursor, no meio da régua
  const girando = estado === "testando";
  const lit = (chave, tom) => (pose.lit === chave ? "lit" : tom);
  const zA = 16.7 + pose.folga + COMPRIMENTO[papel] + 3; // altura da lança: a ponta fica a "folga" sobre o cursor
  const zs = zA - 3;
  const P = [];
  const em = (grupo, ...ps) => P.push(...ps.flat().map((p) => ({ ...p, grupo })));

  // ---- base: pés, placa, ranhura, parafusos nas bordas e nas faces, etiqueta, painel de comando (MCP) ----
  em("base", [[-120, -64], [120, -64], [-120, 62], [120, 62]].map(([x, y]) => C(x, y, 7, 0, 5, "lo")));
  em("base", L(-128, -72, 256, 142, 5, 8, "mid", { r: 8, borda: 1.4 }), N(-122, -66, 122, -66, 13, "faint"), N(122, -66, 122, 64, 13, "faint"), N(122, 64, -122, 64, 13, "faint"), N(-122, 64, -122, -66, 13, "faint"));
  em("base", D([[-124, -68, 13], [124, -68, 13], [-124, 66, 13], [124, 66, 13]], 0.7, "hi"), D([-100, -60, -20, 20, 60, 100].map((x) => [x, 70, 9]), 0.5, "mid"), D([-45, -15, 15, 45].map((y) => [128, y, 9]), 0.5, "mid"));
  em("base", L(-124, 56, 40, 12, 13, 0.8, "lo", { r: 2 }), N(-120, 60, -104, 60, 13.8, "lo", true), N(-120, 64.5, -98, 64.5, 13.8, "lo", true)); // traços que representam texto impresso
  em("base", L(-76, 56, 78, 12, 13, 6, "mid", { r: 2, borda: 0.5 }), [-64, -50, -36].flatMap((x) => [C(x, 62, 3.4, 19, 2.5, "hi"), S(x, 62, 3.4, 19, 2.5, 18), N(x, 62, x + 3.4, 62, 21.5, "hi", "start")])); // ponteiro do botão: nasce no centro
  em("base", D([-14, -9, -4].map((x) => [x, 62, 19]), 0.6, "hi"), { tipo: "texto", x: -28, y: 64.5, z: 19, texto: "MCP", tam: 5, tom: "mid", ancora: "start" });
  // fonte de alimentação no fundo, entre a torre e o mostrador: corpo, painel, aletas, chave e led
  em("base", com(L(-86, -68, 56, 26, 13, 14, "mid", { r: 2, borda: 0.6 }), fundo(13)), com(L(-83, -65, 50, 20, 27, 0.6, "lo", { r: 1 }), fundo(27)), [0, 1, 2, 3].map((i) => com(L(-79 + 5 * i, -63, 2.4, 16, 27.6, 0.5, "faint"), fundo(27.6))));
  em("base", com(C(-46, -55, 3, 27.6, 2, "hi"), fundo(27.6)), com(C(-46, -55, 1.6, 29.6, 1.2, "lo"), fundo(29.6)), com(D([[-38, -62, 27.6], [-38, -57, 27.6]], 0.6, "hi"), fundo(27.7)));

  // ---- trilho (dois trilhos, batentes, parafusos) e régua (a pasta/arquivo onde o agente está) ----
  for (const y of [-8.5, 19.5]) em("base", com(L(-100, y, 200, 5, 13, 4, "mid", { r: 1 }), 14));
  for (const y of [-6, 22]) em("base", com(D([-90, -60, -30, 0, 30, 60, 90].map((x) => [x, y, 17]), 0.4, "hi"), 16.9));
  for (const x of [-106, 100]) em("base", com(L(x, -11, 6, 38, 13, 8, "lo", { borda: 0.5 }), 14.5), com(D([[x + 3, -6, 21], [x + 3, 22, 21]], 0.5, "hi"), 14.6));
  em("base", com(L(-118, 40, 236, 12, 13, 2.5, "mid", { r: 1.2, borda: 0.4 }), 14.2), [-122, 118].map((x) => com(L(x, 38, 4, 16, 13, 6, "lo"), 14.3)));
  for (let i = 0; i <= 12; i++) em("base", com(N(-108 + 18 * i, 52, -108 + 18 * i, i % 2 === 0 ? 44.5 : 48, 15.5, i % 2 === 0 ? "hi" : "mid", "end"), 15.6));

  // ---- etiqueta com a sigla do papel ----
  em("etiqueta", { tipo: "texto", x: -102, y: 67, z: 13.8, texto: SIGLA[papel], tam: 6, tom: "hi" });

  // ---- torre da lâmpada (estado): poste fixo, lâmpada com lentes (mais anéis = mais acesa) e bandeira ----
  const tx = -116, ty0 = -60, tom = lit("lampada", pose.lampada);
  em("base", com(C(tx, ty0, 8, 13, 4, "lo"), fundo(13)), com(A(tx, ty0, 6.3, 17, "faint"), fundo(17)), [[17, 11], [30.5, 25.5], [58.5, 5.5]].map(([z, h]) => com(C(tx, ty0, 2.4, z, h, "mid"), fundo(z))));
  for (const z of [28, 56]) em("base", com(C(tx, ty0, 3.6, z, 2.5, "lo"), fundo(z)));
  em("base", com(C(tx, ty0, 7, 64, 2, "lo"), fundo(64)));
  em("torre", com(C(tx, ty0, 6, 66, 9, tom, { borda: 0.4 }), fundo(66)), com(S(tx, ty0, 6, 66, 9, 24, "lo"), fundo(66.1)), com(C(tx, ty0, 4.4, 75, 3, tom === "lit" ? "hi" : tom), fundo(75)));
  for (let i = 0; i < ANEIS[pose.lampada]; i++) em("torre", com(A(tx, ty0, 3.4 - 1.2 * i, 78.01, "hi"), fundo(78.1)));
  if (estado === "concluido") em("torre", com(L(tx + 2.4, ty0 - 0.8, 24, 1.6, 38, 14, "lit"), fundo(38)), [42.5, 47.5].map((z) => com(L(tx + 3, ty0 + 0.8, 22, 0.3, z, 1, "mid"), fundo(z + 0.1))));

  // ---- barra de progresso: calha com 10 encaixes; os segmentos cheios sobem e acendem ----
  // (fica na frente, onde a lança nunca tapa)
  em("base", L(14, 55, 108, 15, 13, 3, "lo", { r: 2, borda: 0.4 }), D(Array.from({ length: 10 }, (_, i) => [24.2 + 10.4 * i, 62.5, 16]), 0.5, "mid"));
  for (let i = 0; i < Math.round(prog * 10); i++) em("enchimento", L(20.2 + 10.4 * i, 58, 8, 9, 16, 3.4, "hi", { r: 0.8 }));

  // ---- mostrador: lê o progresso só durante o teste; fora dele o ponteiro fica no zero ----
  const gx = 33, gy = -53, fi = (i) => ((90 + (270 * i) / 8) * Math.PI) / 180, af = fi(girando ? prog * 8 : 0);
  em("base", com(C(gx, gy, 15, 13, 6, "hi", { borda: 0.8 }), fundo(13)), com(S(gx, gy, 15, 13, 6, 56), fundo(13.1)), com(C(gx, gy, 12.8, 19, 0.4, "lo"), fundo(19)));
  em("mostrador", com(A(gx, gy, 12.8, 19.45, girando ? "mid" : "faint"), fundo(19.5)), com(C(gx, gy, 1.8, 19.4, 1.4, "hi"), fundo(19.7)));
  for (let i = 0; i <= 8; i += 2) em("mostrador", com(N(gx + 9.5 * Math.cos(fi(i)), gy + 9.5 * Math.sin(fi(i)), gx + 12.8 * Math.cos(fi(i)), gy + 12.8 * Math.sin(fi(i)), 19.45, girando ? (i % 4 === 0 ? "hi" : "mid") : "lo", "start"), fundo(19.6)));
  em("agulha", com(N(gx - 2.6 * Math.cos(af), gy - 2.6 * Math.sin(af), gx + 11.2 * Math.cos(af), gy + 11.2 * Math.sin(af), 21, lit("agulha", girando ? "hi" : "mid"), true), fundo(21))); // cauda e ponta da agulha

  // ---- bandeja de planos: 5 pastas em pé, cheias na ordem em que os planos terminam ----
  const cheias = estado === "concluido" ? 5 : Math.min(4, Math.floor(prog * 5));
  em("base", com(L(66, -70, 58, 36, 13, 3, "lo", { r: 1.5, borda: 0.4 }), fundo(13)), com(L(66, -70, 58, 2, 16, 6, "mid"), fundo(16)), com(L(66, -68, 2, 32, 16, 6, "mid"), fundo(16)));
  for (let i = 0; i < cheias; i++) { const x = 71 + 10 * i; em("cartas", com(L(x, -67, 7, 30, 16, 15, i === cheias - 1 ? "hi" : "mid", { r: 0.5 }), fundo(16))); }
  em("frente", com(L(66, -36, 58, 2, 16, 6, "mid"), fundo(16.2)), com(L(122, -68, 2, 32, 16, 6, "mid"), fundo(16.2)));

  // ---- cursor na régua: onde o agente trabalha (e o que ele faz ali) ----
  em("alvo", com(C(cx, ty, 5.4, 15.5, 1.2, lit("cursor", "hi")), 15.8), com(A(cx, ty, 3.2, 16.7, "mid"), 15.9));
  if (estado === "lendo") em("alvo", com(N(cx, 40, cx, 52, 15.5, "mid"), 15.7)); // feixe de leitura, de borda a borda
  if (estado === "escrevendo") em("alvo", com(D([-24, -19, -14, -10.5, -8].map((d) => [cx + d, ty, 15.5]), 0.6, "hi"), 15.7)); // rastro de tinta

  // ---- trava de segurança, só no bloqueio: sobe à frente do carro, com listras de aviso ----
  if (estado === "bloqueado") {
    const xb = pos > 0.75 ? cx - 40 : cx + 40;
    em("trava", com(L(xb - 4, -12, 8, 40, 17, 2, "lo", { borda: 0.3 }), 17.1), com(L(xb - 2.5, -10, 5, 36, 19, 11, "hi", { borda: 0.5 }), 19.1));
    for (let i = 0; i < 4; i++) em("trava", com(K(xb + 2.5, -10 + 9 * i, 19, xb + 2.5, -1 + 9 * i, 30, "mid"), 19.2));
    em("trava", com(D([[xb, -7, 30], [xb, 23, 30]], 0.6, "hi"), 30.1));
  }

  // ---- carro: blocos sobre o trilho, placa, parafusos, rebites, motor, flange e pilar da roda ----
  for (const y of [-6, 22]) for (const dx of [-17, 17]) em("carro", L(cx + dx - 8, y - 5.5, 16, 11, 17, 5, "lo", { borda: 0.4 }));
  em("carro", L(cx - 30, -18, 60, 52, 22, 5, "mid", { r: 3, borda: 0.6 }), N(cx - 26.5, -14.5, cx + 26.5, -14.5, 27, "faint"), N(cx + 26.5, -14.5, cx + 26.5, 30.5, 27, "faint"), N(cx + 26.5, 30.5, cx - 26.5, 30.5, 27, "faint"), N(cx - 26.5, 30.5, cx - 26.5, -14.5, 27, "faint"));
  for (const [sx, sy] of [[-27, -15], [27, -15], [-27, 31], [27, 31]]) em("carro", C(cx + sx, sy, 1.5, 27, 0.9, "mid"), N(cx + sx - 1.1, sy - 0.4, cx + sx + 1.1, sy + 0.4, 27.9, "hi")); // parafusos de cabeça com fenda
  em("carro", L(cx + 6, 17, 22, 10, 27, 0.5, "lo", { r: 1 }), { tipo: "texto", x: cx + 8.5, y: 23, z: 27.5, texto: "CARRO", tam: 4.4, tom: "mid", ancora: "start" }, D([[cx + 8, 18.8, 27.5], [cx + 26, 25.2, 27.5]], 0.45, "hi")); // placa de rótulo
  em("carro", L(cx - 24, 9, 14, 3, 27, 0.2, "faint", { r: 1.4 }), L(cx + 4, 6, 14, 3, 27, 0.2, "faint", { r: 1.4 })); // furos oblongos de ajuste
  em("carro", D([-21, -14, -7, 0, 7, 14, 21].map((d) => [cx + d, 32, 27]), 0.4, "mid"));
  em("carro", C(cx, 0, 9, 27, 2.5, "lo", { borda: 0.4 }), C(cx, 0, 6.6, 29.5, 2.5, "mid"), S(cx, 0, 9, 27, 2.5, 28));
  em("carro", C(cx - 18, -9, 5.4, 27, 6, "mid", { borda: 0.4 }), C(cx - 18, -9, 5.4, 33, 5, "mid", { borda: 0.4 }), S(cx - 18, -9, 5.4, 33, 5, 20), C(cx - 18, -9, 5.4, 38, 2, "mid", { borda: 0.4 }), C(cx - 18, -9, 2.6, 40, 2, "hi"), A(cx - 18, -9, 4.2, 40.05, "lo"));
  em("carro", L(cx + 16, -14.5, 6, 5, 27, 1.5, "lo", { r: 0.8 }), L(cx - 26, 23, 17, 7, 27, 0.6, "lo", { r: 1 }), N(cx - 24, 25, cx - 13, 25, 27.6, "lo", true), N(cx - 24, 28, cx - 17, 28, 27.6, "lo", true));
  em("roda", { tipo: "roda", cx: cx + 19, cy: -12, z: 37.5, r: 9, w: 4, eixo: "y", tom: girando ? "hi" : "lo" }, { tipo: "aro", cx: cx + 19, cy: -10, z: 37.5, r: 8.2, eixo: "y", tom: girando ? "mid" : "faint" });
  const ang = pos * 10; // a roda gira com o carro (como se rolasse na cremalheira)
  // raios: a ponta de dentro fica sob o cubo (pintado depois) e é "libre" porque o conferidor de pontas não conta o cubo como cobertura
  for (let i = 0; i < 3; i++) { const a = ang + (i * 2 * Math.PI) / 3; em("raios", com(K(cx + 19 + 2.4 * Math.cos(a), -10, 37.5 + 2.4 * Math.sin(a), cx + 19 + 8.2 * Math.cos(a), -10, 37.5 + 8.2 * Math.sin(a), girando ? "hi" : "lo", "start"), 34.3)); }
  em("cubo", { tipo: "roda", cx: cx + 19, cy: -9.1, z: 37.5, r: 3.4, w: 1.8, eixo: "y", tom: "mid", ordem: 34.6 });

  // ---- braço: poste (em duas partes, a lança passa por ele), colar, lança, luva e ferramenta ----
  const topo = 84;
  em("braco", C(cx, 0, 3.6, 32, zA - 1 - 32, "mid"), com(C(cx, 0, 6, zA - 1, 8, "lo", { borda: 0.4 }), zA - 1));
  em("braco", com(L(cx - 3.5, 6, 7, 35, zA, 5, "mid", { r: 1.5, borda: 0.4 }), zA), com(N(cx, 6, cx, 41, zA + 5, "lo"), zA + 5.1));
  em("braco", com(C(cx, 0, 3.6, zA + 7, topo - zA - 7, "mid"), zA + 7), com(C(cx, 0, 4.4, topo - 1.5, 1.5, "hi"), topo), com(A(cx, 0, 2.4, topo + 0.01, "lo"), topo + 0.1));
  em("braco", com(C(cx, ty, 5.5, zs, 3, "mid", { borda: 0.4 }), zA + 0.5), com(S(cx, ty, 5.5, zs, 3, 20), zA + 0.6), com(C(cx, ty, 5.5, zs + 3, 7, "mid", { borda: 0.4 }), zA + 0.5));
  for (const p of ferramenta(papel, cx, ty, zs, lit("ponta", "hi"))) em("braco", com(p, zA - 4));

  // ---- bancada: emendas da placa, furos de fixação, faixa lateral e numeração da régua ----
  em("base", N(-128, -30, 128, -30, 13, "faint"), N(54, -72, 54, -30, 13, "faint")); // emendas da placa: de uma borda à outra
  em("base", D([-96, -88, -80, -72, -64, -56, -48, -40, -32, -24, -16, -8, 0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96].map((x) => [x, -24, 13]), 0.42, "lo"),
    D([-96, -88, -80, -72, -64, -56, -48, -40, -32, -24, -16, -8, 0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96].map((x) => [x, -18, 13]), 0.42, "lo"),
    D([76, 88, 100, 112].flatMap((x) => [-24, -14, -4, 6, 16, 26].map((y) => [x, y, 13])), 0.42, "lo"), D([114, 122].flatMap((x) => [-30, -22, -14, -6, 2, 10, 18, 26].map((y) => [x, y, 13])), 0.42, "lo"));
  // faixas laterais da placa: venezianas de ventilação, placa de série (a moldura) e rebites, e mais furos nos cantos vazios
  for (let i = 0; i < 8; i++) em("base", Q("y", 70, 0, -109 + 26 * i, -107 + 26 * i, 6.6, 11.4), Q("x", 128, 0, -53 + 15 * i, -51 + 15 * i, 6.6, 11.4));
  em("base", Q("y", 70, 0, 70, 110, 6.4, 11.6, "lo"), D([[72, 70, 9], [108, 70, 9]], 0.5, "hi"));
  em("base", D([-122, -114, -106].flatMap((x) => [-46, -38, -30, -22, -14].map((y) => [x, y, 13])), 0.42, "lo"));
  [0, 1, 2, 3, 4].forEach((k) => em("base", com({ tipo: "texto", x: -72 + 36 * k - 1.2, y: 42.6, z: 15.5, texto: String(k), tam: 3.4, tom: "lo", ancora: "start" }, 15.7)));

  // ---- fonte de alimentação: venezianas, placa de dados, rebites, tomada de saída e parafusos do tampo ----
  for (let i = 0; i < 8; i++) em("base", com(Q("y", -42, 0, -81 + 6 * i, -79 + 6 * i, 16, 24), fundo(27.8)));
  em("base", com(Q("x", -30, 0, -64, -50, 17, 23), fundo(27.8)), com(D([[-30, -62, 20], [-30, -52, 20]], 0.5, "hi"), fundo(27.9)));
  em("base", com(L(-83, -42, 6, 3.2, 17, 5, "lo", { r: 0.5, borda: 0.3 }), fundo(28)), com(D([[-81, -38.8, 20.2], [-79, -38.8, 20.2]], 0.45, "hi"), fundo(28.1)));
  em("base", com(C(-83.5, -66.5, 1.2, 27, 0.8, "mid"), fundo(27.9)), com(C(-32.5, -66.5, 1.2, 27, 0.8, "mid"), fundo(27.9)), com({ tipo: "texto", x: -60.5, y: -52, z: 27.6, texto: "24V", tam: 4, tom: "mid", ancora: "start" }, fundo(27.9)));

  // ---- torre e mostrador: parafusos da flange e do aro, e a bucha por onde o cabo sai ----
  em("base", com(D([[tx + 5.1, ty0 + 5.1, 17], [tx - 5.1, ty0 + 5.1, 17], [tx + 5.1, ty0 - 5.1, 17], [tx - 5.1, ty0 - 5.1, 17]], 0.6, "hi"), fundo(17.2)), com(L(tx - 2.5, -52.2, 5, 3.6, 13, 4.2, "lo", { r: 0.5, borda: 0.3 }), fundo(17.3)));
  em("base", com(D([45, 135, 225, 315].map((g) => [gx + 13.9 * Math.cos((g * Math.PI) / 180), gy + 13.9 * Math.sin((g * Math.PI) / 180), 19]), 0.6, "hi"), fundo(19.3)));

  // ---- cabos (retos, nos eixos, sobre apoios): torre → fonte, e carro → painel MCP (acompanha o carro; cada ponta termina numa tomada) ----
  // cabo 1: da bucha da torre à tomada da fonte, com duas braçadeiras (blocos entre dois trechos: o cabo para em cada face)
  const OR1 = 14.2;
  em("base", cabo([[tx, -48.6, 14.1], [tx, -45.6, 14.1]], OR1), com(L(tx - 1.8, -45.6, 3.6, 3.2, 13, 3.4, "lo", { r: 0.4, borda: 0.3 }), OR1), com(D([[tx, -44, 16.4]], 0.5, "hi"), OR1 + 0.1));
  em("base", cabo([[tx, -42.4, 14.1], [tx, -37.5, 14.1], [-99.8, -37.5, 14.1]], OR1), com(L(-99.8, -39.1, 3.6, 3.2, 13, 3.4, "lo", { r: 0.4, borda: 0.3 }), OR1), com(D([[-98, -37.5, 16.4]], 0.5, "hi"), OR1 + 0.1));
  em("base", cabo([[-96.2, -37.5, 14.1], [-80, -37.5, 14.1], [-80, -37.5, 19.5], [-80, -38.8, 19.5]], OR1));
  // cabo 2: da tomada do carro à tomada na ponta do painel MCP; acompanha o carro (por isso é redesenhado a cada posição)
  const px = cx + 12, OR2 = 30;
  em("carro", L(px - 4.5, 34, 9, 3.2, 22.6, 4.4, "lo", { r: 0.5, borda: 0.3 }), D([[px - 2, 37.2, 25.2], [px + 2, 37.2, 25.2]], 0.45, "hi"));
  em("base", com(L(2, 59, 4, 5, 14.5, 3.4, "lo", { r: 0.5, borda: 0.3 }), OR2), com(D([[3, 61.5, 17.9], [5, 61.5, 17.9]], 0.45, "hi"), OR2 + 0.1)); // tomada na ponta do painel, de frente para o cabo
  em("cabo", cabo([[px, 37.2, 24.8], [px, 38.8, 24.8], [px, 38.8, 20.5], [10, 38.8, 20.5], [10, 52.6, 20.5]], OR2));
  em("base", com(C(10, 38.8, 1.2, 13, 6.2, "lo"), OR2 - 1), com(L(8.2, 52.6, 3.6, 3.2, 13, 8.6, "mid", { r: 0.4, borda: 0.3 }), OR2), com(D([[10, 54.2, 21.6]], 0.5, "hi"), OR2 + 0.1)); // apoio no canto e braçadeira alta
  em("cabo", cabo([[10, 55.8, 20.5], [10, 61.5, 20.5], [10, 61.5, 15.8], [6, 61.5, 15.8]], OR2));

  const arred = (v) => (typeof v === "number" ? Math.round(v * 100) / 100 : Array.isArray(v) ? v.map(arred) : v);
  return P.map((p) => Object.fromEntries(Object.entries(p).map(([k, v]) => [k, arred(v)])));
}

// frase curta para o texto alternativo do SVG
export const descricaoDoAgente = ({ papel, estado } = {}) => `${TEXTO_PAPEL[papel] ?? TEXTO_PAPEL.construtor}: ${(POSE[estado] ?? POSE.ocioso).texto}`;
