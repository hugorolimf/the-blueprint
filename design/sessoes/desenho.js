// Desenha o estado da tela em HTML: o trilho de workspaces, a lista de sessões, a conversa e o ambiente. O comportamento fica em tela.js.
const $ = (id) => document.getElementById(id);
const esc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const fig = (nome) => `<div class="fig">${FIG[nome] ?? FIG.parado}</div>`;
const doWs = (id) => SESSOES.filter((s) => s.ws === id);

function desenhaTrilho() {
  $("workspaces").innerHTML = WORKSPACES.map((w) => {
    const lista = doWs(w.id), pedem = lista.filter((s) => s.estado === "pede").length;
    const vivas = lista.filter((s) => s.estado !== "encerrada").length;
    const linha = pedem ? `<span class="pede">${pedem} ${pedem > 1 ? "pedem" : "pede"} você</span>` : `<span>${vivas ? `${vivas} ${vivas > 1 ? "abertas" : "aberta"}` : "Nenhuma aberta"}</span>`;
    return `<button class="ws ${w.id === vista.ws ? "ativo" : ""}" data-ws="${w.id}"><b>${esc(w.nome)}</b>${linha}</button>`;
  }).join("");
}

function desenhaLista() {
  const w = WORKSPACES.find((x) => x.id === vista.ws), lista = doWs(w.id);
  $("ws-nome").textContent = w.nome;
  $("ws-pasta").textContent = w.pasta;
  $("ambiente").classList.toggle("ativo", vista.modo === "ambiente");
  $("sessoes").innerHTML = GRUPOS.map(([estado, titulo]) => {
    const doGrupo = lista.filter((s) => s.estado === estado);
    if (!doGrupo.length) return "";
    return `<div class="grupo rotulo">${titulo} · ${doGrupo.length}</div>` + doGrupo.map((s) => `
      <button class="sessao ${vista.modo === "conversa" && s.id === vista.sessao ? "ativa" : ""}" data-sessao="${s.id}">
        ${fig(POSE[s.estado])}
        <div class="txt"><b>${esc(s.nome)}</b><small>${s.acao ? `Ação ${s.acao}` : "Sem ação"} · ${s.quando}</small><span class="estado e-${s.estado}">${TEXTO_ESTADO[s.estado]}</span></div>
      </button>`).join("");
  }).join("");
}

function desenhaItem(i, n) {
  if (i.t === "voce") return `<div class="m-voce">${i.fila ? "<i>Na fila: entra quando o turno acabar</i>" : ""}${esc(i.x)}</div>`;
  if (i.t === "claude") return `<div class="m-claude" ${i.vivo ? 'id="fala-viva"' : ""}>${esc(i.x)}</div>`;
  if (i.t === "sis") return `<div class="m-sis">${esc(i.x)}</div>`;
  if (i.t === "ferr") return `<div class="ferr"><b>${esc(i.nome)}</b><span>${esc(i.alvo)}</span></div>`;
  if (i.t === "sub") return `<details class="sub"><summary>${fig(i.papel)}${esc(i.titulo)}<span>${esc(i.resumo)}</span></summary><div class="sub-corpo">${i.itens.map(desenhaItem).join("")}</div></details>`;
  if (i.fim) return `<div class="pedido fim">${esc(i.ferramenta)}: <code>${esc(i.comando)}</code>. ${esc(i.fim)}</div>`;
  return `<div class="pedido"><div class="rotulo">O Claude pede permissão</div><pre>${esc(i.comando)}</pre><p>${esc(i.motivo)} Pasta: ${esc(WORKSPACES.find((w) => w.id === vista.ws).pasta)}</p>
    <div class="pedido-botoes"><button class="primario" data-faz="permitir">Permitir</button><button class="contorno" data-faz="negar">Negar</button><button class="discreto" data-faz="sempre">Sempre nesta pasta</button></div></div>`;
}

function botoesDaSessao(s) {
  if (s.estado === "fora") return `<button class="discreto" data-faz="trazer">Trazer para o app</button>`;
  const b = [];
  if (s.estado === "trabalhando" || s.estado === "pede") b.push(`<button class="discreto" data-faz="interromper">Interromper</button>`);
  if (s.estado === "pronta") b.push(`<button class="discreto" data-faz="encerrar">Encerrar</button>`);
  if (s.estado === "encerrada") b.push(`<button class="contorno" data-faz="retomar">Retomar</button>`);
  b.push(`<button class="discreto" data-faz="terminal">No terminal</button>`);
  return b.join("");
}

