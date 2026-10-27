// ---- apresentação com IA, editor manual: o HTML do painel, a partir do esquema (apresentacao-ia-esquema.js) e do documento { conteudo, figuras } ----
// Só desenha (strings): os eventos, o salvador e a conversa com o servidor estão em apresentacao-ia-editor.js. Todo texto vindo dos dados passa por esc().
// Os campos do `conteudo` levam data-c="<caminho pontuado>" e data-tipo; os das figuras, data-fig="<id>" e data-campo; os botões de lista, data-ed e data-caminho.
const apEdBotao = (ed, caminho, texto, extra = "") => `<button class="btn mini" type="button" data-ed="${ed}" data-caminho="${esc(caminho)}"${extra}>${texto}</button>`;

// um controle de formulário; `ligacao` = os atributos que dizem onde o valor vai (data-c ou data-fig/data-campo)
function apEdControle(tipo, ligacao, valor, { opcoes = [], rotOp = {}, doc, obrigatorio = false, vazio = "" } = {}) {
  const v = valor ?? "", base = `${ligacao} data-tipo="${tipo}"${obrigatorio ? " required" : ""}`;
  if (tipo === "area") return `<textarea ${base} rows="3">${esc(v)}</textarea>`;
  if (tipo === "data") return `<input type="date" ${base} value="${esc(v)}">`;
  if (tipo === "opcoes" || tipo === "secao") {
    const lista = tipo === "secao" ? AP_IA_SECOES_FIGURA : opcoes;
    return `<select ${base}><option value="">${vazio || tr("Nenhuma")}</option>${lista.map((o) => `<option value="${esc(o)}"${o === v ? " selected" : ""}>${esc(rotOp[o] || o)}</option>`).join("")}</select>`;
  }
  if (tipo === "figura") return `<select ${base}><option value="">${tr("Nenhuma")}</option>${(doc.figuras || []).map((f) => `<option value="${esc(f.id)}"${f.id === v ? " selected" : ""}>${esc(f.titulo || f.id)}</option>`).join("")}</select>`;
  return `<input type="text" ${base} value="${esc(v)}">`;
}
const apEdCampo = (rotulo, controle) => `<label class="ap-ed-c"><span>${rotulo}</span>${controle}</label>`;

function apEdCampos(base, campos, objeto, doc, rot, rotOp) {
  return campos.map(([chave, tipo, obrig, opcoes]) => apEdCampo(rot[chave] || chave, apEdControle(tipo, `data-c="${esc(base ? base + "." + chave : chave)}"`, objeto?.[chave], { opcoes, rotOp, doc, obrigatorio: !!obrig }))).join("");
}

// uma lista (de objetos ou de frases) com seus itens, os botões de cada um e o de adicionar; `sub` desenha as listas de dentro do item
function apEdLista(def, caminho, lista, doc, rot, rotOp) {
  const itens = (lista || []).map((item, i) => {
    const c = `${caminho}.${i}`, ultimo = i === lista.length - 1;
    const botoes = `<div class="ap-ed-botoes">${apEdBotao("sobe", c, "↑", ` aria-label="${esc(tr("Subir"))}"${i ? "" : " disabled"}`)}${apEdBotao("desce", c, "↓", ` aria-label="${esc(tr("Descer"))}"${ultimo ? " disabled" : ""}`)}${apEdBotao("remove", c, tr("Remover"))}</div>`;
    if (def.simples) return `<div class="ap-ed-linha"><input type="text" data-c="${esc(c)}" data-tipo="texto" value="${esc(item)}" aria-label="${esc(def.item + " " + (i + 1))}">${botoes}</div>`;
    const nome = def.marca && item?.[def.marca] ? ` · ${esc(corta(String(item[def.marca]), 40))}` : "";
    const subs = (def.sub || []).map((s) => apEdLista(s, `${c}.${s.chave}`, item?.[s.chave], doc, rot, rotOp)).join("");
    return `<fieldset class="ap-ed-item"><legend>${def.item} ${i + 1}${nome}</legend>${apEdCampos(c, def.campos, item, doc, rot, rotOp)}${subs}${botoes}</fieldset>`;
  }).join("");
  return `<div class="ap-ed-lista"${def.sub ? "" : ""}>${def.titulo && def.chave ? `<p class="ap-ed-sub">${def.titulo}</p>` : ""}${itens}${apEdBotao("adiciona", caminho, def.novo)}</div>`;
}

