// ---- os itens do menu do botão direito: o que aparece para cada alvo, em que grupo e com que ícone (quem desenha é contexto-menu.js, quem escuta é contexto.js) ----
// Grupos, na ordem em que aparecem: abrir (ir e ver), editar (mudar), estado (situação e marcar), prazo (datas), perigo (excluir), geral (voltar e tela cheia).
const GRUPOS_CONTEXTO = ["abrir", "editar", "estado", "prazo", "perigo", "geral"];

/** O nome do alvo para o cabeçalho do menu: o mais específico sob o cursor (plano, ação, blueprint, workspace). */
function tituloDoContexto(alvo, { cartaoWorkspace, linhaBlueprint, acao, plano, idBlueprint }) {
  const bp = (id) => D.blueprints.find((x) => x.id === id);
  const feita = acao && acaoDe(acao.dataset.p, acao.dataset.f);
  if (plano && feita) {
    const it = (plano.dataset.tipo === "t" ? feita.planos : feita.checkpoints).find((x) => x.id === plano.dataset.id);
    return `${plano.dataset.tipo === "t" ? tr("Plano") : tr("Checkpoint")} · ${it?.texto || it?.nome || tr("sem nome")}`;
  }
  if (feita) return `${tr("Ação")} ${numeroDaAcao(bp(acao.dataset.p), feita)} · ${feita.nome || tr("sem nome")}`;
  if (cartaoWorkspace) return `Workspace · ${cartaoWorkspace.querySelector("h2")?.textContent || ""}`;
  if (linhaBlueprint || idBlueprint) return `${tr("Blueprint")} · ${bp(idBlueprint)?.nome || tr("sem nome")}`;
  return workspaceAtual ? `Workspace · ${workspaceAtual.nome}` : "The Blueprint";
}

