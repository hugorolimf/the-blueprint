// O Claude na folha: um personagem pequeno, desenhado com o anatomy, só dados (lista de peças para desenhaEmblema), puro e determinístico.
// É o agente que trabalha no app (contrato em docs/claude-na-folha.md). Inspirado na mascote do Claude Code: corpo largo e laranja,
// dois olhos escuros, braços e quatro pernas curtos. Olha para +x (câmera em AZIMUTE 30: a frente e o lado esquerdo ficam à vista).
// O corpo é o mesmo em tudo; a pose troca só braços, pernas, olhos e o adereço. Os subagentes (explorador, construtor, revisor) são o mesmo
// corpo com UMA ferramenta na mão esquerda e a cor de apoio do papel (base, cinta e ferramenta); a folha os desenha no MESMO tamanho do Claude (ESCALA.subagente = 1, decisão do usuário de 2026-10-22: todos os agentes têm o mesmo tamanho; o papel se vê pela ferramenta e pela cor).
// Cada peça leva `grupo` (base, pernas, corpo, bracos, ferramenta, adereco), para quem animar depois mexer só no que muda.
export const POSES = ["parado", "andando", "trabalhando", "esperando", "bloqueado", "concluido"];
export const PAPEIS = ["explorador", "construtor", "revisor"];
// Animação por quadros (cada quadro é um desenho pronto, guardado uma vez): quantos quadros cada pose tem, em que ordem rodam e quanto dura cada um (ms).
// O quadro 0 de toda pose é o desenho aprovado; os outros mexem só em poucas peças (respirar, piscar, digitar, ponto aceso, lâmpada, pulo, bandeira).
// Fluidez: os quadros intermediários (meio caminho entre dois extremos) e a sequência que vai e volta por eles, a ~8 quadros por segundo; o olho completa o movimento.
export const QUADROS = { parado: 5, andando: 3, trabalhando: 5, esperando: 3, bloqueado: 4, concluido: 3 };
export const SEQUENCIA = {
  parado: [0, 1, 2, 1, 0, 1, 2, 1, 0, 1, 2, 1, 0, 3, 4, 3], // três respirações e uma piscada (a piscada fecha e abre em 3 quadros)
  andando: [0, 2, 1, 2], // um par de pernas sobe, passa pelo meio, o outro par sobe
  trabalhando: [0, 1, 2, 1, 0, 3, 4, 3], // um braço avança e volta, depois o outro; a faísca pulsa junto
  esperando: [0, 1, 2], // o ponto aceso anda de um lado ao outro
  bloqueado: [0, 1, 2, 3], // a lâmpada acende (0 e 1) e apaga (2 e 3) enquanto a barra da cancela sobe e desce
  concluido: [0, 1, 2, 1, 0, 0, 0, 0], // pulinho (sobe e desce) e pausa; a bandeira ondula junto
};
export const RITMO = { parado: 150, andando: 110, trabalhando: 110, esperando: 300, bloqueado: 200, concluido: 120 };
export const GRUPOS = ["base", "pernas", "corpo", "bracos", "ferramenta", "adereco"];
export const AZIMUTE = 30; // a frente do personagem (+x) fica virada para o leitor
export const LIMITE = 400; // teto de peças da figura do agente (decisão da ação 3.7)
export const MAX_SUBAGENTES = 7; // figuras de subagente na folha (1 principal + 7); passando de 5 elas se juntam, sobrepostas; as demais ficam sob a pilha (sem contador "+N")
export const ESCALA = { claude: 1, subagente: 1 }; // largura do SVG do subagente = a do Claude (o enquadramento é o mesmo; foi 70% até 2026-10-22)
// pontos que só entram na caixa do enquadramento (opção `enquadra` de desenhaEmblema): a câmera não muda entre poses nem papéis
export const ENQUADRA = [[-20, -32, 0], [30, 32, 0], [0, 0, 60]];
export const OPCOES = { azimute: AZIMUTE, enquadra: ENQUADRA, limite: LIMITE, sombra: true };

const PULSO = [0, 1, 2, 1, 2]; // trabalhando: quanto a faísca cresce e gira em cada quadro (0 a 2)
const COR = { explorador: "exp", construtor: "con", revisor: "rev" }; // tom de apoio de cada papel (web/css/claude.css)
const SIGLA = { explorador: "EXP", construtor: "CON", revisor: "REV" };
const TEXTO_POSE = { parado: "parado", andando: "andando", trabalhando: "trabalhando", esperando: "aguardando resposta", bloqueado: "bloqueado", concluido: "concluiu" };
const TEXTO_PAPEL = { explorador: "Subagente explorador", construtor: "Subagente construtor", revisor: "Subagente revisor" };

