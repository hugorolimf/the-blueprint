// A tela principal: o Planejar como PÁGINA. A proposta que o agente manda pelo MCP chega escrita como uma página do blueprint, no
// estilo das telas do Obsidian do Hugo: o plano em poucas linhas, o que muda na folha (desenhado como a folha), cada ação com os
// planos e o checkpoint, as perguntas do agente para responder ali mesmo e um só botão cheio: «Aplicar na folha».
// À direita, a memória que o agente usou, as ligações e um pedaço do grafo.
import { txt, ret, fio, botao, rotulo, etiqueta, cartao, caixinha, losango, circulo, quadro } from "./svg.mjs";
import { W, LAT, barra, lateral } from "./moldura.mjs";

const H = 1110, X = LAT + 40, XF = 960, XD = 992, WD = W - 24 - XD;

/** Dia de novembro → x da mini régua (01/11 a 30/11 entre X e XF). */
const dx = (d) => X + ((d - 1) / 30) * (XF - X);

function regua(y) {
  const out = [rotulo(X, y, "O que muda na folha · 2 ações novas · 1 alterada", XF - X)];
  const yr = y + 34;
  out.push(txt(X, yr - 22, "NOV", { tam: 10, c: "tinta" }), fio(X, yr, XF, yr, { c: "tinta" }));
  for (const d of [1, 8, 15, 22, 29]) out.push(fio(dx(d), yr - 6, dx(d), yr, { c: "tinta" }), txt(dx(d) + 4, yr - 10, String(d).padStart(2, "0"), { tam: 9, c: "medio" }));
  out.push(fio(dx(1.5), yr, dx(1.5), y + 150, { c: "tinta", dash: "16 4 2 4" }));
  // a coleta (alterada: ganha um plano) e as duas novas, tracejadas até você aplicar
  const barras = [[2, 9.9, "1.1 Coleta dos medidores", "alterada", [9]], [10, 20.9, "1.2 Painel por sala", "nova", [20]], [21, 28.9, "1.3 Alerta de pico", "nova", [28]]];
  barras.forEach(([a, b, nome, tipo, cps], i) => {
    const by = yr + 22 + i * 34, x0 = dx(a), w = dx(b) - x0;
    out.push(txt(x0, by - 4, nome, { tam: 10, c: "medio" }));
    out.push(ret(x0, by, w, 16, { fill: "papel", stroke: tipo === "nova" ? "tinta" : "medio", dash: tipo === "nova" ? "5 3" : undefined, sw: 1.2 }));
    for (const c of cps) out.push(losango(dx(c + 0.5), by + 8, 5));
    out.push(etiqueta(x0 + w + 8, by + 12, tipo, { cheia: tipo === "nova" })[0]);
  });
  return out.join("");
}

function acaoAberta(y) {
  const w = XF - X, out = [cartao(X, y, w, 176, { marcado: false })];
  out.push(caixinha(X + 18, y + 18, true), etiqueta(X + 42, y + 29, "nova")[0]);
  out.push(txt(X + 92, y + 29, "1.2 · Painel por sala", { tam: 13, caixa: true, peso: 700, trilha: 0.06 }));
  out.push(txt(X + w - 18, y + 29, "10/11 → 20/11 · 11 dias", { tam: 10, caixa: true, c: "medio", fim: true }));
  out.push(txt(X + 42, y + 50, "Mostra o consumo de cada sala, por dia e por semana, a partir das leituras guardadas na 1.1.", { tam: 11, c: "medio" }));
  const xm = X + w * 0.58;
  out.push(rotulo(X + 42, y + 78, "Planos · 3", xm - X - 66), rotulo(xm, y + 78, "Checkpoint", X + w - 18 - xm));
  [["Somar as leituras por sala", "~4 h"], ["Uma barra por sala no painel", "~6 h"], ["Mostrar o dia e a semana", "~2 h"]].forEach(([t, h], i) => {
    const py = y + 100 + i * 22;
    out.push(caixinha(X + 42, py - 10), txt(X + 64, py, t, { tam: 11.5 }), txt(xm - 24, py, h, { tam: 10, c: "medio", fim: true }));
  });
  out.push(losango(xm + 6, y + 96, 6), txt(xm + 20, y + 100, "Painel no ar · 20/11", { tam: 11.5 }));
  out.push(txt(xm, y + 128, "Instruções: ler a memória", { tam: 10.5, c: "medio" }), txt(xm, y + 144, "«Salas do escritório» e a", { tam: 10.5, c: "medio" }), txt(xm, y + 160, "Decisão 04 antes de começar.", { tam: 10.5, c: "medio" }));
  return out.join("");
}

