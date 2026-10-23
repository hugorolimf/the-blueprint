import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { grande, pequena, png } from "../claude-plugin/plugins/blueprint-terminal/assets/marca.mjs";

const pasta = new URL("../claude-plugin/plugins/blueprint-terminal/assets/", import.meta.url);
const imagem = readFileSync(new URL("marca.png", pasta));
assert(imagem.length < 2 * 1024 * 1024);
assert.equal(imagem.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");
assert.deepEqual(Buffer.from(png, "base64"), imagem);
for (const { columns, rows, cells } of [grande, pequena]) {
  const bytes = Buffer.from(cells, "base64");
  assert.equal(bytes.length, columns * rows * 12);
  assert.deepEqual(bytes, readFileSync(new URL(`marca-${columns}x${rows}.cells`, pasta)));
  for (let i = 0; i < bytes.length; i += 12) {
    assert([0x20, 0x2580, 0x2584].includes(bytes.readUInt32LE(i)));
    if (bytes.readUInt32LE(i) === 0x2580) assert.notEqual(bytes.readUInt32LE(i + 4), 0x1000000);
    for (const color of [bytes.readUInt32LE(i + 4), bytes.readUInt32LE(i + 8)]) {
      assert(color <= 0xffffff || color === 0x1000000);
    }
  }
}
console.log("marca do terminal: PNG e células dos dois tamanhos válidos");
