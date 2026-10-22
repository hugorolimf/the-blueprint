// Grava uma página HTML com a figura do agente numa pose (para o `lines.mjs` e o `drive.mjs` da skill anatomy conferirem no navegador).
// Uso: node scripts/pagina-figura.mjs <saida.html> [--tipos linha,risco] [--papel construtor] [--estado escrevendo] [--posicao 0.5] [--progresso 0.6] [--tema escuro|claro]
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { desenhaEmblema } from "../emblemas.mjs";
import { pecasDoAgente } from "../agentes-figura.mjs";

const raiz = resolve(import.meta.dirname, "..");
const saida = resolve(process.argv[2] || join(raiz, "target", "figura.html"));
const arg = (nome, padrao) => { const i = process.argv.indexOf(`--${nome}`); return i > 0 ? process.argv[i + 1] : padrao; };
const pose = { papel: arg("papel", "construtor"), estado: arg("estado", "escrevendo"), posicao: Number(arg("posicao", 0.5)), progresso: Number(arg("progresso", 0.6)) };
const tema = arg("tema", "escuro");
const so = arg("tipos", "").split(",").filter(Boolean); // só estes tipos de peça (para achar quem deixa ponta solta); as lajes ficam, para as linhas terem onde apoiar
const todas = pecasDoAgente(pose);
const lista = so.length ? todas.filter((p) => so.includes(p.tipo) || p.tipo === "laje" || p.tipo === "cilindro" || p.tipo === "roda") : todas;
const e = desenhaEmblema(lista, { largura: 880, altura: 680, pad: 3, progresso: 1, sombra: false, limite: 400 });
const claro = ".emblema{--anatomy-paper:#f3f7fd;--anatomy-face:#e3ecf8;--anatomy-top:#fff;--anatomy-shade-0:#c3d3e8;--anatomy-shade-1:#cddbee;--anatomy-shade-2:#d7e3f2;--anatomy-shade-3:#e1eaf6;--anatomy-hi:#0b2a57;--anatomy-mid:#3d6aa3;--anatomy-lo:#7a9fcb;--anatomy-faint:#bcd0e8;--anatomy-lit:#000;--anatomy-lit-top:#cfdff3;--anatomy-lit-shade:#b0c6e3;--anatomy-glow:#6d9bd6;--anatomy-dot:#0b2a57}";
const fundo = tema === "claro" ? "#f3f7fd" : "radial-gradient(ellipse at 50% 40%,#1250a0 0,#0c3a78 62%,#082b5e 100%)";
mkdirSync(dirname(saida), { recursive: true });
writeFileSync(saida, `<!doctype html><meta charset="utf-8"><title>Figura do agente</title><style>html,body{margin:0;background:${fundo}}${readFileSync(join(raiz, "web/css/marca.css"), "utf8")}${tema === "claro" ? claro : ""}svg{display:block}</style>
<svg class="emblema" width="${e.largura}" height="${e.altura}" viewBox="0 0 ${e.largura} ${e.altura}">${e.svg}</svg>`);
console.log(saida);
