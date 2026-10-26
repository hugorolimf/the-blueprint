// ---- avisos da sincronia: conflito e item apagado, com o nome da outra pessoa (docs/colaboracao.md, seção 6, cenários 2 a 4) ----
// Um aviso por vez, num quadro fixo (#sinc-aviso) com botões opcionais (Desfazer, Copiar); some sozinho depois de alguns segundos.
let sincAvisoT = 0;
// cada rótulo é um tr() literal, para o verificador de catálogos (scripts/i18n-completude.mjs) enxergar a chave
const SINC_CAMPOS = { texto: () => tr("o título"), nome: () => tr("o nome"), descricao: () => tr("a descrição"), obs: () => tr("as instruções"), inicio: () => tr("o início"), fim: () => tr("o fim"), feito: () => tr("a marcação"), status: () => tr("a situação"), data: () => tr("a data"), horas: () => tr("as horas") };
const sincCampo = (k) => (Object.hasOwn(SINC_CAMPOS, k) ? SINC_CAMPOS[k]() : tr("um campo"));

function sincAviso(texto, botoes = []) {
  let el = $("sinc-aviso");
  if (!el) {
    el = document.createElement("div");
    el.id = "sinc-aviso";
    el.setAttribute("role", "status");
    el.setAttribute("aria-live", "polite");
    document.body.append(el);
  }
  el.replaceChildren(Object.assign(document.createElement("span"), { textContent: texto }));
  for (const [rotulo, acao] of botoes) {
    const b = Object.assign(document.createElement("button"), { type: "button", className: "btn mini", textContent: rotulo });
    b.onclick = () => { acao(); el.hidden = true; };
    el.append(b);
  }
  el.hidden = false;
  clearTimeout(sincAvisoT);
  sincAvisoT = setTimeout(() => { el.hidden = true; }, botoes.length ? 12000 : 7000);
}

/** Quem é, para o texto: eu em outra janela, outra pessoa ou alguém que não ficou no histórico (um script, o banco antigo). */
const sincQuem = (quem) => (!quem ? null : quem.id === aoVivoEu() ? "" : quem.nome || tr("Alguém"));

/** O que o servidor respondeu ao meu lote: conflitos (ficou o meu, com Desfazer) e operações ignoradas (item ou pai apagado, id repetido). */
function sincResposta(resposta, ops) {
  for (const c of resposta.conflitos || []) {
    const quem = sincQuem(c.quem), campo = sincCampo(c.campo);
    const texto = quem === "" ? tr("Você mudou isto em outra janela; ficou o desta.") : quem ? tr("{quem} tinha mudado {campo}; ficou o seu.", { quem, campo }) : tr("Alguém tinha mudado {campo}; ficou o seu.", { campo });
    const desfazer = () => { const t = opsAchar(arquivo, c.tipo, c.id); if (!t) return; if (c.deles === null) delete t.no[c.campo]; else t.no[c.campo] = c.deles; mudou(); };
    sincAviso(texto, [[tr("Desfazer"), desfazer]]);
  }
  let resalvar = false;
  for (const ig of resposta.ignoradas || []) {
    const op = ops[ig.indice];
    if (!op) continue;
    if (ig.codigo === "id-existe") { // raro (ids aleatórios): o item ganha outro id e vai no próximo lote
      const t = opsAchar(arquivo, op.tipo, op.id);
      if (t) { t.no.id = novoId(); resalvar = true; }
      continue;
    }
    const texto = Object.values(op.campos || op.valor || {}).filter((v) => typeof v === "string" && v).join("\n");
    const quem = sincQuem(ig.quem);
    aplicarOp(arquivo, { op: "apagar", tipo: op.tipo, id: op.id }); // o item não existe mais no servidor: sai da tela também
    const msg = quem === "" ? tr("Você apagou este item em outra janela; esta mudança não foi gravada.") : tr("{quem} apagou este item; sua mudança não foi gravada.", { quem: quem || tr("Alguém") });
    sincAviso(msg, texto ? [[tr("Copiar o texto"), () => navigator.clipboard?.writeText(texto).catch(() => {})]] : []);
    aoVivoRedesenhar();
  }
  if (resalvar) salvar();
}
