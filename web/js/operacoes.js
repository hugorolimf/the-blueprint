// ---- operações por item (docs/colaboracao.md, seção 6): a página continua editando o objeto `arquivo`; ao salvar, manda só a diferença ----
// opsEntre(base, tela): item novo vira `criar` (o mais alto da subárvore), campo diferente vira `alterar` com o `antes`, pai ou ordem relativa
// diferente vira `mover` (só os itens fora da maior subsequência comum, para os vizinhos que "andaram junto" não virarem operação) e item
// sumido vira `apagar` (o mais alto, depois dos `mover`, para o filho que saiu de um item apagado não se perder). A posição de cada criar e
// mover é calculada aplicando as operações numa cópia da base, com a mesma regra do servidor, então aplicar o lote na base dá a tela.
// aplicarOp(arvore, op) aplica uma operação; aplicarRemota(base, tela, op) aplica a de outra pessoa sem pisar no que esta tela ainda não mandou.
const OPS_FILHAS = { workspace: ["blueprints"], blueprint: ["acoes"], acao: ["planos", "checkpoints", "prototipos"] };
const OPS_TIPO_DA_LISTA = { workspaces: "workspace", blueprints: "blueprint", acoes: "acao", planos: "plano", checkpoints: "checkpoint", prototipos: "prototipo" };
const OPS_LISTA_DO_TIPO = { workspace: "workspaces", blueprint: "blueprints", acao: "acoes", plano: "planos", checkpoint: "checkpoints", prototipo: "prototipos" };
const OPS_PAI = { blueprint: "workspace", acao: "blueprint", plano: "acao", checkpoint: "acao", prototipo: "acao" };
const OPS_RAIZ = "sm";

const opsChave = (tipo, id) => tipo + ":" + id;
const opsListas = (tipo, raiz) => (raiz ? ["workspaces", ...OPS_FILHAS.workspace] : OPS_FILHAS[tipo] || []);
const opsClone = (x) => (x === undefined ? undefined : JSON.parse(JSON.stringify(x)));
function opsIgual(a, b) {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null || Array.isArray(a) !== Array.isArray(b)) return false;
  if (Array.isArray(a)) return a.length === b.length && a.every((x, i) => opsIgual(x, b[i]));
  const ka = Object.keys(a), kb = Object.keys(b);
  return ka.length === kb.length && ka.every((k) => Object.prototype.hasOwnProperty.call(b, k) && opsIgual(a[k], b[k]));
}

/** Todos os itens por "tipo:id": o nó, o tipo, a chave do pai e a lista em que está. */
function opsIndice(raiz) {
  const mapa = new Map();
  const visita = (no, tipo, chave, ehRaiz) => {
    for (const lista of opsListas(tipo, ehRaiz)) {
      if (!Array.isArray(no[lista])) continue;
      const tipoFilho = OPS_TIPO_DA_LISTA[lista];
      no[lista].forEach((filho) => {
        if (!filho || typeof filho.id !== "string") return;
        const c = opsChave(tipoFilho, filho.id);
        mapa.set(c, { no: filho, tipo: tipoFilho, pai: chave, paiNo: no, lista });
        visita(filho, tipoFilho, c, false);
      });
    }
  };
  mapa.set(opsChave("workspace", OPS_RAIZ), { no: raiz, tipo: "workspace", pai: null, paiNo: null, lista: null });
  visita(raiz, "workspace", opsChave("workspace", OPS_RAIZ), true);
  return mapa;
}

/** Os campos do item que `alterar` compara (sem o id e sem as listas filhas). */
function opsCampos(tipo, no, ehRaiz) {
  const fora = new Set(["id", ...opsListas(tipo, ehRaiz)]);
  return Object.keys(no).filter((k) => !fora.has(k));
}

/** A maior subsequência comum entre duas listas de ids: os que ficaram no lugar. */
function opsLcs(a, b) {
  const m = a.length, n = b.length, t = Array.from({ length: m + 1 }, () => new Uint16Array(n + 1));
  for (let i = m - 1; i >= 0; i--) for (let j = n - 1; j >= 0; j--) t[i][j] = a[i] === b[j] ? t[i + 1][j + 1] + 1 : Math.max(t[i + 1][j], t[i][j + 1]);
  const fica = new Set();
  for (let i = 0, j = 0; i < m && j < n;) {
    if (a[i] === b[j]) { fica.add(a[i]); i++; j++; } else if (t[i + 1][j] >= t[i][j + 1]) i++; else j++;
  }
  return fica;
}

function opsAchar(raiz, tipo, id) {
  if (tipo === "workspace" && id === OPS_RAIZ) return { no: raiz, lista: null };
  return opsIndice(raiz).get(opsChave(tipo, id)) || null;
}

function aplicarOp(raiz, op) {
  const achado = opsAchar(raiz, op.tipo, op.id);
  const inserir = (item) => {
    const pai = op.tipo === "workspace" ? { no: raiz } : opsAchar(raiz, OPS_PAI[op.tipo], op.pai);
    if (!pai) return false;
    const lista = OPS_LISTA_DO_TIPO[op.tipo];
    if (!Array.isArray(pai.no[lista])) pai.no[lista] = [];
    pai.no[lista].splice(Math.min(Number.isInteger(op.posicao) ? op.posicao : Infinity, pai.no[lista].length), 0, item);
    return true;
  };
  switch (op.op) {
    case "alterar":
      if (!achado) return false;
      for (const [k, v] of Object.entries(op.campos || {})) { if (v === null) delete achado.no[k]; else achado.no[k] = opsClone(v); }
      return true;
    case "criar":
      if (achado) return false;
      return inserir({ ...opsClone(op.valor), id: op.id });
    case "mover": {
      if (!achado || !achado.lista) return false;
      const lista = achado.paiNo[achado.lista], i = lista.indexOf(achado.no);
      if (op.tipo !== "workspace" && !opsAchar(raiz, OPS_PAI[op.tipo], op.pai)) return false;
      lista.splice(i, 1);
      return inserir(achado.no) || (lista.splice(i, 0, achado.no), false);
    }
    case "apagar":
      if (!achado || !achado.lista) return false;
      achado.paiNo[achado.lista].splice(achado.paiNo[achado.lista].indexOf(achado.no), 1);
      return true;
  }
  return false;
}

