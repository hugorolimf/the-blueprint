// ---- decisões do blueprint: o card no Editar (criar, editar, remover) e o cartão da proposta no Planejar ----
// O dado é `blueprint.decisoes = [{ id, titulo, escolha, motivo, estado: "aberta"|"decidida", em }]` (opcional: sem decisão o campo nem existe).
// O agente propõe em `blueprint.decisoes`; o servidor devolve cada uma já completa, e aqui o usuário marca o que aplicar (c.decisoesNovas, com `inc`).
const rotuloDaDecisao = (estado) => (estado === "aberta" ? tr("Em aberto") : tr("Decidida"));
const decisoesMarcadas = (c) => (c.decisoesNovas || []).filter((x) => x.inc).length;

function htmlDecisoesEd(p) {
  const lista = p.decisoes || [];
  const opcao = (x, v) => `<option value="${v}"${x.estado === v ? " selected" : ""}>${rotuloDaDecisao(v)}</option>`;
  const linha = (x) => `<div class="decisao-ed" data-dc="${esc(x.id)}"><div class="decisao-topo"><input data-c="titulo" placeholder="${tr("O que foi decidido (ex.: Servidor)")}" aria-label="${tr("Título da decisão")}" value="${esc(x.titulo)}"><select data-c="estado" aria-label="${tr("Estado da decisão")}">${opcao(x, "aberta")}${opcao(x, "decidida")}</select><button class="x" data-act="del-decisao" type="button" aria-label="${tr("Remover decisão")}">×</button></div><label class="f">${tr("Escolha")}<textarea data-c="escolha" rows="2" placeholder="${tr("O que foi escolhido")}">${esc(x.escolha)}</textarea></label><label class="f">${tr("Motivo")}<textarea data-c="motivo" rows="2" placeholder="${tr("Por que")}">${esc(x.motivo)}</textarea></label></div>`;
  return `<div class="card"><h3>${tr("Decisões")}<small>${lista.length || ""}</small></h3>${lista.length ? lista.map(linha).join("") : `<p class="vazio">${tr("Nenhuma decisão registrada. Stack, escopo e regras escolhidas ficam aqui, fora das instruções das ações.")}</p>`}<div class="acoes"><button class="btn mini" data-act="add-decisao" type="button">${tr("+ Decisão")}</button></div><p class="dica">${tr("O seu agente propõe decisões em Planejar; você escolhe quais aplicar. Uma decisão em aberto ainda espera a escolha.")}</p></div>`;
}
function novaDecisao(p) {
  const x = { id: novoId(), titulo: "", escolha: "", motivo: "", estado: "aberta", em: local(new Date()) };
  (p.decisoes ||= []).push(x);
  return x;
}
const LIMITE_DECISAO = { titulo: 120, escolha: 2000, motivo: 3000 };
function editarDecisao(p, id, campo, valor) {
  const x = (p.decisoes || []).find((y) => y.id === id);
  if (!x) return;
  x[campo] = campo === "estado" ? (valor === "aberta" ? "aberta" : "decidida") : String(valor).slice(0, LIMITE_DECISAO[campo] || 0);
}
function removerDecisao(p, id) {
  p.decisoes = (p.decisoes || []).filter((x) => x.id !== id);
  if (!p.decisoes.length) delete p.decisoes;
}

// ---- Planejar: as decisões que a proposta traz ----------------------------------------------------------------------------
function htmlDecisoesProposta(c, p) {
  const ds = c.decisoesNovas || [];
  if (!ds.length) return "";
  const linha = (d, j) => {
    const ja = d.id && (p.decisoes || []).some((x) => x.id === d.id);
    return `<label class="rem-item decisao-prop"><input type="checkbox" data-pc="dec-inc" data-j="${j}"${d.inc ? " checked" : ""}><span><b>${esc(d.titulo)}</b> <span class="badge${ja ? "" : " nova"}">${ja ? tr("Alterada") : tr("Nova")}</span> <span class="badge">${rotuloDaDecisao(d.estado)}</span>${d.escolha ? `<div class="md">${mdHtml(d.escolha)}</div>` : ""}${d.motivo ? `<small>${tr("Motivo")}: ${esc(d.motivo)}</small>` : ""}</span></label>`;
  };
  return `<div class="pf"><div class="pf-topo"><span class="badge nova">${tr("Decisões")}</span><b style="letter-spacing:.08em;text-transform:uppercase;font-size:12px">${trn(ds.length, ["{n} decisão", "{n} decisões"])}</b><span style="margin-left:auto"><button class="btn mini" data-pact="dec-todas" type="button">${tr("Marcar")}</button> <button class="btn mini" data-pact="dec-nenhuma" type="button">${tr("Nenhum")}</button></span></div><div>${ds.map(linha).join("")}</div></div>`;
}
// ao aplicar: cria as decisões novas e atualiza (por id) as que já existem; devolve quantas foram gravadas
function aplicaDecisoes(p, c) {
  let n = 0;
  (c.decisoesNovas || []).filter((d) => d.inc && d.titulo.trim()).forEach((d) => {
    const campos = { titulo: d.titulo, escolha: d.escolha || "", motivo: d.motivo || "", estado: d.estado === "aberta" ? "aberta" : "decidida", em: local(new Date()) };
    const x = d.id && (p.decisoes || []).find((y) => y.id === d.id);
    if (x) Object.assign(x, campos); else (p.decisoes ||= []).push({ id: novoId(), ...campos });
    n++;
  });
  return n;
}
