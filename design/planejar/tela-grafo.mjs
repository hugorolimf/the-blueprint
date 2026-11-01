// O Grafo do workspace: tudo o que está ligado (blueprints, ações, páginas, memórias, decisões e as pastas de código), desenhado
// como a folha, em traço sobre o papel. Clicar abre a página; a escolhida mostra um resumo no canto.
import { txt, ret, fio, botao, rotulo, losango, circulo, quadro, COR } from "./svg.mjs";
import { W, LAT, barra, lateral } from "./moldura.mjs";

const H = 800;

/** As formas de cada tipo: blueprint (placa), ação (barra), página (folha dobrada), memória (círculo), decisão (losango), código (pasta). */
const FORMA = {
  blueprint: (x, y, m) => ret(x - 16, y - 10, 32, 20, { fill: m ? "tinta" : "papel", stroke: "tinta", sw: 1.4 }),
  acao: (x, y, m, nova) => ret(x - 14, y - 5, 28, 10, { fill: "papel", stroke: m ? "tinta" : "medio", dash: nova ? "4 2" : undefined }),
  pagina: (x, y) => `<path d="M${x - 7} ${y - 9}H${x + 3}L${x + 7} ${y - 5}V${y + 9}H${x - 7}Z" fill="${COR.papel}" stroke="${COR.tinta}"/>`,
  memoria: (x, y, m) => circulo(x, y, m ? 9 : 6, { fill: m ? "tinta" : "papel", stroke: m ? "tinta" : "medio", sw: m ? 2 : 1 }),
  decisao: (x, y) => losango(x, y, 7),
  codigo: (x, y) => `<path d="M${x - 9} ${y - 6}H${x - 3}L${x - 1} ${y - 3}H${x + 9}V${y + 7}H${x - 9}Z" fill="${COR.papel}" stroke="${COR.medio}"/>`,
};

// [id, tipo, x, y, nome, marcada?, nova?]
const NOS = [
  ["ws", "blueprint", 760, 130, "Laboratório de ideias"],
  ["mc", "blueprint", 560, 360, "Monitor de consumo"], ["pf", "blueprint", 1010, 360, "Portal do fornecedor"],
  ["a11", "acao", 430, 300, "1.1 Coleta"], ["a12", "acao", 440, 470, "1.2 Painel por sala", false, true], ["a13", "acao", 560, 540, "1.3 Alerta de pico", false, true],
  ["pl", "pagina", 690, 470, "Planejar nº 3"], ["pg", "pagina", 670, 270, "Página do blueprint"],
  ["m1", "memoria", 360, 410, "Salas do escritório"], ["m2", "memoria", 340, 220, "Rede do 2º andar", true], ["m3", "memoria", 760, 560, "Prazo da diretoria"],
  ["m4", "memoria", 790, 300, "Texto em pt-BR"], ["d4", "decisao", 560, 220, "Decisão 04 · Banco local"],
  ["c1", "codigo", 340, 600, "~/Projects/monitor"], ["c2", "codigo", 1150, 520, "~/Projects/portal"],
  ["b1", "acao", 1110, 280, "1.1 Login do fornecedor"], ["b2", "acao", 940, 500, "1.2 Envio de notas"], ["m5", "memoria", 1170, 410, "SSO da empresa"],
];
const LIGA = [["ws", "mc"], ["ws", "pf"], ["mc", "a11"], ["mc", "a12"], ["mc", "a13"], ["mc", "pl"], ["mc", "pg"], ["pl", "a12"], ["pl", "a13"], ["pl", "m1"], ["pl", "m2"], ["pl", "m3"],
  ["a11", "m2"], ["a11", "c1"], ["a12", "m1"], ["mc", "d4"], ["d4", "a11"], ["m4", "mc"], ["m4", "pf"], ["pf", "b1"], ["pf", "b2"], ["b1", "m5"], ["pf", "c2"], ["m2", "d4"]];

