// ---- apresentação com IA, editor manual: as operações (lógica pura, sem DOM; o editor está em apresentacao-ia-editor.js) ----
// Contrato em docs/interno/apresentacao-ia.md ("Operations"). O editor manda UMA operação por mudança: aqui elas se aplicam a uma cópia local (o que o usuário vê antes de o
// servidor responder), se invertem (Desfazer) e se reaplicam sobre o documento que o servidor devolve (o agente pode ter mexido em outro ponto). O documento é
// { conteudo, figuras }; os caminhos são pontuados e relativos ao `conteudo` ("etapas.2.titulo", índice de lista a partir de 0).
// Operação interna `__figuras` (troca a lista inteira de figuras, vai ao servidor como `figuras`): só para duplicar e para desfazer a remoção de uma figura.
const apOpsClone = (v) => (v === undefined ? undefined : JSON.parse(JSON.stringify(v)));
const apOpsPartes = (caminho) => String(caminho ?? "").split(".").filter((p) => p !== "");
const apOpsErro = (codigo) => Object.assign(new Error(codigo), { codigo });
const apOpsIdx = (p, lista) => { const n = Number(p); return Number.isInteger(n) && n >= 0 && n < lista.length ? n : -1; };

// o nó que contém o último trecho do caminho; `criar` faz objetos que faltam no meio (a última chave de um `definir` pode não existir)
function apOpsPai(conteudo, caminho, criar = false) {
  const partes = apOpsPartes(caminho), ultimo = partes.pop();
  if (ultimo === undefined) throw apOpsErro("caminho");
  let no = conteudo;
  for (const p of partes) {
    if (Array.isArray(no)) { const i = apOpsIdx(p, no); if (i < 0) throw apOpsErro("caminho"); no = no[i]; }
    else if (no && typeof no === "object") { if (!(p in no) && criar) no[p] = {}; if (!(p in no)) throw apOpsErro("caminho"); no = no[p]; }
    else throw apOpsErro("caminho");
  }
  if (!no || typeof no !== "object") throw apOpsErro("caminho");
  return { no, ultimo, paiCaminho: partes.join(".") };
}
const apOpsJunta = (pai, chave) => (pai ? `${pai}.${chave}` : String(chave));
function apOpsLista(conteudo, caminho) {
  let no = conteudo;
  for (const p of apOpsPartes(caminho)) no = Array.isArray(no) ? no[apOpsIdx(p, no)] : no?.[p];
  if (!Array.isArray(no)) throw apOpsErro("nao-e-lista");
  return no;
}
function apOpsFigura(doc, id) {
  const i = (doc.figuras || []).findIndex((f) => f.id === id);
  if (i < 0) throw apOpsErro("figura-desconhecida");
  return i;
}

