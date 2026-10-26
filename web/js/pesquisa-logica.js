// Pesquisa: a correspondência de texto, sem DOM (a tela está em pesquisa-folha.js e pesquisa-campo.js; teste: pesquisa-logica.test.mjs).
// Sem acento e sem caixa; a consulta vira termos separados por espaço e TODOS têm de aparecer no item (nos campos dele, juntos).
// Cada campo diz onde achou: "nome", "descricao" ou "instrucoes". Os trechos voltam como [início, fim) no texto ORIGINAL (para destacar).
// shortcut: o termo é achado como pedaço de texto (não por palavra inteira), como a busca antiga; mudar se a lista de resultados ficar ruidosa.

// Texto sem acento e minúsculo, com o índice de cada letra no texto original (a normalização pode mudar o tamanho de uma letra).
function pesqNorma(texto) {
  const original = String(texto ?? ""), mapa = [];
  let s = "";
  for (let i = 0; i < original.length; i++) {
    const n = original[i].normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
    s += n;
    for (let k = 0; k < n.length; k++) mapa.push(i);
  }
  return { s, mapa };
}
const pesqTermos = (consulta) => [...new Set(pesqNorma(consulta).s.split(/\s+/).filter(Boolean))];

// os trechos do texto original onde algum termo aparece (juntados quando se tocam); [] se nenhum
function pesqTrechos(texto, termos) {
  const { s, mapa } = pesqNorma(texto), achados = [];
  for (const t of termos) for (let i = s.indexOf(t); i >= 0; i = s.indexOf(t, i + t.length)) achados.push([mapa[i], mapa[i + t.length - 1] + 1]);
  achados.sort((a, b) => a[0] - b[0]);
  const fim = [];
  for (const a of achados) { const u = fim[fim.length - 1]; if (u && a[0] <= u[1]) u[1] = Math.max(u[1], a[1]); else fim.push([a[0], a[1]]); }
  return fim;
}

// `campos`: { nome: texto, descricao: texto, ... }. Casa quando cada termo aparece em algum campo; devolve os campos em que algo apareceu, ou null.
function pesqCampos(campos, termos) {
  if (!termos.length) return null;
  const ns = Object.entries(campos).map(([k, v]) => [k, pesqNorma(v).s]);
  if (!termos.every((t) => ns.some(([, s]) => s.includes(t)))) return null;
  return ns.filter(([, s]) => termos.some((t) => s.includes(t))).map(([k]) => k);
}

const pesqAcao = (f, termos) => pesqCampos({ nome: f.nome, instrucoes: f.obs }, termos);
const pesqPlano = (t, termos) => pesqCampos({ nome: t.texto, descricao: t.descricao }, termos);
const pesqCheckpoint = (m, termos) => pesqCampos({ nome: m.nome }, termos);

// Um blueprint inteiro: { proprio, acoes: [{ id, propria, planos: [{id, onde}], checkpoints: [{id, onde}] }], n: { acoes, planos, checkpoints, total } }.
// `acoes` só traz as que têm algo a mostrar (a própria ação casa ou algum plano/checkpoint dela); `proprio` é o casamento do nome/descrição do blueprint.
function pesqBlueprint(p, termos, acoes = p.acoes || []) {
  const res = { proprio: pesqCampos({ nome: p.nome, descricao: p.descricao }, termos), acoes: [], n: { acoes: 0, planos: 0, checkpoints: 0, total: 0 } };
  if (!termos.length) return res;
  for (const f of acoes) {
    const propria = pesqAcao(f, termos);
    const planos = (f.planos || []).map((t) => ({ id: t.id, onde: pesqPlano(t, termos) })).filter((x) => x.onde);
    const checkpoints = (f.checkpoints || []).map((m) => ({ id: m.id, onde: pesqCheckpoint(m, termos) })).filter((x) => x.onde);
    if (!propria && !planos.length && !checkpoints.length) continue;
    res.acoes.push({ id: f.id, propria, planos, checkpoints });
    res.n.acoes += propria ? 1 : 0; res.n.planos += planos.length; res.n.checkpoints += checkpoints.length;
  }
  res.n.total = res.n.acoes + res.n.planos + res.n.checkpoints;
  return res;
}
// o blueprint aparece na lista do workspace? (nome, descrição, ou qualquer ação, plano ou checkpoint)
const pesqCasaBlueprint = (r) => !!r.proprio || r.acoes.length > 0;
