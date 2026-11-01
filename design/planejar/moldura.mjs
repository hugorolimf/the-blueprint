// A moldura comum das telas do protótipo: a barra de cima (a mesma do «Novo blueprint» nativo) e a coluna da esquerda com as páginas
// do workspace, como a árvore de arquivos do Obsidian: cada blueprint tem a sua página, o Planejar, a Memória, as Decisões e as ações.
import { txt, ret, fio, botao, rotulo, etiqueta, COR } from "./svg.mjs";

export const W = 1280;
export const LAT = 252;

/** A barra de cima: «← Voltar à folha», o rótulo e o título da página, o estado e os botões da direita. */
export function barra(rot, titulo, estado, botoes = []) {
  const out = [ret(0, 0, W, 64, { fill: COR.barra, op: 0.92 }), ret(0, 62, W, 2, { fill: "tinta" })];
  const [b, bw] = botao(24, 16, "← Voltar à folha");
  out.push(b);
  const x = 24 + bw + 16;
  out.push(txt(x, 25, rot, { tam: 10, caixa: true, c: "medio", trilha: 0.16 }), txt(x, 46, titulo, { tam: 17, caixa: true, peso: 700, trilha: 0.06 }));
  let bx = W - 24;
  for (const [t, op] of [...botoes].reverse()) {
    const [, l] = botao(0, 0, t, op);
    bx -= l;
    out.push(botao(bx, 16, t, op)[0]);
    bx -= 10;
  }
  if (estado) out.push(txt(bx - 6, 36, estado, { tam: 10, caixa: true, c: "medio", trilha: 0.12, fim: true }));
  return out.join("");
}

/** Uma linha da árvore: recuo, sinal (▾ ▸ ·), o nome, um número à direita; `ativa` fica invertida como o botão marcado. */
function linha(y, nivel, sinal, nome, { ativa = false, num, tag, fraca = false } = {}) {
  const x = 18 + nivel * 16, out = [];
  if (ativa) out.push(ret(8, y - 15, LAT - 16, 22, { fill: "tinta" }));
  const c = ativa ? "papel" : fraca ? "medio" : "tinta";
  out.push(txt(x, y, sinal, { tam: 11, c: ativa ? "papel" : "medio" }), txt(x + 16, y, nome, { tam: 12, c, peso: ativa ? 700 : 400, max: LAT - x - 60 }));
  if (num != null) out.push(txt(LAT - 18, y, num, { tam: 10, c: ativa ? "papel" : "medio", fim: true }));
  if (tag) out.push(etiqueta(LAT - 18 - 48, y - 1, tag, { cheia: false, c: ativa ? "papel" : "medio" })[0]);
  return out.join("");
}

/** A coluna da esquerda. `ativa`: "blueprint" | "planejar" | "memoria" | "decisoes" | "grafo". */
export function lateral(h, ativa) {
  const out = [ret(0, 64, LAT, h - 64, { fill: "#0c264b", op: 0.72 }), fio(LAT, 64, LAT, h, { c: "baixo" })];
  out.push(rotulo(18, 96, "Laboratório de ideias", LAT - 36));
  let y = 128;
  const l = (...a) => { out.push(linha(y, ...a)); y += 26; };
  l(0, "▾", "Monitor de consumo", {});
  l(1, "·", "Página do blueprint", { ativa: ativa === "blueprint" });
  l(1, "·", "Planejar", { ativa: ativa === "planejar", tag: "nº 3" });
  l(1, "·", "Memória", { ativa: ativa === "memoria", num: 12 });
  l(1, "·", "Decisões", { ativa: ativa === "decisoes", num: 4 });
  l(1, "▸", "1.1 Coleta dos medidores", {});
  l(1, "▸", "1.2 Painel por sala", { fraca: true, tag: "nova" });
  l(0, "▸", "Portal do fornecedor", {});
  y += 14;
  out.push(rotulo(18, y, "Do workspace", LAT - 36));
  y += 32;
  l(0, "·", "Grafo", { ativa: ativa === "grafo" });
  l(0, "·", "Memória do workspace", { num: 5 });
  l(0, "·", "Como rodamos local", {});
  l(0, "·", "Convenções do repositório", {});
  out.push(fio(18, h - 58, LAT - 18, h - 58, { c: "baixo", op: 0.8 }), botao(18, h - 46, "+ Nova página", { mini: true })[0]);
  out.push(txt(LAT - 18, h - 29, "Ctrl+K busca", { tam: 10, c: "medio", fim: true }));
  return out.join("");
}
