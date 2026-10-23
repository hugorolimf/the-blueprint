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
function iniciarMensagem(status, j, numero) {
  if (status === 200 || j?.ok) return `Terminal aberto: o Claude vai começar a ação ${numero}.`;
  if (j?.codigo === "sem-pasta" || status === 409) return "Esta pasta não está ligada a um blueprint. Em Editar, preencha “Pasta do projeto” neste blueprint e tente de novo.";
  if (j?.codigo === "sem-terminal" || status === 501) return "Não encontrei um terminal para abrir (Windows Terminal ou o prompt do Windows).";
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
async function iniciarComClaude(p, f) {
  const numero = numeroDaAcao(p, f), ocupada = agentesDoBlueprint(p.id).some((x) => x.acaoId === f.id && x.estado !== "ocioso");
  const ok = await confirmar({
    titulo: `Iniciar a ação ${numero} com o Claude?`,
    texto: `Abre um terminal novo com o Claude Code e gasta tokens.${ocupada ? " Já há um agente trabalhando nesta ação; o novo vai trabalhar junto com ele." : ""} Iniciar a ação ${numero} «${f.nome || "sem nome"}»?`,
    rotulo: "Iniciar",
  });
  if (!ok) return;
  iniciarAviso("Abrindo o terminal…");
  try {
    const r = await fetch("/api/mcp/iniciar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: workspaceAtual?.id || "sm", blueprintId: p.id, acaoId: f.id }) });
    const j = await r.json().catch(() => ({}));
    iniciarAviso(iniciarMensagem(r.status, j, numero), !r.ok);
  } catch { iniciarAviso("Não consegui falar com o app para abrir o terminal. Ele está aberto?", true); }
}
