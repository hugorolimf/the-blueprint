// ---- o Claude na folha, parte DOM: uma figura por agente ativo, numa camada à parte (#claude-camada) para o render() da folha não a perder ----
// A lógica (quem aparece, onde, quantos) está em claude-logica.js. Aqui: ler no SVG da folha onde fica o plano, posicionar a figura, andar até lá.
// Zero trabalho em repouso: cada pose é desenhada uma vez (cache); só enquanto a figura se desloca há UMA animação (WAAPI) e um relógio que alterna os 2 quadros,
// e os dois acabam no fim do deslocamento. Parada, a pose anima por quadros num relógio à parte (claude-vida.js), só com a figura à vista. Com prefers-reduced-motion ou aba oculta ela aparece no destino sem andar.
const camadaClaude = $("claude-camada");
const claudeFiguras = new Map(); // nome do agente (ou "~evento") -> { el, lugar, x, y, chave, anim, relogio }
const claudeSvgs = new Map(); // papel|pose|quadro -> html do SVG (claudeChaveSvg)
let claudeMod = null, claudePedido = false, claudeAjuste = 0, claudeTela, claudeTrocou = false; // claudeTrocou: a tela mudou, então as figuras só aparecem no lugar (não andam entre telas)

function claudeCarrega() { // só quando há agente: traz o desenho (agentes-claude.mjs) e o kit
  if (claudePedido) return;
  claudePedido = true;
  Promise.all([import("/agentes-claude.mjs"), emblemasProntos]).then(([m]) => { claudeMod = m; claudeCamadaAtualiza(); montaRobo(); }).catch(() => { claudePedido = false; });
}
function claudeSvg(papel, pose, quadro = 0) {
  const chave = claudeChaveSvg(papel, pose, quadro);
  if (!claudeSvgs.has(chave)) {
    const args = { pose, papel, quadro }, l = claudeLarguraDe(papel), e = EMB.desenhaEmblema(claudeMod.pecasDoClaude(args), { ...claudeMod.OPCOES, largura: 220, altura: 170, pad: 6 });
    claudeSvgs.set(chave, `<svg class="emblema" viewBox="0 0 220 170" width="${l}" height="${claudeAlturaDe(papel)}" role="img" aria-label="${esc(claudeMod.descricaoDoClaude(args))}">${e.svg}</svg>`);
  }
  return claudeSvgs.get(chave);
}
// a faixa fixa #robo-agente usa a mesma figura (do Claude principal) no lugar do robozinho
const claudeMiniatura = (ev) => (claudeMod && EMB ? claudeSvg(null, claudeMod.poseDoEstado(claudeEstadoDoEvento(ev), false)) : "");

