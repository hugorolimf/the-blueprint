// ---- ao vivo: as mudanças das outras pessoas (e das outras janelas) chegam por GET /api/eventos (SSE) e entram por item ----------------
// Cada lote vem com `id:` = versão; o navegador reconecta sozinho mandando Last-Event-ID e o servidor reenvia o que faltou, ou manda
// `recarregar` quando há buraco (app reiniciado, buffer curto, PUT no meio): aí a página faz o GET inteiro e junta com sincMesclar.
// A tela é redesenhada uma vez por quadro (requestAnimationFrame) e espera quem está digitando. Nada roda em repouso: só o SSE aberto.
// Ver docs/colaboracao.md, seção 6.
const CLIENTE = "c-" + Math.random().toString(36).slice(2, 10);
let aoVivoES = null, aoVivoWs = "", aoVivoQuadro = 0, aoVivoAdiado = false;
let aoVivoBase = null; // o servidor como esta página o conhece (a BASE das operações); a tela é `arquivo`
const aoVivoVisto = new Map(); // workspace -> última versão aplicada dele (o que veio antes disso é eco ou repetição)
let aoVivoVersaoGet = 0; // a versão do último GET inteiro: o ponto de partida de todo workspace
let aoVivoMinhas = []; // {versao, ops, em} das gravações desta janela: um lote mais velho que o meu não pisa nos campos que eu gravei depois
const aoVivoEu = () => (typeof sessao !== "undefined" && sessao?.membro?.id) || "local"; // a sessão vem de sessao.js (frente TELAS); sem ela é o modo local

/** Um retrato do arquivo como vai ao servidor (sem o estado só da tela). */
const instantaneoDe = (a) => JSON.parse(JSON.stringify({ ...a, chats: (a.chats || []).map(({ pensando, ...c }) => c) }));

/** O arquivo acabou de vir inteiro do servidor (GET): ele é a nova base e a contagem de versões recomeça dele. */
function aoVivoAssumir(versao) {
  aoVivoBase = instantaneoDe(arquivo);
  aoVivoVersaoGet = Number(versao) || 0;
  aoVivoVisto.clear();
  aoVivoMinhas = [];
  aoVivoConectar(true);
}

/** Abre (ou troca) a assinatura do workspace aberto. `forcar` reabre mesmo no mesmo workspace (depois de um GET). */
function aoVivoConectar(forcar = false) {
  if (typeof EventSource !== "function" || !aoVivoBase) return;
  const ws = workspaceAtual?.id || "sm";
  if (!forcar && aoVivoES && ws === aoVivoWs) return;
  aoVivoES?.close();
  aoVivoWs = ws;
  const desde = aoVivoVisto.get(ws) || aoVivoVersaoGet;
  // `blueprint`: quem só lê um blueprint (o visitante do link) assina só ele; quem lê o workspace recebe o workspace todo de qualquer jeito
  const bp = typeof tela === "string" && tela ? `&blueprint=${encodeURIComponent(tela)}` : "";
  const es = new EventSource(`/api/eventos?workspace=${encodeURIComponent(ws)}${bp}&desde=${desde}&cliente=${CLIENTE}`);
  aoVivoES = es;
  es.addEventListener("mudancas", (e) => aoVivoLote(JSON.parse(e.data)));
  es.addEventListener("substituido", () => aoVivoRecarregar());
  es.addEventListener("recarregar", () => aoVivoRecarregar());
  es.addEventListener("presenca", (e) => typeof presencaRecebida === "function" && presencaRecebida(JSON.parse(e.data)));
  es.addEventListener("sessao-encerrada", () => { es.close(); typeof semSessao === "function" && semSessao(); });
}
const aoVivoAberto = () => !!aoVivoES && aoVivoES.readyState !== 2;
window.addEventListener("hashchange", () => aoVivoConectar());

