// ---- apresentação com IA: "Conversar com o agente" (o painel ao lado do visor) ----
// O usuário pede um ajuste em texto; o app o entrega a uma sessão do Claude (pelo canal dela, POST /api/mcp/apresentacao/pedido) ou abre uma aba do terminal do app
// com o pedido (POST /api/terminal/iniciar com `ajuste`). O agente muda SÓ o que foi pedido, com as ferramentas de edição (guia: apresentacao_ajustar); não reescreve a apresentação.
// Só por clique, só loopback. A escolha da sessão vale até a tela fechar (só memória): sessão com canal já escolhida não pergunta de novo. Zero trabalho em repouso.
let apCv = null; // { raiz, bp, id, escolha, historico: [{texto, estado, nota}], ocupado }

function apCvDesenhaHistorico() {
  const el = apCv?.raiz.querySelector(".ap-cv-hist");
  if (!el) return;
  el.innerHTML = apCv.historico.map((h) => `<li data-estado="${h.estado}"><b>${h.sistema ? tr("App") : tr("Você")}</b> ${esc(h.texto)}${h.nota ? `<small>${esc(h.nota)}</small>` : ""}</li>`).join("");
  el.scrollTop = el.scrollHeight;
}
function apCvAbrir(raiz, { bp, id }) {
  raiz.classList.add("ap-cv");
  apCv = { raiz, bp, id, escolha: apCv?.bp === bp ? apCv.escolha : null, historico: apCv?.id === id ? apCv.historico : [], ocupado: false };
  raiz.innerHTML = `<p class="ap-dica">${tr("Diga o que mudar. O agente muda só o que você pedir, direto na apresentação, sem refazer o resto. Também dá para digitar o pedido no terminal onde ele está.")}</p>
<ol class="ap-cv-hist" aria-live="polite" aria-label="${esc(tr("Pedidos desta tela"))}"></ol>
<label class="ap-ed-c"><span>${tr("Seu pedido")}</span><textarea class="ap-cv-texto" rows="4" maxlength="2000" placeholder="${esc(tr("Ex.: deixe o texto da etapa 3 mais curto e troque a data de fim para 12 de novembro"))}"></textarea></label>
<button class="btn cheio" type="button" data-cv="enviar">${tr("Enviar ao agente")}</button><p class="ap-ia-msg" role="status"></p>`;
  raiz.onclick = (e) => { if (e.target.closest('[data-cv="enviar"]')) apCvEnviar(); };
  raiz.onkeydown = (e) => { if (e.key === "Enter" && (e.ctrlKey || e.metaKey) && e.target.matches(".ap-cv-texto")) { e.preventDefault(); apCvEnviar(); } };
  apCvDesenhaHistorico();
}
function apCvFechar() { if (apCv) { apCv.raiz.onclick = apCv.raiz.onkeydown = null; apCv.raiz = { querySelector: () => null }; } }
function apCvMsg(texto, erro) {
  const el = apCv?.raiz.querySelector(".ap-ia-msg");
  if (el) { el.textContent = texto || ""; el.classList.toggle("erro", !!erro); }
}
// o agente (ou outra janela) mexeu na apresentação: uma linha no histórico para o usuário saber por que o visor recarregou
function apCvAtualizou() {
  if (!apCv) return;
  apCv.historico.push({ texto: tr("O agente atualizou a apresentação."), estado: "ok", sistema: true });
  apCvDesenhaHistorico();
}

// a sessão que vale para este pedido: a lembrada (se ainda está aberta e recebe pelo canal) ou o que o usuário escolher no diálogo
async function apCvEscolha(texto) {
  const sessoes = await sessoesAbertas(apCv.bp);
  const lembrada = apCv.escolha?.tipo === "sessao" ? sessoes.find((s) => s.id === apCv.escolha.id && s.canal) : null;
  if (lembrada) return { sessao: lembrada };
  const primeira = sessoes.findIndex((s) => s.canal);
  const escolha = await escolherSessao({ titulo: tr("Enviar o pedido ao agente?"), texto: `${tr("Gasta tokens.")} ${apIaTextoAjuste(texto).slice(0, 160)}`, sessoes, padrao: primeira >= 0 ? primeira : "app", rotulo: tr("Enviar") });
  if (escolha) apCv.escolha = escolha.sessao ? { tipo: "sessao", id: escolha.sessao.id } : { tipo: escolha.app ? "app" : "nova" };
  return escolha;
}
async function apCvEnviar() {
  if (!apCv || apCv.ocupado) return;
  const campo = apCv.raiz.querySelector(".ap-cv-texto"), texto = apIaTextoAjuste(campo.value);
  if (!texto) return apCvMsg(tr("Escreva o que você quer mudar."), true);
  if (typeof repositorioConvidadoNoApp === "function" && repositorioConvidadoNoApp()) return apCvMsg(tr("Conversar com o agente só funciona no app local, não na janela da equipe."), true);
  if (typeof poderesDaSessao === "function" && poderesDaSessao(sessao).soLeitura) return apCvMsg(tr("O seu papel só lê este workspace: não dá para pedir mudanças daqui."), true);
  apCv.ocupado = true;
  const entrada = { texto, estado: "enviando", nota: "" };
  try {
    const escolha = await apCvEscolha(texto);
    if (!escolha) return;
    const ws = workspaceAtual?.id || "sm", ids = { workspaceId: ws, blueprintId: apCv.bp.id, apresentacaoId: apCv.id };
    apCv.historico.push(entrada);
    apCvDesenhaHistorico();
    campo.value = "";
    let r, j, como;
    if (escolha.app) { // uma aba do terminal do app com o pedido (a mesma pasta, o Claude com o canal ligado)
      como = "app";
      r = await fetch("/api/terminal/iniciar", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(apIaCorpoTerminalAjuste({ ...ids, texto, titulo: tr("Ajuste: {titulo}", { titulo: apCv.bp.nome || "" }) })) });
      j = await r.json().catch(() => ({}));
      if (r.ok && j.sessao) terminalAbrirSessao(j.sessao);
    } else {
      como = escolha.sessao ? "sessao" : "externo";
      r = await fetch("/api/mcp/apresentacao/pedido", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(apIaCorpoPedidoAjuste({ ...ids, texto, sessao: escolha.sessao })) });
      j = await r.json().catch(() => ({}));
      if (escolha.sessao && !escolha.sessao.canal && r.ok && j.prompt) { // sem canal não há entrega: o pedido vai para a área de transferência
        try { await navigator.clipboard.writeText(j.prompt); entrada.estado = "ok"; entrada.nota = tr("Pedido copiado: cole na sessão."); }
        catch { entrada.estado = "erro"; entrada.nota = tr("Não consegui copiar sozinho. O pedido é: {prompt}", { prompt: j.prompt }); }
        return;
      }
    }
    entrada.estado = r.ok ? "ok" : "erro";
    entrada.nota = apIaMensagemAjuste(r.status, j, como);
    apCvMsg(entrada.nota, !r.ok);
  } catch { entrada.estado = "erro"; entrada.nota = tr("Não consegui falar com o app. Ele está aberto?"); apCvMsg(entrada.nota, true); }
  finally { apCv.ocupado = false; apCvDesenhaHistorico(); }
}
