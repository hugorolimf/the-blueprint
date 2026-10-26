// ---- a folha da ação (a nota "Instruções da ação") e o botão "Iniciar com o Claude": abre uma sessão nova do Claude Code em terminal novo (POST /api/mcp/iniciar) ----
// Contrato em docs/claude-na-folha.md ("Claude esperando e iniciar a ação"). Só por clique confirmado do usuário; o app não manda nada a uma sessão já aberta.
// Quem chama: nota.js (o ícone de instruções da ação), claude-camada.js (clique no Claude esperando).
function abreFolhaDaAcao(pid, fid, ancora) {
  const p = D.blueprints.find((x) => x.id === pid), f = fid && p?.acoes.find((x) => x.id === fid);
  if (!p) return;
  const dono = f || p; // a folha da ação, ou a observação do blueprint quando não há ação
  abreNota({
    rotulo: f ? tr("Instruções da ação") : tr("Observação do blueprint"), titulo: f ? `${numeroDaAcao(p, f)} · ${f.nome || tr("sem nome")}` : p.nome || tr("Sem nome"), texto: dono.obs || "",
    editar: { dica: f ? tr("O que fazer nesta ação, passo a passo: o roteiro, as decisões, as dependências e os riscos…") : tr("Anotações do blueprint…"), salvar: (v) => { const novo = v.trim() ? v : ""; if (novo === (dono.obs || "")) return; dono.obs = novo; mudou(); } },
    acoes: [...(f && f.status !== "concluido" ? [{ rotulo: tr("Iniciar com o Claude"), fn: () => iniciarComClaude(p, f) }] : []), ...(f ? [{ rotulo: tr("Conversar com o plano"), fn: () => abreConversa(p.id, f.id) }] : [])], // conversa.js
  }, ancora);
  if (f) ldtNaFolha(p, f); // o que o agente fez nesta ação (linha-do-tempo.js)
}

