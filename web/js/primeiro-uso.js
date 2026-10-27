// ---- primeiro uso: a entrada sem nenhum blueprint mostra 3 caminhos (abrir o exemplo, conectar o agente, criar um blueprint) ----
// Some sozinha assim que existe um blueprint. O exemplo entra como um workspace NOVO (todo id trocado, datas trazidas para perto de hoje),
// gravado por /api/mudancas como qualquer workspace criado à mão (rede.js): nunca toca nos dados que já existem, e sai com
// "Remover exemplo" (o mesmo apagar do workspace, com confirmação). O modelo vem de GET /api/exemplo, no idioma da página.
const EXEMPLO_HOJE = "2026-10-20"; // o "hoje" em que o data.exemplo.json foi escrito: as datas andam o mesmo tanto que o dia de hoje

/** O data.exemplo.json como um workspace novo: os blueprints da raiz, ids novos em tudo o que é item, datas deslocadas para `hojeIso`. */
function exemploComoWorkspace(modelo, hojeIso, gerarId = novoId) {
  const desloca = Math.round(dia(hojeIso) - dia(EXEMPLO_HOJE)) || 0;
  const data = (iso) => (ISO.test(iso || "") ? deDia(dia(iso) + desloca) : iso);
  const novo = (o) => ({ ...o, id: gerarId() });
  const blueprints = (modelo.blueprints || []).map((p) => ({
    ...novo(p),
    acoes: (p.acoes || []).map((a) => ({ ...novo(a), inicio: data(a.inicio), fim: data(a.fim), planos: (a.planos || []).map(novo), checkpoints: (a.checkpoints || []).map((m) => ({ ...novo(m), data: data(m.data) })), prototipos: [] })),
  }));
  const nome = String(modelo.empresa || tr("Exemplo"));
  return { id: gerarId(), nome, empresa: nome, descricao: tr("Dados fictícios para conhecer o app. Remova quando quiser."), icone: "blueprint", exemplo: true, blueprints };
}
const temBlueprint = (a) => workspacesDe(a).some((c) => c.dados.blueprints.length);
const workspaceExemplo = (a) => (a.workspaces || []).find((c) => c.exemplo === true);

function primeiroUso() {
  const el = $("primeiro-uso");
  const local = !(typeof hospedado === "function" && hospedado()); // no modo de equipe quem cria workspace é o dono: a tela de convidado não oferece
  const vazio = carregado && local && !temBlueprint(arquivo), exemplo = carregado && local && workspaceExemplo(arquivo);
  el.hidden = !vazio && !exemplo;
  if (vazio) {
    const cartao = (acao, n, titulo, texto) => `<button class="pu-cartao" type="button" data-pu="${acao}"><span class="pu-n">${n}</span><strong>${titulo}</strong><span>${texto}</span><span class="pu-ir" aria-hidden="true">→</span></button>`;
    el.innerHTML = `<p class="pu-rot">${tr("Primeiros passos")}</p><h2 id="pu-titulo">${tr("Nenhum blueprint ainda. Por onde começar?")}</h2><div class="pu-cartoes" role="group" aria-labelledby="pu-titulo">`
      + cartao("exemplo", "01", tr("Abrir o exemplo"), tr("Um workspace com dois blueprints fictícios, para ver o app cheio. Os seus dados não mudam e ele sai com um clique."))
      + cartao("agente", "02", tr("Conectar seu agente"), tr("Registre o MCP no Claude Code ou no Codex e peça um plano: a proposta aparece aqui para você aplicar."))
      + cartao("criar", "03", tr("Criar um blueprint"), tr("Escreva o primeiro blueprint à mão, com ações, planos e checkpoints."))
      + `</div><p class="pu-estado" id="pu-estado" role="status"></p>`;
  } else if (exemplo) {
    el.innerHTML = `<p class="pu-linha">${tr("O workspace «{nome}» é o exemplo: dados fictícios, separados dos seus.", { nome: esc(exemplo.nome) })} <button class="btn mini" type="button" data-pu="remover">${tr("Remover exemplo")}</button></p>`;
  } else el.innerHTML = "";
}
async function abrirExemplo(botao) {
  if (salvandoWorkspace) return;
  salvandoWorkspace = true; // carregar() espera: o arquivo não é trocado no meio da gravação
  botao.disabled = true;
  $("pu-estado").textContent = tr("Abrindo o exemplo…");
  let ws = null;
  try {
    const r = await fetch("/api/exemplo", { cache: "no-store" });
    if (!r.ok) throw new Error(r.status);
    ws = limpa(exemploComoWorkspace(await r.json(), hoje));
    arquivo.workspaces.push(ws);
    await gravarDados();
    location.hash = "/c/" + encodeURIComponent(ws.id);
  } catch {
    if (ws) arquivo.workspaces = arquivo.workspaces.filter((c) => c !== ws);
    $("pu-estado").textContent = tr("Não foi possível abrir o exemplo. Tente novamente.");
  } finally { salvandoWorkspace = false; botao.disabled = false; }
}
// Conectar e Criar abrem no workspace raiz: a navegação fecha as telas abertas, então a tela pedida abre depois dela
function naRaiz(depois) {
  const alvo = "#/c/sm";
  if (location.hash === alvo) return depois();
  addEventListener("hashchange", () => depois(), { once: true });
  location.hash = alvo;
}
$("primeiro-uso").addEventListener("click", (e) => {
  const b = e.target.closest("[data-pu]");
  if (!b) return;
  const acao = b.dataset.pu;
  if (acao === "exemplo") abrirExemplo(b);
  else if (acao === "agente") naRaiz(() => { agenteTela(true, { novo: true }); $("cu-abrir").click(); });
  else if (acao === "criar") naRaiz(() => editar(true, { novo: true }));
  else if (acao === "remover") { const w = workspaceExemplo(arquivo); if (w) excluirWorkspace(w.id); }
});
