import assert from "node:assert/strict";
import { carregar } from "./teste-comum.mjs";

const { L } = carregar();
// sha256 conhecido e impressão estável (ordem das chaves não importa, o conteúdo importa)
assert.equal(L.sha256("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
assert.equal(L.sha256(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
assert.equal(L.impressao({ b: 1, a: [1, { y: 2, x: 3 }] }), L.impressao({ a: [1, { x: 3, y: 2 }], b: 1 }));
assert.notEqual(L.impressao({ a: 1 }), L.impressao({ a: 2 }));

// público cliente: palavra inteira, não prefixo (cliente não é cli, aplicativo não é api), e horas por plano
assert.equal(L.auditarCliente({ titulo: "Aplicativo para o cliente", texto: "O assistente atende no WhatsApp e cobra por Pix." }).length, 0);
const achados = L.auditarCliente({ titulo: "Plano com API e servidor", etapas: [{ texto: "Rodar o Webhook em 3 horas" }, { texto: "Levar 2h" }], interno: { x: "Rust e SQLite" } });
assert.equal(achados.map((a) => a.termo).sort().join(","), "api,horas por plano,horas por plano,servidor,webhook");
assert.ok(achados.every((a) => a.campo !== "interno.x"), "o campo interno não conta");
assert.equal(achados.find((a) => a.termo === "api").campo, "titulo");
assert.ok(L.auditarCliente({ t: "Banco de dados em nuvem" }).some((a) => a.termo === "banco de dados"), "frase com espaços");
console.log("auditoria.test.mjs: ok");
