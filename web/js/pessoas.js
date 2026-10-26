// ---- painel Pessoas do workspace (modo hospedado; docs/colaboracao.md, seções 3 e 5) ----------------------------------
// Módulo ES carregado por import() só quando o painel abre (sessao.js). Lista quem tem acesso, com papel e última atividade; o dono troca o
// papel e remove (o servidor recusa tirar o último dono: 409 ultimo-dono); o supervisor só lista. Os links de convite ficam em pessoas-convites.js.
import { montarConvites } from "./pessoas-convites.js";

export const PAPEIS_MEMBRO = ["dono", "colaborador", "supervisor"];

// "há 5 min", "há 3 h", "há 2 dias"; sem data: "nunca"
export function haQuanto(iso, agora = Date.now()) {
  const t = Date.parse(iso || "");
  if (!Number.isFinite(t)) return tr("nunca");
  const min = Math.max(0, Math.floor((agora - t) / 60000));
  if (min < 2) return tr("agora há pouco");
  if (min < 60) return tr("há {n} min", { n: min });
  const h = Math.floor(min / 60);
  if (h < 24) return tr("há {n} h", { n: h });
  return trn(Math.floor(h / 24), ["há {n} dia", "há {n} dias"]);
}
// as linhas da lista: quem é você vem primeiro; o dono pode mudar e remover os outros (a própria linha não, para não se trancar fora)
export function linhasPessoas(membros, souDono, agora = Date.now()) {
  return [...(membros || [])]
    .sort((a, b) => (b.voce ? 1 : 0) - (a.voce ? 1 : 0) || (PAPEIS_MEMBRO.indexOf(a.papel) - PAPEIS_MEMBRO.indexOf(b.papel)) || String(a.nome).localeCompare(String(b.nome)))
    .map((m) => ({
      id: m.id, nome: m.nome, papel: m.papel, voce: !!m.voce,
      escopo: m.blueprintId ? tr("só um blueprint") : tr("workspace inteiro"),
      quando: haQuanto(m.ultimaAtividade, agora),
      podeMudar: !!souDono && !m.voce,
    }));
}

let workspaceId = "";
const corpo = () => $("equipe-corpo");
export async function abrirPessoas() {
  const opcoes = workspacesDaEquipe(["dono", "supervisor"]);
  if (!opcoes.length) return;
  if (!opcoes.some((w) => w.id === workspaceId)) workspaceId = (opcoes.find((w) => w.id === workspaceAtual?.id) || opcoes[0]).id;
  $("equipe-titulo").textContent = tr("Pessoas");
  const escolha = opcoes.length > 1
    ? `<label class="eq-ws">${tr("Workspace")}<select id="eq-ws">${opcoes.map((w) => `<option value="${esc(w.id)}"${w.id === workspaceId ? " selected" : ""}>${esc(w.nome)}</option>`).join("")}</select></label>` : "";
  corpo().innerHTML = `${escolha}<section class="eq-secao" aria-labelledby="eq-pessoas-tit"><h4 id="eq-pessoas-tit">${tr("Quem tem acesso")}</h4><p class="eq-status" id="eq-pessoas-status" role="status"></p><div id="eq-pessoas"></div></section><section class="eq-secao" id="eq-convites" hidden></section>`;
  $("eq-ws")?.addEventListener("change", (e) => { workspaceId = e.target.value; abrirPessoas(); });
  if (!$("equipe").open) $("equipe").showModal();
  await listar();
  const souDono = papelNoEscopo(sessao.acessos, workspaceId) === "dono";
  $("eq-convites").hidden = !souDono;
  if (souDono) montarConvites($("eq-convites"), workspaceId);
}
async function listar() {
  const status = $("eq-pessoas-status"), souDono = papelNoEscopo(sessao.acessos, workspaceId) === "dono";
  status.textContent = tr("carregando…");
  let membros;
  try { ({ membros } = await equipeApi(`/api/pessoas?workspace=${encodeURIComponent(workspaceId)}`)); }
  catch (e) { status.textContent = e.message; return; }
  status.textContent = trn(membros.length, ["{n} pessoa", "{n} pessoas"]);
  const opcao = (m, p) => `<option value="${p}"${m.papel === p ? " selected" : ""}>${esc(nomeDoPapel(p))}</option>`;
  $("eq-pessoas").innerHTML = `<ul class="eq-lista">${linhasPessoas(membros, souDono).map((m) => `<li data-membro="${esc(m.id)}">
    <span class="eq-avatar" aria-hidden="true">${esc(String(m.nome).slice(0, 1).toUpperCase())}</span>
    <span class="eq-nome"><strong>${esc(m.nome)}</strong>${m.voce ? ` <small>${tr("(você)")}</small>` : ""}<small>${esc(m.escopo)} · ${esc(m.quando)}</small></span>
    ${m.podeMudar ? `<select data-papel aria-label="${esc(tr("Papel de {nome}", { nome: m.nome }))}">${PAPEIS_MEMBRO.map((p) => opcao(m, p)).join("")}</select><button class="btn mini perigo" type="button" data-remover>${tr("Remover")}</button>` : `<span class="eq-papel">${esc(nomeDoPapel(m.papel))}</span>`}
  </li>`).join("")}</ul>`;
  $("eq-pessoas").querySelectorAll("[data-papel]").forEach((s) => s.addEventListener("change", () => mudarPapel(s.closest("li").dataset.membro, s.value)));
  $("eq-pessoas").querySelectorAll("[data-remover]").forEach((b) => b.addEventListener("click", () => remover(b.closest("li"))));
}
async function mudarPapel(membroId, papel) {
  try { await equipeApi("/api/pessoas/papel", { workspaceId, membroId, papel }); await listar(); $("eq-pessoas-status").textContent = tr("Papel atualizado."); }
  catch (e) { await listar(); $("eq-pessoas-status").textContent = e.message; }
}
async function remover(li) {
  const nome = li.querySelector(".eq-nome strong").textContent;
  if (!(await confirmar({ titulo: tr("Remover {nome}?", { nome }), texto: tr("{nome} perde o acesso a este workspace na hora, e a chave de MCP dessa pessoa deixa de valer.", { nome }), rotulo: tr("Remover") }))) return;
  try { await equipeApi("/api/pessoas/remover", { workspaceId, membroId: li.dataset.membro }); await listar(); $("eq-pessoas-status").textContent = tr("{nome} foi removido.", { nome }); }
  catch (e) { $("eq-pessoas-status").textContent = e.message; }
}