// ---- onde ficam as coisas na folha: lê as caixas (x, y, w, h) no SVG e converte para a camada; vazio se o lugar não está desenhado ----
function claudeAncoras(svg) {
  const k = svg.getBoundingClientRect(), c = camadaClaude.getBoundingClientRect(), fator = k.width / (svg.viewBox.baseVal.width || k.width || 1);
  const caixa = (r) => r && { x: k.left - c.left + r.x.baseVal.value * fator, y: k.top - c.top + r.y.baseVal.value * fator, w: r.width.baseVal.value * fator, h: r.height.baseVal.value * fator };
  const q = (s) => host.querySelector(s);
  return {
    plano: (id) => caixa(q(`.robo-linha[data-plano="${CSS.escape(id)}"]`)),
    acao: (id) => (q(`.acao[data-f="${CSS.escape(id)}"] .cota`) ? caixa(q(`.acao[data-f="${CSS.escape(id)}"] .caixa`)) : null), // a barra da ação, só com ela aberta (as compactas não têm espaço para a figura)
    barra: (id) => caixa(q(`.acao[data-f="${CSS.escape(id)}"] .caixa`)), // a barra da ação, aberta ou compacta (o agente fica ao lado dela quando a linha do plano não está desenhada)
    // a chave da seção (o projeto) da ação: a da seção em que a ação está desenhada ou, se ela não está à vista (filtro, período), a do projeto dela nos dados
    secao: (id) => {
      const g = q(`.acao[data-f="${CSS.escape(id)}"]`)?.closest("[data-proj]")?.querySelector("[data-projeto], [data-toggle]");
      if (g) return g.dataset.projeto ?? g.dataset.toggle;
      const p = D.blueprints.find((x) => x.id === tela), f = p?.acoes.find((x) => x.id === id);
      if (!f) return null;
      return p.projetos?.length ? chaveProjeto(p, projetoDaAcao(p, f)) : p.id;
    },
    caixaSecao: (chave) => caixa(q(`[data-projeto="${CSS.escape(chave)}"] .cab, [data-toggle="${CSS.escape(chave)}"] .cab`)),
    lugar: (id) => (tela ? { tipo: "cab", ...caixa(q(`[data-proj="${CSS.escape(id)}"] .cab`)) } : { tipo: "cartao", ...caixa(q(`[data-goto="${CSS.escape(id)}"] .moldura`)) }),
  };
}
// o Claude esperando (só da interface): a próxima ação do blueprint aberto, ou null (na lista de blueprints não há espera)
function claudeEsperaDe(id, ags) {
  const p = tela === id ? D.blueprints.find((x) => x.id === id) : null, f = p && claudeProximaAcao(p, ags);
  return f ? { p, f } : null;
}
function claudeItensDaTela(ancoras) { // todos os itens da tela: o blueprint aberto ou, na lista, um grupo por cartão
  const ids = tela ? [tela] : D.blueprints.map((p) => p.id), todos = [];
  for (const id of ids) {
    const ags = agentesDoBlueprint(id), esp = claudeEsperaDe(id, ags);
    if (!ags.length && !esp) continue;
    const base = ancoras.lugar(id), onde = { barra: (fid) => !!ancoras.barra(fid), secao: (fid) => (ancoras.caixaSecao(ancoras.secao(fid) || "")?.w ? ancoras.secao(fid) : null) };
    const itens = ags.length ? claudeItens(ags, (pid) => !!ancoras.plano(pid), onde) : [];
    if (esp) itens.push(claudeItemEspera(esp.p, esp.f, numeroDaAcao(esp.p, esp.f), (fid) => !!ancoras.acao(fid), itens, onde.secao)); // fora do limite de figuras: não é agente
    for (const it of itens) {
      const [tipoLugar, idLugar] = it.lugar.split(/:(.*)/s), caixa = tipoLugar === "plano" ? ancoras.plano(idLugar) : tipoLugar === "acao" ? ancoras.acao(idLugar) : tipoLugar === "barra" ? ancoras.barra(idLugar) : tipoLugar === "secao" ? ancoras.caixaSecao(idLugar) : base;
      if (caixa?.w) todos.push({ ...it, caixa, tipo: tipoLugar === "cab" || tipoLugar === "secao" ? base.tipo : "plano", bp: id }); // cabeçalho (do blueprint ou da seção) tem o seu jeito de apoiar; o resto fica ao lado da linha/barra
    }
  }
  return todos;
}

