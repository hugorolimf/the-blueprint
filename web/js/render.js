// Passa de 350 linhas (teto de 500) de propósito: `render` é uma função só, que mede e desenha a lista e a folha com dezenas de variáveis locais
// compartilhadas; cortá-la ao meio mudaria o desenho. Próximo passo: extrair as medidas e cada seção do desenho em funções recebendo o contexto.
function render() {
  if (!carregado && !falhou) { host.innerHTML = ""; return; } // só mostra a folha quando os dados chegaram
  document.body.classList.remove("carregando");
  workspaceAtual = workspaceDaRota(arquivo, location.hash);
  D = workspaceAtual?.dados || arquivo;
  mcpWorkspaceMudou();
  responsavelCarrega(); // responsavel-logica.js: as pessoas do workspace, para atribuir planos
  repositorioAoAbrir(); // repositorio-logica.js: o aviso Configurar projeto, só quando o blueprint aberto muda
  const criandoWorkspace = location.hash === "#/workspace/novo";
  $("workspace-nova").hidden = !criandoWorkspace;
  document.body.classList.toggle("criando-workspace", criandoWorkspace);
  const naEntrada = !workspaceAtual;
  document.body.classList.toggle("na-entrada", naEntrada);
  $("entrada").hidden = !naEntrada;
  const W = Math.max(host.clientWidth, 960);
  larg = host.clientWidth;
  const FM = 10, IL = 36;
  const L = IL + 34, R = W - IL - 50;
  hoje = hojeSimulado || local(new Date());
  const hojeD = dia(hoje);
  const foco = tela ? D.blueprints.find((p) => p.id === tela) : null;
  const acoesTodas = (foco ? [foco] : D.blueprints).flatMap(foco ? acoesVisiveis : acoesDe);
  $("periodo-pendentes").setAttribute("aria-pressed", String(!!(foco && soPendentes[foco.id])));
  $("periodo-andamento").setAttribute("aria-pressed", String(!!(foco && soAndamento[foco.id])));
  $("periodo-meus").hidden = !responsavelAtivo(); // só no modo hospedado, para quem grava
  $("periodo-meus").setAttribute("aria-pressed", String(!!(foco && soMeus[foco.id])));
  $("periodo-andamento-n").textContent = foco ? agentesDoBlueprint(foco.id).length : 0; // quantos agentes trabalham neste blueprint ("0" se nenhum)
  const janela = janelaPeriodo(acoesTodas, hoje, foco ? periodos[tela] : null);
  const { t0, t1 } = janela;
  const ymin = Math.floor(janela.inicio / 12), mesInicial = janela.inicio % 12;
  const NM = janela.fim - janela.inicio + 1;
  const X = (d, a = L, b = R) => a + ((d - t0) / (t1 - t0)) * (b - a);
  const mesD = (m) => dia(mesIso(janela.inicio + m) + "-01");
  const xt = X(Math.min(Math.max(hojeD, t0), t1));
  if (foco) janelasRenderizadas[tela] = { inicio: mesIso(janela.inicio), fim: mesIso(janela.fim) };
  let n = 0;
  const nova = () => false; // tudo é desenhado de uma vez: só a folha se abre, o conteúdo já está lá
  const anima = (_novo, _cls, base = "") => (base ? 'class="' + base + '"' : "");

  // qual tela: a lista de blueprints (inicial) ou o blueprint de um blueprint
  if (tela && carregado && !D.blueprints.some((p) => p.id === tela)) { tela = null; history.replaceState(null, "", location.pathname + location.search); }
  const todos = D.blueprints.map((p, pi) => ({ p, pi, ok: projConcluido(p) }));
  const atual = tela ? todos.find((x) => x.p.id === tela) : null;
  const { mapa, onde: ondeAchou } = pesqLista(todos, consulta); // pesquisa-folha.js: os cartões que casam e onde cada um casou
  const total = 1 + D.blueprints.length;
  document.body.classList.toggle("em-blueprint", !!atual);
  if (criandoWorkspace) {
    host.innerHTML = ""; atualizaArte();
    if (desenrolarNaProxima && nomeUsuario) {
      atualizarCriacaoWorkspace();
      desenrolarNaProxima = false;
      desenrolar($("workspace-nova"), $("workspace-nova"));
    }
    return;
  }
  if (naEntrada) {
    $("entrada-marca").textContent = tr("Workspaces");
    $("entrada-titulo").textContent = tr("Seus workspaces");
    $("entrada-descricao").textContent = tr("Escolha um workspace para continuar.");
    $("entrada-voltar").hidden = true;
    $("entrada-novo").hidden = true;
    const workspaces = workspacesDe(arquivo);
    $("entrada-status").textContent = falhou ? tr("Não foi possível atualizar os workspaces. Tentando novamente…") : trn(workspaces.length, ["{n} workspace · arraste para reordenar (Alt + setas)", "{n} workspaces · arraste para reordenar (Alt + setas)"]);
    $("entrada-blueprints").innerHTML = workspaces.map((c) => {
      const pecas = pecasWorkspace(c, EMB);
      return `<a class="blueprint-cartao" data-workspace="${esc(c.id)}" draggable="false" aria-keyshortcuts="Alt+ArrowLeft Alt+ArrowRight Alt+ArrowUp Alt+ArrowDown" title="${esc(tr("Arraste para reordenar · Alt + setas no teclado"))}" href="#/c/${encodeURIComponent(c.id)}"><span class="cartao-tipo">${tr("Workspace")}</span><div class="cartao-arte" aria-hidden="true">${pecas ? previaMarca(pecas, 280, 120) : ""}</div><h2 title="${esc(c.nome)}">${esc(c.nome)}</h2><p title="${esc(c.dados.descricao || tr("Blueprints e planejamento do workspace"))}">${esc(c.dados.descricao || tr("Blueprints e planejamento do workspace"))}</p><footer><span>${trn(c.dados.blueprints.length, ["{n} blueprint", "{n} blueprints"])}</span><span>${tr("Abrir espaço →")}</span></footer></a>`;
    }).join("") + equipesCartoes() + `<button class="blueprint-cartao cartao-adicionar" type="button" data-adicionar-workspace><span class="mais" aria-hidden="true">+</span><strong>${tr("Criar workspace")}</strong><span>${tr("Um espaço para agrupar blueprints.")}</span></button>`;
    host.innerHTML = "";
    abrirEntrada();
    return;
  }

  // ---- medidas da lista de blueprints
  const TH = 152, GAP = 24, yMapa = 128;
  const minCartao = mapa.length > 1 ? 400 : 320;
  const cols = Math.min(3, Math.max(1, Math.floor((R - L + GAP) / (minCartao + GAP))));
  const tw = (R - L - (cols - 1) * GAP) / cols;
  const mapEnd = yMapa + 48 + Math.ceil((mapa.length + 1) / cols) * (TH + GAP) - GAP;
  const RH = 124, PH = 96, FOLGA = 76, baixo = 110, MINIMA = Math.max(560, innerHeight); // cada folha ocupa a janela inteira, no mínimo, e cresce com o conteúdo
  const COLW = Math.max(240, Math.min(430, Math.floor((R - L - 30) / 2)));
  const yRuler = tela ? 204 : 160, y0 = yRuler + 90;

  // ---- medidas do blueprint: a folha cresce com o conteúdo, sem limite de altura
  let bloco = null;
  if (atual) {
    const { p, pi, ok } = atual;
    const acoes = acoesVisiveis(p).filter((f) => f.a < t1 && f.b >= t0).map((f) => {
      const rotulo = `${numeroDaAcao(p, f, pi)} · ${f.nome || tr("SEM NOME")}`;
      const w = rotulo.length * (CHAR + 0.35) + 34 + (f.prototipos?.length ? 30 : 0); // +0,35 por letra: o espaçamento de 0,06em do texto SVG
      const x0 = X(Math.max(f.a, t0)), x1 = X(Math.min(f.b + 1, t1));
      const nt = f.planos.length, nm = f.checkpoints.length, ncol = (nt ? 1 : 0) + (nm ? 1 : 0);
      const blockW = ncol ? ncol * COLW + (ncol - 1) * 30 : 0;
      const bw = Math.max(x1 - x0, BARRA_MIN), bx0 = Math.max(L, Math.min(x0, R - bw)); // barra aberta: começa na data de início, com largura mínima
      // título, planos e checkpoints formam um bloco alinhado à esquerda, ao centro ou à direita da folha, conforme o terço do tempo em que a barra está (só 3 alinhamentos: acompanha o tempo e continua em colunas)
      const bloco3 = Math.min(Math.max(w, blockW), R - L - 8), terco = (bx0 + bw / 2 - L) / (R - L);
      const lx = terco < 1 / 3 ? L + 4 : terco > 2 / 3 ? R - bloco3 : (L + R - bloco3) / 2, bx = lx;
      const linhasPlanos = f.planos.map((t) => linhasDoPlano(t.texto || tr("sem texto"), Math.floor((COLW - 28) / 7.3)));
      const alturaPlanos = linhasPlanos.reduce((s, linhas) => s + Math.max(22, linhas.length * 14 + 8), 0);
      return { ...f, linhasPlanos, rotulo, x0, x1, bw, bx0, lx, lw: w, bx, blockW, alt: ncol ? 100 + 34 + Math.max(alturaPlanos, nm * 22) + 44 : RH };
    });
    const medida = (aberto) => ({ aberto, RH, PH, FOLGA, de: aberto ? (f) => Math.min(f.lx, f.bx, f.bx0) : (f) => f.x0, ate: aberto ? (f) => Math.max(f.lx + f.lw, f.bx0 + f.bw + 56, f.bx + f.blockW) + 24 : (f) => f.x1 + 6 });
    const grupos = gruposDoBlueprint(p, acoes, acoesDe(p)), filtrando = soPendentes[p.id] || soAndamento[p.id] || soMeus[p.id], GAP_SECAO = 28, secoes = [];
    let ys = y0;
    const poe = (sec) => { const disp = disposicaoFolha(sec.acoes, medida(sec.aberto)); secoes.push({ ...sec, ...disp, y: ys }); ys += disp.alto - FOLGA + GAP_SECAO; };
    if (!grupos) poe({ p, pi, numero: pad(pi + 1), nome: p.nome, descricao: p.descricao, total: p.acoes.length, obs: p.obs, links: p.links, aberto: abertos[p.id] === true, toggle: `data-toggle="${esc(p.id)}"`, acoes, ...resumo(acoesDe(p)) });
    else grupos.filter((g) => !filtrando || g.acoes.length).forEach((g) => poe({ p, pi, numero: pad(g.numero), nome: g.nome, descricao: g.descricao || (g.total ? `${fmt(deDia(g.ini))} → ${fmt(deDia(g.fim))}` : ""), total: g.total, aberto: g.aberto, toggle: `data-projeto="${esc(g.chave)}"`, acoes: g.acoes, concl: g.concl, horas: g.horas, ok: g.ok, futuro: g.futuro, geral: g.geral })); // cada projeto é uma seção, como um blueprint
    bloco = { p, pi, ok, secoes, y: y0, alto: secoes.length ? ys - y0 - GAP_SECAO + FOLGA : PH + 36 + FOLGA, ...resumo(acoesDe(p)) };
  }
  const alturaCapa = Math.max(MINIMA, mapEnd + 150 + baixo + IL);
  const alturaProj = (b) => Math.max(MINIMA, y0 + b.alto + 40 + baixo + IL);

  // ---- peças comuns a toda folha ------------------------------------------------------------
  const moldura = (out, H) => {
    out.push(`<defs><pattern id="hach" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2="6" stroke="#9cc4ee" stroke-width="1.5"/></pattern></defs>`);
    out.push(`<rect class="ln" x="${FM}" y="${FM}" width="${W - 2 * FM}" height="${H - 2 * FM}"/>`);
    out.push(`<rect class="ln" style="stroke-width:2.2" x="${IL}" y="${IL}" width="${W - 2 * IL}" height="${H - 2 * IL}"/>`);
    for (let m = 0; m <= NM; m++) {
      const xa = X(mesD(m));
      out.push(`<path class="ln l" d="M${xa} ${FM}V${IL}M${xa} ${H - IL}V${H - FM}"/>`);
      if (m < NM) {
        const xc = (xa + X(mesD(m + 1))) / 2;
        out.push(`<text class="mid" x="${xc}" y="${FM + 17}" font-size="10" text-anchor="middle">${m + 1}</text><text class="mid" x="${xc}" y="${H - FM - 9}" font-size="10" text-anchor="middle">${m + 1}</text>`);
      }
    }
    for (let k = 0; IL + k * 120 < H - IL; k++) {
      const ya = IL + k * 120, yb = Math.min(ya + 120, H - IL), letra = String.fromCharCode(65 + (k % 26));
      out.push(`<path class="ln l" d="M${FM} ${ya}H${IL}M${W - IL} ${ya}H${W - FM}"/>`);
      out.push(`<text class="mid" x="${FM + 13}" y="${(ya + yb) / 2 + 4}" font-size="10" text-anchor="middle">${letra}</text><text class="mid" x="${W - FM - 13}" y="${(ya + yb) / 2 + 4}" font-size="10" text-anchor="middle">${letra}</text>`);
    }
    out.push(`<path class="ln l" d="M${FM} ${H - IL}H${IL}M${W - IL} ${H - IL}H${W - FM}"/>`);
  };
  const regua = (out, H) => {
    const yBand = H - IL - baixo;
    out.push(`<g class="regua"><path class="ln" d="M${L} ${yRuler}H${R}"/>`); // .regua: a barra de datas (barras.js a anima)
    for (let m = 0; m <= NM; m++) {
      const x = X(mesD(m));
      out.push(`<path class="ln" d="M${x} ${yRuler - 10}V${yRuler}"/><path class="guia" d="M${x} ${yRuler}V${yBand - 20}"/>`);
      if (m < NM) out.push(`<g class="lk" role="button" tabindex="0" data-mes="${mesIso(janela.inicio + m)}" aria-label="Selecionar ${mesIso(janela.inicio + m)}" aria-pressed="${periodos[tela]?.aguardando && periodos[tela].inicio === mesIso(janela.inicio + m) ? "true" : "false"}"><rect fill="${periodos[tela]?.aguardando && periodos[tela].inicio === mesIso(janela.inicio + m) ? "#ffffff33" : "transparent"}" x="${x}" y="${yRuler - 32}" width="${X(mesD(m + 1)) - x}" height="30"/><text x="${(x + X(mesD(m + 1))) / 2}" y="${yRuler - 16}" font-size="11" text-anchor="middle">${MESES[(mesInicial + m) % 12]}</text></g>`);
      if (m < NM && (m === 0 || (mesInicial + m) % 12 === 0)) out.push(`<text class="mid" x="${x + 3}" y="${yRuler - 34}" font-size="10">${ymin + Math.floor((mesInicial + m) / 12)}</text>`);
    }
    for (let d = t0 + 7; d < t1; d += 7) out.push(`<path class="ln m" d="M${X(d)} ${yRuler - 4}V${yRuler}"/>`);
    if (hojeD >= t0 && hojeD < t1) out.push(`<path class="eixo" d="M${xt} ${yRuler}V${yBand - 20}"/>`);
    if (hojeD >= t0 && hojeD < t1) out.push(`<g class="hoje-chip"><rect x="${xt - 46}" y="${yRuler + 8}" width="92" height="20" fill="#eaf4ff"/><text x="${xt}" y="${yRuler + 22}" font-size="11" font-weight="700" text-anchor="middle" class="hoje-t" style="fill:#0c3a78">${tr("HOJE")} ${fmt(hoje)}</text></g>`);
    out.push(`</g>`);
  };
  const rodape = (out, H, folha, desenho) => {
    const yBand = H - IL - baixo, ly = yBand - 4;
    const cw = 440, ch = 96, cx = R - cw + 16, cy2 = H - IL - ch - 4;
    out.push(`<path class="ln" d="M${L} ${yBand - 8}H${R}"/>`);
    if (!atual) {
      out.push(`<text class="mid cap" x="${L}" y="${ly + 60}" font-size="8">${tr("BARRA = AÇÃO · PREENCHIDO = FEITO · VAZIO = FALTA")}</text>`);
      out.push(`<rect class="ln" x="${L + 270}" y="${ly + 60 - 8}" width="22" height="8"/><rect class="hach" x="${L + 271}" y="${ly + 60 - 7}" width="20" height="6"/><text class="mid cap" x="${L + 298}" y="${ly + 60}" font-size="8">${tr("CONCLUÍDA")}</text>`);
      out.push(`<rect class="ln" style="stroke-width:1.8" x="${L + 360}" y="${ly + 60 - 8}" width="22" height="8"/><rect class="prog" x="${L + 361}" y="${ly + 60 - 7}" width="10" height="6"/><text class="mid cap" x="${L + 388}" y="${ly + 60}" font-size="8">${tr("EM ANDAMENTO")}</text>`);
      out.push(`<rect class="ln m" style="stroke-dasharray:6 4" x="${L + 470}" y="${ly + 60 - 8}" width="22" height="8"/><text class="mid cap" x="${L + 498}" y="${ly + 60}" font-size="8">${tr("A FAZER")}</text>`);
    } // legenda da lista: a barra é a ação; na folha do blueprint o rótulo vai no próprio desenho
    const selecionado = atual ? tr("Marque planos e checkpoints clicando neles · passe o mouse sobre uma ação para ver os detalhes.") : tr("Clique num blueprint para abrir o blueprint dele.");
    const maxSel = Math.max(1, Math.floor((cx - L - 18) / 7.7));
    out.push(`<text class="sel mid" data-max="${maxSel}" x="${L}" y="${ly + 100}" font-size="11">${esc(corta(selecionado, maxSel))}</text>`);
    out.push(`<rect class="ln" style="stroke-width:1.6" x="${cx}" y="${cy2}" width="${cw}" height="${ch}"/>`);
    out.push(`<path class="ln m" d="M${cx} ${cy2 + 36}H${cx + cw}M${cx} ${cy2 + 66}H${cx + cw}M${cx + 150} ${cy2 + 66}V${cy2 + ch}M${cx + 250} ${cy2 + 66}V${cy2 + ch}M${cx + 340} ${cy2 + 66}V${cy2 + ch}"/>`);
    out.push(`<text class="mid cap" x="${cx + 8}" y="${cy2 + 11}" font-size="8">${tr("Workspace")}</text><text x="${cx + 8}" y="${cy2 + 28}" font-size="16" font-weight="700" class="cap">${esc(corta(D.empresa, 28))}</text>`);
    out.push(`<text class="mid cap" x="${cx + 8}" y="${cy2 + 47}" font-size="8">${tr("Tela")}</text><text x="${cx + 8}" y="${cy2 + 61}" font-size="12" font-weight="700" class="cap">${esc(corta(desenho, 38))}</text>`);
    const acoesCarimbo = atual ? acoesDe(atual.p) : D.blueprints.flatMap((p) => acoesDe(p)), feitasCarimbo = acoesCarimbo.filter((x) => x.status === "concluido").length;
    const progCarimbo = atual ? progressoDe(atual.p) : D.blueprints.length ? D.blueprints.reduce((t, x) => t + progressoDe(x), 0) / D.blueprints.length : 0;
    [[tr("Data"), hoje.split("-").reverse().join("/"), 0], [tr("Progresso"), `${Math.round(progCarimbo * 100)}%`, 150], [tr("Folha"), `${folha}/${total}`, 250], [tr("Ações"), `${feitasCarimbo}/${acoesCarimbo.length}`, 340]].forEach(([k, v, off]) => {
      out.push(`<text class="mid cap" x="${cx + off + 8}" y="${cy2 + 77}" font-size="8">${k}</text><text x="${cx + off + 8}" y="${cy2 + 91}" font-size="12" font-weight="700">${esc(v)}</text>`);
    });
  };
  // a marca da folha: peças já "construídas" (sólidas) seguem o progresso; as pendentes ficam só no traço
  const marcaFolha = (pecas, { centro = MINIMA / 2, maxLado = MINIMA * 0.6, progresso = 1, titulo = "", blueprint = "", fundo = false } = {}) => {
    if (!EMB || !pecas?.length) return "";
    const lado = Math.round(Math.min(W * (fundo ? 0.55 : 0.3), maxLado, fundo ? MINIMA : 420)), larg = Math.round(lado * 1.35);
    const e = EMB.desenhaEmblema(pecas, { largura: larg, altura: lado - 18, progresso, sombra: false });
    if (!e) return "";
    const pct = Math.round(progresso * 100);
    const info = tr("MARCA · {pct}% CONSTRUÍDA ({feitas} de {total} peças)", { pct, feitas: e.feitas, total: e.total });
    return `<svg class="emblema marca${fundo ? " marca-fundo" : ""}" x="${W / 2 - larg / 2}" y="${centro - lado / 2}" width="${larg}" height="${lado}" viewBox="0 0 ${larg} ${lado}" role="img" aria-label="${esc(info)}" data-info="${esc(info)}" data-pct="${pct}" data-feitas="${e.feitas}" data-total="${e.total}" data-tit="${esc(titulo)}" data-proj="${esc(blueprint)}">${e.svg}<text class="mid legenda-marca" x="${larg / 2}" y="${lado - 3}" text-anchor="middle" font-size="10">${pct}% CONSTRUÍDA</text></svg>`;
  };
  const arteCartao = (pecas, x, y, w, h, progresso = 1) => {
    if (!EMB || !pecas?.length) return "";
    const e = EMB.desenhaEmblema(pecas, { largura: w, altura: h, pad: 2, progresso, sombra: false });
    return `<svg class="emblema" x="${x}" y="${y}" width="${w}" height="${h}" viewBox="0 0 ${e.largura} ${e.altura}" aria-hidden="true">${e.svg}</svg>`;
  };
  const folhaSvg = (H, out, rotulo) => `<svg class="desenho" xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" role="group" aria-label="${esc(rotulo)}">${out.join("")}</svg>`;

  // ---- tela inicial: a lista de blueprints -----------------------------------------------------
  const capa = () => {
    const H = alturaCapa, out = [];
    moldura(out, H);
    const comAcoes = todos.filter((x) => x.p.acoes.length);
    const geralTodos = comAcoes.length ? comAcoes.reduce((t, x) => t + resumo(acoesDe(x.p)).geral, 0) / comAcoes.length : 0;
    out.push(marcaFolha(pecasWorkspace(workspaceAtual, EMB), { centro: H / 2, maxLado: MINIMA * 0.55, progresso: geralTodos, titulo: D.empresa || "Workspace", fundo: true })); // a marca do workspace acompanha o progresso dos seus blueprints como uma marca d'água de fundo
    out.push(`<text data-titulo="nome" x="${L}" y="${IL + 36}" font-size="18" font-weight="700" class="cap">${esc(D.empresa)} · ${esc(D.titulo)}</text>`);
    out.push(`<text data-titulo class="mid cap" x="${L}" y="${IL + 56}" font-size="11">${tr("Blueprints · cada um abre o seu próprio blueprint")}</text>`);
    out.push(`<text class="mid cap" x="${L}" y="${yMapa + 6}" font-size="10">${tr("Clique para abrir o blueprint · segure e arraste para reordenar")}</text>`);
    mapa.forEach(({ p, pi, ok }, i) => {
      const col = i % cols, row = Math.floor(i / cols);
      const tx = L + col * (tw + GAP), ty = yMapa + 48 + row * (TH + GAP);
      const fs = acoesDe(p), r = resumo(fs);
      const nv = nova("t:" + p.id);
      const artW = Math.min(116, Math.max(76, Math.round(tw * 0.25))), artX = tx + 12, artY = ty + 26, artH = TH - 52;
      const textoX = artX + artW + 14, textoW = tx + tw - 16 - textoX;
      const nome = corta(p.nome || tr("SEM NOME"), Math.max(8, Math.floor(textoW / 10.2)));
      const descricao = corta(p.descricao || "", Math.max(12, Math.floor(textoW / 7.5)));
      const info = (!p.acoes.length ? tr("SEM AÇÕES AINDA") : ok ? tr("✓ CONCLUÍDO · {n} AÇÕES", { n: p.acoes.length }) : r.futuro ? tr("PLANEJADO") : tr("{concl}/{total} AÇÕES · {pct}% · FALTAM {faltam} ITENS", { concl: r.concl, total: p.acoes.length, pct: Math.round(r.geral * 100), faltam: fs.reduce((t, f) => t + f.it[1] - f.it[0], 0) })) + (r.horas ? ` · ${r.horas} H` : "");
      const aria = tr("Blueprint {n}, {nome}: {info}. Abrir o blueprint. Segure e arraste para reordenar", { n: pi + 1, nome: p.nome, info });
      out.push(`<g data-goto="${esc(p.id)}" class="alvo${ok ? " ok" : ""}" tabindex="0" role="button" aria-label="${esc(aria)}">`);
      out.push(`<rect class="moldura" x="${tx}" y="${ty}" width="${tw}" height="${TH}"/>`);
      out.push(`<g ${anima(nv, "surge")}>`);
      out.push(arteCartao(p.emblema?.pecas?.length ? p.emblema.pecas : EMB?.CONSTRUCAO, artX, artY, artW, artH, r.geral));
      out.push(`<text x="${textoX}" y="${ty + 44}" font-size="15" font-weight="700" class="cap">${esc(nome)}</text>`);
      out.push(`<text class="mid cap" x="${textoX}" y="${ty + 70}" font-size="11">${esc(descricao)}</text>`);
      const achou = ondeAchou.get(p.id); // pesquisando: no lugar do andamento, onde a consulta casou
      out.push(`<text class="mid${achou ? " pesq-onde" : ""}" x="${textoX}" y="${ty + 95}" font-size="10">${esc(corta(achou || info, Math.floor(textoW / 7)))}</text>`);
      if (fs.length) {
        // como numa folha de projeto: a barra é o blueprint inteiro (hachurado = feito, vazio = falta) e, embaixo, a linha de cota
        // com as datas de início e fim, o "hoje" e os checkpoints em losango (cheio = feito)
        const x0 = textoX, x1 = tx + tw - 16, yb = ty + 106, yc = ty + 130;
        const ini = Math.min(...fs.map((f) => f.a)), fim = Math.max(...fs.map((f) => f.b + 1));
        const px = (d) => x0 + ((d - ini) / (fim - ini || 1)) * (x1 - x0), fmt = (d) => deDia(d).slice(8) + "/" + deDia(d).slice(5, 7);
        // uma célula por ação, juntas por situação (concluídas | em andamento | a fazer), com um divisor mais largo entre os grupos; a largura é a quantidade de itens
        const ordem = { concluido: 0, andamento: 1, planejado: 2 }, cel = capaCelulas([...fs].sort((u, v) => ordem[u.status] - ordem[v.status] || u.a - v.a), x1 - x0); // muitas ações: um bloco por situação (capa-agrupa.js)
        const vao = (i) => (i && cel[i].status !== cel[i - 1].status ? 7 : 2), peso = (f) => Math.max(1, f.it[1]);
        const total = cel.reduce((t, f) => t + peso(f), 0), util = x1 - x0 - cel.reduce((t, _, i) => t + (i ? vao(i) : 0), 0);
        let cx = x0;
        cel.forEach((f, i) => {
          if (i) { cx += vao(i); if (vao(i) > 2) out.push(`<path class="eixo" style="stroke-dasharray:none" d="M${cx - 3.5} ${yb - 3}V${yb + 15}"/>`); }
          const w = util * peso(f) / total, estilo = f.status === "concluido" ? "" : f.status === "andamento" ? ' style="stroke-width:1.8"' : ' style="stroke-dasharray:6 4"';
          const miolo = f.status === "concluido" ? `<rect class="hach" x="${cx + 1}" y="${yb + 1}" width="${Math.max(0, w - 2)}" height="10"/>` : f.pct > 0 ? `<rect class="prog" x="${cx + 1}" y="${yb + 1}" width="${Math.max(0, (w - 2) * f.pct)}" height="10"/>` : "";
          out.push(`<g><title>${esc(capaTitulo(f, () => numeroDaAcao(p, f, pi)))}</title><rect class="ln${f.status === "planejado" ? " m" : ""}"${estilo} x="${cx}" y="${yb}" width="${w}" height="12"/>${miolo}</g>`);
          cx += w;
        });
        out.push(`<path class="ln l" d="M${x0} ${yc}H${x1}M${x0} ${yc - 4}V${yc + 4}M${x1} ${yc - 4}V${yc + 4}"/>`);
        capaMarcos(fs.flatMap((f) => f.checkpoints).filter((m) => dia(m.data) >= ini && dia(m.data) <= fim).map((m) => ({ x: px(dia(m.data)), feito: !!m.feito }))).forEach((g) => out.push(capaMarcoSvg(g, yc)));
        if (hojeD >= ini && hojeD <= fim) out.push(`<path class="eixo" style="stroke-dasharray:none" d="M${px(hojeD)} ${yc - 7}V${yc + 7}"/>`);
        out.push(`<text class="mid" x="${x0}" y="${ty + 146}" font-size="8">${fmt(ini)}</text><text class="mid" x="${x1}" y="${ty + 146}" font-size="8" text-anchor="end">${fmt(fim - 1)}</text>`);
      }
      out.push(`</g></g>`);
      if (nv) n++;
    });
    const novoIndice = mapa.length, novaColuna = novoIndice % cols, novaLinha = Math.floor(novoIndice / cols);
    const nx = L + novaColuna * (tw + GAP), ny = yMapa + 48 + novaLinha * (TH + GAP);
    const artW = Math.min(108, Math.max(76, Math.round(tw * 0.24))), artX = nx + 12, artY = ny + 26, artH = TH - 52;
    const textoX = artX + artW + 14, plusX = nx + tw - 56, textoW = plusX - 12 - textoX;
    out.push(`<g data-adicionar-blueprint tabindex="0" role="button" aria-label="${esc(tr("Criar novo blueprint"))}" class="alvo criar"><rect class="moldura" x="${nx}" y="${ny}" width="${tw}" height="${TH}"/>${arteCartao(EMB?.CONSTRUCAO, artX, artY, artW, artH)}<text class="cap" x="${textoX}" y="${ny + 68}" font-size="12">${esc(corta(tr("NOVO BLUEPRINT"), Math.floor(textoW / 7.2)))}</text><text class="mid cap" x="${textoX}" y="${ny + 94}" font-size="10">${esc(corta(tr("ADICIONAR BLUEPRINT"), Math.floor(textoW / 6)))}</text><rect class="mais-borda" x="${plusX}" y="${ny + 58}" width="36" height="36"/><path class="mais-novo" d="M${plusX + 10} ${ny + 76}h16M${plusX + 18} ${ny + 68}v16"/></g>`);
    if (falhou) out.push(`<text class="aviso cap" x="${L}" y="${yMapa + 40}" font-size="12" font-weight="700">${tr("Servidor indisponível · abra o The Blueprint (beta) (blueprint.exe) e use http://127.0.0.1:3000")}</text>`);
    rodape(out, H, 1, tr("Lista de blueprints"));
    return folhaSvg(H, out, tr("Lista de blueprints da {empresa}", { empresa: D.empresa }));
  };

  // ---- blueprint de um blueprint -----------------------------------------------------------------
  const folhaBlueprint = (b) => {
    const numero = b.pi + 2;
    const H = alturaProj(b), out = [];
    const { p, pi, y: yb } = b;
    const agsP = agentesDoBlueprint(p.id); // as figuras do Claude ficam na camada à parte (claude-camada.js); aqui só o destaque do plano
    moldura(out, H);
    out.push(marcaFolha(p.emblema?.pecas, { centro: H / 2, progresso: b.geral, titulo: p.nome || "Blueprint", blueprint: p.id })); // a marca deste blueprint, sempre no centro da janela, construída conforme o progresso
    out.push(`<text data-titulo="nome" x="${L}" y="${IL + 36}" font-size="18" font-weight="700" class="cap">${esc(D.empresa)} · ${esc(D.titulo)}</text>`);
    out.push(`<text data-titulo class="mid cap" x="${L}" y="${IL + 56}" font-size="11">${tr("Folha {n} de {total} · Blueprint {bp} · {nome}", { n: numero, total, bp: pad(pi + 1), nome: esc(corta(p.nome || tr("Sem nome"), 60)) })}</text>`);
    regua(out, H);
    const secao = (s) => {
      const yb = s.y;
      cabecalhoSecao(out, s, { L, R });
      if (!s.total) out.push(`<text class="mid cap" x="${L + 40}" y="${yb + PH + 8}" font-size="11">${tr("Sem ações · abra Editar para adicionar")}</text>`);
      else if (!s.acoes.length) out.push(`<text class="mid" x="${L}" y="${yb + PH + 30}" font-size="12">${soAndamento[p.id] ? tr("Nada em andamento neste blueprint.") : soPendentes[p.id] ? tr("Nada pendente neste blueprint.") : tr("Nenhuma ação neste período.")}</text>`);
      s.acoes.forEach((f) => {
        const nf = nova((s.aberto ? "f:" : "c:") + f.id);
        const dias = Math.round(f.b - f.a) + 1;
        let w = Math.max(f.x1 - f.x0, 6);
        const extra = (f.tf[1] ? ` · ${f.tf[0]}/${f.tf[1]} ${tr("PLANOS")}` : "") + (f.mf[1] ? ` · ${f.mf[0]}/${f.mf[1]} ${tr("CHECKPOINTS")}` : "") + (f.obs?.trim() ? ` · ${tr("COM INSTRUÇÕES")}` : "");
        const abrir = s.aberto ? "" : " · " + tr("CLIQUE PARA ABRIR AQUI"); // com as ações recolhidas, clicar abre e leva até o item
        const info = `${f.rotulo.toUpperCase()} · ${ROTULO[f.status].toUpperCase()} · ${Math.round(f.pct * 100)}% · ${fmt(f.inicio)} → ${fmt(f.fim)} · ${dias} ${tr("dias")}${extra}${abrir}`;
        const aria = `${f.rotulo}, ${ROTULO[f.status]}, ${Math.round(f.pct * 100)}%, ${fmt(f.inicio)} a ${fmt(f.fim)}${extra.replaceAll("·", ",").toLowerCase()}`;
        const checkpoints = (cy, r, g) => (f.checkpoints || []).filter((m) => dia(m.data) >= t0 && dia(m.data) < t1).map((m) => {
          const mxp = g ? g.x0 + ((dia(m.data) + 0.5 - f.a) / (f.b + 1 - f.a)) * (g.x1 - g.x0) : Math.min(R, Math.max(L, X(dia(m.data) + 0.5)));
          const mi = `${tr("CHECKPOINT")}: ${(m.nome || tr("SEM NOME")).toUpperCase()} · ${fmt(m.data)} · ${m.feito ? tr("FEITO") : tr("PENDENTE")}${abrir}`;
          return `<g class="checkpoint" data-m="${esc(m.id)}" data-info="${esc(mi)}"><circle cx="${mxp}" cy="${cy}" r="${r + 5}" fill="transparent"/><path class="dm${m.feito ? " ok" : ""}" d="M${mxp} ${cy - r}l${r} ${r}l${-r} ${r}l${-r} ${-r}z"/></g>`;
        }).join("");
        const dados = `class="acao" tabindex="0" role="group" aria-label="${esc(aria)}" data-st="${f.status}" data-p="${esc(p.id)}" data-f="${esc(f.id)}" data-info="${esc(info)}"`;
        if (!s.aberto) {
          const by = yb + PH - 12 + s.rowY[f.linha];
          out.push(`<g ${dados} data-ir>`);
          out.push(`<rect class="hit" x="${f.x0 - 10}" y="${by - 7}" width="${Math.max(w, 24) + 20}" height="24"/>`); // área de clique: a linha inteira da ação, não só o traço da barra
          out.push(`<rect ${anima(nf, "desenha", "caixa")} pathLength="1" x="${f.x0}" y="${by}" width="${w}" height="10"/>`);
          if (f.status === "concluido") out.push(`<rect ${anima(nf, "surge", "hach")} x="${f.x0 + 1}" y="${by + 1}" width="${w - 2}" height="8"/>`);
          else if (f.status === "andamento") out.push(`<rect ${anima(nf, "surge", "prog")} x="${f.x0 + 1}" y="${by + 1}" width="${Math.max(0, (w - 2) * f.pct)}" height="8"/>`);
          out.push(checkpoints(by + 5, 4.5));
          out.push(`<rect class="realce" x="${f.x0 - 3}" y="${by - 3}" width="${w + 6}" height="16"/></g>`);
          if (nf) n++;
          return;
        }
        const g = { ...f, x0: f.bx0, x1: f.bx0 + f.bw }; // barra e cota abertas usam a largura mínima
        w = f.bw;
        const fy = yb + PH + s.rowY[f.linha], by = fy + 20, bh = 26;
        out.push(`<g ${dados}>`);
        out.push(`<rect x="${Math.min(g.x0, f.lx) - 6}" y="${fy}" width="${Math.max(w + 12, f.lw + 12)}" height="${RH - 30}" fill="transparent"/>`);
        out.push(`<text x="${f.lx}" y="${fy + 12}" font-size="11" class="cap">${esc(f.rotulo)}</text>`);
        out.push(`<rect ${anima(nf, "desenha", "caixa")} pathLength="1" x="${g.x0}" y="${by}" width="${w}" height="${bh}"/>`);
        if (f.status === "concluido") {
          const cx = Math.max(g.x0 + 12, g.x1 - 17), cy = by + bh / 2;
          out.push(`<rect ${anima(nf, "surge", "hach")} x="${g.x0 + 1}" y="${by + 1}" width="${w - 2}" height="${bh - 2}"/>`);
          if (w >= 100) out.push(`<g class="selo"><circle cx="${cx}" cy="${cy}" r="8" fill="#0c3a78" stroke="#eaf4ff"/><path class="ln" d="M${cx - 3.6} ${cy}l2.6 3l4.6-6"/></g>`);
        } else if (f.status === "andamento") {
          out.push(`<rect ${anima(nf, "surge", "prog")} x="${g.x0 + 1}" y="${by + 1}" width="${Math.max(0, (w - 2) * f.pct)}" height="${bh - 2}"/>`);
        }
        // porcentagem da ação: vem dos planos e checkpoints marcados (ou do valor digitado, se a ação não tiver nenhum)
        out.push(`<text data-horas-dias ${anima(nf, "surge")} x="${Math.min(g.x1 + 20, R + 4)}" y="${by + 17}" font-size="12" font-weight="700">${Math.round(f.pct * 100)}%</text>`);
        if (f.it[1]) out.push(`<text class="mid" data-horas-dias ${anima(nf, "surge")} x="${Math.min(g.x1 + 20, R + 4)}" y="${by + 31}" font-size="9">${f.it[0]}/${f.it[1]}</text>`);
        out.push(checkpoints(by + bh / 2, 7, g));
        out.push(`<rect class="realce" x="${g.x0 - 4}" y="${by - 4}" width="${w + 8}" height="${bh + 8}"/>`);
        // cota: duração entre as duas datas, com traços oblíquos nas pontas
        const cy = fy + 72;
        out.push(`<g class="cota"><path class="ln m" d="M${g.x0} ${by + bh + 2}V${cy + 6}M${g.x1} ${by + bh + 2}V${cy + 6}M${g.x0} ${cy}H${g.x1}M${g.x0 - 4} ${cy + 4}l8 -8M${g.x1 - 4} ${cy + 4}l8 -8"/>`);
        out.push(`<text class="mid" x="${(g.x0 + g.x1) / 2}" y="${cy - 5}" font-size="10" text-anchor="middle">${trn(dias, ["{n} DIA", "{n} DIAS"])}</text>`);
        out.push(datasCota(g, cy));
        // planos e checkpoints da ação: caixas de marcar direto no blueprint
        if (f.planos.length || f.checkpoints.length) {
          const base = fy + 112, maxc = Math.floor((COLW - 28) / 7.3);
          const cab = (x, titulo, feitos, tot) => `<text class="mid cap" x="${x}" y="${base}" font-size="9">${titulo} · ${feitos}/${tot}</text><path class="ln l" d="M${x} ${base + 7}h${COLW - 20}"/>`;
          const atr = (tipo, it, info2) => `class="chk${it.feito ? " ok" : ""}" tabindex="0" role="checkbox" aria-checked="${!!it.feito}" aria-label="${esc(info2)}" data-chk="${esc(it.id)}" data-tipo="${tipo}" data-p="${esc(p.id)}" data-f="${esc(f.id)}" data-id="${esc(it.id)}" data-info="${esc(info2)}"`;
          const gl = [];
          let cx0 = f.bx;
          if (f.planos.length) {
            gl.push(cab(cx0, tr("Planos"), f.tf[0], f.tf[1]));
            let deslocamento = 0;
            f.planos.forEach((t, i) => {
              const yy = base + 30 + deslocamento, nome = t.texto || tr("sem texto"), linhas = f.linhasPlanos[i], altura = Math.max(22, linhas.length * 14 + 8);
              deslocamento += altura;
              const nomeNormalizado = nome.replace(/\s+/gu, " ").trim();
              let posicao = 0;
              const textoLinhas = linhas.map((linha, n) => { posicao += linha.length; const espaco = nomeNormalizado[posicao] === " " ? (posicao++, " ") : ""; return `<tspan x="${cx0 + 22}" dy="${n ? 14 : 0}">${esc(linha + espaco)}</tspan>`; }).join("");
              gl.push(`<g ${atr("t", t, tr("PLANO: {nome} · {estado} · clique para marcar", { nome, estado: t.feito ? tr("FEITO") : tr("PENDENTE") }))}><rect fill="transparent" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="${altura}"/><rect class="foco2" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="${altura}"/><rect class="cx${t.feito ? " ok" : ""}" x="${cx0}" y="${yy - 11}" width="13" height="13"/>${t.feito ? `<path class="cxm" d="M${cx0 + 3} ${yy - 4}l3 3l5-6"/>` : ""}<text class="tx" x="${cx0 + 22}" y="${yy}" font-size="11">${linhas.length === 1 ? esc(linhas[0]) : textoLinhas}</text></g>`);
              const aqui = agsP.filter((x) => x.planoId === t.id);
              if (aqui.length) gl.push(`<rect class="robo-linha${roboClasse(aqui[0])}" data-plano="${esc(t.id)}" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="${altura}"/>`);
            });
            cx0 += COLW + 30;
          }
          if (f.checkpoints.length) {
            gl.push(cab(cx0, tr("Checkpoints"), f.mf[0], f.mf[1]));
            f.checkpoints.forEach((m, i) => {
              const yy = base + 30 + i * 22, nome = m.nome || tr("sem nome"), dt = dia(m.data) === dia(m.data) ? " · " + fmt(m.data) : "";
              gl.push(`<g ${atr("m", m, tr("CHECKPOINT: {nome}{dt} · {estado} · clique para marcar", { nome, dt, estado: m.feito ? tr("FEITO") : tr("PENDENTE") }))}><rect fill="transparent" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/><rect class="foco2" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/><path class="dm${m.feito ? " ok" : ""}" d="M${cx0 + 6.5} ${yy - 12}l6.5 6.5l-6.5 6.5l-6.5 -6.5z"/><text class="tx" x="${cx0 + 22}" y="${yy}" font-size="11">${esc(corta(nome + dt, maxc))}</text></g>`);
            });
          }
          out.push(`<g ${anima(nf, "surge")}>${gl.join("")}</g>`);
        }
        if (f.prototipos?.length) out.push(protoSvg(f.lx + (f.rotulo.length * (CHAR + 0.35)) + 46, fy - 1, trn(f.prototipos.length, ["PROTÓTIPO DA AÇÃO {rotulo}: {titulos} · clique para ver", "PROTÓTIPOS DA AÇÃO {rotulo}: {titulos} · clique para ver"], { rotulo: f.rotulo.toUpperCase(), titulos: f.prototipos.map((x) => x.titulo.toUpperCase()).join(" · ") }), `${p.id}|${f.id}`));
        out.push(notaSvg(f.lx + (f.rotulo.length * (CHAR + 0.35)) + 16, fy - 1, tr("INSTRUÇÕES DA AÇÃO {rotulo}: {texto}", { rotulo: f.rotulo.toUpperCase(), texto: f.obs?.trim() ? umaLinha(f.obs, 200) : tr("ainda sem instruções · clique para escrever") }), `${p.id}|${f.id}`));
        out.push(`</g>`);
        if (nf) n++;
      });
      out.push(`</g>`);
    };
    b.secoes.forEach(secao);
    rodape(out, H, numero, p.nome || tr("Sem nome"));
    return folhaSvg(H, out, `Blueprint ${p.nome}`);
  };

  host.innerHTML = bloco ? folhaBlueprint(bloco) : capa();
  horasAtualiza();
  posicionaMarca();
  requestAnimationFrame(barraAjusta); // barra-topo.js: a barra começa depois do título (no quadro seguinte: medir aqui no meio dispararia as transições da folha)
  const alvo = [...host.querySelectorAll(".acao")].find((g) => g.getAttribute("aria-label") === selecionada);
  if (alvo) marcar(alvo);
  if (desenrolarNaProxima) { desenrolarNaProxima = false; desenrolar(); sugereF11(bloco ? "blueprint" : "lista"); }
  montaRobo();
  if (roboRolar) { roboRolar = false; requestAnimationFrame(() => (host.querySelector(".robo-linha") || camadaClaude.querySelector("[data-robo-nome]"))?.scrollIntoView({ block: "center", behavior: reduz() ? "auto" : "smooth" })); }

  pesquisaAposRender(mapa); // pesquisa-folha.js: contador, destaque e (no blueprint) o filtro dos itens
  requestAnimationFrame(atualizaRolagem);
}

