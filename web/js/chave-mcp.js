// ---- Minha chave de MCP (modo hospedado; docs/colaboracao.md, seções 4 e 5) --------------------------------------------
// Cada membro gera a própria chave e conecta o próprio agente (Claude Code, Codex) ao servidor da equipe, com o papel dele. A chave aparece
// UMA vez, junto com os comandos prontos; depois a tela mostra só o prefixo (bpk_xxxx) e as datas. Gerar outra troca a anterior.
// Módulo ES carregado por import() só quando o painel abre (sessao.js).

// os comandos do contrato (seção 4); o servidor manda os dele em "comandos" e eles têm preferência
export function comandosDaChave(url, chave) {
  const env = `--env BLUEPRINT_URL=${url} --env BLUEPRINT_CHAVE=${chave}`;
  return { claude: `claude mcp add blueprint-equipe ${env} -- blueprint --mcp`, codex: `codex mcp add blueprint-equipe ${env} -- blueprint --mcp` };
}
const dataCurta = (iso) => { const d = new Date(iso || ""); return Number.isFinite(d.getTime()) ? d.toLocaleString(I18N.ativo, { dateStyle: "short", timeStyle: "short" }) : tr("nunca"); };

export async function abrirChave() {
  $("equipe-titulo").textContent = tr("Minha chave de MCP");
  $("equipe-corpo").innerHTML = `<section class="eq-secao"><p>${tr("Com a chave, o seu agente lê e atualiza os blueprints deste servidor com o seu papel. Ela é pessoal: não a compartilhe.")}</p>
    <p class="eq-status" id="eq-chave-status" role="status">${tr("carregando…")}</p>
    <div class="eq-acoes"><button class="btn cheio" type="button" id="eq-chave-gerar">${tr("Gerar chave")}</button><button class="btn perigo" type="button" id="eq-chave-revogar" hidden>${tr("Revogar")}</button></div>
    <div class="eq-link" id="eq-chave-nova" hidden></div></section>`;
  $("eq-chave-gerar").onclick = gerar;
  $("eq-chave-revogar").onclick = revogar;
  if (!$("equipe").open) $("equipe").showModal();
  $("equipe").addEventListener("close", () => { const n = $("eq-chave-nova"); if (n) { n.hidden = true; n.textContent = ""; } }, { once: true }); // a chave some da tela ao fechar
  await estado();
}
async function estado() {
  try {
    const { chave } = await equipeApi("/api/chave");
    $("eq-chave-status").textContent = chave
      ? tr("Chave {prefixo}… criada em {criada}; último uso: {usada}.", { prefixo: chave.prefixo, criada: dataCurta(chave.criadaEm), usada: dataCurta(chave.usadaEm) })
      : tr("Você ainda não tem chave.");
    $("eq-chave-gerar").textContent = chave ? tr("Trocar chave") : tr("Gerar chave");
    $("eq-chave-revogar").hidden = !chave;
  } catch (e) { $("eq-chave-status").textContent = e.message; }
}
async function gerar() {
  if (!$("eq-chave-revogar").hidden && !(await confirmar({ titulo: tr("Trocar a chave?"), texto: tr("A chave atual deixa de valer na hora; o agente que a usa precisa da nova."), rotulo: tr("Trocar chave") }))) return;
  try {
    const j = await equipeApi("/api/chave", {});
    const c = j.comandos?.claude && j.comandos?.codex ? j.comandos : comandosDaChave(location.origin, j.chave);
    const n = $("eq-chave-nova");
    n.hidden = false;
    n.innerHTML = `<small>${tr("A chave só aparece agora. Copie o comando do seu agente e rode no terminal.")}</small>
      <b>${tr("Chave")}</b><code>${esc(j.chave)}</code><button class="btn mini" type="button" data-copiar="${esc(j.chave)}">${tr("Copiar")}</button>
      <b>Claude Code</b><code>${esc(c.claude)}</code><button class="btn mini" type="button" data-copiar="${esc(c.claude)}">${tr("Copiar")}</button>
      <b>Codex</b><code>${esc(c.codex)}</code><button class="btn mini" type="button" data-copiar="${esc(c.codex)}">${tr("Copiar")}</button>`;
    n.querySelectorAll("[data-copiar]").forEach((b) => b.addEventListener("click", async () => {
      try { await navigator.clipboard.writeText(b.dataset.copiar); b.textContent = tr("Copiado"); }
      catch { getSelection().selectAllChildren(b.previousElementSibling); $("eq-chave-status").textContent = tr("Selecionei o texto: copie com Ctrl+C."); }
    }));
    await estado();
  } catch (e) { $("eq-chave-status").textContent = e.message; }
}
async function revogar() {
  if (!(await confirmar({ titulo: tr("Revogar a chave?"), texto: tr("O agente que usa esta chave para de acessar o servidor na hora."), rotulo: tr("Revogar") }))) return;
  try { await equipeApi("/api/chave/revogar", {}); const n = $("eq-chave-nova"); n.hidden = true; n.textContent = ""; await estado(); }
  catch (e) { $("eq-chave-status").textContent = e.message; }
}
