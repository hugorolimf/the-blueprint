// ---- apresentação com IA, editor manual: o painel ao lado do visor (eventos, estado e conversa com o servidor) ----
// O HTML vem de apresentacao-ia-editor-html.js, as operações de apresentacao-ia-ops.js e o salvamento de apresentacao-ia-salvar.js. Cada mudança vira UMA operação (nunca o
// documento inteiro: o agente pode estar mudando outro ponto) aplicada à cópia local e mandada em lote para POST /api/mcp/apresentacao/editar. Zero trabalho em repouso.
let apEd = null; // { raiz, ids, doc, salvador, aviso, aoSalvo, aoOutro }

const apEdNet = async (url, corpo) => {
  const r = await fetch(url, corpo ? { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) } : { cache: "no-store" });
  return { status: r.status, json: await r.json().catch(() => ({})) };
};
const apEdFraseEstado = (e) => ({ salvo: tr("Salvo"), pendente: tr("Alterações pendentes…"), salvando: tr("Salvando…"), erro: tr("Não salvo: erro de conexão. A próxima mudança tenta de novo.") })[e] || "";
function apEdMostraEstado(e) {
  const el = apEd?.raiz.querySelector(".ap-ed-estado");
  if (el) { el.textContent = apEdFraseEstado(e); el.dataset.estado = e; }
  const d = apEd?.raiz.querySelector('[data-ed="desfaz"]');
  if (d) d.disabled = !apEd.salvador.pode();
}
function apEdMostraAvisos(lista, erro = "") {
  const el = apEd?.raiz.querySelector(".ap-ed-avisos");
  if (!el) return;
  const itens = [...(erro ? [`<li class="erro">${esc(erro)}</li>`] : []), ...lista.map((a) => `<li>${esc(a)}</li>`)];
  el.innerHTML = itens.length ? `<p class="ap-ed-sub">${erro ? tr("O servidor recusou a mudança") : tr("Avisos do servidor")}</p><ul>${itens.join("")}</ul>` : "";
}

// onde o usuário está digitando, para devolver o foco (e o cursor) depois de redesenhar
function apEdFoco() {
  const a = document.activeElement;
  if (!a || !apEd.raiz.contains(a) || !a.matches("input,textarea,select,button")) return null;
  return { sel: a.dataset.c ? `[data-c="${CSS.escape(a.dataset.c)}"]` : a.dataset.fig && a.dataset.campo ? `[data-fig="${CSS.escape(a.dataset.fig)}"][data-campo="${a.dataset.campo}"]${a.dataset.j ? `[data-j="${a.dataset.j}"][data-k="${a.dataset.k}"]` : ""}` : "", ini: a.selectionStart, fim: a.selectionEnd };
}
function apEdDesenha(foco = apEdFoco()) {
  const corpo = apEd.raiz.querySelector(".ap-ed-secoes"), topo = corpo.scrollTop;
  const abertas = new Set([...corpo.querySelectorAll("details.ap-ed-sec[open]")].map((d) => d.dataset.sec));
  const abertasAv = new Set([...corpo.querySelectorAll("details.ap-ed-avancado[open]")].map((d) => d.closest("[data-figura]")?.dataset.figura));
  corpo.innerHTML = apEdHtml(apEd.doc, abertas.size || corpo.childElementCount ? abertas : null);
  for (const d of corpo.querySelectorAll("details.ap-ed-avancado")) if (abertasAv.has(d.closest("[data-figura]")?.dataset.figura)) d.open = true;
  corpo.scrollTop = topo;
  if (foco?.sel) { const el = corpo.querySelector(foco.sel); if (el) { el.focus({ preventScroll: true }); try { if (foco.ini != null) el.setSelectionRange(foco.ini, foco.fim); } catch {} } }
}

// aplica à cópia local e entrega ao salvador; uma operação que não cabe no documento (a lista mudou) não vai ao servidor
function apEdMuda(ops, { caminho = "", junta = false, redesenha = false, fragmento = "" } = {}) {
  const r = apOpsAplicaLista(apEd.doc, ops);
  if (r.falhas.length) { apEdMostraAvisos([], tr("Essa mudança já não cabe no documento. Recarregando o que está salvo.")); return apEdRecarrega(true); }
  apEd.doc = r.doc;
  apEd.fragmento = fragmento || apOpsFragmento(caminho || ops[0]?.caminho || "");
  apEd.salvador.mudar(ops, r.inversos, { caminho, junta });
  if (redesenha) apEdDesenha();
  apEdMostraEstado(apEd.salvador.estado());
}

