let nomeUsuario = "", editandoNome = false, salvandoPerfil = false, salvandoWorkspace = false;
function mostrarPerfil() {
  document.body.classList.toggle("sem-perfil", !nomeUsuario);
  $("login").hidden = !!nomeUsuario;
  $("perfil-nome").textContent = nomeUsuario;
  $("perfil-avatar").textContent = nomeUsuario.slice(0, 1).toUpperCase();
}
mostrarPerfil();
$("login-form").onsubmit = async (e) => {
  e.preventDefault();
  if (!carregado) return;
  const nome = $("login-nome").value.trim();
  if (!nome) { $("login-nome").setCustomValidity("Digite seu nome."); $("login-nome").reportValidity(); return; }
  const anterior = arquivo.usuario;
  const botao = e.target.querySelector("button");
  botao.disabled = true;
  salvandoPerfil = true;
  $("login-erro").textContent = tr("Salvando seu nome…");
  arquivo.usuario = { ...arquivo.usuario, nome };
  try {
    await gravarDados();
    nomeUsuario = nome;
    editandoNome = false;
    $("login-erro").textContent = "";
    mostrarPerfil(); desenrolarNaProxima = true; render();
  } catch {
    arquivo.usuario = anterior;
    $("login-erro").textContent = tr("Não foi possível salvar seu nome. Tente novamente.");
  } finally { salvandoPerfil = false; botao.disabled = false; }
};
$("login-nome").oninput = () => $("login-nome").setCustomValidity("");
$("perfil-sair").onclick = () => {
  $("login-nome").value = nomeUsuario;
  editandoNome = true;
  nomeUsuario = "";
  mostrarPerfil(); desenrolar(document.querySelector(".login-folha"), document.querySelector(".login-folha")); $("login-nome").focus();
};
$("entrada-novo").onclick = () => editar(true, { novo: true });
$("entrada-voltar").onclick = () => { location.hash = ""; };
let workspaceSemClique = false;
$("entrada-blueprints").onclick = (e) => {
  if (workspaceSemClique) { e.preventDefault(); e.stopPropagation(); workspaceSemClique = false; return; }
  if (!e.target.closest("[data-adicionar-workspace]")) return;
  $("workspace-form").reset();
  atualizarCriacaoWorkspace();
  location.hash = "/workspace/novo";
};
$("workspace-cancelar").onclick = () => { if (!salvandoWorkspace) location.hash = ""; };
function atualizarCriacaoWorkspace() {
  $("workspace-erro").textContent = "";
  $("workspace-titulo").textContent = $("workspace-nome").value.trim() || "Novo workspace";
  atualizaArte();
}
$("workspace-tela").onclick = () => alternarTelaCheia();
$("workspace-nome").oninput = atualizarCriacaoWorkspace;
async function criarWorkspace(nome, descricao = "", emblema = null, icone = "blueprint") {
  nome = nome.trim().slice(0, 80);
  if (!nome) throw new Error(tr("Digite o nome do workspace."));
  if (workspacesDe(arquivo).some((c) => norm(c.nome) === norm(nome))) throw new Error(tr("Já existe um workspace com esse nome."));
  const workspace = limpa({ id: novoId(), nome, empresa: nome, descricao, emblema, icone, blueprints: [] });
  arquivo.workspaces.push(workspace);
  try { await gravarDados(); return workspace; }
  catch (e) { arquivo.workspaces = arquivo.workspaces.filter((c) => c !== workspace); throw e; }
}
$("workspace-form").onsubmit = async (e) => {
  e.preventDefault();
  if (!carregado || salvandoWorkspace) return;
  salvandoWorkspace = true;
  $("workspace-estado").textContent = tr("salvando…");
  $("workspace-estado").dataset.e = "";
  const controles = $("workspace-nova").querySelectorAll("button,input,textarea,select");
  controles.forEach((c) => { c.disabled = true; });
  $("workspace-erro").textContent = "";
  try {
    const arte = lerArte();
    const novo = await criarWorkspace($("workspace-nome").value, $("workspace-descricao-manual").value.trim(), arte.pecas ? { pecas: arte.pecas } : null, arte.icone);
    location.hash = "/c/" + encodeURIComponent(novo.id);
  } catch (erro) {
    $("workspace-erro").textContent = erro.message === "Failed to fetch" ? tr("Não foi possível concluir. Tente novamente.") : erro.message;
    $("workspace-estado").textContent = tr("erro");
    $("workspace-estado").dataset.e = "erro";
  }
  finally { salvandoWorkspace = false; controles.forEach((c) => { c.disabled = false; }); }
};
const host = $("desenho");
const hojeSimulado = new URLSearchParams(location.search).get("hoje");
let hoje = hojeSimulado || local(new Date());