function apEdFigura(f, i, total, rot, rotOp) {
  const lig = (campo) => `data-fig="${esc(f.id)}" data-campo="${campo}"`;
  const campos = apIaCamposFigura().map(([chave, tipo]) => apEdCampo(rot[chave] || chave, apEdControle(tipo, lig(chave), f[chave], { rotOp, vazio: tr("Nenhuma") }))).join("");
  const chaves = (f.chaves || []).map((k, j) => `<div class="ap-ed-linha ap-ed-chave"><select data-fig="${esc(f.id)}" data-campo="chaves" data-j="${j}" data-k="marca" aria-label="${esc(rot.marca)}">${AP_IA_MARCAS.map((m) => `<option value="${m}"${m === k.marca ? " selected" : ""}>${esc(rotOp[m])}</option>`).join("")}</select><input type="text" data-fig="${esc(f.id)}" data-campo="chaves" data-j="${j}" data-k="rotulo" value="${esc(k.rotulo)}" aria-label="${esc(tr("Texto da chave"))}"><button class="btn mini" type="button" data-ed="chave-remove" data-fig="${esc(f.id)}" data-j="${j}">${tr("Remover")}</button></div>`).join("");
  return `<fieldset class="ap-ed-item" data-figura="${esc(f.id)}"><legend>${tr("Figura")} ${i + 1} · ${esc(f.id)}</legend>${campos}
<p class="ap-ed-sub">${tr("Chaves da legenda")}</p>${chaves}<button class="btn mini" type="button" data-ed="chave-adiciona" data-fig="${esc(f.id)}">${tr("Adicionar chave")}</button>
<details class="ap-ed-avancado"><summary>${tr("Código da figura (avançado)")}</summary><textarea data-fig="${esc(f.id)}" data-campo="codigo" data-tipo="codigo" rows="10" spellcheck="false" aria-label="${esc(tr("Código da figura"))}">${esc(f.codigo || "")}</textarea><p class="ap-ed-sintaxe" role="status"></p></details>
<div class="ap-ed-botoes"><button class="btn mini" type="button" data-ed="figura-duplica" data-fig="${esc(f.id)}">${tr("Duplicar")}</button><button class="btn mini" type="button" data-ed="figura-remove" data-fig="${esc(f.id)}">${tr("Remover")}</button></div></fieldset>`;
}

const apEdSecao = (id, titulo, n, corpo, aberta) => `<details class="ap-ed-sec" data-sec="${id}"${aberta ? " open" : ""}><summary>${titulo}${n === null ? "" : `<span class="ap-ed-n">${n}</span>`}</summary><div class="ap-ed-corpo">${corpo}</div></details>`;

// o painel inteiro; `abertas` = os ids das seções que estavam abertas (a Geral abre na primeira vez)
function apEdHtml(doc, abertas) {
  const c = doc.conteudo || {}, rot = apIaRotulosCampo(), rotOp = apIaRotulosOpcao(), listas = apIaListas();
  const aberta = (id) => (abertas ? abertas.has(id) : id === "geral");
  const secoes = [apEdSecao("geral", tr("Geral"), null, apEdCampos("", apIaGeral(), c, doc, rot, rotOp), aberta("geral"))];
  for (const [id, def] of Object.entries(listas)) {
    if (def.interno && c.publico !== "interno") continue;
    const lista = def.caminho.split(".").reduce((o, k) => o?.[k], c);
    secoes.push(apEdSecao(id, def.titulo, (lista || []).length, apEdLista(def, def.caminho, lista, doc, rot, rotOp), aberta(id)));
  }
  const figs = doc.figuras || [];
  secoes.push(apEdSecao("figuras", tr("Figuras"), figs.length, figs.map((f, i) => apEdFigura(f, i, figs.length, rot, rotOp)).join("") || `<p class="ap-dica">${tr("Esta apresentação não tem figuras.")}</p>`, aberta("figuras")));
  return secoes.join("");
}
