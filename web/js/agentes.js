// ---- agentes ativos: a busca que alimenta as figuras do Claude na folha (claude-camada.js) e o filtro "Em andamento" ----
// Só vem do que os agentes do usuário reportam pelo MCP (o app não abre, não conversa e não executa agentes: sem botão de iniciar nem de conversar).
// Zero trabalho em repouso: só há um relógio até o próximo agente expirar; a folha é redesenhada quando quem trabalha, onde ou em que estado muda.
// Quem concluiu fica no plano, com a bandeira, ATÉ o usuário clicar nele e FECHAR a mensagem de conclusão (aí foi visto: ele sai da folha e o Claude de espera aparece na
// próxima ação). O que já foi visto fica só neste navegador.
const CHAVE_VISTOS = "blueprint.concluidosVistos";
const agentesVistos = (() => {
  try { const v = JSON.parse(localStorage.getItem(CHAVE_VISTOS) || "[]"); return new Set(Array.isArray(v) ? v.filter((x) => typeof x === "string").slice(-200) : []); } catch { return new Set(); }
})();
const agenteChaveVisto = (a) => `${a.workspaceId}|${a.agente}|${a.em}`; // o aviso novo do mesmo agente (outro `em`) volta a precisar de clique
const agenteVisto = (a) => agentesVistos.has(agenteChaveVisto(a));
function agenteMarcaVisto(a) {
  agentesVistos.add(agenteChaveVisto(a));
  try { localStorage.setItem(CHAVE_VISTOS, JSON.stringify([...agentesVistos].slice(-200))); } catch {}
  agentesAtualiza();
  render();
}
let agentesLista = [], agentesVida = 10 * 60 * 1000, agentesExpira = 0; // agentesVida: sem aviso por esse tempo, o agente vai para o repouso
const ESTADO_AG = { lendo: tr("Lendo"), escrevendo: tr("Escrevendo"), testando: tr("Testando"), aguardando: tr("Aguardando"), bloqueado: tr("Bloqueado"), concluido: tr("Concluído"), ocioso: tr("Parado") };
const PAPEL_AG = { explorador: "Explorador", construtor: "Construtor", revisor: "Revisor" };

const haQuanto = (iso) => {
  const s = Math.max(0, Math.round((Date.now() - Date.parse(iso)) / 1000));
  return s < 10 ? tr("agora") : s < 60 ? tr("há {n} s", { n: s }) : s < 3600 ? tr("há {n} min", { n: Math.round(s / 60) }) : tr("há {n} h", { n: Math.round(s / 3600) });
};

async function agentesBusca() {
  let mudou = false;
  try {
    const r = await fetch("/api/mcp/agentes", { cache: "no-store" });
    if (!r.ok) return; // servidor sem a lista (o Node antigo): sem figuras
    const j = await r.json();
    agentesVida = j.vidaMs || agentesVida;
    const assina = (l) => l.map((a) => `${a.agente}|${a.papel}|${a.blueprintId}|${a.acaoId}|${a.planoId}|${a.estado}`).join(";"), antes = assina(agentesLista);
    agentesLista = Array.isArray(j.agentes) ? j.agentes : [];
    mudou = assina(agentesLista) !== antes;
  } catch { return; }
  agentesAtualiza();
  if (mudou) render(); // as figuras da folha seguem a lista: só redesenha quando quem trabalha, onde e em que estado mudou
}
// Quem ficou agentesVida sem avisar NÃO some: repousa, parado, sem plano, no cabeçalho do blueprint. Concluído e ainda não visto não tem prazo: fica no plano até o usuário
// ler e clicar em "Concluído"; concluído e visto sai da folha (agentesAtivos não o devolve). O mesmo botão dispensa o agente parado (aguardando ou em repouso) até ele avisar de novo.
const agenteVenceEm = (a) => (a.estado === "concluido" ? Infinity : Date.parse(a.em) + agentesVida); // quando ele vai para o repouso por falta de aviso
const agenteRepousa = (a) => a.estado === "ocioso" || (a.estado !== "concluido" && Date.now() >= agenteVenceEm(a));
// os agentes do workspace aberto (os que repousam vêm como "ocioso", sem ação nem plano: a folha os põe parados no cabeçalho)
const agentesDoWorkspace = () => agentesLista.filter((a) => a.workspaceId === workspaceAtual?.id);
const agenteEhSub = (a) => claudeEhSub(a, agentesDoWorkspace()); // terminar a tarefa dele não é terminar o trabalho: sem mensagem de conclusão, e some sozinho (claudeSubSumiu)
const agentesAtivos = () => agentesDoWorkspace().filter((a) => !claudeSubSumiu(a, agentesDoWorkspace()) && !(agenteVisto(a) && (["concluido", "aguardando", "ocioso"].includes(a.estado) || agenteRepousa(a)))).map((a) => (agenteRepousa(a) ? { ...a, estado: "ocioso", acaoId: "", planoId: "", repouso: true } : a));

function agentesAtualiza() {
  const trabalhando = agentesLista.filter((a) => a.workspaceId === workspaceAtual?.id && !agenteRepousa(a) && a.estado !== "concluido");
  clearTimeout(agentesExpira); // um relógio só, até o próximo agente ir para o repouso
  if (typeof terminalAtualizaAbas === "function") terminalAtualizaAbas(); // o estado das abas do terminal do app vem destes avisos
  if (typeof focoDepoisDoRender === "function") focoDepoisDoRender(); // o modo foco e a faixa do plano acompanhado seguem a presença dos agentes (foco.js)
  const lista = agentesDoWorkspace(), prazos = [...trabalhando.map(agenteVenceEm), ...lista.filter((a) => a.estado === "concluido" && claudeEhSub(a, lista) && !claudeSubSumiu(a, lista)).map(claudeSubSomeEm)]; // e o subagente que concluiu, até sumir
  if (prazos.length) agentesExpira = setTimeout(() => { agentesAtualiza(); render(); }, Math.max(1000, Math.min(...prazos) - Date.now() + 500));
}