function acaoFechada(y, tag, nome, resumo, cheia) {
  const w = XF - X;
  return ret(X, y, w, 34, { fill: "#0c264b", stroke: "baixo" }) + caixinha(X + 18, y + 10, true) + etiqueta(X + 42, y + 21, tag, { cheia })[0] +
    txt(X + 118, y + 21, nome, { tam: 12, caixa: true, peso: 700, trilha: 0.06 }) + txt(X + w - 18, y + 21, resumo, { tam: 10.5, c: "medio", fim: true });
}

function perguntas(y) {
  const w = XF - X, out = [rotulo(X, y, "Perguntas do Claude · 2 · a resposta volta para ele", w)];
  const y1 = y + 18;
  out.push(cartao(X, y1, w, 104), etiqueta(X + 18, y1 + 28, "P1")[0]);
  out.push(txt(X + 52, y1 + 28, "De quanto em quanto tempo ler os medidores?", { tam: 13, peso: 500 }));
  let bx = X + 52;
  for (const [t, op] of [["5 min", {}], ["15 min · recomendado", { ativo: true }], ["30 min", {}]]) { const [s, bw] = botao(bx, y1 + 42, t, { mini: true, ...op }); out.push(s); bx += bw + 8; }
  out.push(txt(X + 52, y1 + 88, "O Claude recomenda 15 min: a memória «Rede do 2º andar» diz que o switch cai com tráfego alto.", { tam: 10.5, c: "medio" }));
  const y2 = y1 + 116;
  out.push(cartao(X, y2, w, 64), etiqueta(X + 18, y2 + 26, "P2")[0], txt(X + 52, y2 + 26, "Quem recebe o alerta de pico?", { tam: 13, peso: 500 }));
  out.push(ret(X + 52, y2 + 36, w - 70, 20, { fill: "#071a33", stroke: "baixo" }), txt(X + 60, y2 + 50, "Escreva a resposta…", { tam: 10.5, c: "baixo" }));
  return out.join("");
}

function direita() {
  const out = [rotulo(XD, 104, "Memória que o Claude usou · 3", WD)];
  [["Salas do escritório", "projeto", "8 salas em 2 andares; o 2º tem o switch velho."], ["Rede do 2º andar", "projeto", "o switch cai com tráfego alto: ler devagar."], ["Revisar antes de aplicar", "você", "nada vai à folha sem você aplicar."]].forEach(([t, tipo, d], i) => {
    const y = 120 + i * 66;
    out.push(ret(XD, y, WD, 56, { fill: "#0c264b", stroke: "baixo" }), txt(XD + 12, y + 20, t, { tam: 12, peso: 500, max: WD - 80 }), etiqueta(XD + WD - 12 - 62, y + 19, tipo, { cheia: false, c: "medio" })[0]);
    out.push(txt(XD + 12, y + 40, d, { tam: 10, c: "medio", max: WD - 24 }));
  });
  out.push(rotulo(XD, 340, "Ligações · 4", WD));
  [["←", "Decisão 04 · Banco local"], ["←", "1.1 Coleta dos medidores"], ["→", "Página do blueprint"], ["→", "Portal do fornecedor"]].forEach(([s, t], i) => {
    out.push(txt(XD, 366 + i * 22, s, { tam: 11, c: "medio" }), txt(XD + 18, 366 + i * 22, t, { tam: 11.5, max: WD - 20 }), fio(XD + 18, 369 + i * 22, XD + 18 + Math.min(WD - 20, t.length * 6.7), 369 + i * 22, { c: "medio", op: 0.6 }));
  });
  out.push(rotulo(XD, 478, "No grafo", WD), ret(XD, 492, WD, 170, { fill: "#071a33", op: 0.55, stroke: "baixo" }));
  const c = [XD + WD / 2, 576], nos = [[-82, -50, "m"], [-92, 18, "m"], [-30, 62, "m"], [70, -48, "d"], [88, 16, "a"], [40, 62, "a"], [0, -62, "b"]];
  for (const [x, y] of nos) out.push(fio(c[0], c[1], c[0] + x, c[1] + y, { c: "medio", op: 0.7 }));
  for (const [x, y, t] of nos) {
    const px = c[0] + x, py = c[1] + y;
    out.push(t === "m" ? circulo(px, py, 6, { fill: "papel", stroke: "medio" }) : t === "d" ? losango(px, py, 7) : t === "b" ? ret(px - 9, py - 6, 18, 12, { fill: "papel", stroke: "tinta" }) : ret(px - 7, py - 5, 14, 10, { fill: "papel", stroke: "medio", dash: "3 2" }));
  }
  out.push(circulo(c[0], c[1], 11, { fill: "tinta", stroke: "tinta" }), txt(c[0], c[1] + 28, "nº 3", { tam: 9, c: "tinta", meio: true }));
  out.push(botao(XD, 676, "Abrir o grafo", { mini: true })[0]);
  out.push(rotulo(XD, 740, "Propostas anteriores", WD), txt(XD, 764, "nº 2 · aplicada · 29/10", { tam: 11, c: "medio" }), txt(XD, 786, "nº 1 · descartada · 27/10", { tam: 11, c: "medio" }));
  return out.join("");
}