// ---- eventos ----
const apEdValor = (tipo, chave, bruto) => (bruto === "" && ["situacao", "figura", "ate", "secao"].includes(chave) ? null : bruto);
function apEdCampoMudou(el, final) {
  if (el.matches("[data-c]")) {
    const caminho = el.dataset.c, chave = caminho.split(".").pop();
    if (el.required && !el.value.trim()) { el.setAttribute("aria-invalid", "true"); return; } // o título não pode ficar vazio: não vai ao servidor
    el.removeAttribute("aria-invalid");
    return apEdMuda([{ op: "definir", caminho, valor: apEdValor(el.dataset.tipo, chave, el.value) }], { caminho, junta: !final, redesenha: caminho === "publico" });
  }
  const id = el.dataset.fig, campo = el.dataset.campo, f = apEd.doc.figuras.find((x) => x.id === id);
  if (!f) return;
  if (campo === "codigo") return apEdCodigo(el, f, final);
  if (campo === "chaves") { // a lista de chaves vai inteira (é pequena)
    const novo = (f.chaves || []).map((k) => ({ ...k })), j = Number(el.dataset.j);
    if (!novo[j]) return;
    novo[j][el.dataset.k] = el.value;
    return apEdMuda([{ op: "figura_definir", figura: id, campo, valor: novo }], { caminho: `figuras.${id}.chaves.${j}`, junta: !final });
  }
  apEdMuda([{ op: "figura_definir", figura: id, campo, valor: apEdValor(el.dataset.tipo, campo, el.value) }], { caminho: `figuras.${id}.${campo}`, junta: !final, fragmento: campo === "secao" ? apOpsFragmentoSecao(el.value) : "" });
}

// o código da figura: a sintaxe se confere SÓ por new Function (nunca se executa); o texto vai como troca do trecho inteiro (`figura_trecho`)
function apEdSintaxe(codigo) {
  try { new Function(`return (${codigo})`); return ""; } // shortcut: só confere a gramática, não o que a figura faz; a biblioteca mostra o erro de execução no cartão
  catch (e) { return String(e.message || e); }
}
function apEdCodigo(el, f, final) {
  const msg = el.parentElement.querySelector(".ap-ed-sintaxe"), erro = apEdSintaxe(el.value);
  msg.textContent = erro ? tr("Erro de sintaxe: {erro}. A figura vai mostrar o cartão de erro.", { erro }) : tr("Sintaxe válida.");
  msg.classList.toggle("erro", !!erro);
  if (!final || el.value === f.codigo) return;
  apEdMuda([{ op: "figura_trecho", figura: f.id, de: f.codigo, para: el.value }], { caminho: `figuras.${f.id}.codigo` });
}

function apEdBotaoClicado(b) {
  const ed = b.dataset.ed, caminho = b.dataset.caminho || "", id = b.dataset.fig;
  const [pai, ultimo] = [caminho.split(".").slice(0, -1).join("."), Number(caminho.split(".").pop())];
  if (ed === "desfaz") { const inv = apEd.salvador.desfazer(); if (!inv) return; apEd.doc = apOpsAplicaLista(apEd.doc, inv).doc; apEdDesenha(null); apEdMostraEstado("pendente"); apEd.fragmento = ""; return; }
  if (ed === "adiciona") { // o item novo vem do esquema; as listas de dentro do item seguem o nome da última parte do caminho
    const chave = caminho.split(".").pop(), novo = (AP_IA_NOVO[chave] || (() => ""))(apEd.doc);
    return apEdMuda([{ op: "inserir", caminho, valor: novo }], { caminho, redesenha: true });
  }
  if (ed === "remove") return apEdMuda([{ op: "remover", caminho }], { caminho, redesenha: true });
  if (ed === "sobe" || ed === "desce") return apEdMuda([{ op: "mover", caminho, para: ultimo + (ed === "sobe" ? -1 : 1) }], { caminho: pai, redesenha: true });
  if (ed === "figura-remove") return apEdMuda([{ op: "figura_remover", figura: id }], { caminho: "figuras", redesenha: true });
  if (ed === "figura-duplica") {
    const f = apEd.doc.figuras.find((x) => x.id === id);
    if (!f) return;
    let n = 2; while (apEd.doc.figuras.some((x) => x.id === `${id}-${n}`)) n++;
    return apEdMuda([{ op: "__figuras", valor: [...apEd.doc.figuras, { ...apOpsClone(f), id: `${id}-${n}`, titulo: `${f.titulo || id} (${tr("cópia")})`, secao: null }] }], { caminho: "figuras", redesenha: true });
  }
  if (ed === "chave-adiciona" || ed === "chave-remove") {
    const f = apEd.doc.figuras.find((x) => x.id === id);
    if (!f) return;
    const chaves = (f.chaves || []).map((k) => ({ ...k }));
    if (ed === "chave-adiciona") chaves.push(AP_IA_NOVO.chaves()); else chaves.splice(Number(b.dataset.j), 1);
    return apEdMuda([{ op: "figura_definir", figura: id, campo: "chaves", valor: chaves }], { caminho: `figuras.${id}.chaves`, redesenha: true });
  }
}

