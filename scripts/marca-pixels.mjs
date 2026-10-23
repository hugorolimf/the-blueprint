// A marca do terminal desenhada pixel a pixel: o emblema isométrico do app vira uma mancha azul em 32×16 células,
// então aqui a folha de blueprint fica de frente, com barras de cronograma e um checkpoint amarelo (mesma paleta do app).
const COR = { borda: 0x8dbaea, papel: 0x124a9c, grade: 0x1c5db8, linha: 0x5f8fd0, barra: 0xeaf4ff, tenue: 0x8dbaea, ponto: 0xffd24a };

// [x0, y0, x1, y1, cor] em pixels (x1 e y1 inclusos); as peças de baixo vêm primeiro.
const FOLHA_32 = [
  [2, 2, 29, 29, "papel"], [11, 10, 11, 27, "grade"], [17, 10, 17, 27, "grade"], [23, 10, 23, 27, "grade"],
  [6, 5, 15, 6, "barra"], [18, 5, 25, 6, "tenue"], [5, 9, 26, 9, "linha"],
  [6, 12, 14, 13, "barra"], [11, 17, 22, 18, "barra"], [17, 22, 25, 23, "barra"],
  [24, 11, 24, 11, "ponto"], [23, 12, 25, 12, "ponto"], [22, 13, 26, 13, "ponto"], [23, 14, 25, 14, "ponto"], [24, 15, 24, 15, "ponto"],
];
const FOLHA_16 = [
  [1, 1, 14, 14, "papel"],
  [3, 3, 7, 3, "barra"], [10, 3, 12, 3, "tenue"], [3, 5, 12, 5, "linha"],
  [3, 7, 7, 7, "barra"], [6, 9, 10, 9, "barra"], [8, 11, 12, 11, "barra"],
  [11, 6, 11, 6, "ponto"], [10, 7, 12, 7, "ponto"], [11, 8, 11, 8, "ponto"],
];

/** tam 32 ou 16 → { tam, rgba } com a folha de frente, borda de 1 px e os cantos cortados; fora dela, transparente. */
export function desenharMarca(tam) {
  const rgba = new Uint8Array(tam * tam * 4);
  const pixel = (x, y, cor) => { const i = (y * tam + x) * 4; rgba.set([cor >> 16, (cor >> 8) & 255, cor & 255, 255], i); };
  const [x0, y0, x1, y1] = tam === 32 ? [2, 2, 29, 29] : [1, 1, 14, 14];
  for (const [ax, ay, bx, by, nome] of tam === 32 ? FOLHA_32 : FOLHA_16)
    for (let y = ay; y <= by; y++) for (let x = ax; x <= bx; x++) pixel(x, y, COR[nome]);
  for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) {
    const canto = (x === x0 || x === x1) && (y === y0 || y === y1);
    if (canto) rgba.fill(0, (y * tam + x) * 4, (y * tam + x) * 4 + 4);
    else if (x === x0 || x === x1 || y === y0 || y === y1) pixel(x, y, COR.borda);
  }
  return { tam, rgba };
}
