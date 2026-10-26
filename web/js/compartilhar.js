// ---- Compartilhar um workspace pelo próprio app (só no app local; ação "Compartilhar um workspace pelo próprio app") ---------------
// O app aberto atende os convidados numa porta própria, por um túnel HTTPS da Cloudflare, e essa porta só mostra o workspace compartilhado.
// API (só na porta local): GET /api/compartilhar -> {estado, url?, workspaceId?, erro?, porta?}; POST /api/compartilhar/ligar {workspaceId};
// POST /api/compartilhar/desligar; o SSE do app avisa "compartilhar" e sessao.js repassa para cá (compartilharMudou). Com o túnel ligado, a
// lista de pessoas e os links de convite são os de pessoas.js (o usuário local é o dono), e os links saem no endereço do túnel.
// Módulo ES carregado por import() quando o diálogo abre (sessao.js); usa o <dialog id="equipe"> dos painéis de equipe.
import { montarPessoas } from "./pessoas.js";

export const ESTADOS = ["desligado", "baixando", "ligando", "ligado", "erro"];
// a resposta do servidor (GET, POST ou o evento) no formato que a tela usa; estado desconhecido vira erro
export function normalizar(j) {
  const estado = ESTADOS.includes(j?.estado) ? j.estado : "erro";
  const workspaces = Array.isArray(j?.workspaces) ? j.workspaces.map(String) : j?.workspaceId ? [String(j.workspaceId)] : [];
  return { estado, url: String(j?.url || ""), workspaces, erro: String(j?.erro || "") };
}
export const corpoDoLigar = (workspaceId) => (workspaceId ? { workspaceId } : { erro: "falta-workspace" });
// o que a tela mostra, sempre sobre o workspace aberto: título, texto (lido pelo leitor de tela), o botão (ligar, desligar, cancelar ou
// nenhum), a barra de progresso e o endereço. Vários workspaces dividem o mesmo túnel; cada convidado só vê o workspace do convite.
export function vistaDoEstado(s, wsAberto, nomeDe = (id) => id) {
  const nome = nomeDe(wsAberto), este = s.workspaces.includes(wsAberto);
  switch (s.estado) {
    case "baixando": return { titulo: tr("Baixando o túnel…"), texto: tr("Só na primeira vez: o app baixa o cloudflared, o programa da Cloudflare que cria o endereço público."), botao: "cancelar", ocupado: true, url: "" };
    case "ligando": return { titulo: tr("Ligando o túnel…"), texto: tr("Pedindo um endereço público à Cloudflare; leva alguns segundos."), botao: "cancelar", ocupado: true, url: "" };
    case "ligado": return este
      ? { titulo: tr("Compartilhando «{nome}»", { nome }), texto: tr("Quem tiver um link de convite entra por este endereço e só vê este workspace."), botao: "desligar", ocupado: false, url: s.url }
      : { titulo: tr("«{nome}» não está compartilhado", { nome }), texto: tr("O túnel já está ligado para outro workspace. Ligue para incluir «{nome}» no mesmo endereço: cada convidado só vê o workspace do próprio convite.", { nome }), botao: "ligar", ocupado: false, url: "" };
    case "erro": return { titulo: tr("Não foi possível compartilhar"), texto: s.erro || tr("O túnel não ligou. Tente de novo."), botao: "ligar", ocupado: false, url: "" };
    default: return { titulo: tr("Compartilhamento desligado"), texto: tr("Ligue para criar um endereço público para «{nome}»: só este workspace fica visível para quem você convidar.", { nome }), botao: "ligar", ocupado: false, url: "" };
  }
}
const ROTULO = { ligar: () => tr("Ligar"), desligar: () => tr("Desligar"), cancelar: () => tr("Cancelar") };
const nomeDoWorkspace = (id) => workspacesDe(arquivo).find((w) => w.id === id)?.nome || id;