const L = (x, y, w, d, z, h, tom = "claude", o = {}) => ({ tipo: "laje", x, y, w, d, z, h, tom, ...o });
const C = (cx, cy, r, z, h, tom = "claude", o = {}) => ({ tipo: "cilindro", cx, cy, r, z, h, tom, ...o });
const R = (cx, cy, z, r, w, tom) => ({ tipo: "roda", cx, cy, z, r, w, eixo: "x", tom }); // disco de frente para o leitor
const N = (x, y, x2, y2, z, tom = "faint") => ({ tipo: "linha", x, y, x2, y2, z, tom });
const K = (x, y, z, x2, y2, z2, tom = "olho", libre = true) => ({ tipo: "risco", x, y, z, x2, y2, z2, tom, ...(libre ? { libre } : {}) }); // `libre`: raio, sobrancelha, traço de texto
const D = (pts, tam = 0.5, tom = "hi") => ({ tipo: "pontos", pts, tam, tom });

// ferramenta de cada papel, em pé sobre a mão (hx, hy) à altura hz: lupa, martelo, prancheta
function ferramenta(papel, hx, hy, hz) {
  if (papel === "explorador") return [C(hx, hy, 1.4, hz, 8, "exp"), R(hx, hy, hz + 14, 6, 2.4, "exp"), R(hx + 1.5, hy, hz + 14, 4.4, 1, "papel")]; // cabo, aro e vidro
  if (papel === "construtor") return [C(hx, hy, 1.6, hz, 15, "con"), L(hx - 3, hy - 6, 6, 12, hz + 15, 6, "aco", { r: 1, borda: 0.4 }), L(hx - 3.3, hy + 3.4, 6.6, 2.4, hz + 14.8, 6.4, "con")]; // cabo, cabeça e anel
  return [L(hx - 0.9, hy - 6, 1.8, 12, hz, 17, "rev", { r: 1, borda: 0.5 }), L(hx + 0.9, hy - 4.6, 0.5, 9.2, hz + 2, 11.5, "papel"), L(hx + 0.9, hy - 2.5, 0.9, 5, hz + 14, 2.4, "aco"), // prancheta, folha e prendedor
    ...[6.5, 9, 11.5].map((z) => K(hx + 1.45, hy - 3.4, hz + z, hx + 1.45, hy + 3.4, hz + z, "olho"))];
}

