// ---- tela Roadmap: o roadmap do próprio app (roadmap.json → roadmap-dados.js), por tema e por estado, com filtros, busca, teclado e animação ----
// Módulo ES carregado sob demanda (roadmap-entrada.js). Zero trabalho em repouso: nada anima sozinho depois da entrada; movimento reduzido desliga tudo.
// Todo texto de interface passa por tr(); o conteúdo dos itens vem do JSON, em inglês ou português conforme o idioma ativo.
import { ROADMAP } from "/web/js/roadmap-dados.js";

const ESTADOS = ["entregue", "agora", "proximo", "depois"];
const rotulos = () => ({ entregue: tr("Entregue"), agora: tr("Fazendo agora"), proximo: tr("Em seguida"), depois: tr("Depois") });
const idioma = () => (I18N.ativo === "pt-BR" ? "pt" : "en");
const reduzido = () => matchMedia("(prefers-reduced-motion: reduce)").matches;
const CIRC = 2 * Math.PI * 15;

let raiz = null, filtro = { estado: "", tema: "", q: "" }, abertos = new Set(), origem = null, aoTecla = null;

const norm2 = (s) => String(s || "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
const itensDoTema = (t) => ROADMAP.itens.filter((i) => i.tema === t.id);
const textoDe = (i) => i[idioma()] || i.en;
const casa = (i) => (!filtro.estado || i.estado === filtro.estado) && (!filtro.tema || i.tema === filtro.tema)
  && (!filtro.q || norm2(`${textoDe(i).titulo} ${textoDe(i).resumo} ${textoDe(i).detalhe || ""}`).includes(norm2(filtro.q)));

function anel(feitos, total) {
  const p = total ? feitos / total : 0;
  return `<svg class="rm-anel" viewBox="0 0 40 40" width="44" height="44" role="img" aria-label="${esc(tr("{a} de {b} entregues", { a: feitos, b: total }))}"><circle class="rm-anel-fundo" cx="20" cy="20" r="15"/><circle class="rm-anel-feito" cx="20" cy="20" r="15" style="--c:${CIRC.toFixed(2)};--p:${p.toFixed(3)}"/><text x="20" y="24" text-anchor="middle" font-size="10">${Math.round(p * 100)}%</text></svg>`;
}

function htmlCartao(i, n) {
  const x = textoDe(i), aberto = abertos.has(i.id);
  return `<article class="rm-card" data-id="${esc(i.id)}" data-estado="${i.estado}" tabindex="0" role="button" aria-expanded="${aberto}" style="--i:${n}"><span class="rm-ponto" aria-hidden="true"></span><h3>${esc(x.titulo)}</h3><p class="rm-resumo-item">${esc(x.resumo)}</p>${x.detalhe ? `<div class="rm-detalhe" ${aberto ? "" : "hidden"}><p>${esc(x.detalhe)}</p></div>` : ""}</article>`;
}

function html() {
  const rot = rotulos(), lang = idioma();
  const contagem = Object.fromEntries(ESTADOS.map((e) => [e, ROADMAP.itens.filter((i) => i.estado === e).length]));
  const chipsEstado = [`<button class="rm-chip" type="button" data-estado="" aria-pressed="${!filtro.estado}">${tr("Todos")} · ${ROADMAP.itens.length}</button>`, ...ESTADOS.map((e, k) => `<button class="rm-chip" type="button" data-estado="${e}" aria-pressed="${filtro.estado === e}" title="${k + 1}">${rot[e]} · ${contagem[e]}</button>`)].join("");
  const chipsTema = [`<button class="rm-chip" type="button" data-tema="" aria-pressed="${!filtro.tema}">${tr("Todos os temas")}</button>`, ...ROADMAP.temas.map((t) => `<button class="rm-chip" type="button" data-tema="${t.id}" aria-pressed="${filtro.tema === t.id}">${esc(t[lang])}</button>`)].join("");
  let n = 0;
  const faixas = ROADMAP.temas.map((t) => {
    const itens = itensDoTema(t), feitos = itens.filter((i) => i.estado === "entregue").length;
    const cols = ESTADOS.map((e) => `<div class="rm-col" data-estado="${e}"><h4>${rot[e]}</h4>${itens.filter((i) => i.estado === e).map((i) => htmlCartao(i, n++)).join("")}</div>`).join("");
    return `<section class="rm-faixa" data-tema="${t.id}"><header>${anel(feitos, itens.length)}<div><h2>${esc(t[lang])}</h2><small>${tr("{a} de {b} entregues", { a: feitos, b: itens.length })}</small></div></header><div class="rm-colunas">${cols}</div></section>`;
  }).join("");
  const marca = typeof EMB !== "undefined" && EMB && typeof previaMarca === "function" ? `<div class="rm-marca" aria-hidden="true">${previaMarca(EMB.BLUEPRINT, 150, 104)}</div>` : "";
  return `<div class="rm-topo"><button class="btn rm-voltar" type="button">${tr("← Voltar")}</button>${marca}<div class="rm-titulo"><h1>${tr("Roadmap")}</h1><p>${esc(ROADMAP.resumo[lang])}</p><small>${tr("Atualizado em {data}", { data: esc(ROADMAP.atualizado) })}</small></div><label class="rm-busca"><span class="rm-so-leitor">${tr("Buscar no roadmap…")}</span><input type="search" placeholder="${esc(tr("Buscar no roadmap…"))}" value="${esc(filtro.q)}" autocomplete="off"></label></div>
<div class="rm-filtros" role="group" aria-label="${esc(tr("Filtros"))}"><div>${chipsEstado}</div><div>${chipsTema}</div></div>
<p class="rm-vazio" hidden>${tr("Nenhum item com esses filtros.")}</p><div class="rm-grade">${faixas}</div><p class="rm-dica">${tr("Atalhos: setas navegam, Enter abre, / busca, 1 a 4 filtram, 0 limpa, Esc fecha.")}</p>`;
}

// mostra e esconde sem refazer a tela: o que ficou fora do filtro some; o que fica reentra (a animação só roda se o movimento não estiver reduzido)
function aplicaFiltro(animar = true) {
  let visiveis = 0;
  for (const c of raiz.querySelectorAll(".rm-card")) {
    const item = ROADMAP.itens.find((i) => i.id === c.dataset.id), ok = casa(item);
    if (ok) visiveis++;
    if (c.hidden === !ok && !(ok && animar)) continue;
    c.hidden = !ok;
    if (ok && animar && !reduzido()) { c.classList.remove("rm-entra"); void c.offsetWidth; c.classList.add("rm-entra"); }
  }
  for (const f of raiz.querySelectorAll(".rm-faixa")) f.hidden = !f.querySelector(".rm-card:not([hidden])");
  for (const col of raiz.querySelectorAll(".rm-col")) col.classList.toggle("rm-col-vazia", !col.querySelector(".rm-card:not([hidden])"));
  raiz.querySelector(".rm-vazio").hidden = visiveis > 0;
  for (const b of raiz.querySelectorAll(".rm-filtros [data-estado]")) b.setAttribute("aria-pressed", String(b.dataset.estado === filtro.estado));
  for (const b of raiz.querySelectorAll(".rm-filtros [data-tema]")) b.setAttribute("aria-pressed", String(b.dataset.tema === filtro.tema));
}

function alterna(c, aberto) {
  const d = c.querySelector(".rm-detalhe");
  aberto ??= !abertos.has(c.dataset.id);
  aberto ? abertos.add(c.dataset.id) : abertos.delete(c.dataset.id);
  c.setAttribute("aria-expanded", String(aberto));
  if (d) d.hidden = !aberto;
}

const visiveisCartoes = () => [...raiz.querySelectorAll(".rm-card:not([hidden])")].filter((c) => !c.closest("[hidden]"));
function tecla(e) {
  if (!raiz || raiz.hidden) return;
  const busca = raiz.querySelector(".rm-busca input"), noCampo = document.activeElement === busca;
  if (e.key === "Escape") {
    e.preventDefault();
    const aberto = raiz.querySelector('.rm-card[aria-expanded="true"]');
    if (noCampo && busca.value) { busca.value = ""; filtro.q = ""; aplicaFiltro(); }
    else if (aberto && !noCampo) alterna(aberto, false);
    else fecharRoadmap();
    return;
  }
  if (noCampo) { if (e.key === "ArrowDown") { e.preventDefault(); visiveisCartoes()[0]?.focus(); } return; }
  if (e.key === "/") { e.preventDefault(); busca.focus(); return; }
  if (/^[0-4]$/.test(e.key) && !e.ctrlKey && !e.metaKey && !e.altKey) { filtro.estado = e.key === "0" ? "" : ESTADOS[+e.key - 1]; aplicaFiltro(); return; }
  const cartoes = visiveisCartoes(), i = cartoes.indexOf(document.activeElement);
  const vai = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[e.key];
  if (vai && cartoes.length) { e.preventDefault(); cartoes[i < 0 ? 0 : Math.min(cartoes.length - 1, Math.max(0, i + vai))].focus(); }
  else if (e.key === "Home" && cartoes.length) { e.preventDefault(); cartoes[0].focus(); }
  else if (e.key === "End" && cartoes.length) { e.preventDefault(); cartoes.at(-1).focus(); }
  else if ((e.key === "Enter" || e.key === " ") && document.activeElement?.classList.contains("rm-card")) { e.preventDefault(); alterna(document.activeElement); }
}

export async function abrirRoadmap(quemAbriu) {
  try { await emblemasProntos; } catch {} // a marca do anatomy no topo precisa do módulo das figuras (proposta.js)
  raiz ||= document.getElementById("roadmap-tela");
  origem = quemAbriu || null;
  filtro = { estado: "", tema: "", q: "" };
  raiz.innerHTML = html();
  raiz.hidden = false;
  document.body.classList.add("rm-aberto");
  raiz.classList.remove("rm-pronto");
  raiz.scrollTop = 0;
  aplicaFiltro(false);
  requestAnimationFrame(() => requestAnimationFrame(() => raiz.classList.add("rm-pronto"))); // os anéis enchem depois do primeiro quadro
  raiz.querySelector(".rm-voltar").onclick = fecharRoadmap;
  raiz.querySelector(".rm-busca input").oninput = (e) => { filtro.q = e.target.value.trim(); aplicaFiltro(); };
  raiz.querySelector(".rm-filtros").onclick = (e) => {
    const b = e.target.closest("button"); if (!b) return;
    if ("estado" in b.dataset) filtro.estado = b.dataset.estado; else filtro.tema = b.dataset.tema;
    aplicaFiltro();
  };
  raiz.querySelector(".rm-grade").onclick = (e) => { const c = e.target.closest(".rm-card"); if (c) alterna(c); };
  if (!aoTecla) addEventListener("keydown", (aoTecla = tecla));
  raiz.querySelector(".rm-voltar").focus({ preventScroll: true });
}

export function fecharRoadmap() {
  if (!raiz || raiz.hidden) return;
  raiz.hidden = true;
  document.body.classList.remove("rm-aberto");
  if (location.hash === "#/roadmap") location.hash = "#/";
  origem?.focus?.({ preventScroll: true });
}
