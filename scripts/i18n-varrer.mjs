// Abre cada tela do app em inglês (en-US) e lista o texto visível que ainda está em português, ignorando o que vem dos dados (nomes de blueprints, ações etc.).
// É o teste de completude da interface: o que sobra aqui é texto que falta traduzir. Usa os dados de exemplo e um servidor temporário.
//   node scripts/i18n-varrer.mjs [blueprint.exe]      lista; sai com 1 se sobrar texto em português (para o CI quando a conversão acabar)
import { join, resolve } from "node:path";
import { abrirAmbiente, dormir, raiz } from "./gravar/nucleo.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const A = await abrirAmbiente({ exe, idioma: "en-US", largura: 1440, altura: 900 });
const COLETAR = `(() => {
  const dados = new Set(); const juntar = (v) => { if (typeof v === "string") dados.add(v); else if (v && typeof v === "object") Object.values(v).forEach(juntar); };
  juntar(typeof arquivo !== "undefined" ? arquivo : {});
  const texto = new Set(), ehDado = (s) => [...dados].some((d) => d.length > 2 && (d.includes(s) || s.includes(d)));
  const acento = /[áàâãéêíóôõúçÁÀÂÃÉÊÍÓÔÕÚÇ]/, palavras = /(^|[^\\p{L}])(de|da|do|dos|das|para|com|sem|ainda|nenhum|nenhuma|abrir|fechar|salvar|ação|ações|plano|planos|aplicar|descartar|criar|adicionar|remover|editar|marcar|voltar|nome|data|prazo|hoje|erro|sobre|este|esta|seu|sua|você)([^\\p{L}]|$)/iu;
  const add = (s) => { s = (s || "").replace(/\\s+/g, " ").trim(); if (s.length < 3 || ehDado(s)) return; if (acento.test(s) || palavras.test(s)) texto.add(s.slice(0, 140)); };
  const visivel = (e) => { const r = e.getBoundingClientRect(); const cs = getComputedStyle(e); return cs.display !== "none" && cs.visibility !== "hidden" && !e.closest("[hidden]"); };
  const w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let n = w.nextNode(); n; n = w.nextNode()) { const p = n.parentElement; if (p && !["SCRIPT", "STYLE"].includes(p.tagName) && visivel(p)) add(n.nodeValue); }
  for (const e of document.querySelectorAll("[aria-label],[title],[placeholder]")) if (visivel(e)) for (const a of ["aria-label", "title", "placeholder"]) add(e.getAttribute(a));
  return [...texto];
})()`;
const resultado = new Map();
const tela = async (nome, preparar) => { await preparar(); await dormir(900); const achados = await A.js(COLETAR); for (const t of achados) if (!resultado.has(t)) resultado.set(t, nome); };
try {
  const d = await (await fetch(A.url + "/api/dados")).json();
  const bp = d.blueprints.find((b) => b.acoes?.some((a) => a.planos?.length)), acao = bp.acoes.find((a) => a.planos?.length);
  await tela("lista de workspaces", () => A.ir("#/", 2500));
  await tela("workspace", () => A.ir("#/c/sm", 2500));
  await tela("folha", () => A.ir(`#/p/${bp.id}`, 3200));
  await tela("menu da ação", async () => { await A.js(`document.querySelector('.acao[data-f="${acao.id}"]')?.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 500, clientY: 400 }))`); });
  await A.js(`document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }))`);
  await A.post("/api/mcp/proposta", { workspaceId: "sm", blueprintId: bp.id, mensagem: "Teste", acoes: [{ nome: "Ação nova de teste", inicio: "2026-12-14", fim: "2026-12-15", planos: [{ texto: "Plano de teste", horas: 1, descricao: "Escopo: a.\nResultado esperado: b.\nValidação: c." }], checkpoints: [{ nome: "Checkpoint de teste", data: "2026-12-15" }] }], remover: [{ acao_id: acao.id, motivo: "teste" }] });
  await dormir(1500);
  await tela("Planejar com proposta", async () => { await A.js(`document.getElementById("b-ia").click()`); await dormir(1500); });
  await tela("Planejar: editar a ação", async () => { await A.js(`document.querySelector("#agente-tela [data-pact=editar-acao]")?.click()`); });
  await tela("Como conectar", async () => { await A.js(`document.getElementById("cu-abrir")?.click()`); });
  await A.js(`document.getElementById("b-ia").click()`);
  await tela("Editar blueprint", async () => { await A.ir(`#/p/${bp.id}`, 2500); await A.js(`editar(true)`); });
  await tela("Editar ação", async () => { await A.js(`document.querySelectorAll(".mn-item[data-sel]")[1]?.click()`); });
} finally { A.fechar(); }
const por = new Map();
for (const [t, n] of resultado) (por.get(n) || por.set(n, []).get(n)).push(t);
for (const [n, l] of por) { console.log(`\n## ${n} (${l.length})`); for (const t of l) console.log("  " + t); }
console.log(`\n${resultado.size} texto(s) ainda em português em ${por.size} tela(s).${A.erros.length ? " Erros no console: " + A.erros.length : ""}`);
process.exit(resultado.size ? 1 : 0);
