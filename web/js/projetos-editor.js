// ---- projetos no Editar (criar, renomear, remover, escolher o da ação) e no Planejar (o projeto no cartão da proposta) ----
// O dado é `blueprint.projetos = [{ id, nome, descricao? }]` e `acao.projetoId` (ausente = Geral). Remover um projeto devolve as ações dele ao Geral.
const nomeDoProjeto = (p, id) => (p.projetos || []).find((x) => x.id === id)?.nome || "";
const contaAcoesDoProjeto = (p, id) => p.acoes.filter((f) => f.projetoId === id).length;

function htmlProjetosEd(p) {
  const lista = p.projetos || [];
  const linha = (x) => { const n = contaAcoesDoProjeto(p, x.id); return `<div class="projeto-ed" data-pj="${esc(x.id)}"><input data-c="nome" placeholder="${tr("Nome do projeto")}" aria-label="${tr("Nome do projeto")}" value="${esc(x.nome)}"><input data-c="descricao" placeholder="${tr("Descrição (uma linha, aparece sob o nome na folha)")}" aria-label="${tr("Descrição do projeto")}" value="${esc(x.descricao || "")}"><small>${trn(n, ["{n} ação", "{n} ações"])}</small><button class="x" data-act="del-projeto" type="button" aria-label="${tr("Remover projeto")}">×</button></div>`; };
  return `<div class="card"><h3>${tr("Projetos")}<small>${lista.length || ""}</small></h3>${lista.length ? lista.map(linha).join("") : `<p class="vazio">${tr("Sem projetos: todas as ações ficam juntas. Crie projetos para agrupar as ações do blueprint na folha.")}</p>`}<div class="acoes"><button class="btn mini" data-act="add-projeto" type="button">${tr("+ Projeto")}</button></div><p class="dica">${tr("Na folha cada projeto é uma seção numerada (01, 02…) com o cabeçalho de um blueprint e as ações dele; a ordem segue o início da primeira ação. A ação sem projeto fica em “Geral”; escolha o projeto dela em Período e situação.")}</p></div>`;
}
// o campo "Projeto" da ação (só aparece quando o blueprint tem projetos)
function htmlProjetoDaAcao(p, f) {
  if (!p.projetos?.length) return "";
  const atual = (p.projetos.some((x) => x.id === f.projetoId) ? f.projetoId : "");
  return `<label class="f">${tr("Projeto")}<select data-c="projetoId"><option value="">${tr("Geral (sem projeto)")}</option>${p.projetos.map((x) => `<option value="${esc(x.id)}"${x.id === atual ? " selected" : ""}>${esc(x.nome || tr("Sem nome"))}</option>`).join("")}</select></label>`;
}
function novoProjeto(p) {
  const x = { id: novoId(), nome: "" };
  (p.projetos ||= []).push(x);
  return x;
}
function renomearProjeto(p, id, valor, campo = "nome") { // campo: "nome" ou "descricao"
  const x = (p.projetos || []).find((y) => y.id === id);
  if (x) x[campo] = campo === "descricao" ? valor.slice(0, 200) : valor;
}
function removerProjeto(p, id) {
  p.acoes.forEach((f) => { if (f.projetoId === id) delete f.projetoId; });
  p.projetos = (p.projetos || []).filter((x) => x.id !== id);
  if (!p.projetos.length) delete p.projetos;
  projetosAbertos.delete(chaveProjeto(p, id));
  guardaProjetosAbertos();
}

// ---- Planejar: o projeto da ação na proposta -------------------------------------------------------------------------------
// No cartão o projeto é uma chave: "" = Geral, o id de um projeto que já existe, ou "novo:<nome>" (projeto que a própria proposta cria).
function projetoDaProposta(f, existente) {
  if (f.projeto === undefined) return existente?.projetoId || ""; // a proposta não falou do projeto: a ação fica onde está
  return f.projeto ? f.projeto.id || `novo:${f.projeto.nome}` : "";
}
function htmlProjetoDaProposta(pf, p) {
  const novos = conv(p.id).projetosNovos || [], existentes = p.projetos || [];
  if (!novos.length && !existentes.length) return "";
  const opcao = (valor, nome) => `<option value="${esc(valor)}"${valor === (pf.projeto || "") ? " selected" : ""}>${esc(nome)}</option>`;
  return `<label class="f">${tr("Projeto")}<select data-pc="projeto">${opcao("", tr("Geral (sem projeto)"))}${existentes.map((x) => opcao(x.id, x.nome || tr("Sem nome"))).join("")}${novos.map((n) => opcao(`novo:${n}`, tr("{nome} (novo)", { nome: n }))).join("")}</select></label>`;
}
const htmlProjetosNovos = (c) => (c.projetosNovos?.length ? `<p class="dica">${tr("Projetos que esta proposta cria: {nomes}.", { nomes: `<strong>${c.projetosNovos.map(esc).join(", ")}</strong>` })}</p>` : "");
// ao aplicar: cria os projetos novos que ainda não existem (por nome) e devolve o id do projeto escolhido no cartão ("" = Geral)
function criaProjetosNovos(p, c) {
  (c.projetosNovos || []).forEach((nome) => { if (!(p.projetos || []).some((x) => norm(x.nome) === norm(nome))) (p.projetos ||= []).push({ id: novoId(), nome }); });
}
function idDoProjetoEscolhido(p, chave) {
  if (!chave) return "";
  if (chave.startsWith("novo:")) return (p.projetos || []).find((x) => norm(x.nome) === norm(chave.slice(5)))?.id || "";
  return (p.projetos || []).some((x) => x.id === chave) ? chave : "";
}
