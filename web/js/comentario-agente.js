// ---- "Enviar comentário ao agente" (botão direito num plano): o usuário escreve o comentário, escolhe a sessão do Claude e clica em Enviar (é a confirmação).
// Vai pelo canal da sessão como pedido do usuário (POST /api/mcp/conversa, tipo "comentario"); sessão sem canal ou "Só copiar" copia o texto. O comentário
// não é salvo no plano. Contrato em docs/claude-na-folha.md ("Enviar comentário do plano e conversar com o plano"). Quem chama: contexto-itens.js.
async function enviarComentarioAoAgente(pid, fid, tid) {
  const p = D.blueprints.find((x) => x.id === pid), f = p?.acoes.find((x) => x.id === fid), t = f?.planos.find((x) => x.id === tid);
  if (!t) return;
  const sessoes = await sessoesAbertas(p);
  const d = document.createElement("dialog");
  d.className = "confirmar escolher-sessao comentario-agente";
  d.setAttribute("aria-label", tr("Enviar comentário ao agente"));
  const padrao = sessoes.findIndex((s) => s.canal);
  const linhas = sessoes.map((s, i) => `<label><input type="radio" name="sessao" value="${i}"${i === padrao ? " checked" : ""}> <span>${tr("Sessão em {nome}", { nome: esc(conversaNomeSessao(s)) })}</span> <small>${s.canal ? tr("recebe o pedido direto") : tr("sem canal: só copia o pedido")}</small></label>`).join("");
  d.innerHTML = `<div class="cu-topo"><h3>${esc(tr("Enviar comentário ao agente"))}</h3></div>
    <p class="cu-corpo">${esc(tr("Plano «{plano}» da ação {numero}. O Claude recebe o comentário como um pedido seu.", { plano: t.texto || tr("sem nome"), numero: numeroDaAcao(p, f) }))}</p>
    <div class="cu-opcoes"><textarea data-ca-texto maxlength="800" rows="4" aria-label="${esc(tr("Comentário"))}" placeholder="${esc(tr("O que o Claude deve saber ou mudar neste plano…"))}"></textarea>
      ${linhas}<label><input type="radio" name="sessao" value="copiar"${padrao < 0 ? " checked" : ""}> <span>${tr("Só copiar o texto")}</span> <small>${tr("cole você mesmo na sessão")}</small></label>
      <p class="cv-aviso" role="status"></p></div>
    <div class="confirmar-acoes"><button class="btn" type="button" data-r="0">${tr("Cancelar")}</button><button class="btn perigo" type="button" data-r="1">${tr("Enviar")}</button></div>`;
  const avisa = (texto, erro) => { const el = d.querySelector(".cv-aviso"); el.textContent = texto; el.classList.toggle("erro", !!erro); };
  d.addEventListener("click", (e) => { if (e.target === d) d.close(); });
  d.querySelector('[data-r="0"]').addEventListener("click", () => d.close());
  const enviar = d.querySelector('[data-r="1"]');
  enviar.addEventListener("click", async () => {
    const texto = d.querySelector("[data-ca-texto]").value.trim(), v = d.querySelector('input[name="sessao"]:checked')?.value;
    if (!texto) return avisa(tr("Escreva o comentário."), true);
    enviar.disabled = true;
    avisa(tr("Enviando…"));
    const r = await conversaPost(p, f, { tipo: "comentario", planoId: t.id, texto }, v === "copiar" || v == null ? null : sessoes[+v]);
    avisa(r.texto, !r.ok);
    if (!r.ok) { enviar.disabled = false; return; }
    // o resultado fica à vista no próprio diálogo (a folha não tem outro aviso visível); só resta fechar
    enviar.hidden = true;
    d.querySelector('[data-r="0"]').textContent = tr("Fechar");
  });
  d.addEventListener("close", () => d.remove());
  document.body.append(d);
  d.showModal();
  d.querySelector("textarea").focus();
}
