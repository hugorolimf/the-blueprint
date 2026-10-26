// ---- as equipes a que este app já entrou, na tela de workspaces (só no app local; src/equipes.rs, src/http/equipes.rs) ----------------------
// A sessão da equipe é o cookie da janela dela (30 dias, perfil persistente); aqui o app só lembra o endereço e os nomes, para a pessoa reabrir o
// espaço de trabalho colaborativo com um clique, sem o link de convite. A lista é lida ao abrir e quando a janela volta ao foco (sem relógio).
let equipesLocais = [];
let equipesTexto = "[]";
async function equipesBusca() {
  if (hospedado()) return; // dentro da janela da equipe não há "outras equipes"
  try {
    const r = await fetch("/api/equipes", { cache: "no-store" });
    if (!r.ok) return;
    const j = await r.json(), texto = JSON.stringify(j.equipes || []);
    if (texto === equipesTexto) return;
    equipesTexto = texto; equipesLocais = j.equipes || [];
    if (carregado && !workspaceAtual) render(); // só a tela de workspaces mostra as equipes
  } catch {} // app sem a rota (binário antigo) ou sem resposta: a lista só não aparece
}
const hostDaEquipe = (origem) => String(origem).replace(/^https:\/\//, "");
function equipesCartoes() {
  return equipesLocais.map((e) => {
    const nomes = (e.workspaces || []).map((w) => w.nome).filter(Boolean), host = hostDaEquipe(e.origem);
    const titulo = nomes.join(", ") || host, quem = e.membro ? tr("Você entra como {nome}", { nome: e.membro }) : host;
    return `<div class="blueprint-cartao" role="link" tabindex="0" data-equipe-abrir="${esc(e.origem)}" title="${esc(host)}"><span class="cartao-tipo">${tr("Equipe")}</span><div class="cartao-arte" aria-hidden="true">${typeof EMB !== "undefined" && EMB ? previaMarca(EMB.BLUEPRINT, 280, 120) : ""}</div><h2 title="${esc(titulo)}">${esc(titulo)}</h2><p title="${esc(quem)}">${esc(quem)} · ${esc(host)}</p><footer><button class="btn" type="button" data-equipe-esquecer="${esc(e.origem)}">${tr("Tirar da lista")}</button><span>${tr("Abrir espaço →")}</span></footer></div>`;
  }).join("");
}
async function equipePedido(rota, origem) {
  const r = await fetch(rota, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ origem }) });
  const j = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(j.erro || tr("Não foi possível concluir. Tente novamente."));
  return j;
}
async function equipeAbrir(origem) {
  try { await equipePedido("/api/equipes/abrir", origem); $("entrada-status").textContent = tr("Abrindo a equipe na janela do app…"); }
  catch (erro) { $("entrada-status").textContent = erro.message; }
}
async function equipeEsquecer(origem) {
  try { const j = await equipePedido("/api/equipes/esquecer", origem); equipesLocais = j.equipes || []; equipesTexto = JSON.stringify(equipesLocais); render(); }
  catch (erro) { $("entrada-status").textContent = erro.message; }
}
$("entrada-blueprints").addEventListener("click", (e) => {
  const esquecer = e.target.closest("[data-equipe-esquecer]"), abrir = e.target.closest("[data-equipe-abrir]");
  if (esquecer) equipeEsquecer(esquecer.dataset.equipeEsquecer);
  else if (abrir) equipeAbrir(abrir.dataset.equipeAbrir);
});
$("entrada-blueprints").addEventListener("keydown", (e) => {
  const abrir = e.target.matches?.("[data-equipe-abrir]") && e.target;
  if (abrir && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); equipeAbrir(abrir.dataset.equipeAbrir); }
});
addEventListener("focus", equipesBusca);