export function telaPlanejar() {
  const out = [barra("Planejar · Monitor de consumo", "Proposta nº 3", "Esperando você · chegou 09:40", [["Memória"], ["Grafo"]]), lateral(H, "planejar")];
  out.push(txt(X, 104, "Proposta nº 3 · escrita pelo Claude · 01/11 09:40 · ~/Projects/monitor", { tam: 10, caixa: true, c: "medio", trilha: 0.12 }));
  out.push(txt(X, 142, "Painel de consumo por sala", { tam: 26, peso: 500, trilha: -0.02 }));
  out.push(txt(X, 172, "O Claude leu a memória do blueprint e a Decisão 04 e propõe duas ações novas e um ajuste na coleta.", { tam: 12, c: "medio" }));
  out.push(txt(X, 192, "Responda as perguntas e aplique: o que estiver marcado vai para a folha.", { tam: 12, c: "medio" }));
  out.push(cartao(X, 212, XF - X, 118), rotulo(X + 22, 240, "O plano em três linhas"));
  ["Coleta: calibrar os medidores do 2º andar antes da primeira semana de dados.", "Painel por sala: o consumo de cada sala, por dia e por semana, de 10 a 20/11.", "Alerta de pico: avisar quando uma sala passa do limite, até 28/11."].forEach((t, i) => {
    out.push(txt(X + 22, 268 + i * 22, `${i + 1}`, { tam: 12, peso: 700 }), txt(X + 42, 268 + i * 22, t, { tam: 12 }));
  });
  out.push(regua(360));
  out.push(acaoFechada(530, "alterada", "1.1 · Coleta dos medidores", "+ 1 plano: calibrar os medidores do 2º andar", false));
  out.push(acaoAberta(574));
  out.push(acaoFechada(760, "nova", "1.3 · Alerta de pico", "21/11 → 28/11 · 2 planos · 1 checkpoint", true));
  out.push(perguntas(830));
  out.push(fio(X, 1046, XF, 1046, { c: "tinta", op: 0.7 }));
  const [b1, w1] = botao(X, 1060, "Aplicar na folha · 3 ações", { cheio: true });
  out.push(b1, botao(X + w1 + 10, 1060, "Descartar")[0], txt(XF, 1080, "Ctrl+Enter aplica · o desmarcado não entra", { tam: 10, c: "medio", fim: true }));
  out.push(direita());
  return quadro("pl", W, H, out.join(""), "Planejar: a proposta nº 3 como página, com o que muda na folha, as ações, as perguntas do Claude e a memória que ele usou");
}