export function pecasDoClaude({ pose, papel, quadro } = {}) {
  pose = POSES.includes(pose) ? pose : "parado";
  papel = PAPEIS.includes(papel) ? papel : null;
  quadro = Number.isInteger(quadro) && quadro > 0 && quadro < QUADROS[pose] ? quadro : 0; // quadros por pose em QUADROS; andando alterna o par de pernas e o braço que avançam
  const cor = papel ? COR[papel] : null;
  const andando = pose === "andando", trab = pose === "trabalhando", blq = pose === "bloqueado", esp = pose === "esperando", fim = pose === "concluido";
  const parado = pose === "parado";
  const zb = 12 + (andando && quadro ? (quadro === 2 ? 0.5 : 1) : 0) + (parado && quadro < 3 ? quadro * 0.5 : 0) + (fim ? [0, 1, 2][quadro] : 0); // base do corpo: sobe até 1 ao respirar, até 2 no pulinho, 0,5 a 1 andando
  const P = [];
  const em = (grupo, ...ps) => P.push(...ps.flat().map((p) => ({ ...p, grupo })));

  // ---- base: placa (na cor do papel), ranhura, parafusos e etiqueta ----
  em("base", L(-20, -32, 50, 64, 0, 4, cor ?? "lo", { r: 5, borda: 0.8 }), D([[-17, -29, 4], [27, -29, 4], [-17, 29, 4], [27, 29, 4]], 0.6));
  const ranhura = cor ? "olho" : "faint";
  em("base", N(-16, -28, 26, -28, 4, ranhura), N(26, -28, 26, 28, 4, ranhura), N(26, 28, -16, 28, 4, ranhura), N(-16, 28, -16, -28, 4, ranhura));
  em("base", { tipo: "texto", x: -14, y: -25, z: 4, texto: papel ? SIGLA[papel] : "CLD", tam: 4.5, tom: "hi", ancora: "start" });
  em("base", [[-12, -6], [-3, 3], [6, 12]].map(([a, b]) => ({ tipo: "quadro", plano: "y", x: a, y: 32, x2: b, y2: 32, z: 1, z2: 3, tom: ranhura })), // frestas de ventilação nas duas faces à vista da placa
    [[-18, -10], [-4, 4], [10, 18]].map(([a, b]) => ({ tipo: "quadro", plano: "x", x: 30, y: a, x2: 30, y2: b, z: 1, z2: 3, tom: ranhura })));

  // ---- pernas: quatro, com sapata; andando, um par sobe e avança ----
  [-16.5, -5.5, 5.5, 16.5].forEach((y, i) => {
    const sobe = andando ? (quadro === 2 ? 1.5 : i % 2 === quadro ? 3 : 0) : 0, dx = sobe; // andando: quadro 2 é o meio do passo (os dois pares a meia altura)
    em("pernas", L(-5.2 + dx, y - 4, 10.4, 8, 4 + sobe, 1.6, "claude", { r: 0.8, borda: 0.4 }), L(-4 + dx, y - 3, 8, 6, 4 + sobe, zb - 4 - sobe, "claude", { borda: 0.5 }), L(-4.4 + dx, y - 3.4, 8.8, 6.8, zb - 2, 1.2, "olho")); // sapata, perna (até o corpo) e junta sob o corpo
  });

  // ---- corpo: bloco, tampa, cinta do papel, distintivo na lateral (a faísca do Claude) e os olhos ----
  em("corpo", L(-11, -22, 22, 44, zb, 26, "claude", { r: 4, borda: 1 }));
  em("corpo", N(-7, -18, 7, -18, zb + 26, "olho"), N(7, -18, 7, 18, zb + 26, "olho"), N(7, 18, -7, 18, zb + 26, "olho"), N(-7, 18, -7, -18, zb + 26, "olho"));
  if (cor) em("corpo", L(11, -22.7, 0.8, 45.4, zb + 1.5, 3.2, cor), L(-11.7, 22, 23.4, 0.8, zb + 1.5, 3.2, cor));
  em("corpo", { tipo: "quadro", plano: "y", x: -6, y: 22.05, x2: 6, y2: 22.05, z: zb + 11, z2: zb + 21, tom: "olho" }); // moldura do distintivo na face y=22
  em("corpo", [[-4, 0], [0, 4], [-2.8, 2.8], [-2.8, -2.8]].map(([a, b]) => K(a, 22.05, zb + 16 + b, -a, 22.05, zb + 16 - b))); // faísca: quatro raios que se cruzam no centro
  let [ez, eh] = blq ? [17, 3.5] : fim ? [17.5, 2.4] : trab ? [14, 6] : esp ? [16, 7] : [15, 7]; // altura e tamanho dos olhos: franzidos no bloqueio, "fechados" ao concluir
  if (parado && quadro >= 3) [ez, eh] = quadro === 3 ? [16.4, 3.6] : [17.8, 1.2]; // piscada: meio fechado e fechado
  for (const ey of [-8, 8]) em("corpo", L(11, ey - 1.7, 1.2, 3.4, zb + ez, eh, "olho", { r: 0.4 }));
  em("corpo", D([[-7, -18, zb + 26], [7, -18, zb + 26], [-7, 18, zb + 26], [7, 18, zb + 26]], 0.5), // parafusos da tampa
    D([[12.3, -8, zb + ez + eh - 1.4], [12.3, 8, zb + ez + eh - 1.4]], 0.45, "hi")); // brilho nos olhos
  em("corpo", [-4.5, 0, 4.5].map((y) => L(11, y - 0.6, 0.3, 1.2, zb + 5.5, 4.5, "olho"))); // respiros na frente, embaixo dos olhos
  if (blq) em("corpo", K(11.3, -11.5, zb + 22.5, 11.3, -5, zb + 20.2), K(11.3, 11.5, zb + 22.5, 11.3, 5, zb + 20.2)); // sobrancelhas

  // ---- braços (e a mão esquerda, onde a ferramenta fica em pé) ----
  let mao; // [x, y, z] do topo do braço esquerdo
  if (fim) { // os dois para o alto
    em("bracos", L(-3, -28.5, 6, 6.5, zb + 3, 18), L(-3, 22, 6, 6.5, zb + 3, 18));
    mao = [0, 25.5, zb + 21];
  } else if (trab || blq || esp) { // para a frente: no teclado (trabalhando), com a palma (bloqueado) ou cruzados (aguardando)
    const c = esp ? 6 : 11;
    const dig = trab ? [[0, 0], [-1, 1], [-2, 2], [1, -1], [2, -2]][quadro] : [0, 0]; // digitando: um braço avança e o outro recua, passando pelo meio
    em("bracos", L(11, -22, c + dig[0], 6, zb + 2, 6), L(11, 16, c + dig[1], 6, zb + 2, 6));
    mao = [14 + dig[1], 20.5, zb + 8];
  } else { // ao lado do corpo; andando, balançam
    const b = andando ? [-3, 3, 0][quadro] : 0;
    em("bracos", L(-4 - b, -30, 8, 8, zb + 3, 6), L(-4 + b, 22, 8, 8, zb + 3, 6));
    mao = [b, 26, zb + 9];
  }
  if (papel) em("ferramenta", ferramenta(papel, ...mao).map((p, i) => ({ ...p, ordem: 40 + i / 100 }))); // a ferramenta fica à frente do corpo e do distintivo

  // ---- adereço de cada pose: o que o olhar segue é a única peça lit ----
  if (trab && !papel) { // mesinha com teclado (grade de teclas de borda a borda)
    em("adereco", { ...L(13, -24, 16, 48, 4, 10, "mid", { r: 1.5, borda: 0.5 }), ordem: 13 }, { ...L(15, -13, 11, 26, 14, 1, "aco", { r: 0.5 }), ordem: 13.5 });
    em("adereco", [18.7, 22.3].map((x) => ({ ...N(x, -13, x, 13, 15, "olho"), ordem: 13.6 })), [-7, 0, 7].map((y) => ({ ...N(15, y, 26, y, 15, "olho"), ordem: 13.6 })));
  }
  if (trab) em("adereco", R(0, 0, zb + 38, 3.4 + 0.4 * PULSO[quadro], 1, "lit"), [0, 45, 90, 135].map((g) => { const a = ((g + 11 * PULSO[quadro]) * Math.PI) / 180, c = (8 + 0.75 * PULSO[quadro]) * Math.cos(a), s = (8 + 0.75 * PULSO[quadro]) * Math.sin(a); return K(0, c, zb + 38 + s, 0, -c, zb + 38 - s, "hi"); })); // faísca acesa sobre a cabeça
  if (esp) em("adereco", [-6, 0, 6].map((y) => R(0, y, zb + 34, 1.9, 1.2, y === [6, 0, -6][quadro] ? "lit" : "mid"))); // "..." de quem espera: o ponto aceso anda de um lado ao outro
  if (blq) { // cancela à frente: dois postes, barra listrada e a lâmpada acesa no poste visível
    const na_frente = (p, i) => ({ ...p, ordem: 45 + i / 100 }); // a cancela é a peça mais próxima do leitor
    em("adereco", [C(27, -23, 1.5, 4, 14, "aco"), C(27, 23, 1.5, 4, 14, "aco"), ...[-25, -15, -5, 5, 15].map((y, i) => L(25.6, y, 2.4, 10, 12 + [0, 0.75, 1.5, 0.75][quadro], 4, i % 2 ? "claude" : "papel")), C(27, 23, 2.2, 18, 2.6, quadro >= 2 ? "mid" : "lit")].map(na_frente));
  }
  if (fim) em("adereco", C(-15.5, 29.5, 0.9, 4, 34, "aco"), L(-16, 16.1 + [0, 1.2, 2.4][quadro], 1, 13.4 - [0, 1.2, 2.4][quadro], 30, 8, "lit", { r: 0.4 })); // o pano da bandeira ondula // bandeira de concluído na placa
  const arred = (v) => (typeof v === "number" ? Math.round(v * 100) / 100 : Array.isArray(v) ? v.map(arred) : v);
  return P.map((p) => Object.fromEntries(Object.entries(p).map(([k, v]) => [k, arred(v)])));
}

// pose a partir do contrato de registrar_progresso: `estado` (lendo|escrevendo|testando|aguardando|bloqueado|concluido, ou nada = parado)
// e `andando` (a folha sabe que o agente acabou de mudar de `onde`/plano): andando só vale enquanto ele trabalha
export function poseDoEstado(estado, andando = false) {
  if (estado === "bloqueado" || estado === "concluido") return estado;
  if (estado === "aguardando") return "esperando";
  if (estado === "lendo" || estado === "escrevendo" || estado === "testando") return andando ? "andando" : "trabalhando";
  return "parado";
}

export const descricaoDoClaude = ({ pose, papel } = {}) => `${TEXTO_PAPEL[papel] ?? "Claude"}: ${TEXTO_POSE[pose] ?? TEXTO_POSE.parado}`;