/** `{ titulo, itens }` do alvo: cada item é `{ rotulo, fn, icone, grupo, perigo? }`. */
function itensDoContexto(alvo) {
  const itens = [];
  const add = (grupo, icone, rotulo, fn, extra = {}) => itens.push({ grupo, icone, rotulo, fn, ...extra });
  const cartaoWorkspace = alvo?.closest("#entrada-blueprints a.blueprint-cartao[href]");
  const novoWorkspace = alvo?.closest("#entrada-blueprints [data-adicionar-workspace]");
  const linhaBlueprint = alvo?.closest("[data-goto]");
  const acao = alvo?.closest(".acao[data-p][data-f]");
  const plano = alvo?.closest(".chk[data-p][data-f][data-id]");
  const nota = alvo?.closest(".nota[data-obs]");
  const marca = alvo?.closest(".marca");
  const link = alvo?.closest("a.lk[href]");
  const figura = alvo?.closest("#claude-camada [data-robo-nome]"); // a figura do Claude: o plano em que ele trabalha (claude-camada.js põe os ids)
  const criandoWorkspace = location.hash === "#/workspace/novo";
  const idBlueprint = linhaBlueprint?.dataset.goto || acao?.dataset.p || plano?.dataset.p || nota?.dataset.obs.split("|")[0] || tela;

  if (cartaoWorkspace) add("abrir", "abrir", tr("Abrir workspace"), () => { location.hash = cartaoWorkspace.getAttribute("href").slice(1); });
  if (cartaoWorkspace && cartaoWorkspace.dataset.workspace !== "sm") add("perigo", "lixo", "Excluir workspace…", () => excluirWorkspace(cartaoWorkspace.dataset.workspace), { perigo: true });
  if (novoWorkspace) add("editar", "mais", "Criar workspace", () => novoWorkspace.click());
  if (linhaBlueprint) {
    add("abrir", "abrir", tr("Abrir blueprint"), () => irBlueprint(linhaBlueprint.dataset.goto));
    add("perigo", "lixo", "Excluir blueprint…", () => excluirBlueprint(linhaBlueprint.dataset.goto), { perigo: true });
  }
  if (acao) {
    const feita = acaoDe(acao.dataset.p, acao.dataset.f), dono = D.blueprints.find((x) => x.id === acao.dataset.p);
    if (feita && efetiva(feita).status !== "concluido") add("estado", "play", tr("Iniciar com o Claude…"), () => iniciarComClaude(dono, feita)); // pede confirmação (gasta tokens, abre terminal)
    if (feita) add("estado", "doc", tr("Conversar com o plano…"), () => abreConversa(acao.dataset.p, acao.dataset.f)); // conversa.js
    if (feita) for (const { grupo, icone, rotulo, fn, ...extra } of itensDeSituacao(acao.dataset.p, feita)) add(grupo, icone, rotulo, fn, extra);
    add("editar", "lapis", tr("Editar ação"), () => editar(true, { proj: acao.dataset.p, sel: acao.dataset.f }));
    // abre o Editar na ação e aciona o "+ Plano" ou "+ Checkpoint" dela: o campo do item novo já fica em foco
    const adicionar = (botao) => () => { editar(true, { proj: acao.dataset.p, sel: acao.dataset.f }); document.querySelector(`#mn-detalhe [data-act="${botao}"]`)?.click(); };
    add("editar", "mais", tr("Adicionar plano"), adicionar("add-plano"));
    add("editar", "losango", tr("Adicionar checkpoint"), adicionar("add-checkpoint"));
    if (acaoDe(acao.dataset.p, acao.dataset.f)?.prototipos?.length) add("abrir", "tela", tr("Ver protótipo"), () => abreProtos(acao.dataset.p, acao.dataset.f));
    add("abrir", "alvo", tr("Ir para esta ação"), () => irPara(acao.dataset.p, acao.dataset.f, alvo.closest(".checkpoint")?.dataset.m));
    add("perigo", "lixo", tr("Excluir ação…"), () => excluirAcao(acao.dataset.p, acao.dataset.f, true), { perigo: true });
  }
  if (plano) {
    const feito = plano.getAttribute("aria-checked") === "true";
    add("estado", feito ? "circulo" : "check", feito ? "Marcar como pendente" : "Marcar como feito", () => alternarItem(plano.dataset.p, plano.dataset.f, plano.dataset.tipo, plano.dataset.id));
    if (plano.dataset.tipo === "t") for (const { icone, rotulo, fn } of itensDeResponsavel(plano.dataset.p, plano.dataset.f, plano.dataset.id)) add("editar", icone, rotulo, fn); // responsavel-logica.js (só no modo hospedado)
    if (plano.dataset.tipo === "t") add("estado", "doc", tr("Enviar comentário ao agente…"), () => enviarComentarioAoAgente(plano.dataset.p, plano.dataset.f, plano.dataset.id)); // comentario-agente.js
    const noPlano = plano.dataset.tipo === "t" && agentesAtivos().find((a) => a.planoId === plano.dataset.id); // o agente que trabalha neste plano, se houver
    if (noPlano) add("abrir", "alvo", tr("Acompanhar este agente"), () => focoAbre(noPlano.agente)); // foco.js
  }
  if (figura && !figura.dataset.espera) add("abrir", "alvo", tr("Acompanhar este agente"), () => focoAbre(figura.dataset.roboNome));
  if (nota) add("abrir", "doc", nota.dataset.obs.split("|")[1] ? "Abrir instruções" : "Abrir observações", () => abreNotaObs(nota));
  if (marca) add("abrir", "losango", "Ver marca", () => abreNotaMarca(marca));
  if (link) add("abrir", "externo", tr("Abrir link em nova aba"), () => link.click());

  if (!hub.hidden) {
    add("geral", "voltar", "Voltar ao blueprint", () => agenteTela(false));
  } else if (!ed.hidden) {
    const itemAcao = alvo?.closest(".mn-item[data-sel]");
    if (itemAcao && itemAcao.dataset.sel !== "blueprint") {
      add("editar", "lapis", tr("Editar ação"), () => itemAcao.click());
      add("perigo", "lixo", tr("Excluir ação…"), () => excluirAcao(edProj, itemAcao.dataset.sel, true), { perigo: true });
    }
    add("geral", "voltar", "Voltar ao blueprint", () => editar(false));
    add("editar", "mais", "Criar blueprint", () => $("b-novo").click());
  } else if (criandoWorkspace) {
    add("geral", "voltar", tr("Voltar aos espaços"), () => { location.hash = ""; });
  } else if (workspaceAtual) {
    if (idBlueprint) {
      if (idBlueprint !== tela && !linhaBlueprint) add("abrir", "abrir", tr("Abrir blueprint"), () => irBlueprint(idBlueprint));
      add("editar", "lapis", tr("Editar blueprint"), () => editar(true, { proj: idBlueprint }));
      add("editar", "estrela", tr("Planejar este blueprint"), () => agenteTela(true, { proj: idBlueprint }));
      if (idBlueprint === tela) {
        const bp = D.blueprints.find((x) => x.id === idBlueprint), aberto = bp?.projetos?.length ? algumProjetoAberto(bp) : abertos[idBlueprint] === true;
        add("geral", aberto ? "recolher" : "expandir", aberto ? tr("Recolher ações") : tr("Expandir ações"), () => abrirOuFecharTudo(idBlueprint, !aberto));
      }
    } else {
      add("editar", "lapis", tr("Editar dados"), () => { editar(true); $("geral").open = true; });
      add("editar", "mais", "Novo blueprint", () => editar(true, { novo: true }));
    }
    add("geral", "voltar", tela ? tr("Voltar aos blueprints") : tr("Voltar aos espaços"), () => { if (tela) irHome(); else location.hash = ""; });
  } else if (nomeUsuario) {
    if (!novoWorkspace) add("editar", "mais", "Criar workspace", () => $("entrada-blueprints").querySelector("[data-adicionar-workspace]")?.click());
  } else {
    add("geral", "entrar", tr("Entrar no sistema"), () => $("login-nome").focus());
  }
  add("geral", "tela_cheia", document.fullscreenElement ? tr("Sair da tela cheia") : tr("Entrar em tela cheia"), alternarTelaCheia);
  itens.sort((a, b) => GRUPOS_CONTEXTO.indexOf(a.grupo) - GRUPOS_CONTEXTO.indexOf(b.grupo)); // estável: dentro do grupo vale a ordem em que foram somados
  return { titulo: tituloDoContexto(alvo, { cartaoWorkspace, linhaBlueprint, acao, plano, idBlueprint }), itens };
}
