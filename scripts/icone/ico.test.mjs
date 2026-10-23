// node scripts/icone/ico.test.mjs: confere o cabeçalho e o diretório do .ico montado por empacotarIco.
import assert from "node:assert/strict";
import { empacotarIco } from "./ico.mjs";

const pngs = [{ tam: 256, png: Buffer.from("aaaa") }, { tam: 16, png: Buffer.from("bb") }];
const ico = empacotarIco(pngs);
assert.equal(ico.readUInt16LE(2), 1, "tipo ícone");
assert.equal(ico.readUInt16LE(4), 2, "duas imagens");
assert.equal(ico.readUInt8(6), 0, "256 vira 0 no diretório");
assert.equal(ico.readUInt8(6 + 16), 16);
for (const [i, { png }] of pngs.entries()) {
  const tam = ico.readUInt32LE(6 + 16 * i + 8), pos = ico.readUInt32LE(6 + 16 * i + 12);
  assert.deepEqual(ico.subarray(pos, pos + tam), png, `imagem ${i} no lugar certo`);
}
assert.equal(ico.length, 6 + 32 + 6, "nada sobra no fim");
console.log("ico: ok");
