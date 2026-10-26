// ---- Responsável por plano (modo hospedado; docs/colaboracao.md): `plano.responsavel = { id, agente? }`, só rótulo, não permissão ----
// `id` é o de uma pessoa do workspace; `agente: true` é o Claude dessa pessoa. Escolhido em Editar e no menu do botão direito; o filtro "Meus planos" mostra só os da pessoa
// da sessão; o agente dela, pelo MCP, vê o plano como seu (`ler_blueprint` traz `voce` e `responsavel`). No modo local há uma pessoa só: o campo fica escondido.
let responsavelMembros = []; // [{ id, nome, papel, voce }] do workspace aberto
let responsavelWs = "";

const responsavelAtivo = () => hospedado() && !!sessao.membro && !sessao.visitante && !poderesDaSessao(sessao).soLeitura;

// o valor do <select>: "" (ninguém), "m1" (a pessoa) ou "m1:agente" (o Claude dela)
const responsavelValor = (r) => (r && typeof r.id === "string" && r.id ? r.id + (r.agente ? ":agente" : "") : "");
function responsavelDeValor(v) {
  const [id, marca] = String(v || "").split(":");
  return id ? (marca === "agente" ? { id, agente: true } : { id }) : null;
}
const responsavelEhMeu = (t) => !!(sessao.membro && t?.responsavel?.id === sessao.membro.id);

// as pessoas que podem receber plano: quem grava (dono e colaborador). Quem já é responsável e saiu do workspace continua listado, para a escolha não sumir
function responsavelOpcoes(atual) {
  const opcoes = [{ valor: "", rotulo: tr("Sem responsável") }];
  for (const m of responsavelMembros.filter((x) => x.papel === "dono" || x.papel === "colaborador")) {
    opcoes.push({ valor: m.id, rotulo: m.voce ? tr("{nome} (você)", { nome: m.nome }) : m.nome }, { valor: m.id + ":agente", rotulo: tr("Claude de {nome}", { nome: m.nome }) });
  }
  if (atual && !opcoes.some((o) => o.valor === atual)) opcoes.push({ valor: atual, rotulo: tr("(pessoa que saiu)") });
  return opcoes;
}
function responsavelSeletor(t) {
  if (!responsavelAtivo()) return "";
  const atual = responsavelValor(t.responsavel);
  return `<select data-c="responsavel" aria-label="${tr("Responsável pelo plano")}" title="${tr("Quem faz este plano: uma pessoa ou o Claude dela")}">${responsavelOpcoes(atual).map((o) => `<option value="${esc(o.valor)}"${o.valor === atual ? " selected" : ""}>${esc(o.rotulo)}</option>`).join("")}</select>`;
}
function responsavelAtribui(plano, valor) {
  const r = responsavelDeValor(valor);
  if (r) plano.responsavel = r; else delete plano.responsavel;
}

// o item "Responsável" do menu do botão direito num plano: atribuir a mim, ao meu Claude, ou tirar
function itensDeResponsavel(pid, fid, tid) {
  if (!responsavelAtivo()) return [];
  const plano = D.blueprints.find((p) => p.id === pid)?.acoes.find((f) => f.id === fid)?.planos.find((t) => t.id === tid);
  if (!plano) return [];
  const eu = sessao.membro.id, atribui = (r) => () => { if (r) plano.responsavel = r; else delete plano.responsavel; mudou(); };
  const itens = [];
  if (!(plano.responsavel?.id === eu && !plano.responsavel.agente)) itens.push({ icone: "alvo", rotulo: tr("Atribuir a mim"), fn: atribui({ id: eu }) });
  if (!(plano.responsavel?.id === eu && plano.responsavel.agente)) itens.push({ icone: "play", rotulo: tr("Atribuir ao meu Claude"), fn: atribui({ id: eu, agente: true }) });
  if (plano.responsavel) itens.push({ icone: "circulo", rotulo: tr("Tirar o responsável"), fn: atribui(null) });
  return itens;
}

// "Meus planos": por blueprint, como "Pendentes" e "Em andamento"
const soMeus = {};
const planosFiltrados = (f) => (f.planos || []).filter(responsavelEhMeu);

// as pessoas do workspace aberto, uma vez por workspace (chamado por render())
function responsavelCarrega() {
  if (!responsavelAtivo()) { responsavelMembros = []; responsavelWs = ""; return; }
  const ws = workspaceAtual?.id || "";
  if (!ws || ws === responsavelWs) return;
  responsavelWs = ws;
  equipeApi(`/api/pessoas/para-atribuir?workspace=${encodeURIComponent(ws)}`)
    .then((r) => { responsavelMembros = Array.isArray(r?.membros) ? r.membros : []; if (typeof ed !== "undefined" && !ed.hidden) montaManual(); })
    .catch(() => { responsavelWs = ""; });
}