/** Um lote de outra pessoa (ou janela): aplica cada operação na base e na tela, menos o eco desta janela e o que já foi visto. */
function aoVivoLote(lote) {
  const ws = lote.workspaceId, v = Number(lote.versao);
  if (v <= (aoVivoVisto.get(ws) || aoVivoVersaoGet)) return;
  aoVivoVisto.set(ws, v);
  aoVivoMinhas = aoVivoMinhas.filter((m) => m.versao > v || Date.now() - m.em < 600000); // as minhas ficam 10 min para o aviso "mudou o que você editou"
  if (lote.cliente === CLIENTE) return; // o eco do meu próprio lote: a base já o tem
  const quem = lote.autor?.nome || tr("Alguém");
  let mudou = false;
  for (const op of lote.ops || []) {
    const depois = aoVivoMinhas.filter((m) => m.versao > v); // gravações minhas que o servidor aplicou depois deste lote: elas venceram
    const pisado = (campo) => depois.some((m) => m.ops.some((o) => o.op === "alterar" && o.tipo === op.tipo && o.id === op.id && campo in o.campos));
    let efetiva = op;
    if (op.op === "alterar") {
      const campos = Object.fromEntries(Object.entries(op.campos || {}).filter(([k]) => !pisado(k)));
      if (!Object.keys(campos).length) continue;
      efetiva = { ...op, campos };
      const editei = Object.keys(campos).filter((k) => aoVivoMinhas.some((m) => m.ops.some((o) => o.op === "alterar" && o.tipo === op.tipo && o.id === op.id && k in o.campos)));
      if (editei.length && lote.autor?.id !== aoVivoEu()) sincAviso(tr("{quem} mudou {campo} que você editou.", { quem, campo: sincCampo(editei[0]) }));
    }
    const r = aplicarRemota(aoVivoBase, arquivo, efetiva);
    if (r.conflitos.length) sincAviso(tr("{quem} também mudou {campo}; o seu fica quando salvar.", { quem, campo: sincCampo(r.conflitos[0]) }));
    if (r.perdeu) sincAviso(tr("{quem} apagou um item que você estava editando.", { quem }));
    mudou = true;
  }
  if (mudou) aoVivoRedesenhar();
}

/** Redesenha uma vez por quadro; quem está digitando não tem a tela trocada (espera o foco sair). */
function aoVivoRedesenhar() {
  if (digitando()) { aoVivoAdiado = true; return; }
  if (aoVivoQuadro) return;
  aoVivoQuadro = requestAnimationFrame(() => {
    aoVivoQuadro = 0;
    workspaceAtual = workspaceDaRota(arquivo, location.hash);
    D = workspaceAtual?.dados || arquivo;
    concluidosAntes = concluidosAgora();
    render();
  });
}
document.addEventListener("focusout", () => {
  if (aoVivoAdiado) setTimeout(() => { if (aoVivoAdiado && !digitando()) { aoVivoAdiado = false; aoVivoRedesenhar(); } }, 200);
});

/** Buraco no que chegou (ou um PUT de script): traz tudo, junta com o que está na tela e manda depois o que faltava. */
let aoVivoRecarregando = false;
async function aoVivoRecarregar() {
  if (aoVivoRecarregando) return;
  aoVivoRecarregando = true;
  try {
    const r = await fetch("/api/dados", { cache: "no-store" });
    if (r.status === 401) return typeof semSessao === "function" && semSessao();
    if (!r.ok) throw new Error(r.status);
    const texto = await r.text();
    const deles = instantaneoDe(preparaArquivo(JSON.parse(texto)));
    const minha = instantaneoDe(arquivo), base = aoVivoBase || deles;
    arquivo = preparaArquivo(sincMesclar(base, minha, deles));
    ultimo = texto;
    versaoDados = r.headers.get("x-dados-versao") || versaoDados;
    aoVivoBase = deles; // a base é o servidor; o que só a tela tem vira operação no próximo salvar
    aoVivoVersaoGet = Number(versaoDados) || 0;
    aoVivoVisto.clear();
    aoVivoMinhas = [];
    aoVivoConectar(true);
    aoVivoRedesenhar();
    if (opsEntre(aoVivoBase, instantaneoDe(arquivo)).length) salvar();
  } catch {} finally { aoVivoRecarregando = false; }
}

/** Depois de gravar: a base ganha as operações que o servidor aceitou e a página guarda a versão para ordenar os lotes que chegarem. */
function aoVivoGravou(versao, ops, resposta) {
  const fora = new Set((resposta.ignoradas || []).map((x) => x.indice));
  ops.forEach((op, i) => { if (!fora.has(i)) aplicarOp(aoVivoBase, op); });
  aoVivoMinhas.push({ versao: Number(versao), ops: ops.filter((_, i) => !fora.has(i)), em: Date.now() });
  if (aoVivoMinhas.length > 200) aoVivoMinhas.shift();
}