let estado = normalizar({ estado: "desligado" }), wsAberto = "", relogio = 0, equipeDe = "";
export async function abrirCompartilhar() {
  wsAberto = workspaceAtual?.id || "sm";
  equipeDe = "";
  $("equipe-titulo").textContent = tr("Compartilhar");
  $("equipe-corpo").innerHTML = `<section class="eq-secao cp" aria-labelledby="cp-titulo">
    <div role="status" aria-live="polite" aria-atomic="true"><h4 id="cp-titulo"></h4><p id="cp-texto"></p></div>
    <progress class="cp-progresso" id="cp-progresso" aria-label="${esc(tr("Ligando o compartilhamento"))}" hidden></progress>
    <div class="eq-link" id="cp-endereco" hidden></div>
    <p class="eq-dica">${tr("O app precisa ficar aberto enquanto as pessoas usam. O endereço muda a cada vez que você liga: depois de religar, mande os links de novo.")}</p>
    <div class="eq-acoes"><button class="btn cheio" type="button" id="cp-botao"></button></div>
  </section><div id="cp-equipe"></div>`;
  $("cp-botao").onclick = alternar;
  if (!$("equipe").open) $("equipe").showModal();
  $("equipe").addEventListener("close", () => { clearTimeout(relogio); equipeDe = ""; }, { once: true });
  await atualizar();
  if (!$("cp-botao")?.hidden) $("cp-botao").focus(); // o foco vai para a ação principal (Esc fecha: é um <dialog> modal)
}
// o SSE avisou: só redesenha com o diálogo do Compartilhar aberto
export function compartilharMudou(j) {
  if ($("equipe").open && $("cp-botao")) j && j.estado ? mostrar(normalizar(j)) : atualizar();
}
async function atualizar() {
  try { mostrar(normalizar(await equipeApi("/api/compartilhar"))); }
  catch (e) { mostrar({ ...estado, estado: "erro", erro: e.status === 404 ? tr("Esta versão do app não compartilha workspaces.") : e.message }); }
}
function mostrar(s) {
  if (!$("cp-botao")) return; // o diálogo já mostra outro painel
  estado = s;
  const v = vistaDoEstado(s, wsAberto, nomeDoWorkspace), b = $("cp-botao");
  $("cp-titulo").textContent = v.titulo;
  $("cp-texto").textContent = v.texto;
  $("cp-progresso").hidden = !v.ocupado;
  b.hidden = !v.botao;
  b.dataset.acao = v.botao;
  b.textContent = v.botao ? ROTULO[v.botao]() : "";
  b.className = v.botao === "ligar" ? "btn cheio" : "btn perigo";
  const caixa = $("cp-endereco");
  caixa.hidden = !v.url;
  if (v.url && caixa.dataset.url !== v.url) {
    caixa.dataset.url = v.url;
    caixa.innerHTML = `<code id="cp-url">${esc(v.url)}</code><button class="btn mini" type="button" id="cp-copiar">${tr("Copiar")}</button><small>${tr("Este é o endereço público. Quem não tem link de convite não entra.")}</small>`;
    $("cp-copiar").onclick = async () => {
      try { await navigator.clipboard.writeText(v.url); $("cp-copiar").textContent = tr("Copiado"); }
      catch { getSelection().selectAllChildren($("cp-url")); }
    };
  }
  // shortcut: enquanto baixa ou liga, confere de novo a cada 1,5 s (só com o diálogo aberto) caso o aviso do SSE não chegue; o SSE basta quando o servidor avisar cada passo
  clearTimeout(relogio);
  if (v.ocupado) relogio = setTimeout(atualizar, 1500);
  const ws = s.estado === "ligado" && s.workspaces.includes(wsAberto) ? wsAberto : "";
  if (ws !== equipeDe) {
    equipeDe = ws;
    if (ws) montarPessoas($("cp-equipe"), ws, true);
    else $("cp-equipe").textContent = "";
  }
}
async function alternar() {
  const b = $("cp-botao"), acao = b.dataset.acao;
  if (acao === "desligar" && !(await confirmar({ titulo: tr("Parar de compartilhar este workspace?"), texto: tr("Quem usa este workspace perde o acesso na hora. Sem nenhum workspace compartilhado, o túnel desliga e o endereço muda ao ligar de novo."), rotulo: tr("Desligar") }))) return;
  b.disabled = true;
  try {
    // cancelar (baixando ou ligando) desliga tudo; desligar tira só o workspace aberto
    const j = acao === "ligar" ? await equipeApi("/api/compartilhar/ligar", corpoDoLigar(wsAberto)) : await equipeApi("/api/compartilhar/desligar", acao === "desligar" ? { workspaceId: wsAberto } : {});
    j && j.estado ? mostrar(normalizar(j)) : await atualizar();
  } catch (e) { mostrar({ ...estado, estado: "erro", erro: e.message }); }
  finally { b.disabled = false; }
}
