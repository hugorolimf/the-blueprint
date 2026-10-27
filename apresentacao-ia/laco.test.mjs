import assert from "node:assert/strict";
import { carregar } from "./teste-comum.mjs";

const { L } = carregar();
// segue: aproxima do alvo sem passar, e em movimento calmo salta
let v = 0;
for (let i = 0; i < 30; i++) { const novo = L.segue(v, 10, 1 / 60, 0.2, false); assert.ok(novo >= v && novo <= 10); v = novo; }
assert.ok(v > 5 && v < 10);
assert.equal(L.segue(0, 10, 1 / 60, 0.2, true), 10);
// mola: chega ao alvo e para (devolve false), sem passar de 1,6x numa subida simples
const m = { x: 0, v: 0 };
let passos = 0, maximo = 0;
while (L.mola(m, 12, 1 / 60, 360, 24, false) && passos++ < 600) maximo = Math.max(maximo, m.x);
assert.equal(m.x, 12); assert.equal(m.v, 0); assert.ok(maximo < 12 * 1.6, "overshoot pequeno");
assert.equal(L.mola({ x: 0, v: 0 }, 5, 1 / 60, 360, 24, true), false);
// sem navegador não há matchMedia: calmo() é falso e o laço sem observador não lança
assert.equal(L.calmo(), false);
console.log("laco.test.mjs: ok");
