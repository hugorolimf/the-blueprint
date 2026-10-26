// ---- links de convite do workspace (só o dono; docs/colaboracao.md, seções 4 e 5) ---------------------------------------
// Criar com papel, escopo e validade; o link (com o segredo) aparece UMA vez, só na resposta da criação ou da troca, para copiar; depois a
// lista mostra só o estado, com Trocar (revoga e cria outro igual) e Revogar. O link público é o convite de visitante: um blueprint, sem nome,
// sem limite de usos, que abre a folha ou a apresentação.
export const PAPEIS_CONVITE = ["colaborador", "supervisor", "dono", "visitante"];
export const VALIDADES = [1, 24, 168, 720]; // horas: 1 hora, 1 dia, 7 dias (padrão), 30 dias

// valida o formulário e monta o corpo de POST /api/convites; devolve { erro } com um código da seção 5 quando não serve
export function corpoDoConvite({ workspaceId, papel, blueprintId = "", abre = "folha", validadeHoras = 168, usos = "1" }) {
  if (!PAPEIS_CONVITE.includes(papel)) return { erro: "papel-invalido" };
  const horas = Number(validadeHoras);
  if (!Number.isInteger(horas) || horas < 1 || horas > 720) return { erro: "validade-invalida" };
  const visitante = papel === "visitante";
  if (visitante && !blueprintId) return { erro: "falta-blueprint" };
  let usosMax = null;
  if (!visitante && usos !== "" && usos != null) {
    usosMax = Number(usos);
    if (!Number.isInteger(usosMax) || usosMax < 1 || usosMax > 1000) return { erro: "usos-invalidos" };
  }
  return { workspaceId, papel, validadeHoras: horas, usosMax, ...(blueprintId ? { blueprintId } : {}), ...(visitante ? { abre: abre === "apresentacao" ? "apresentacao" : "folha" } : {}) };
}
// ativo, revogado, vencido ou esgotado (a lista mostra os inativos apagados, no fim)
export function estadoDoConvite(c, agora = Date.now()) {
  if (c.revogado) return "revogado";
  if (Date.parse(c.expiraEm) <= agora) return "vencido";
  if (c.usosMax != null && c.usos >= c.usosMax) return "esgotado";
  return "ativo";
}
const rotuloEstado = (e) => ({ ativo: tr("ativo"), revogado: tr("revogado"), vencido: tr("vencido"), esgotado: tr("esgotado") })[e];
const rotuloValidade = (h) => ({ 1: tr("1 hora"), 24: tr("1 dia"), 168: tr("7 dias"), 720: tr("30 dias") })[h];
const dataCurta = (iso) => { const d = new Date(iso); return Number.isFinite(d.getTime()) ? d.toLocaleString(I18N.ativo, { dateStyle: "short", timeStyle: "short" }) : ""; };