// aplica UMA operação ao documento (muda `doc`) e devolve a operação que a desfaz; lança com `codigo` se a operação não cabe no documento
function apOpsAplica(doc, op) {
  const c = doc.conteudo;
  switch (op?.op) {
    case "definir": {
      const { no, ultimo } = apOpsPai(c, op.caminho, true);
      if (Array.isArray(no)) {
        const i = apOpsIdx(ultimo, no); if (i < 0) throw apOpsErro("caminho");
        const antes = no[i]; no[i] = apOpsClone(op.valor);
        return { op: "definir", caminho: op.caminho, valor: antes };
      }
      const existia = ultimo in no, antes = no[ultimo];
      no[ultimo] = apOpsClone(op.valor);
      return existia ? { op: "definir", caminho: op.caminho, valor: antes } : { op: "remover", caminho: op.caminho };
    }
    case "inserir": {
      const lista = apOpsLista(c, op.caminho), i = op.indice == null ? lista.length : Math.min(Math.max(0, op.indice | 0), lista.length);
      lista.splice(i, 0, apOpsClone(op.valor));
      return { op: "remover", caminho: apOpsJunta(op.caminho, i) };
    }
    case "remover": {
      const { no, ultimo, paiCaminho } = apOpsPai(c, op.caminho);
      if (Array.isArray(no)) { const i = apOpsIdx(ultimo, no); if (i < 0) throw apOpsErro("caminho"); const [x] = no.splice(i, 1); return { op: "inserir", caminho: paiCaminho, indice: i, valor: x }; }
      if (!(ultimo in no)) throw apOpsErro("caminho");
      const x = no[ultimo]; delete no[ultimo];
      return { op: "definir", caminho: op.caminho, valor: x };
    }
    case "mover": {
      const { no, ultimo, paiCaminho } = apOpsPai(c, op.caminho);
      const i = Array.isArray(no) ? apOpsIdx(ultimo, no) : -1, para = Math.min(Math.max(0, op.para | 0), (no.length || 1) - 1);
      if (i < 0) throw apOpsErro("caminho");
      no.splice(para, 0, no.splice(i, 1)[0]);
      return { op: "mover", caminho: apOpsJunta(paiCaminho, para), para: i };
    }
    case "figura_definir": {
      const f = doc.figuras[apOpsFigura(doc, op.figura)];
      if (op.campo === "codigo" || op.campo === "id") throw apOpsErro("campo");
      const existia = op.campo in f, antes = f[op.campo];
      f[op.campo] = apOpsClone(op.valor);
      return existia ? { op: "figura_definir", figura: op.figura, campo: op.campo, valor: antes } : { op: "figura_definir", figura: op.figura, campo: op.campo, valor: op.campo === "secao" ? null : "" };
    }
    case "figura_trecho": {
      const f = doc.figuras[apOpsFigura(doc, op.figura)], de = String(op.de ?? ""), codigo = String(f.codigo ?? "");
      const n = de ? codigo.split(de).length - 1 : 0;
      if (n !== 1) throw Object.assign(apOpsErro("trecho-nao-unico"), { ocorrencias: n });
      f.codigo = codigo.replace(de, () => String(op.para ?? ""));
      return { op: "figura_trecho", figura: op.figura, de: String(op.para ?? ""), para: de };
    }
    case "figura_remover": {
      const antes = apOpsClone(doc.figuras), i = apOpsFigura(doc, op.figura);
      doc.figuras.splice(i, 1);
      return { op: "__figuras", valor: antes };
    }
    case "__figuras": {
      const antes = apOpsClone(doc.figuras);
      doc.figuras = apOpsClone(op.valor);
      return { op: "__figuras", valor: antes };
    }
    default: throw apOpsErro("operacao");
  }
}

// aplica uma lista a uma CÓPIA; devolve { doc, inversos (já na ordem em que se desfaz), falhas: [{op, codigo}] }; uma operação que não cabe é pulada, não quebra as outras
function apOpsAplicaLista(doc, ops) {
  const novo = apOpsClone(doc), inversos = [], falhas = [];
  for (const op of ops) {
    try { inversos.unshift(apOpsAplica(novo, op)); }
    catch (e) { falhas.push({ op, codigo: e.codigo || "operacao" }); }
  }
  return { doc: novo, inversos, falhas };
}

// várias digitações no mesmo campo viram uma só (a última vence); o resto segue na ordem
function apOpsJuntar(ops) {
  const saida = [];
  for (const op of ops) {
    const ult = saida[saida.length - 1];
    if (ult && op.op === "definir" && ult.op === "definir" && ult.caminho === op.caminho) ult.valor = op.valor;
    else if (ult && op.op === "figura_definir" && ult.op === "figura_definir" && ult.figura === op.figura && ult.campo === op.campo) ult.valor = op.valor;
    else saida.push(apOpsClone(op));
  }
  return saida;
}

// o corpo de POST /api/mcp/apresentacao/editar para uma lista: a troca da lista de figuras (`__figuras`) vai como `figuras` e quebra o lote em dois pedidos, na ordem
function apOpsCorpos({ workspaceId, blueprintId, apresentacaoId }, ops, autor = "usuario") {
  const base = { workspaceId, blueprintId, apresentacaoId, autor }, corpos = [];
  let lote = [];
  const fecha = () => { if (lote.length) corpos.push({ ...base, operacoes: lote }); lote = []; };
  for (const op of ops) {
    if (op.op === "__figuras") { fecha(); corpos.push({ ...base, figuras: op.valor }); } else lote.push(op);
  }
  fecha();
  return corpos;
}