function desenhaConversa() {
  const s = SESSOES.find((x) => x.id === vista.sessao);
  const w = WORKSPACES.find((x) => x.id === vista.ws);
  if (!s) {
    $("palco").innerHTML = `<div class="vazio">${fig("parado")}<div>Nenhuma sessão aberta neste workspace.</div><button class="contorno" data-faz="abrir-nova">Começar uma sessão</button></div>`;
    return;
  }
  const contexto = s.contexto == null ? "" : ` · Contexto ${s.contexto}%`;
  const pe = s.estado === "fora"
    ? `<div class="fora-faixa"><span>Esta sessão está no seu terminal. Responda por lá; aqui você só acompanha.</span></div>`
    : `<div class="caixa"><textarea id="msg" placeholder="${s.estado === "encerrada" ? "Escrever retoma a sessão" : "Escreva ao Claude. Enter envia."}"></textarea><button class="contorno" data-faz="enviar">Enviar</button></div>`;
  $("palco").innerHTML = `<header class="cab"><div class="cab-txt"><div class="rotulo">${esc(w.blueprint)}${s.acao ? ` · Ação ${s.acao}` : ""}</div><h2>${esc(s.nome)}</h2>
    <div class="meta"><span class="estado e-${s.estado}">${TEXTO_ESTADO[s.estado]}</span> · ${esc(w.modelo)}${contexto} · sessão ${s.id}</div></div><div class="cab-botoes">${botoesDaSessao(s)}</div></header>
    <div class="conversa" id="conversa">${s.itens.map(desenhaItem).join("")}</div>${pe}`;
  const c = $("conversa");
  c.scrollTop = c.scrollHeight;
}

function desenhaAmbiente() {
  const w = WORKSPACES.find((x) => x.id === vista.ws), vivas = doWs(w.id).filter((s) => !["encerrada", "fora"].includes(s.estado)).length;
  const regras = w.regras.length ? w.regras.map((r, n) => `<div class="linha"><span><code>${esc(r)}</code></span><button data-tira="${n}">Tirar</button></div>`).join("") : `<div class="tecla">Nada ainda. Use «Sempre nesta pasta» num pedido de permissão.</div>`;
  $("palco").innerHTML = `<div class="folha"><div class="rotulo">Ambiente de trabalho</div><h2>${esc(w.nome)}</h2><div class="tecla">Tudo o que as sessões deste workspace herdam ao começar.</div>
    <div class="grade">
      <div class="cartao"><div class="rotulo">Onde trabalha</div><div class="linha"><span>Pasta</span><span>${esc(w.pasta)}</span></div><div class="linha"><span>Blueprint</span><span>${esc(w.blueprint)}</span></div><div class="linha"><span>Sessões rodando no app</span><span>${vivas}</span></div></div>
      <div class="cartao"><div class="rotulo">Claude</div><div class="linha"><span>Versão</span><span>2.1.316</span></div><div class="linha"><span>Conta</span><span>A sua, no próprio Claude</span></div><div class="linha"><span>Modelo ao começar</span><span>${esc(w.modelo)}</span></div></div>
      <div class="cartao"><div class="rotulo">Sempre permitido nesta pasta</div>${regras}</div>
      <div class="cartao"><div class="rotulo">Ligado em toda sessão</div><div class="linha"><span>MCP do Blueprint</span><span class="ok">Ligado</span></div><div class="linha"><span>Plugin e hooks</span><span class="ok">Ligados</span></div><div class="tecla">Se algum falhar ao abrir a sessão, o app avisa em vez de seguir sem ele.</div></div>
    </div></div>`;
}

function desenhaNova() {
  const w = WORKSPACES.find((x) => x.id === vista.ws);
  const linhas = w.acoes.map(([n, t]) => `<button class="${vista.nova === n ? "ativo" : ""}" data-acao="${n}"><span>${n} ${esc(t)}</span><small>${esc(w.blueprint)}</small></button>`).join("");
  $("palco").innerHTML = `<div class="folha"><div class="rotulo">Nova sessão · ${esc(w.nome)}</div><h2>No que o Claude vai trabalhar?</h2>
    <div class="acoes">${linhas}<button class="${vista.nova === "" ? "ativo" : ""}" data-acao=""><span>Conversa solta, sem ação</span><small>Fica só neste workspace</small></button></div>
    <textarea class="nova-prompt" id="nova-texto" placeholder="O que ele deve fazer?"></textarea>
    <div class="pedido-botoes"><button class="primario" data-faz="iniciar">Iniciar sessão</button><button class="discreto" data-faz="cancelar">Cancelar</button></div></div>`;
}

function desenha() {
  desenhaTrilho();
  desenhaLista();
  if (vista.modo === "ambiente") desenhaAmbiente();
  else if (vista.modo === "nova") desenhaNova();
  else desenhaConversa();
}