// ---- reconcilia a camada com a folha: cria, move, troca de pose e remove ----
function claudeCamadaAtualiza() {
  const svg = host.querySelector("svg.desenho"), ativos = svg && workspaceAtual && carregado ? agentesAtivos().length || agenteAqui() || (tela && claudeEsperaDe(tela, agentesDoBlueprint(tela))) : null;
  if (ativos) claudeCarrega();
  if (!ativos || !claudeMod || !EMB) { claudeLimpa(new Set()); return; }
  const itens = claudeItensDaTela(claudeAncoras(svg)), vistos = new Set();
  claudeTrocou = tela !== claudeTela;
  claudeTela = tela;
  const grupo = new Map(); // lugar -> larguras dos itens, na ordem
  itens.forEach((it) => { const l = grupo.get(it.bp + it.lugar) || []; l[it.indice] = claudeLarguraItem(it); grupo.set(it.bp + it.lugar, l); });
  const principais = new Map(itens.filter((it) => it.principal).map((it) => [it.bp, it.a]));
  for (const it of itens) {
    const pos = claudePosicao(it.tipo, it.caixa, it.indice, grupo.get(it.bp + it.lugar)), nome = it.a.agente || "~evento";
    vistos.add(nome);
    claudePoe(nome, it, pos, it.espera ? it.a : principais.get(it.bp)); // a espera é do Claude principal (sem papel)
  }
  claudeLimpa(vistos);
}
// Aparecer e sumir: a figura cresce a partir do centro ao entrar na folha e encolhe até o centro ao sair (pedido do usuário, 2026-10-22). A propriedade `scale` compõe com o
// translate do transform e gira em torno do centro da figura. Sem animação com prefers-reduced-motion ou aba oculta (aparece e some na hora).
function claudeEntra(el) {
  if (claudeAnimar()) el.animate([{ scale: "0", opacity: 0 }, { scale: "1", opacity: 1 }], { duration: 380, easing: "cubic-bezier(.2,.8,.3,1)" });
}
function claudeSai(el) {
  if (!claudeAnimar()) { el.remove(); return; }
  el.inert = true; // já saiu: sem clique nem foco enquanto encolhe
  el.dataset.saindo = "";
  el.animate([{ scale: "1", opacity: 1 }, { scale: "0", opacity: 0 }], { duration: 300, easing: "cubic-bezier(.5,0,.8,.4)", fill: "forwards" }).finished.then(() => el.remove(), () => el.remove());
}
function claudeLimpa(vistos) {
  for (const [nome, r] of claudeFiguras) if (!vistos.has(nome)) { claudeParaAndar(r); claudeSoltaVigia(r); claudeSai(r.el); claudeFiguras.delete(nome); }
}
function claudePoe(nome, it, pos, principal) {
  const a = it.a, y = pos.base - claudeAlturaDe(it.papel), lugar = `${it.bp}|${it.lugar}`;
  let r = claudeFiguras.get(nome);
  const novo = !r;
  if (novo) {
    const el = document.createElement("div");
    el.className = "claude-fig";
    el.style.width = `${claudeLarguraDe(it.papel)}px`; // tamanho fixo: os quadros ficam empilhados (position:absolute) e a figura não depende deles
    el.style.height = `${claudeAlturaDe(it.papel)}px`;
    el.tabIndex = 0;
    el.setAttribute("role", "button");
    el.dataset.roboNome = a.agente || "";
    camadaClaude.append(el);
    claudeFiguras.set(nome, r = { el, lugar, x: pos.x, y });
    claudeVigia(r); // o relógio dos quadros (claude-vida.js) só liga com a figura à vista
    el.style.transform = `translate(${pos.x}px,${y}px)`;
    claudeEntra(el);
  }
  if (a.espera) { r.el.dataset.espera = a.acaoId; r.el.dataset.bp = a.blueprintId; } // o clique abre a folha desta ação (abreFolhaDaAcao)
  const f = claudeFigura(a, principal, false, claudeMod.poseDoEstado);
  r.dados = { a, f, extras: it.extras || 0 };
  r.el.style.zIndex = 20 - it.indice; // juntas, a mais perto do lugar (o principal) fica por cima
  claudeRotula(r);
  if (novo) return;
  const moveu = Math.abs(r.x - pos.x) > 1 || Math.abs(r.y - y) > 1;
  if (moveu && (claudeTrocou || (!r.anim && r.lugar === lugar))) claudeFixa(r, pos.x, y); // outra tela, ou a folha só mudou de medida: sem andar
  else if (r.lugar !== lugar && moveu) claudeAnda(r, pos.x, y);
  else if (r.anim && (Math.abs(r.destX - pos.x) > 1 || Math.abs(r.destY - y) > 1)) claudeAnda(r, pos.x, y); // mudou o destino no meio do caminho: segue dali
  r.lugar = lugar;
}
function claudeFixa(r, x, y) {
  claudeParaAndar(r);
  r.el.style.transform = `translate(${x}px,${y}px)`;
  r.x = x; r.y = y; r.chave = "";
  claudeRotula(r);
}
function claudeRotula(r) { // aria, dica e SVG da pose; durante o deslocamento o SVG só troca no fim
  const { a, f, extras } = r.dados;
  r.el.setAttribute("aria-label", a.espera ? "Claude esperando: abrir a ação para iniciar" : `Ver o que ${a.agente || "o agente"} está fazendo`);
  r.el.title = a.espera ? `${a.texto}. Clique para abrir a ação e iniciar com o Claude.` : `${a.agente || "Agente"}${a.texto ? ": " + umaLinha(a.texto, 140) : ""}${extras ? ` (mais ${extras} agente${extras > 1 ? "s" : ""} junto${extras > 1 ? "s" : ""} aqui)` : ""}`;
  r.el.classList.toggle("erro", f.pose === "bloqueado");
  if (r.chave !== f.chave && !r.anim) { r.el.innerHTML = claudeHtmlPose(f.papel, f.pose); r.chave = f.chave; claudeVidaLiga(r); }
}

