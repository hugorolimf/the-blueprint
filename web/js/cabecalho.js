// ---- cabeçalho de uma seção da folha: círculo com o número, nome, descrição, progresso geral, linha e botão Recolher/Abrir, mais a nota e os links ----
// É o cabeçalho que o blueprint sempre teve; agora serve também a cada projeto (cada projeto é uma seção igual a um blueprint). Só desenha: o clique de
// recolher/abrir é do atributo `s.toggle` (data-toggle do blueprint ou data-projeto do projeto), tratado em navegacao.js e projetos.js.
// `s`: { p, y, numero, nome, descricao, total, concl, horas, ok, futuro, geral, aberto, toggle, obs?, links? }; `{ L, R }`: as margens da folha.
function cabecalhoSecao(out, s, { L, R }) {
  const yb = s.y;
  const nomeH = corta(s.nome || tr("Sem nome"), 48), descH = corta(s.descricao || "", 70);
  out.push(`<g data-proj="${esc(s.p.id)}">`);
  out.push(`<g ${s.toggle} class="alvo" tabindex="0" role="button" aria-expanded="${s.aberto}" aria-label="${esc(s.aberto ? tr("{nome}: recolher ações", { nome: s.nome }) : tr("{nome}: expandir ações", { nome: s.nome }))}"><rect class="cab" x="${L - 8}" y="${yb - 8}" width="${R - L + 16}" height="86"/>`);
  out.push(`<g><circle class="ln" cx="${L + 14}" cy="${yb + 20}" r="14"/><text x="${L + 14}" y="${yb + 25}" font-size="13" font-weight="700" text-anchor="middle">${s.numero}</text>`);
  out.push(`<text x="${L + 40}" y="${yb + 19}" font-size="17" font-weight="700" class="cap">${esc(nomeH)}</text><text class="mid cap" x="${L + 40}" y="${yb + 38}" font-size="11">${esc(descH)}</text></g>`);
  // medidor logo depois do nome, longe da linha de eixo "hoje"
  const px = Math.min(L + 40 + Math.max(nomeH.length * 11.6 + (s.obs?.trim() ? 30 : 0), descH.length * 7.2) + 64, R - 330);
  const contagem = `${tr("{concl}/{total} ações concluídas", { concl: s.concl, total: s.total })}${s.horas ? ` · ${tr("{horas} h de desenvolvimento", { horas: s.horas })}` : ""}`;
  out.push(`<g>` + (!s.total
    ? `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">${tr("Situação")}</text><text x="${px}" y="${yb + 32}" font-size="12" font-weight="700">${tr("SEM AÇÕES AINDA")}</text>`
    : s.ok
      ? `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">${tr("Situação")} · ${contagem}</text><text x="${px}" y="${yb + 32}" font-size="12" font-weight="700">${tr("✓ CONCLUÍDO · 100%")}</text>`
      : s.futuro
        ? `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">${tr("Situação")}</text><text x="${px}" y="${yb + 32}" font-size="12" font-weight="700">${tr("PLANEJADO")}</text>`
        : `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">${tr("Progresso geral")} · ${contagem}</text><rect class="ln" x="${px}" y="${yb + 21}" width="190" height="10"/><rect class="prog" x="${px}" y="${yb + 21}" width="${190 * s.geral}" height="10"/><text x="${px + 200}" y="${yb + 31}" font-size="12" font-weight="700">${Math.round(s.geral * 100)}%</text>`) + `</g>`);
  out.push(`<path class="ln m" pathLength="1" d="M${L} ${yb + 58}H${s.total ? R - 144 : R}"/>`);
  // botão grande com texto: some a dúvida do que fazer e dá um alvo fácil de acertar
  if (s.total) out.push(`<rect class="botao" x="${R - 136}" y="${yb + 44}" width="132" height="30"/><text class="botao-t cap" x="${R - 70}" y="${yb + 63}" font-size="11" text-anchor="middle">${s.aberto ? tr("− Recolher") : `+ ${trn(s.total, ["Abrir {n} ação", "Abrir {n} ações"])}`}</text>`);
  out.push(`</g>`);
  // nota do blueprint: ao lado do nome, abre as observações na gaveta lateral
  if (s.obs?.trim()) out.push(notaSvg(L + 40 + nomeH.length * 11.6 + 10, yb + 3, tr("OBSERVAÇÃO DO BLUEPRINT: {texto}", { texto: umaLinha(s.obs, 200) }), `${s.p.id}|`));
  // links do blueprint: em até duas linhas, alinhados à direita, na ordem cadastrada
  const lks = (s.links || []).filter((l) => urlSegura(l.url));
  const vaga = R - 6 - (px + 280);
  const grupos = [[], []];
  let livre = 0, usado = 0, g = 0;
  lks.forEach((l) => {
    const txt = "↗ " + corta(rotuloDe(l), 18), w = txt.length * CHAR + 12;
    if (usado && usado + 8 + w > vaga) { g++; usado = 0; }
    if (g > 1 || w > vaga) { livre++; return; }
    grupos[g].push({ l, txt, w });
    usado += (usado ? 8 : 0) + w;
  });
  const chips = [];
  grupos.forEach((linha, i) => {
    let x = R - 6 - (linha.reduce((t, c) => t + c.w, 0) + 8 * Math.max(0, linha.length - 1));
    linha.forEach(({ l, txt, w }) => {
      const ry = yb + 2 + i * 20;
      chips.push(`<a class="lk" href="${esc(urlSegura(l.url))}" target="_blank" rel="noopener noreferrer" aria-label="${esc(tr("Abrir {nome} em nova aba", { nome: rotuloDe(l) }))}"><rect fill="transparent" x="${x}" y="${ry}" width="${w}" height="18"/><rect class="foco" x="${x}" y="${ry}" width="${w}" height="18"/><text class="cap" x="${x + 6}" y="${ry + 13}" font-size="11">${esc(txt)}</text></a>`);
      x += w + 8;
    });
  });
  if (chips.length || livre) out.push(`<g>${chips.join("")}${livre ? `<text class="mid" x="${R - 6}" y="${yb + 55}" font-size="10" text-anchor="end">${trn(livre, ["+{n} LINK · VEJA EM EDITAR", "+{n} LINKS · VEJA EM EDITAR"])}</text>` : ""}</g>`);
}
