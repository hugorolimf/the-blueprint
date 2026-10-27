// Auditoria do público cliente e a impressão (sha256) do conteúdo que cada figura explica. L.TERMOS vem de termos-tecnicos.json (o empacotador o injeta).
(function (L) {
  // ---- sha256 síncrono em JS puro (SubtleCrypto é assíncrono e não existe fora de contexto seguro) ----
  const K = new Uint32Array([
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
  ]);
  const utf8 = (texto) => {
    const bytes = [];
    for (const c of unescape(encodeURIComponent(texto))) bytes.push(c.charCodeAt(0));
    return bytes;
  };
  const sha256 = (texto) => {
    const bytes = utf8(texto), bits = bytes.length * 8;
    bytes.push(0x80);
    while (bytes.length % 64 !== 56) bytes.push(0);
    for (let i = 7; i >= 0; i--) bytes.push(i >= 4 ? 0 : (bits >>> (i * 8)) & 255);
    const h = new Uint32Array([0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
    const w = new Uint32Array(64), giro = (x, n) => (x >>> n) | (x << (32 - n));
    for (let o = 0; o < bytes.length; o += 64) {
      for (let i = 0; i < 16; i++) w[i] = (bytes[o + i * 4] << 24) | (bytes[o + i * 4 + 1] << 16) | (bytes[o + i * 4 + 2] << 8) | bytes[o + i * 4 + 3];
      for (let i = 16; i < 64; i++) {
        const s0 = giro(w[i - 15], 7) ^ giro(w[i - 15], 18) ^ (w[i - 15] >>> 3), s1 = giro(w[i - 2], 17) ^ giro(w[i - 2], 19) ^ (w[i - 2] >>> 10);
        w[i] = (w[i - 16] + s0 + w[i - 7] + s1) | 0;
      }
      let [a, b, c, d, e, f, g, hh] = h;
      for (let i = 0; i < 64; i++) {
        const t1 = (hh + (giro(e, 6) ^ giro(e, 11) ^ giro(e, 25)) + ((e & f) ^ (~e & g)) + K[i] + w[i]) | 0;
        const t2 = ((giro(a, 2) ^ giro(a, 13) ^ giro(a, 22)) + ((a & b) ^ (a & c) ^ (b & c))) | 0;
        hh = g; g = f; f = e; e = (d + t1) | 0; d = c; c = b; b = a; a = (t1 + t2) | 0;
      }
      h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
    }
    return Array.from(h, (x) => x.toString(16).padStart(8, "0")).join("");
  };
  // JSON com as chaves em ordem: o mesmo conteúdo dá sempre a mesma impressão
  const estavel = (v) => (v && typeof v === "object" ? (Array.isArray(v) ? "[" + v.map(estavel).join(",") + "]" : "{" + Object.keys(v).sort().map((k) => JSON.stringify(k) + ":" + estavel(v[k])).join(",") + "}") : JSON.stringify(v));
  L.sha256 = sha256;
  L.impressao = (valor) => sha256(estavel(valor));

  // ---- auditoria: nenhuma palavra técnica nem horas por plano no texto que o cliente lê ----
  const textos = (v, caminho, saida) => {
    if (typeof v === "string") saida.push([caminho, v]);
    else if (Array.isArray(v)) v.forEach((x, i) => textos(x, `${caminho}[${i}]`, saida));
    else if (v && typeof v === "object") for (const k of Object.keys(v)) if (k !== "interno") textos(v[k], caminho ? `${caminho}.${k}` : k, saida);
    return saida;
  };
  const escapa = (s) => s.replace(/[.*+?^${}()|[\]\\\/]/g, "\\$&");
  let regras = null;
  const monta = () => (regras ||= (L.TERMOS || []).map((t) => [t, new RegExp(`(?<![\\p{L}\\p{N}_-])${escapa(t.toLowerCase())}(?![\\p{L}\\p{N}_-])`, "u")]));
  const HORAS = /(?<![\p{L}\p{N}_])\d+([.,]\d+)?\s*(h|hs|horas|hours|hrs)(?![\p{L}\p{N}_])/iu;
  // devolve [{ termo, campo, trecho }]; vazio = limpo. O campo `interno` não conta (só existe no público interno).
  L.auditarCliente = (conteudo) => {
    const achados = [];
    for (const [campo, texto] of textos(conteudo, "", [])) {
      const baixo = texto.toLowerCase();
      for (const [termo, re] of monta()) {
        const m = re.exec(baixo);
        if (m) achados.push({ termo, campo, trecho: texto.slice(Math.max(0, m.index - 20), m.index + termo.length + 20) });
      }
      const h = HORAS.exec(texto);
      if (h) achados.push({ termo: "horas por plano", campo, trecho: texto.slice(Math.max(0, h.index - 20), h.index + h[0].length + 20) });
    }
    return achados;
  };
})(L);
