const menuContexto = $("menu-contexto");
let alvoContexto = null;
function fecharMenuContexto(restaurar = false) {
  if (menuContexto.hidden) return;
  menuContexto.hidden = true;
  if (restaurar && alvoContexto?.isConnected) alvoContexto.focus?.({ preventScroll: true });
  alvoContexto = null;
}
document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  if (menuContexto.contains(e.target)) return;
  fecharMenuContexto();
  const alvo = e.target instanceof Element ? e.target : e.target.parentElement;
  alvoContexto = alvo;
  const acoes = [];
  const add = (rotulo, fn) => acoes.push({ rotulo, fn });
  const cartaoWorkspace = alvo?.closest("#entrada-blueprints a.blueprint-cartao[href]");
  const novoWorkspace = alvo?.closest("#entrada-blueprints [data-adicionar-workspace]");
  const linhaBlueprint = alvo?.closest("[data-goto]");
  const acao = alvo?.closest(".acao[data-p][data-f]");
  const plano = alvo?.closest(".chk[data-p][data-f][data-id]");
  const nota = alvo?.closest(".nota[data-obs]");
  const marca = alvo?.closest(".marca");
  const link = alvo?.closest("a.lk[href]");
  const criandoWorkspace = location.hash === "#/workspace/novo";
  const idBlueprint = linhaBlueprint?.dataset.goto || acao?.dataset.p || plano?.dataset.p || nota?.dataset.obs.split("|")[0] || tela;

  if (cartaoWorkspace) add("Abrir workspace", () => { location.hash = cartaoWorkspace.getAttribute("href").slice(1); });
  if (novoWorkspace) add("Criar workspace", () => novoWorkspace.click());
  if (linhaBlueprint) {
    add("Abrir blueprint", () => irBlueprint(linhaBlueprint.dataset.goto));
    add("Excluir blueprint…", () => excluirBlueprint(linhaBlueprint.dataset.goto));
  }
  if (acao) {
    add("Editar ação", () => editar(true, { proj: acao.dataset.p, sel: acao.dataset.f }));
    if (acaoDe(acao.dataset.p, acao.dataset.f)?.prototipos?.length) add("Ver protótipo", () => abreProtos(acao.dataset.p, acao.dataset.f));
    add("Ir para esta ação", () => irPara(acao.dataset.p, acao.dataset.f, alvo.closest(".checkpoint")?.dataset.m));
    add("Excluir ação…", () => excluirAcao(acao.dataset.p, acao.dataset.f, true));
  }
  if (plano) add(plano.getAttribute("aria-checked") === "true" ? "Marcar como pendente" : "Marcar como feito", () => alternarItem(plano.dataset.p, plano.dataset.f, plano.dataset.tipo, plano.dataset.id));
  if (nota) add(nota.dataset.obs.split("|")[1] ? "Abrir instruções" : "Abrir observações", () => abreNotaObs(nota));
  if (marca) add("Ver marca", () => abreNotaMarca(marca));
  if (link) add("Abrir link em nova aba", () => link.click());

  if (!hub.hidden) {
    add("Voltar ao blueprint", () => agenteTela(false));
  } else if (!ed.hidden) {
    const itemAcao = alvo?.closest(".mn-item[data-sel]");
    if (itemAcao && itemAcao.dataset.sel !== "blueprint") {
      add("Editar ação", () => itemAcao.click());
      add("Excluir ação…", () => excluirAcao(edProj, itemAcao.dataset.sel, true));
    }
    add("Voltar ao blueprint", () => editar(false));
    add("Criar blueprint", () => $("b-novo").click());
  } else if (criandoWorkspace) {
    add("Voltar aos espaços", () => { location.hash = ""; });
  } else if (workspaceAtual) {
    if (idBlueprint) {
      if (idBlueprint !== tela && !linhaBlueprint) add("Abrir blueprint", () => irBlueprint(idBlueprint));
      add("Editar blueprint", () => editar(true, { proj: idBlueprint }));
      add("Planejar este blueprint", () => agenteTela(true, { proj: idBlueprint }));
      if (idBlueprint === tela) add(abertos[idBlueprint] ? "Recolher ações" : "Expandir ações", () => alternar(idBlueprint));
    } else {
      add("Editar dados", () => { editar(true); $("geral").open = true; });
    }
    add(tela ? "Voltar aos blueprints" : "Voltar aos espaços", () => { if (tela) irHome(); else location.hash = ""; });
  } else if (nomeUsuario) {
    if (!novoWorkspace) add("Criar workspace", () => $("entrada-blueprints").querySelector("[data-adicionar-workspace]")?.click());
  } else {
    add("Entrar no sistema", () => $("login-nome").focus());
  }
  add(document.fullscreenElement ? "Sair da tela cheia" : "Entrar em tela cheia", alternarTelaCheia);

  menuContexto.replaceChildren(...acoes.map(({ rotulo, fn }) => {
    const botao = document.createElement("button");
    botao.type = "button";
    botao.setAttribute("role", "menuitem");
    botao.tabIndex = -1;
    botao.textContent = rotulo;
    botao.addEventListener("click", () => { fecharMenuContexto(); fn(); });
    return botao;
  }));
  menuContexto.hidden = false;
  const caixa = alvo?.getBoundingClientRect();
  const x = e.clientX || caixa?.left || 8, y = e.clientY || caixa?.bottom || 8;
  menuContexto.style.left = Math.max(8, Math.min(x, innerWidth - menuContexto.offsetWidth - 8)) + "px";
  menuContexto.style.top = Math.max(8, Math.min(y, innerHeight - menuContexto.offsetHeight - 8)) + "px";
  menuContexto.querySelector("button")?.focus({ preventScroll: true });
}, true);
document.addEventListener("pointerdown", (e) => { if (!menuContexto.hidden && !menuContexto.contains(e.target)) fecharMenuContexto(); }, true);
document.addEventListener("keydown", (e) => {
  if (menuContexto.hidden) return;
  const itens = [...menuContexto.querySelectorAll('[role="menuitem"]')];
  const atual = itens.indexOf(document.activeElement);
  if (e.key === "Escape") { e.preventDefault(); e.stopImmediatePropagation(); fecharMenuContexto(true); }
  else if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
    e.preventDefault();
    const proximo = e.key === "Home" ? 0 : e.key === "End" ? itens.length - 1 : (atual + (e.key === "ArrowDown" ? 1 : -1) + itens.length) % itens.length;
    itens[proximo]?.focus();
  } else if (e.key === "Tab") fecharMenuContexto(true);
}, true);
addEventListener("scroll", () => fecharMenuContexto(), true);
addEventListener("resize", () => fecharMenuContexto());
