// ---- tela de edição (só manual) ------------------------------------
const ed = $("editor");
let edProj = null, edSel = "blueprint";
function contextoChat(id) {
  return `${workspaceAtual?.id || "sm"}:${id}`;
}
// guarda a proposta pendente de cada blueprint (e a do blueprint novo) junto com os dados
function conv(id) {
  const contexto = contextoChat(id);
  arquivo.chats ||= [];
  arquivo.chatsAtivos ||= {};
  let c = arquivo.chats.find((c) => c.contexto === contexto && c.id === arquivo.chatsAtivos[contexto]) || arquivo.chats.filter((c) => c.contexto === contexto).at(-1);
  if (!c) {
    c = { id: novoId(), contexto, criadoEm: new Date().toISOString(), msgs: [], proposta: null, emblema: null, dadosBlueprint: null, lote: null, pensando: false, intencao: null, acaoAlvo: "", consulta: [] };
    arquivo.chats.push(c);
  }
  arquivo.chatsAtivos[contexto] = c.id;
  return c;
}
const NOVO = "__novo"; // o blueprint que ainda está sendo criado na tela "Novo blueprint"
let rascunho = { id: NOVO, nome: "", descricao: "", obs: "", links: [], acoes: [], emblema: null };
const blueprintEd = () => (edProj === NOVO ? rascunho : D.blueprints.find((p) => p.id === edProj) || null);
const acaoEd = (p) => p.acoes.find((f) => f.id === edSel);
const SIT = [["planejado", tr("Planejada")], ["andamento", tr("Em desenvolvimento")], ["concluido", tr("Concluída")]];
const progEditavel = (f) => f.status === "andamento" && !efetiva(f).it[1];
const ultimoDe = (el, sel) => [...el.querySelectorAll(sel)].pop();
const durTxt = (f) => (dia(f.inicio) === dia(f.inicio) && dia(f.fim) === dia(f.fim) && dia(f.fim) >= dia(f.inicio) ? trn(dia(f.fim) - dia(f.inicio) + 1, ["{n} dia", "{n} dias"]) : "");

// tela cheia: as telas abrem em tela cheia (dá para desligar em Dados gerais)
const telaCheiaPref = () => { try { return localStorage.getItem("blueprint.telaCheia") !== "0"; } catch { return true; } };
// o sistema nunca entra em tela cheia sozinho: só sugere F11, uma vez por tipo de tela (sugereF11)
function pedirTelaCheia() {}
// no app (WebView2) a Fullscreen API só estica a página dentro da janela; a tela cheia do Windows vem do Rust via window.ipc
let telaCheiaApp = false;
const rotuloTela = (cheia) => { $("b-tela").textContent = $("ed-tela").textContent = $("ag-tela").textContent = $("workspace-tela").textContent = cheia ? tr("⤡ Sair da tela cheia") : tr("⛶ Tela cheia"); };
function telaCheiaNativa(cheia) { telaCheiaApp = cheia; rotuloTela(cheia); }
function alternarTelaCheia() {
  if (window.chrome?.webview) window.ipc?.postMessage("tela-cheia");
  else if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
  else document.documentElement.requestFullscreen?.().catch(() => {});
}
document.addEventListener("fullscreenchange", () => rotuloTela(!!document.fullscreenElement));
document.addEventListener("keydown", (e) => {
  if (!window.ipc || !(e.key === "F11" || (e.key === "Escape" && telaCheiaApp))) return;
  e.preventDefault();
  alternarTelaCheia();
});
$("b-tela").onclick = $("ed-tela").onclick = $("ag-tela").onclick = alternarTelaCheia;

// ---- abrir e fechar ----------------------------------------------------------------------------
function editar(abre, op = {}) {
  if (abre === !ed.hidden) { if (abre) montaEditor(); return; }
  if (abre) {
    if (op.novo) {
      rascunho = { id: NOVO, nome: "", descricao: "", obs: "", links: [], acoes: [], emblema: null };
      edProj = NOVO;
      
      $("nv-nome").value = "";
      $("nv-desc").value = "";
    } else edProj = op.proj ?? tela ?? D.blueprints.find((x) => x.id === edProj)?.id ?? D.blueprints[0]?.id ?? null;
    edSel = op.sel || "blueprint";
    ed.hidden = false;
    document.body.classList.add("editando");
    document.documentElement.classList.add("editando");
    $("folha").inert = true;
    dizer("pronto");
    montaEditor(); // já montada: a animação só abre a folha, o conteúdo está lá
    sugereF11(op.novo ? "novo" : "editar");
    desenrolar(ed, ed);
    (op.novo ? $("nv-nome") : $("mn-detalhe").querySelector("input,textarea"))?.focus({ preventScroll: true });
  } else {
    ed.hidden = true;
    if (edProj === NOVO) edProj = null;
    ed.getAnimations().forEach((a) => a.cancel());
    document.body.classList.remove("editando");
    document.documentElement.classList.remove("editando");
    $("folha").inert = false;
    carregar();
    // a tela de baixo continua desenhada: fechar o editor não a anima de novo
  }
}
$("ed-fechar").onclick = () => editar(false);