function opsEntre(base, atual) {
  const B = opsIndice(base), A = opsIndice(atual), sim = opsClone(base), criados = new Set(), estrutura = [], alteracoes = [], apagar = [];
  // a cópia do item novo só com os filhos que também são novos (os existentes entram depois, por `mover`)
  const soNovos = (no, tipo) => {
    const c = { ...no };
    for (const lista of opsListas(tipo, false)) {
      if (!Array.isArray(no[lista])) continue;
      const tf = OPS_TIPO_DA_LISTA[lista];
      c[lista] = no[lista].filter((f) => !B.has(opsChave(tf, f.id))).map((f) => { criados.add(opsChave(tf, f.id)); return soNovos(f, tf); });
    }
    return opsClone(c);
  };
  // logo depois do irmão anterior (na tela) que já está no lugar na cópia; 0 se não há
  const posicao = (paiNo, lista, filhos, i, tipoFilho, eu) => {
    const daCopia = (paiNo ? opsAchar(sim, ...paiNo) : null)?.no?.[lista] || [];
    const ids = daCopia.map((x) => x.id).filter((x) => x !== eu);
    for (let j = i - 1; j >= 0; j--) { const k = ids.indexOf(filhos[j].id); if (k >= 0) return k + 1; }
    return 0;
  };
  const visita = (no, tipo, chave, ehRaiz, ident) => {
    for (const lista of opsListas(tipo, ehRaiz)) {
      const filhos = Array.isArray(no[lista]) ? no[lista].filter((f) => f && typeof f.id === "string") : [];
      const tf = OPS_TIPO_DA_LISTA[lista];
      const naBase = B.get(chave)?.no?.[lista] || [];
      const comuns = filhos.filter((f) => B.get(opsChave(tf, f.id))?.pai === chave).map((f) => f.id);
      const ordemBase = naBase.filter((f) => A.get(opsChave(tf, f.id))?.pai === chave).map((f) => f.id);
      const fica = opsLcs(comuns, ordemBase);
      filhos.forEach((f, i) => {
        const c = opsChave(tf, f.id), pai = tf === "workspace" ? {} : { pai: ident[1] };
        if (criados.has(c)) { /* já foi dentro do valor do pai novo */ }
        else if (!B.has(c)) {
          const op = { op: "criar", tipo: tf, id: f.id, ...pai, posicao: posicao(ident, lista, filhos, i, tf, f.id), valor: soNovos(f, tf) };
          criados.add(c); estrutura.push(op); aplicarOp(sim, op);
        } else if (B.get(c).pai !== chave || !fica.has(f.id)) {
          const op = { op: "mover", tipo: tf, id: f.id, ...pai, posicao: posicao(ident, lista, filhos, i, tf, f.id) };
          estrutura.push(op); aplicarOp(sim, op);
        }
        visita(f, tf, c, false, [tf, f.id]);
      });
    }
  };
  visita(atual, "workspace", opsChave("workspace", OPS_RAIZ), true, ["workspace", OPS_RAIZ]);
  for (const [c, a] of A) {
    const b = B.get(c);
    if (!b) continue;
    const ehRaiz = a.pai === null, campos = {}, antes = {};
    for (const k of new Set([...opsCampos(a.tipo, b.no, ehRaiz), ...opsCampos(a.tipo, a.no, ehRaiz)])) {
      if (opsIgual(a.no[k], b.no[k])) continue;
      campos[k] = a.no[k] === undefined ? null : opsClone(a.no[k]);
      antes[k] = b.no[k] === undefined ? null : opsClone(b.no[k]);
    }
    if (Object.keys(campos).length) alteracoes.push({ op: "alterar", tipo: a.tipo, id: ehRaiz ? OPS_RAIZ : a.no.id, campos, antes });
  }
  for (const [c, b] of B) if (b.pai !== null && !A.has(c) && A.has(b.pai)) apagar.push({ op: "apagar", tipo: b.tipo, id: b.no.id });
  return [...estrutura, ...alteracoes, ...apagar];
}

/** A operação de outra pessoa: entra na base inteira e na tela menos nos campos que esta tela mudou e ainda não mandou. */
function aplicarRemota(base, tela, op) {
  const r = { conflitos: [], perdeu: false };
  if (op.op === "alterar") {
    const b = opsAchar(base, op.tipo, op.id), t = opsAchar(tela, op.tipo, op.id), campos = {};
    for (const [k, v] of Object.entries(op.campos || {})) {
      const meu = t && b && !opsIgual(t.no[k], b.no[k]);
      if (!meu) campos[k] = v;
      else if (!opsIgual(t.no[k], v === null ? undefined : v)) r.conflitos.push(k);
    }
    aplicarOp(base, op);
    if (Object.keys(campos).length) aplicarOp(tela, { ...op, campos });
    return r;
  }
  if (op.op === "apagar") {
    const b = opsAchar(base, op.tipo, op.id), t = opsAchar(tela, op.tipo, op.id);
    r.perdeu = !!(t && (!b || !opsIgual(t.no, b.no)));
  }
  aplicarOp(base, op);
  aplicarOp(tela, op);
  return r;
}