// ---- andar: um deslocamento por transform; a pose "andando" alterna os 2 quadros só enquanto dura ----
const claudeAnimar = () => !reduz() && !document.hidden && typeof Element.prototype.animate === "function";
function claudeParaAndar(r) {
  clearInterval(r.relogio);
  r.relogio = 0;
  claudeVidaPara(r);
  if (r.anim) { const an = r.anim; r.anim = null; an.onfinish = an.oncancel = null; an.cancel(); }
}
function claudeAnda(r, x, y) {
  const { a, f } = r.dados;
  const de = r.el.getBoundingClientRect(), c = camadaClaude.getBoundingClientRect(), x0 = de.left - c.left, y0 = de.top - c.top;
  claudeParaAndar(r);
  r.x = x; r.y = y; r.destX = x; r.destY = y;
  r.el.style.transform = `translate(${x}px,${y}px)`;
  if (!claudeAnimar()) { r.chave = ""; claudeRotula(r); return; }
  const anda = claudeMod.poseDoEstado(claudeEstadoDoEvento(a), true) === "andando";
  if (anda) { // os quadros do passo ficam prontos; o relógio só liga um e desliga os outros, na ordem de SEQUENCIA.andando
    const seq = claudeMod.SEQUENCIA.andando;
    r.el.innerHTML = claudeQuadrosUnicos(seq).map((q) => `<span data-q="${q}"${q ? "" : ' class="on"'}>${claudeSvg(f.papel, "andando", q)}</span>`).join("");
    r.chave = "andando";
    let i = 0;
    r.relogio = setInterval(() => { const q = String(seq[++i % seq.length]); r.el.querySelectorAll("span").forEach((s) => s.classList.toggle("on", s.dataset.q === q)); }, claudeMod.RITMO.andando);
  }
  const fim = () => { claudeParaAndar(r); r.chave = ""; claudeRotula(r); };
  const dist = Math.hypot(x - x0, y - y0);
  r.anim = r.el.animate([{ transform: `translate(${x0}px,${y0}px)` }, { transform: `translate(${x}px,${y}px)` }], { duration: Math.min(1600, Math.max(400, dist * 4)), easing: "ease-in-out" });
  r.anim.onfinish = r.anim.oncancel = fim;
}
document.addEventListener("visibilitychange", () => { if (document.hidden) claudeFiguras.forEach((r) => r.anim?.finish()); }); // aba oculta: chega ao destino
addEventListener("resize", () => { cancelAnimationFrame(claudeAjuste); claudeAjuste = requestAnimationFrame(claudeCamadaAtualiza); }); // a folha mudou de largura: reposiciona

// clicar na figura abre a folhinha do que o agente faz agora (só leitura; abreNotaRobo está em robo.js)
const claudeDe = (e) => e.target.closest?.("[data-robo-nome]");
const claudeAbre = (f) => (f.dataset.espera ? abreFolhaDaAcao(f.dataset.bp, f.dataset.espera, f) : abreNotaRobo(f)); // a espera abre a folha da ação; o agente, o que ele faz
camadaClaude.addEventListener("click", (e) => { const f = claudeDe(e); if (f) claudeAbre(f); });
camadaClaude.addEventListener("keydown", (e) => { const f = (e.key === "Enter" || e.key === " ") && claudeDe(e); if (f) { e.preventDefault(); claudeAbre(f); } });