let D = { empresa: "", titulo: "", rev: "A", exemplo: false, blueprints: [] };
let arquivo = D;
// O documento raiz continua sendo a SM; os outros espaços guardam seus próprios blueprints.
const workspacesDe = (a) => {
  const todas = [{ id: "sm", nome: String(a.empresa || "Meu workspace"), icone: a.icone || "blueprint", dados: a }, ...(a.workspaces || []).map((c) => ({ id: c.id, nome: c.nome, icone: c.icone || "blueprint", dados: c }))];
  const posicoes = new Map((Array.isArray(a.ordemWorkspaces) ? a.ordemWorkspaces : []).map((id, i) => [id, i]));
  return todas.sort((x, y) => (posicoes.get(x.id) ?? Infinity) - (posicoes.get(y.id) ?? Infinity));
};
async function excluirWorkspace(id) {
  const w = (arquivo.workspaces || []).find((c) => c.id === id); // o workspace raiz (sm) é o próprio arquivo: não se remove
  if (!w) return;
  const n = (w.blueprints || []).length;
  if (!(await confirmar({ titulo: tr("Excluir workspace?"), texto: tr(`O workspace "{nome}" será apagado{extra}. Não dá para desfazer.`, { nome: w.nome || tr("Sem nome"), extra: n ? tr(", junto com {blueprints} e todas as ações", { blueprints: trn(n, ["{n} blueprint", "{n} blueprints"]) }) : "" }), rotulo: tr("Excluir workspace") }))) return;
  const antes = { workspaces: arquivo.workspaces, ordem: arquivo.ordemWorkspaces };
  arquivo.workspaces = arquivo.workspaces.filter((c) => c.id !== id);
  if (Array.isArray(arquivo.ordemWorkspaces)) arquivo.ordemWorkspaces = arquivo.ordemWorkspaces.filter((x) => x !== id);
  try { await gravarDados(); }
  catch { arquivo.workspaces = antes.workspaces; arquivo.ordemWorkspaces = antes.ordem; $("entrada-status").textContent = tr("Não foi possível excluir o workspace."); return; }
  render();
}
function salvarOrdemWorkspaces(ids, movido) {
  const visiveis = new Set(ids), ordem = workspacesDe(arquivo).map((c) => c.id);
  let i = 0;
  arquivo.ordemWorkspaces = ordem.map((id) => visiveis.has(id) ? ids[i++] : id);
  const cartoes = new Map([...$("entrada-blueprints").querySelectorAll("[data-workspace]")].map((el) => [el.dataset.workspace, el]));
  const fragmento = document.createDocumentFragment();
  ids.forEach((id) => { if (cartoes.has(id)) fragmento.append(cartoes.get(id)); });
  $("entrada-blueprints").insertBefore(fragmento, $("entrada-blueprints").querySelector("[data-adicionar-workspace]"));
  const nome = workspacesDe(arquivo).find((c) => c.id === movido)?.nome || "Workspace";
  $("entrada-status").textContent = tr("{nome} movido para {pos} de {total}. Salvando…", { nome, pos: ids.indexOf(movido) + 1, total: ids.length });
  clearTimeout(timer);
  timer = setTimeout(async () => {
    const gravacao = timer;
    try { await gravarDados(); $("entrada-status").textContent = tr("Ordem dos workspaces salva."); }
    catch { $("entrada-status").textContent = tr("A ordem mudou, mas não foi possível salvá-la."); }
    finally { if (timer === gravacao) timer = 0; }
  }, 400);
}
let arrastoWorkspace = null;
const ghostWorkspace = () => document.querySelector(".fantasma-workspace");
function iniciarArrastoWorkspace() {
  if (!arrastoWorkspace || arrastoWorkspace.ativo) return;
  arrastoWorkspace.ativo = true;
  document.body.classList.add("arrastando");
  const c = workspacesDe(arquivo).find((x) => x.id === arrastoWorkspace.id);
  const f = document.createElement("div");
  f.className = "fantasma fantasma-workspace";
  f.textContent = `↔ ${c?.nome || "Workspace"}`;
  f.style.left = arrastoWorkspace.mx + "px";
  f.style.top = arrastoWorkspace.my + "px";
  document.body.appendChild(f);
  $("entrada-blueprints").querySelector(`[data-workspace="${CSS.escape(arrastoWorkspace.id)}"]`)?.classList.add("arrastando-workspace");
}
function terminarArrastoWorkspace(cancelado) {
  if (!arrastoWorkspace) return;
  clearTimeout(arrastoWorkspace.timer);
  const a = arrastoWorkspace;
  arrastoWorkspace = null;
  $("entrada-blueprints").querySelectorAll(".soltar,.arrastando-workspace").forEach((el) => el.classList.remove("soltar", "arrastando-workspace"));
  document.body.classList.remove("arrastando");
  ghostWorkspace()?.remove();
  if (!a.ativo) return;
  workspaceSemClique = true;
  setTimeout(() => { workspaceSemClique = false; }, 0);
  if (cancelado || !a.alvo || a.alvo === a.id) return;
  const ids = [...$("entrada-blueprints").querySelectorAll("[data-workspace]")].map((el) => el.dataset.workspace);
  const de = ids.indexOf(a.id), para = ids.indexOf(a.alvo);
  if (de < 0 || para < 0) return;
  ids.splice(para, 0, ids.splice(de, 1)[0]);
  salvarOrdemWorkspaces(ids, a.id);
  $("entrada-blueprints").querySelector(`[data-workspace="${CSS.escape(a.id)}"]`)?.focus({ preventScroll: true });
}
$("entrada-blueprints").addEventListener("pointerdown", (e) => {
  if (e.button !== 0) return;
  const el = e.target.closest("[data-workspace]");
  if (!el) return;
  arrastoWorkspace = { id: el.dataset.workspace, x: e.clientX, y: e.clientY, mx: e.clientX, my: e.clientY, tipo: e.pointerType, ativo: false, alvo: null };
  if (e.pointerType !== "mouse") arrastoWorkspace.timer = setTimeout(iniciarArrastoWorkspace, 220);
});
addEventListener("pointermove", (e) => {
  if (!arrastoWorkspace) return;
  const a = arrastoWorkspace;
  a.mx = e.clientX; a.my = e.clientY;
  if (!a.ativo && Math.hypot(e.clientX - a.x, e.clientY - a.y) > (a.tipo === "mouse" ? 6 : 12)) {
    if (a.tipo === "mouse") iniciarArrastoWorkspace(); else return terminarArrastoWorkspace(true);
  }
  if (!a.ativo) return;
  const f = ghostWorkspace();
  if (f) { f.style.left = e.clientX + "px"; f.style.top = e.clientY + "px"; }
  const alvo = document.elementFromPoint(e.clientX, e.clientY)?.closest("[data-workspace]")?.dataset.workspace ?? null;
  if (alvo !== a.alvo) {
    $("entrada-blueprints").querySelectorAll(".soltar").forEach((el) => el.classList.remove("soltar"));
    a.alvo = alvo;
    if (alvo && alvo !== a.id) $("entrada-blueprints").querySelector(`[data-workspace="${CSS.escape(alvo)}"]`)?.classList.add("soltar");
  }
});
addEventListener("pointerup", () => terminarArrastoWorkspace(false));
addEventListener("pointercancel", () => terminarArrastoWorkspace(true));
addEventListener("keydown", (e) => {
  if (!e.altKey || !["ArrowLeft", "ArrowUp", "ArrowRight", "ArrowDown"].includes(e.key)) return;
  const el = e.target.closest?.("#entrada-blueprints [data-workspace]");
  if (!el) return;
  e.preventDefault();
  const ids = [...$("entrada-blueprints").querySelectorAll("[data-workspace]")].map((card) => card.dataset.workspace);
  const de = ids.indexOf(el.dataset.workspace), delta = e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 1, para = de + delta;
  if (para < 0 || para >= ids.length) return;
  ids.splice(para, 0, ids.splice(de, 1)[0]);
  salvarOrdemWorkspaces(ids, el.dataset.workspace);
  $("entrada-blueprints").querySelector(`[data-workspace="${CSS.escape(el.dataset.workspace)}"]`)?.focus({ preventScroll: true });
});
const pecasWorkspace = (c, kit) => c?.dados.emblema?.pecas || kit?.MARCAS?.[c?.icone] || kit?.BLUEPRINT;
const workspaceDaRota = (a, hash) => {
  const c = hash.match(/^#\/c\/(.+)$/), p = hash.match(/^#\/p\/(.+)$/);
  return workspacesDe(a).find((x) => c ? x.id === decodeURIComponent(c[1]) : p ? x.dados.blueprints.some((blueprint) => blueprint.id === decodeURIComponent(p[1])) : false) || null;
};
let workspaceAtual = null;
let carregado = false, falhou = false;
let selecionada = null;
let consulta = "";

const abertos = {}; // só vale até recarregar
const lerHash = () => { const m = location.hash.match(/^#\/p\/(.+)$/); return m ? decodeURIComponent(m[1]) : null; };
let tela = lerHash(); // null = lista de blueprints; um id = blueprint daquele blueprint
let desenrolarNaProxima = true; // a primeira tela também se abre
if (tela) abertos[tela] = true; // abrir direto pelo endereço também mostra as ações