export function telaGrafo() {
  const out = [barra("Grafo · Laboratório de ideias", "Tudo ligado", "18 páginas · 24 ligações", [["Planejar"], ["Memória"]]), lateral(H, "grafo")];
  const x0 = LAT + 32;
  let bx = x0;
  for (const [t, ativo] of [["Tudo", true], ["Só este blueprint", false], ["Memória", false], ["Código", false]]) {
    const [s, w] = botao(bx, 84, t, { mini: true, ativo });
    out.push(s);
    bx += w + 8;
  }
  const pos = Object.fromEntries(NOS.map((n) => [n[0], n]));
  const marcada = new Set(LIGA.filter(([a, b]) => a === "m2" || b === "m2").flat());
  for (const [a, b] of LIGA) {
    const [, , xa, ya] = pos[a], [, , xb, yb] = pos[b], forte = a === "m2" || b === "m2";
    out.push(fio(xa, ya, xb, yb, { c: forte ? "tinta" : "baixo", sw: forte ? 1.4 : 1, op: forte ? 1 : 0.8 }));
  }
  for (const [id, tipo, x, y, nome, m, nova] of NOS) {
    const apagado = !marcada.has(id) && id !== "m2";
    out.push(`<g${apagado ? ' opacity=".55"' : ""}>${FORMA[tipo](x, y, m, nova)}${txt(x, y + (tipo === "blueprint" ? 26 : 22), nome, { tam: tipo === "blueprint" ? 11 : 10, caixa: tipo === "blueprint", peso: tipo === "blueprint" ? 700 : 400, c: tipo === "blueprint" || m ? "tinta" : "medio", meio: true })}</g>`);
  }
  // a legenda
  const ly = H - 40;
  out.push(rotulo(x0, ly - 26, "Legenda"));
  let lx = x0;
  for (const [tipo, nome] of [["blueprint", "blueprint"], ["acao", "ação"], ["pagina", "página"], ["memoria", "memória"], ["decisao", "decisão"], ["codigo", "pasta de código"]]) {
    const f = tipo === "blueprint" ? 18 : 14;
    out.push(FORMA[tipo](lx + f, ly, false), txt(lx + 2 * f + 6, ly + 4, nome, { tam: 10.5, c: "medio" }));
    lx += 2 * f + 24 + nome.length * 6.5;
  }
  // o resumo da escolhida
  const cx = W - 24 - 300, cy = H - 214;
  out.push(ret(cx, cy, 300, 190, { fill: "#0c264b", stroke: "tinta" }), `<path d="M${cx} ${cy + 22}V${cy}H${cx + 22}" fill="none" stroke="${COR.tinta}" stroke-width="2"/>`);
  out.push(txt(cx + 16, cy + 26, "Memória · projeto", { tam: 10, caixa: true, c: "medio", trilha: 0.14 }), txt(cx + 16, cy + 52, "Rede do 2º andar", { tam: 15, peso: 500 }));
  out.push(txt(cx + 16, cy + 76, "O switch velho cai com tráfego alto:", { tam: 11, c: "medio" }), txt(cx + 16, cy + 93, "ler os medidores devagar.", { tam: 11, c: "medio" }));
  out.push(txt(cx + 16, cy + 120, "Ligada a 4: 1.1 Coleta, Decisão 04,", { tam: 10.5, c: "medio" }), txt(cx + 16, cy + 136, "Planejar nº 3, Monitor de consumo.", { tam: 10.5, c: "medio" }));
  out.push(botao(cx + 16, cy + 152, "Abrir", { mini: true })[0]);
  out.push(txt(W - 24, 104, "Arraste para mover · roda aproxima · clique abre", { tam: 10, c: "medio", fim: true }));
  return quadro("gr", W, H, out.join(""), "Grafo do workspace: blueprints, ações, páginas, memórias, decisões e pastas de código ligados; a memória escolhida acende as ligações dela");
}