// a resposta de POST /api/mcp/iniciar vira uma frase para o usuário
function iniciarMensagem(status, j, numero, emSessao) {
  if (status === 200 || j?.ok) return emSessao ? tr("Pedido enviado à sessão: o Claude vai começar a ação {numero}.", { numero }) : tr("Terminal aberto: o Claude vai começar a ação {numero}.", { numero });
  if (j?.codigo === "sessao-caiu") return tr("Essa sessão não está mais conectada ao app. Escolha outra ou abra uma nova.");
  if (j?.codigo === "sessao-de-outra-pasta") return tr("Essa sessão não é de uma pasta deste blueprint. Escolha outra ou abra uma nova.");
  if (j?.codigo === "sem-pasta" || status === 409) return tr("Esta pasta não está ligada a um blueprint. Em Editar, preencha “Pasta do projeto” neste blueprint e tente de novo.");
  if (j?.codigo === "sem-terminal" || status === 501) return tr("Não encontrei um terminal para abrir (Windows Terminal ou o prompt do Windows).") + (j?.erro ? " " + tr("Motivo: {erro}", { erro: j.erro }) : "");
  if (status === 404) return tr("O app não achou esta ação. Recarregue a página e tente de novo.");
  return j?.erro || tr("Não foi possível iniciar a ação (erro {status}).", { status });
}
// onde o resultado aparece: na própria folha da ação, se ainda aberta (a tela do blueprint não tem outro aviso visível); senão no aviso comum
function iniciarAviso(texto, erro) {
  if (notaEl.hidden) return aviso(texto);
  let el = notaEl.querySelector(".iniciar-aviso");
  if (!el) { el = document.createElement("p"); el.className = "iniciar-aviso"; el.setAttribute("role", "status"); notaEl.querySelector(".acoes").before(el); }
  el.textContent = texto;
  el.classList.toggle("erro", !!erro);
}
// as sessões do Claude já abertas neste blueprint ({id, pasta, canal}); sem o app ou com erro, lista vazia (o diálogo volta a ser o de sempre)
async function sessoesAbertas(p) {
  try {
    const r = await fetch(`/api/mcp/sessoes?workspace=${encodeURIComponent(workspaceAtual?.id || "sm")}&blueprint=${encodeURIComponent(p.id)}`);
    return r.ok ? (await r.json()).sessoes || [] : [];
  } catch { return []; }
}
// o diálogo com a escolha: "Nova sessão" (padrão) ou uma sessão aberta. Devolve null (cancelou), {nova: true} ou {sessao}
function escolherSessao({ titulo, texto, sessoes }) {
  return new Promise((resolve) => {
    const d = document.createElement("dialog");
    d.className = "confirmar escolher-sessao";
    d.setAttribute("role", "alertdialog");
    d.setAttribute("aria-label", titulo);
    const nome = (s) => (s.pasta || "").replace(/[\\/]+$/, "").split(/[\\/]/).pop() || s.pasta;
    const linhas = sessoes.map((s, i) => `<label><input type="radio" name="sessao" value="${i}"> <span>${tr("Sessão em {nome}", { nome: esc(nome(s)) })}</span> <small>${s.canal ? tr("recebe o pedido direto") : tr("sem canal: só copia o pedido")}</small></label>`).join("");
    d.innerHTML = `<div class="cu-topo"><h3>${esc(titulo)}</h3></div><p class="cu-corpo">${esc(texto)}</p><div class="cu-opcoes"><label><input type="radio" name="sessao" value="nova" checked> <span>${tr("Nova sessão")}</span> <small>${tr("abre um terminal novo")}</small></label>${linhas}</div><div class="confirmar-acoes"><button class="btn" type="button" data-r="0">${tr("Cancelar")}</button><button class="btn perigo" type="button" data-r="1">${tr("Iniciar")}</button></div>`;
    d.addEventListener("click", (e) => { if (e.target === d) d.close("0"); });
    d.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => d.close(b.dataset.r)));
    d.addEventListener("close", () => {
      const v = d.querySelector('input[name="sessao"]:checked')?.value;
      d.remove();
      resolve(d.returnValue !== "1" ? null : v === "nova" ? { nova: true } : { sessao: sessoes[Number(v)] });
    });
    document.body.append(d);
    d.showModal();
    d.querySelector('[data-r="0"]').focus();
  });
}
async function iniciarComClaude(p, f) {
  const numero = numeroDaAcao(p, f), ocupada = agentesDoBlueprint(p.id).some((x) => x.acaoId === f.id && x.estado !== "ocioso");
  const sessoes = await sessoesAbertas(p);
  const base = `${tr("Gasta tokens.")}${ocupada ? " " + tr("Já há um agente trabalhando nesta ação; o novo vai trabalhar junto com ele.") : ""} ${tr("Iniciar a ação {numero} «{nome}»?", { numero, nome: f.nome || tr("sem nome") })}`;
  let escolha = { nova: true };
  if (sessoes.length) escolha = await escolherSessao({ titulo: tr("Iniciar a ação {numero} com o Claude?", { numero }), texto: base, sessoes });
  else if (!(await confirmar({ titulo: tr("Iniciar a ação {numero} com o Claude?", { numero }), texto: tr("Abre um terminal novo com o Claude Code. {base}", { base }), rotulo: tr("Iniciar") }))) escolha = null;
  if (!escolha) return;
  const sessao = escolha.sessao;
  iniciarAviso(sessao ? (sessao.canal ? tr("Enviando o pedido à sessão…") : tr("Copiando o pedido…")) : tr("Abrindo o terminal…"));
  try {
    const corpo = { workspaceId: workspaceAtual?.id || "sm", blueprintId: p.id, acaoId: f.id };
    if (sessao) Object.assign(corpo, { sessaoId: sessao.id, ...(sessao.canal ? {} : { semCanal: true }) });
    const r = await fetch("/api/mcp/iniciar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
    const j = await r.json().catch(() => ({}));
    if (sessao && !sessao.canal && r.ok && j.prompt) { // sem canal não há entrega: o pedido vai para a área de transferência
      try { await navigator.clipboard.writeText(j.prompt); iniciarAviso(tr("Pedido copiado: cole na sessão para o Claude começar a ação {numero}.", { numero })); }
      catch { iniciarAviso(tr("Não consegui copiar sozinho. O pedido é: {prompt}", { prompt: j.prompt }), true); }
      return;
    }
    iniciarAviso(iniciarMensagem(r.status, j, numero, !!sessao), !r.ok);
  } catch { iniciarAviso(tr("Não consegui falar com o app para abrir o terminal. Ele está aberto?"), true); }
}
