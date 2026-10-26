// A parte pura do terminal do app (web/js/terminal-logica.js, script clássico num contexto isolado, como o navegador faz): tema a partir dos
// tokens, contraste, fonte, rótulo e estado da aba, links, atalhos (F6 sai, Esc e Ctrl+C ficam com o Claude) e a fila de teclas.
//   node terminal-logica.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ctx = vm.createContext({ navigator: { languages: ["pt-BR"] }, location: { search: "" }, Intl });
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), ctx, { filename: "i18n.js" });
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "terminal-logica.js"), "utf8") + "\nthis.api = { termTema, termFonteValida, termRotulo, termLinks, termTecla, termEnviador, TERM_FONTE };", ctx);
const A = ctx.api;
const J = (x) => JSON.parse(JSON.stringify(x));

// ---- tema: as cores do app; todo texto ANSI passa de 4,5:1 sobre o papel azul (WCAG AA)
const tokens = Object.fromEntries([...readFileSync(join(import.meta.dirname, "web", "css", "tokens.css"), "utf8").matchAll(/--(\w+):(#[0-9a-f]{6})/gi)].map((m) => [m[1], m[2]]));
const tema = A.termTema(tokens);
assert.equal(tema.background, tokens.paper);
assert.equal(tema.foreground, tokens.ink);
assert.equal(tema.cursor, tokens.ink);
const lum = (hex) => { const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)); return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]; };
const contraste = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
for (const k of ["foreground", "red", "green", "yellow", "blue", "magenta", "cyan", "white", "brightRed", "brightGreen", "brightYellow", "brightBlue", "brightMagenta", "brightCyan", "brightWhite"])
  assert(contraste(tema[k], tema.background) >= 4.5, `${k} ${tema[k]} tem contraste ${contraste(tema[k], tema.background).toFixed(2)} sobre o papel`);

// ---- fonte: entre 10 e 28, padrão 14 para lixo
assert.equal(A.termFonteValida("abc"), 14);
assert.equal(A.termFonteValida(3), 10);
assert.equal(A.termFonteValida(99), 28);
assert.equal(A.termFonteValida("16"), 16);

// ---- rótulo e estado da aba: o último aviso do agente na mesma ação; sem aviso, "Rodando"; terminada, com o código
const s = { id: "t1", titulo: "1.2 · Tela", acaoId: "a1", blueprintId: "b1", viva: true };
const ESTADOS = { escrevendo: "Escrevendo", testando: "Testando" };
assert.deepEqual(J(A.termRotulo(s, [], ESTADOS)), { texto: "1.2 · Tela", estado: "Rodando", viva: true });
const ags = [{ acaoId: "a1", blueprintId: "b1", estado: "escrevendo", em: "2026-10-26T10:00:00Z" }, { acaoId: "a1", blueprintId: "b1", estado: "testando", em: "2026-10-26T10:05:00Z" }, { acaoId: "a2", blueprintId: "b1", estado: "escrevendo", em: "2026-10-26T11:00:00Z" }];
assert.equal(A.termRotulo(s, ags, ESTADOS).estado, "Testando", "o aviso mais novo da ação da sessão");
assert.equal(A.termRotulo({ ...s, blueprintId: "b9" }, ags, ESTADOS).estado, "Rodando", "aviso de outro blueprint não conta");
assert.deepEqual(J(A.termRotulo({ ...s, viva: false, codigo: 1 }, ags, ESTADOS)), { texto: "1.2 · Tela", estado: "Terminou (código 1)", viva: false });
assert.equal(A.termRotulo({ id: "t7", viva: false, codigo: null }).texto, "Sessão t7");

// ---- links: http(s) sem a pontuação final
assert.deepEqual(J(A.termLinks("veja https://exemplo.com/a?b=1. e (http://x.io/p)")), [{ inicio: 5, fim: 30, url: "https://exemplo.com/a?b=1" }, { inicio: 35, fim: 48, url: "http://x.io/p" }]);
assert.deepEqual(J(A.termLinks("nada aqui, nem ftp://x nem http://")), []);

// ---- atalhos
const k = (key, m = {}) => ({ type: "keydown", key, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...m });
assert.equal(A.termTecla(k("Escape")), "", "Esc é do Claude (interromper)");
assert.equal(A.termTecla(k("c", { ctrlKey: true })), "", "Ctrl+C sem seleção é do Claude");
assert.equal(A.termTecla(k("c", { ctrlKey: true }), true), "copiar", "com seleção, copia");
assert.equal(A.termTecla(k("C", { ctrlKey: true, shiftKey: true })), "copiar");
assert.equal(A.termTecla(k("v", { ctrlKey: true })), "colar");
assert.equal(A.termTecla(k("F6")), "sair");
assert.equal(A.termTecla(k("F6", { shiftKey: true })), "sair-voltar");
assert.equal(A.termTecla(k("PageDown", { ctrlKey: true })), "proxima");
assert.equal(A.termTecla(k("PageUp", { ctrlKey: true })), "anterior");
assert.equal(A.termTecla(k("=", { ctrlKey: true })), "maior");
assert.equal(A.termTecla(k("-", { ctrlKey: true })), "menor");
assert.equal(A.termTecla(k("0", { ctrlKey: true })), "normal");
assert.equal(A.termTecla(k("`", { ctrlKey: true })), "painel");
assert.equal(A.termTecla({ ...k("F6"), type: "keyup" }), "", "só no keydown");
assert.equal(A.termTecla(k("Tab", { shiftKey: true })), "", "Shift+Tab é do Claude (modo)");

// ---- fila de teclas: um POST por vez, na ordem, juntando o que chega no meio
const enviados = [];
let solta;
const enviar = A.termEnviador((t) => { enviados.push(t); return new Promise((r) => (solta = r)); });
enviar("a"); enviar("b"); enviar("c");
assert.deepEqual(enviados, ["a"], "o segundo espera o primeiro");
solta();
await new Promise((r) => setTimeout(r, 0));
assert.deepEqual(enviados, ["a", "bc"], "o que chegou no meio vai junto, na ordem");
solta();
await new Promise((r) => setTimeout(r, 0));
const falha = A.termEnviador(() => Promise.reject(new Error("rede")));
falha("x"); // erro de rede não trava a fila nem estoura
await new Promise((r) => setTimeout(r, 0));
console.log("terminal-logica: ok (tema e contraste, fonte, abas, links, atalhos, fila de teclas)");