let raiz = null, ws = "";
export function montarConvites(el, workspaceId) {
  raiz = el; ws = workspaceId;
  const bps = (workspacesDe(arquivo).find((w) => w.id === ws)?.dados.blueprints || []);
  raiz.innerHTML = `<h4>${tr("Links de convite")}</h4>
  <form class="eq-form" id="eq-conv-form">
    <label>${tr("Papel")}<select name="papel">${PAPEIS_CONVITE.map((p) => `<option value="${p}">${esc(p === "visitante" ? tr("Visitante (link público)") : nomeDoPapel(p))}</option>`).join("")}</select></label>
    <label>${tr("Escopo")}<select name="blueprintId"><option value="">${tr("Workspace inteiro")}</option>${bps.map((b) => `<option value="${esc(b.id)}">${esc(b.nome || tr("Sem nome"))}</option>`).join("")}</select></label>
    <label data-so="visitante" hidden>${tr("Abre")}<select name="abre"><option value="folha">${tr("A folha do blueprint")}</option><option value="apresentacao">${tr("A apresentação")}</option></select></label>
    <label>${tr("Validade")}<select name="validadeHoras">${VALIDADES.map((h) => `<option value="${h}"${h === 168 ? " selected" : ""}>${rotuloValidade(h)}</option>`).join("")}</select></label>
    <label data-so="membro">${tr("Quantas pessoas")}<select name="usos"><option value="1">${tr("Uma pessoa")}</option><option value="10">${tr("Até 10 pessoas")}</option><option value="">${tr("Sem limite")}</option></select></label>
    <p class="eq-dica" id="eq-conv-dica"></p>
    <button class="btn cheio" type="submit">${tr("Criar link")}</button>
  </form>
  <div class="eq-link" id="eq-link" hidden></div>
  <p class="eq-status" id="eq-conv-status" role="status"></p>
  <ul class="eq-lista eq-convites" id="eq-conv-lista"></ul>`;
  const f = $("eq-conv-form");
  const mostra = () => {
    const visitante = f.papel.value === "visitante";
    raiz.querySelector('[data-so="visitante"]').hidden = !visitante;
    raiz.querySelector('[data-so="membro"]').hidden = visitante;
    $("eq-conv-dica").textContent = explicaPapel(f.papel.value);
  };
  f.papel.addEventListener("change", mostra); mostra();
  f.addEventListener("submit", async (e) => {
    e.preventDefault();
    const corpo = corpoDoConvite({ workspaceId: ws, papel: f.papel.value, blueprintId: f.blueprintId.value, abre: f.abre.value, validadeHoras: f.validadeHoras.value, usos: f.usos.value });
    if (corpo.erro) { $("eq-conv-status").textContent = corpo.erro === "falta-blueprint" ? tr("O link público mostra um blueprint: escolha qual no Escopo.") : textoDoErro(corpo.erro); return; }
    await criar("/api/convites", corpo, tr("Link criado."));
  });
  listarConvites();
}
async function criar(rota, corpo, ok) {
  try { const j = await equipeApi(rota, corpo); mostrarLink(j.link); $("eq-conv-status").textContent = ok; await listarConvites(); }
  catch (e) { $("eq-conv-status").textContent = e.message; }
}
// o segredo só existe nesta resposta: mostrado uma vez, com Copiar; fechar o painel o apaga da tela
function mostrarLink(link) {
  const caixa = $("eq-link");
  caixa.hidden = false;
  caixa.innerHTML = `<code id="eq-link-texto">${esc(link)}</code><button class="btn mini" type="button" id="eq-link-copiar">${tr("Copiar")}</button><small>${tr("Este link só aparece agora. Copie e mande para a pessoa; depois dá só para trocar ou revogar.")}</small>`;
  $("eq-link-copiar").onclick = async () => {
    try { await navigator.clipboard.writeText(link); $("eq-link-copiar").textContent = tr("Copiado"); }
    catch { getSelection().selectAllChildren($("eq-link-texto")); $("eq-conv-status").textContent = tr("Selecionei o link: copie com Ctrl+C."); }
  };
  $("equipe").addEventListener("close", () => { caixa.hidden = true; caixa.textContent = ""; }, { once: true });
}
async function listarConvites() {
  let convites;
  try { ({ convites } = await equipeApi(`/api/convites?workspace=${encodeURIComponent(ws)}`)); }
  catch (e) { $("eq-conv-status").textContent = e.message; return; }
  const bps = new Map((workspacesDe(arquivo).find((w) => w.id === ws)?.dados.blueprints || []).map((b) => [b.id, b.nome]));
  const ordenados = convites.map((c) => ({ ...c, estado: estadoDoConvite(c) })).sort((a, b) => (a.estado === "ativo" ? 0 : 1) - (b.estado === "ativo" ? 0 : 1) || String(b.criadoEm).localeCompare(String(a.criadoEm)));
  $("eq-conv-lista").innerHTML = ordenados.length ? ordenados.map((c) => `<li data-convite="${esc(c.id)}" class="eq-${c.estado}">
    <span class="eq-nome"><strong>${esc(c.papel === "visitante" ? tr("Link público") : nomeDoPapel(c.papel))}</strong>
    <small>${esc(c.blueprintId ? bps.get(c.blueprintId) || c.blueprintId : tr("Workspace inteiro"))}${c.papel === "visitante" ? " · " + esc(c.abre === "apresentacao" ? tr("abre a apresentação") : tr("abre a folha")) : ""}</small>
    <small>${esc(rotuloEstado(c.estado))} · ${esc(c.usosMax == null ? trn(c.usos, ["{n} uso", "{n} usos"]) : tr("{usos} de {max} usos", { usos: c.usos, max: c.usosMax }))} · ${esc(tr("vence {data}", { data: dataCurta(c.expiraEm) }))}${c.criadoPor ? " · " + esc(tr("por {nome}", { nome: c.criadoPor.nome })) : ""}</small></span>
    ${c.estado === "ativo" ? `<button class="btn mini" type="button" data-trocar>${tr("Trocar")}</button><button class="btn mini perigo" type="button" data-revogar>${tr("Revogar")}</button>` : ""}
  </li>`).join("") : `<li class="eq-vazio">${tr("Nenhum link ainda.")}</li>`;
  $("eq-conv-lista").querySelectorAll("[data-trocar]").forEach((b) => b.addEventListener("click", () => criar("/api/convites/trocar", { id: b.closest("li").dataset.convite }, tr("Link trocado: o anterior deixou de valer."))));
  $("eq-conv-lista").querySelectorAll("[data-revogar]").forEach((b) => b.addEventListener("click", () => revogar(b.closest("li").dataset.convite)));
}
async function revogar(id) {
  if (!(await confirmar({ titulo: tr("Revogar este link?"), texto: tr("O link para de funcionar na hora. Quem já entrou continua, menos visitantes, que saem junto."), rotulo: tr("Revogar") }))) return;
  try { await equipeApi("/api/convites/revogar", { id }); $("eq-conv-status").textContent = tr("Link revogado."); await listarConvites(); }
  catch (e) { $("eq-conv-status").textContent = e.message; }
}