// o que o servidor devolveu (documento novo) com as operações que o usuário ainda não mandou por cima: o agente pode ter mudado outro ponto sem apagar o que se digitava
const apOpsReaplicar = (docServidor, pendentes) => apOpsAplicaLista(docServidor, pendentes);

// as frases dos erros do servidor (o `codigo` é o do contrato) e dos erros de operação
function apOpsFraseErro(status, j) {
  const f = {
    "falta-titulo": tr("O título não pode ficar vazio."),
    "publico-invalido": tr("O público precisa ser Cliente ou Interno."),
    "nivel-invalido": tr("O nível precisa ser Rápido, Padrão ou Completo."),
    "interno-no-cliente": tr("Uma apresentação para o cliente não pode ter a seção Ações: ela mostraria o lado técnico."),
    "figuras-invalidas": tr("A lista de figuras não é válida."),
    "figuras-demais": tr("São figuras demais: o limite é 12."),
    "figura-id-invalido": tr("O identificador de uma figura não é válido."),
    "figura-id-repetido": tr("Duas figuras têm o mesmo identificador."),
    "codigo-invalido": tr("O código de uma figura não é texto."),
    "codigo-grande": tr("O código de uma figura passa de 200 KB."),
    "conteudo-grande": tr("O conteúdo passa de 128 KB."),
    "apresentacao-grande": tr("A apresentação passa de 2 MB."),
    "apresentacao-nao-encontrada": tr("Esta apresentação não existe mais. Volte à lista."),
    "caminho-invalido": tr("Esse campo já não existe na apresentação. O painel voltou ao que está salvo."),
    "operacao-invalida": tr("A mudança não cabe na apresentação como ela está agora."),
    "figura-nao-encontrada": tr("Essa figura já não existe na apresentação."),
    "trecho-ausente": tr("O trecho do código que você quer trocar não está mais lá."),
    "trecho-ambiguo": tr("O trecho do código aparece mais de uma vez: não dá para saber qual trocar."),
  };
  return f[j?.codigo] || j?.erro || tr("Não foi possível salvar (erro {status}).", { status });
}

// o fragmento (#) da página gerada que corresponde ao campo editado: o visor recarrega nessa seção (os ids são os de apresentacao-ia/biblioteca/secoes.js)
const apOpsFragmentoSecao = (secao) => ({ visao: "s-visao", recebe: "s-entregas", etapas: "s-etapas", linha: "s-linha", precisamos: "s-voce", decisoes: "s-decisoes", riscos: "s-decisoes", acoes: "s-acoes" })[secao] || "s-visao";
// o destino para o visor rolar (postMessage "ap-ir" da página gerada): a seção e, nas etapas, o número da etapa (1 a n)
function apOpsDestino(caminho, doc) {
  const [a, b] = apOpsPartes(caminho);
  if (a === "figuras") { const f = (doc?.figuras || []).find((x) => x.id === b); return { secao: f?.secao ? ({ recebe: "entregas", precisamos: "voce", riscos: "decisoes" })[f.secao] || f.secao : "etapas" }; }
  if (a === "etapas" && /^\d+$/.test(b || "")) return { secao: "etapas", etapa: Number(b) + 1 };
  return { secao: ({ entregas: "entregas", etapas: "etapas", precisamosDeVoce: "voce", decisoes: "decisoes", riscos: "decisoes", interno: "acoes" })[a] || "visao" };
}
function apOpsFragmento(caminho) {
  const [a, b] = apOpsPartes(caminho);
  if (a === "etapas" && /^\d+$/.test(b || "")) return `etapas-${Number(b) + 1}`; // a página restaura "#etapas-3"
  const mapa = { entregas: "s-entregas", etapas: "s-etapas", precisamosDeVoce: "s-voce", decisoes: "s-decisoes", riscos: "s-decisoes", interno: "s-acoes", figuras: "s-etapas" };
  return mapa[a] || "s-visao";
}