// ---- carregar e recarregar do servidor ----
async function apEdRecarrega(descartaPendentes = false) {
  try {
    const r = await apEdNet(`/api/mcp/apresentacao?id=${encodeURIComponent(apEd.ids.apresentacaoId)}`);
    if (!apEd || r.status !== 200) return apEdMostraAvisos([], r.status === 404 ? apOpsFraseErro(404, { codigo: "apresentacao-nao-encontrada" }) : tr("Não consegui ler a apresentação (erro {status}).", { status: r.status }));
    if (descartaPendentes) apEd.salvador.descartar();
    // o que o usuário ainda não mandou vai por cima do que o servidor trouxe: nada do que ele digita se perde
    apEd.doc = apOpsReaplicar({ conteudo: r.json.conteudo || {}, figuras: r.json.figuras || [] }, apEd.salvador.pendentes()).doc;
    apEd.editadoEm = r.json.editadoEm || "";
    apEdDesenha();
    apEdMostraEstado(apEd.salvador.estado());
    return r.json;
  } catch { apEdMostraAvisos([], tr("Não consegui falar com o app. Ele está aberto?")); }
}
// o servidor avisou de uma mudança (SSE "apresentacao"): se não foi a nossa, é do agente (ou de outra janela)
async function apEdOutroAutor() {
  if (!apEd) return false;
  if (apEd.salvador.ocupado()) { apEd.recarregarDepois = true; return false; } // o que o usuário digitou sai primeiro; o aviso do agente é tratado logo depois
  const antes = apEd.salvador.ultimoEm(), j = await apEdRecarrega();
  if (!j) return false;
  const nosso = j.editadoEm && j.editadoEm === antes;
  if (!nosso) apEd.aoOutro?.();
  return !nosso;
}

function apEdAbrir(raiz, { ids, aoSalvo, aoOutro, aoFoco }) {
  apEdFechar();
  raiz.innerHTML = `<div class="ap-ed-topo"><b>${tr("Editar")}</b><span class="ap-ed-estado" role="status" aria-live="polite"></span><button class="btn mini" type="button" data-ed="desfaz" disabled>${tr("Desfazer")}</button></div>
<p class="ap-dica">${tr("As mudanças são salvas sozinhas. O agente pode mudar outros pontos ao mesmo tempo: nada do que você escreveu se perde.")}</p><div class="ap-ed-avisos" role="status"></div><div class="ap-ed-secoes"></div>`;
  const esperar = (ms, fn) => { const t = setTimeout(fn, ms); return () => clearTimeout(t); };
  apEd = { raiz, ids, doc: { conteudo: {}, figuras: [] }, fragmento: "", aoSalvo, aoOutro, aoFoco };
  apEd.salvador = apIaSalvador({
    ids, esperar, postar: (corpo) => apEdNet("/api/mcp/apresentacao/editar", corpo),
    aoEstado: (e) => { apEdMostraEstado(e); if (e === "salvo" && apEd) { apEd.aoSalvo?.(apEd.fragmento); if (apEd.recarregarDepois) { apEd.recarregarDepois = false; apEdOutroAutor(); } } },
    aoAvisos: (l) => apEdMostraAvisos(l),
    aoRecusa: (status, j) => { apEdMostraAvisos([], apOpsFraseErro(status, j)); apEdRecarrega(true); }, // volta ao último salvo
  });
  raiz.addEventListener("input", (e) => { if (e.target.matches("[data-c],[data-fig]") && !e.target.matches("select,[type=date]")) apEdCampoMudou(e.target, false); });
  raiz.addEventListener("change", (e) => { if (e.target.matches("[data-c],[data-fig]")) apEdCampoMudou(e.target, true); });
  raiz.addEventListener("click", (e) => { const b = e.target.closest("[data-ed]"); if (b) apEdBotaoClicado(b); });
  raiz.addEventListener("focusin", (e) => { // o visor rola até a seção do campo em foco, sem recarregar (postMessage "ap-ir" da página gerada)
    const el = e.target;
    if (!apEd || !el.matches?.("[data-c],[data-fig]")) return;
    apEd.aoFoco?.(apOpsDestino(el.dataset.c || `figuras.${el.dataset.fig}`, apEd.doc));
  });
  return apEdRecarrega();
}
async function apEdFechar() { // sai mandando o que faltava; o painel some, o estado vai embora
  if (!apEd) return;
  const s = apEd.salvador;
  apEd = null;
  await s.descarregar();
}
