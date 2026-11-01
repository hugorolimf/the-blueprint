// A Memória do blueprint: o que o agente precisa saber, no formato das memórias do Claude Code (um fato por nota, com o porquê e
// como aplicar, e um tipo: você, como trabalhar, projeto, referência). O agente lê no começo da sessão e escreve quando aprende;
// o Hugo corrige ou apaga aqui. À direita, a memória escolhida inteira, com as ligações.
import { txt, ret, fio, botao, rotulo, etiqueta, cartao, quadro } from "./svg.mjs";
import { W, LAT, barra, lateral } from "./moldura.mjs";

const H = 800, X = LAT + 40, XF = 892, XD = 924, WD = W - 24 - XD;

const MEMORIAS = [
  ["Salas do escritório", "projeto", ["8 salas em 2 andares; a 204 e a 205", "dividem o mesmo medidor."], "31/10 · usada em 3 propostas"],
  ["Rede do 2º andar", "projeto", ["O switch velho cai com tráfego alto:", "ler os medidores devagar."], "30/10 · usada em 2 propostas"],
  ["Revisar antes de aplicar", "como trabalhar", ["Nada vai para a folha sem o Hugo", "aplicar no Planejar."], "28/10 · você escreveu"],
  ["Texto em pt-BR, código em inglês", "como trabalhar", ["A interface e as mensagens em pt-BR;", "nomes no código seguem o projeto."], "27/10 · usada em 5 propostas"],
  ["Medidores falam Modbus TCP", "referência", ["Manual do fabricante: porta 502,", "registro 30001 = kWh acumulado."], "29/10 · usada em 1 proposta"],
  ["Prazo da diretoria", "projeto", ["Painel por sala apresentado na", "reunião de 30/11."], "01/11 · nova hoje"],
];

function cartaoMemoria(x, y, w, [titulo, tipo, linhas, pe], marcado) {
  const out = [cartao(x, y, w, 112, { marcado })];
  out.push(txt(x + 16, y + 28, titulo, { tam: 13, peso: 500, max: w - 130 }));
  const ew = etiqueta(0, 0, tipo, { cheia: false })[1];
  out.push(etiqueta(x + w - 16 - ew, y + 27, tipo, { cheia: false, c: marcado ? "tinta" : "medio" })[0]);
  linhas.forEach((l, i) => out.push(txt(x + 16, y + 52 + i * 17, l, { tam: 11, c: "medio" })));
  out.push(fio(x + 16, y + 88, x + w - 16, y + 88, { c: "baixo", op: 0.7 }), txt(x + 16, y + 103, pe, { tam: 9.5, caixa: true, c: "medio", trilha: 0.1 }));
  return out.join("");
}

function detalhe() {
  const out = [ret(XD - 16, 64, W - XD + 16, H - 64, { fill: "#0c264b", op: 0.8 }), fio(XD - 16, 64, XD - 16, H, { c: "baixo" })];
  out.push(txt(XD, 100, "Memória · projeto", { tam: 10, caixa: true, c: "medio", trilha: 0.14 }));
  out.push(txt(XD, 128, "Salas do escritório", { tam: 20, peso: 500 }));
  const corpo = ["O escritório tem 8 salas em 2 andares.", "A 204 e a 205 dividem o mesmo medidor:", "o consumo das duas sai somado e é", "dividido pela área (60% e 40%)."];
  corpo.forEach((l, i) => out.push(txt(XD, 158 + i * 19, l, { tam: 12 })));
  const bloco = (y, rot, linhas) => { out.push(rotulo(XD, y, rot, WD)); linhas.forEach((l, i) => out.push(txt(XD, y + 26 + i * 18, l, { tam: 11.5, c: "medio" }))); };
  bloco(258, "Por quê", ["O painel por sala mostrava a 205", "vazia; o Hugo explicou em 31/10."]);
  bloco(344, "Como aplicar", ["Toda soma por sala usa a tabela", "de áreas; nunca ler a 205 sozinha."]);
  out.push(rotulo(XD, 430, "Ligações · 3", WD));
  ["Rede do 2º andar", "Proposta nº 3", "1.2 Painel por sala"].forEach((t, i) => {
    const y = 456 + i * 22;
    out.push(txt(XD, y, "→", { tam: 11, c: "medio" }), txt(XD + 18, y, t, { tam: 11.5 }), fio(XD + 18, y + 3, XD + 18 + t.length * 6.75, y + 3, { c: "medio", op: 0.6 }));
  });
  const [b, bw] = botao(XD, 540, "Editar");
  out.push(b, botao(XD + bw + 10, 540, "Apagar")[0]);
  out.push(txt(XD, 604, "Escrita pelo Claude em 31/10.", { tam: 10.5, c: "medio" }), txt(XD, 622, "Corrigida por você em 01/11.", { tam: 10.5, c: "medio" }));
  out.push(txt(XD, 660, "O agente lê pelo MCP (ler_memoria)", { tam: 10.5, c: "medio" }), txt(XD, 678, "e escreve com lembrar.", { tam: 10.5, c: "medio" }));
  return out.join("");
}

export function telaMemoria() {
  const out = [barra("Memória · Monitor de consumo", "12 memórias", "O Claude lê antes de planejar", [["Planejar"], ["Grafo"]]), lateral(H, "memoria")];
  out.push(txt(X, 104, "O que o Claude precisa saber deste blueprint. Ele lê no começo de cada", { tam: 12, c: "medio" }));
  out.push(txt(X, 124, "sessão e escreve quando aprende algo. Você corrige ou apaga aqui.", { tam: 12, c: "medio" }));
  let bx = X;
  for (const [t, ativo] of [["Tudo · 12", true], ["Você · 3", false], ["Como trabalhar · 4", false], ["Projeto · 4", false], ["Referências · 1", false]]) {
    const [s, w] = botao(bx, 146, t, { mini: true, ativo });
    out.push(s);
    bx += w + 8;
  }
  const cw = (XF - X - 16) / 2;
  MEMORIAS.forEach((m, i) => out.push(cartaoMemoria(X + (i % 2) * (cw + 16), 192 + Math.floor(i / 2) * 128, cw, m, i === 0)));
  out.push(txt(X, 590, "+ 6 memórias mais antigas", { tam: 11, c: "medio" }), botao(X, 606, "+ Nova memória", { mini: true })[0]);
  out.push(detalhe());
  return quadro("me", W, H, out.join(""), "Memória do blueprint: cartões por tipo e a memória escolhida inteira, com o porquê, como aplicar e as ligações");
}
