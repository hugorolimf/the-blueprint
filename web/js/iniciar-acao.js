// ---- a folha da ação (a nota "Instruções da ação") e o botão "Iniciar com o Claude": abre uma sessão nova do Claude Code em terminal novo (POST /api/mcp/iniciar) ----
// Contrato em docs/claude-na-folha.md ("Claude esperando e iniciar a ação"). Só por clique confirmado do usuário; o app não manda nada a uma sessão já aberta.
// Quem chama: nota.js (o ícone de instruções da ação), claude-camada.js (clique no Claude esperando).
function abreFolhaDaAcao(pid, fid, ancora) {
  const p = D.blueprints.find((x) => x.id === pid), f = fid && p?.acoes.find((x) => x.id === fid);
  if (!p) return;
  const dono = f || p; // a folha da ação, ou a observação do blueprint quando não há ação
  abreNota({
    rotulo: f ? "Instruções da ação" : "Observação do blueprint", titulo: f ? `${numeroDaAcao(p, f)} · ${f.nome || "sem nome"}` : p.nome || "Sem nome", texto: dono.obs || "",
    editar: { dica: f ? "O que fazer nesta ação, passo a passo: o roteiro, as decisões, as dependências e os riscos…" : "Anotações do blueprint…", salvar: (v) => { const novo = v.trim() ? v : ""; if (novo === (dono.obs || "")) return; dono.obs = novo; mudou(); } },
    acoes: f && f.status !== "concluido" ? [{ rotulo: "Iniciar com o Claude", fn: () => iniciarComClaude(p, f) }] : [],
  }, ancora);
}

// a resposta de POST /api/mcp/iniciar vira uma frase para o usuário
function iniciarMensagem(status, j, numero, emSessao) {
  if (status === 200 || j?.ok) return emSessao ? `Pedido enviado à sessão: o Claude vai começar a ação ${numero}.` : `Terminal aberto: o Claude vai começar a ação ${numero}.`;
  if (j?.codigo === "sessao-caiu") return "Essa sessão não está mais conectada ao app. Escolha outra ou abra uma nova.";
  if (j?.codigo === "sessao-de-outra-pasta") return "Essa sessão não é de uma pasta deste blueprint. Escolha outra ou abra uma nova.";
  if (j?.codigo === "sem-pasta" || status === 409) return "Esta pasta não está ligada a um blueprint. Em Editar, preencha “Pasta do projeto” neste blueprint e tente de novo.";
  if (j?.codigo === "sem-terminal" || status === 501) return `Não encontrei um terminal para abrir (Windows Terminal ou o prompt do Windows).${j?.erro ? ` Motivo: ${j.erro}` : ""}`;
  if (status === 404) return "O app não achou esta ação. Recarregue a página e tente de novo.";
  return j?.erro || `Não foi possível iniciar a ação (erro ${status}).`;
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
    const linhas = sessoes.map((s, i) => `<label><input type="radio" name="sessao" value="${i}"> <span>Sessão em ${esc(nome(s))}</span> <small>${s.canal ? "recebe o pedido direto" : "sem canal: só copia o pedido"}</small></label>`).join("");
    d.innerHTML = `<div class="cu-topo"><h3>${esc(titulo)}</h3></div><p class="cu-corpo">${esc(texto)}</p><div class="cu-opcoes"><label><input type="radio" name="sessao" value="nova" checked> <span>Nova sessão</span> <small>abre um terminal novo</small></label>${linhas}</div><div class="confirmar-acoes"><button class="btn" type="button" data-r="0">Cancelar</button><button class="btn perigo" type="button" data-r="1">Iniciar</button></div>`;
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
  const base = `Gasta tokens.${ocupada ? " Já há um agente trabalhando nesta ação; o novo vai trabalhar junto com ele." : ""} Iniciar a ação ${numero} «${f.nome || "sem nome"}»?`;
  let escolha = { nova: true };
  if (sessoes.length) escolha = await escolherSessao({ titulo: `Iniciar a ação ${numero} com o Claude?`, texto: base, sessoes });
  else if (!(await confirmar({ titulo: `Iniciar a ação ${numero} com o Claude?`, texto: `Abre um terminal novo com o Claude Code e ${base[0].toLowerCase()}${base.slice(1)}`, rotulo: "Iniciar" }))) escolha = null;
  if (!escolha) return;
  const sessao = escolha.sessao;
  iniciarAviso(sessao ? (sessao.canal ? "Enviando o pedido à sessão…" : "Copiando o pedido…") : "Abrindo o terminal…");
  try {
    const corpo = { workspaceId: workspaceAtual?.id || "sm", blueprintId: p.id, acaoId: f.id };
    if (sessao) Object.assign(corpo, { sessaoId: sessao.id, ...(sessao.canal ? {} : { semCanal: true }) });
    const r = await fetch("/api/mcp/iniciar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
    const j = await r.json().catch(() => ({}));
    if (sessao && !sessao.canal && r.ok && j.prompt) { // sem canal não há entrega: o pedido vai para a área de transferência
      try { await navigator.clipboard.writeText(j.prompt); iniciarAviso("Pedido copiado: cole na sessão para o Claude começar a ação " + numero + "."); }
      catch { iniciarAviso("Não consegui copiar sozinho. O pedido é: " + j.prompt, true); }
      return;
    }
    iniciarAviso(iniciarMensagem(r.status, j, numero, !!sessao), !r.ok);
  } catch { iniciarAviso("Não consegui falar com o app para abrir o terminal. Ele está aberto?", true); }
}
