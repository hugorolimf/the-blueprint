// Passa de 350 linhas (teto de 500) de propósito: `render` é uma função só, que mede e desenha a lista e a folha com dezenas de variáveis locais
// compartilhadas; cortá-la ao meio mudaria o desenho. Próximo passo: extrair as medidas e cada seção do desenho em funções recebendo o contexto.
function render() {
  if (!carregado && !falhou) { host.innerHTML = ""; return; } // só mostra a folha quando os dados chegaram
  document.body.classList.remove("carregando");
  workspaceAtual = workspaceDaRota(arquivo, location.hash);
  D = workspaceAtual?.dados || arquivo;
  mcpWorkspaceMudou();
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
  const q = norm(consulta.trim());
  hoje = hojeSimulado || local(new Date());
  const hojeD = dia(hoje);
  const foco = tela ? D.blueprints.find((p) => p.id === tela) : null;
  const acoesTodas = (foco ? [foco] : D.blueprints).flatMap(acoesDe);
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
  const mapa = q ? todos.filter((x) => casa(x.p, q)) : todos;
  const total = 1 + D.blueprints.length;
  document.body.classList.toggle("em-blueprint", !!atual);
  if (criandoWorkspace) {
    host.innerHTML = "";
    if (desenrolarNaProxima && nomeUsuario) {
      atualizarCriacaoWorkspace();
      desenrolarNaProxima = false;
      desenrolar($("workspace-nova"), $("workspace-nova"));
    }
    return;
  }
  if (naEntrada) {
    $("entrada-marca").textContent = "Workspaces";
    $("entrada-titulo").textContent = "Seus workspaces";
    $("entrada-descricao").textContent = "Escolha um workspace para continuar.";
    $("entrada-voltar").hidden = true;
    $("entrada-novo").hidden = true;
    const workspaces = workspacesDe(arquivo);
    $("entrada-status").textContent = falhou ? "Não foi possível atualizar os workspaces. Tentando novamente…" : `${workspaces.length} workspace${workspaces.length === 1 ? "" : "s"} · arraste para reordenar (Alt + setas)`;
    $("entrada-blueprints").innerHTML = workspaces.map((c) => {
      const pecas = pecasWorkspace(c, EMB);
      return `<a class="blueprint-cartao" data-workspace="${esc(c.id)}" draggable="false" aria-keyshortcuts="Alt+ArrowLeft Alt+ArrowRight Alt+ArrowUp Alt+ArrowDown" title="Arraste para reordenar · Alt + setas no teclado" href="#/c/${encodeURIComponent(c.id)}"><span class="cartao-tipo">Workspace</span><div class="cartao-arte" aria-hidden="true">${pecas ? previaMarca(pecas, 280, 120) : ""}</div><h2 title="${esc(c.nome)}">${esc(c.nome)}</h2><p title="${esc(c.dados.descricao || "Blueprints e planejamento do workspace")}">${esc(c.dados.descricao || "Blueprints e planejamento do workspace")}</p><footer><span>${c.dados.blueprints.length} blueprint(s)</span><span>Abrir espaço →</span></footer></a>`;
    }).join("") + `<button class="blueprint-cartao cartao-adicionar" type="button" data-adicionar-workspace><span class="mais" aria-hidden="true">+</span><strong>Criar workspace</strong><span>Um espaço para agrupar blueprints.</span></button>`;
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
    const acoes = acoesDe(p).filter((f) => f.a < t1 && f.b >= t0).map((f) => {
      const rotulo = `${pi + 1}.${f.idx + 1} · ${f.nome || "SEM NOME"}`;
      const w = rotulo.length * CHAR + 26 + (f.prototipos?.length ? 26 : 0);
      const x0 = X(Math.max(f.a, t0)), x1 = X(Math.min(f.b + 1, t1));
      const nt = f.planos.length, nm = f.checkpoints.length, ncol = (nt ? 1 : 0) + (nm ? 1 : 0);
      const blockW = ncol ? ncol * COLW + (ncol - 1) * 30 : 0;
      const lx = Math.max(L, Math.min(x0, R - w));
      const bx = Math.max(L, Math.min(lx, R - blockW));
      const bw = Math.max(x1 - x0, BARRA_MIN), bx0 = Math.max(L, Math.min(x0, R - bw)); // barra aberta: começa na data de início, com largura mínima
      return { ...f, rotulo, x0, x1, bw, bx0, lx, lw: w, bx, blockW, alt: ncol ? 100 + 34 + Math.max(nt, nm) * 22 + 44 : RH };
    });
    const aberto = abertos[p.id] === true;
    const nl = aberto
      ? linhas(acoes, (f) => Math.min(f.lx, f.bx, f.bx0), (f) => Math.max(f.lx + f.lw, f.bx0 + f.bw + 56, f.bx + f.blockW) + 24)
      : linhas(acoes, (f) => f.x0, (f) => f.x1 + 6);
    const alts = Array(nl).fill(RH);
    acoes.forEach((f) => { alts[f.linha] = Math.max(alts[f.linha], f.alt); });
    const rowY = [];
    let corpo = 0;
    alts.forEach((h, i) => { rowY[i] = corpo; corpo += h; });
    const alto = (!acoes.length ? PH + 36 : PH + (aberto ? corpo : nl * 24 + 10)) + FOLGA;
    bloco = { p, pi, ok, acoes, aberto, y: y0, alto, rowY, ...resumo(acoesDe(p)) };
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
    out.push(`<path class="ln" d="M${L} ${yRuler}H${R}"/>`);
    for (let m = 0; m <= NM; m++) {
      const x = X(mesD(m));
      out.push(`<path class="ln" d="M${x} ${yRuler - 10}V${yRuler}"/><path class="guia" d="M${x} ${yRuler}V${yBand - 20}"/>`);
      if (m < NM) out.push(`<g class="lk" role="button" tabindex="0" data-mes="${mesIso(janela.inicio + m)}" aria-label="Selecionar ${mesIso(janela.inicio + m)}" aria-pressed="${periodos[tela]?.aguardando && periodos[tela].inicio === mesIso(janela.inicio + m) ? "true" : "false"}"><rect fill="${periodos[tela]?.aguardando && periodos[tela].inicio === mesIso(janela.inicio + m) ? "#ffffff33" : "transparent"}" x="${x}" y="${yRuler - 32}" width="${X(mesD(m + 1)) - x}" height="30"/><text x="${(x + X(mesD(m + 1))) / 2}" y="${yRuler - 16}" font-size="11" text-anchor="middle">${MESES[(mesInicial + m) % 12]}</text></g>`);
      if (m < NM && (m === 0 || (mesInicial + m) % 12 === 0)) out.push(`<text class="mid" x="${x + 3}" y="${yRuler - 34}" font-size="10">${ymin + Math.floor((mesInicial + m) / 12)}</text>`);
    }
    for (let d = t0 + 7; d < t1; d += 7) out.push(`<path class="ln m" d="M${X(d)} ${yRuler - 4}V${yRuler}"/>`);
    if (hojeD >= t0 && hojeD < t1) out.push(`<path class="eixo" d="M${xt} ${yRuler}V${yBand - 20}"/>`);
    if (hojeD >= t0 && hojeD < t1) out.push(`<g><rect x="${xt - 46}" y="${yRuler + 8}" width="92" height="20" fill="#eaf4ff"/><text x="${xt}" y="${yRuler + 22}" font-size="11" font-weight="700" text-anchor="middle" class="hoje-t" style="fill:#0c3a78">HOJE ${fmt(hoje)}</text></g>`);
  };
  const rodape = (out, H, folha, desenho) => {
    const yBand = H - IL - baixo, ly = yBand - 4;
    const cw = 440, ch = 96, cx = R - cw + 16, cy2 = H - IL - ch - 4;
    out.push(`<path class="ln" d="M${L} ${yBand - 8}H${R}"/><text class="mid cap" x="${L}" y="${ly + 18}" font-size="10">Legenda</text>`);
    out.push(`<rect class="ln" x="${L}" y="${ly + 34}" width="34" height="14"/><rect class="hach" x="${L + 1}" y="${ly + 35}" width="32" height="12"/><text x="${L + 44}" y="${ly + 45}" font-size="11">CONCLUÍDO</text>`);
    out.push(`<rect class="ln" style="stroke-width:1.8" x="${L + 150}" y="${ly + 34}" width="34" height="14"/><rect class="prog" x="${L + 151}" y="${ly + 35}" width="20" height="12"/><text x="${L + 194}" y="${ly + 45}" font-size="11">EM DESENVOLVIMENTO · cheio = progresso</text>`);
    out.push(`<rect class="ln m" style="stroke-dasharray:6 4" x="${L + 530}" y="${ly + 34}" width="34" height="14"/><text x="${L + 574}" y="${ly + 45}" font-size="11">PLANEJADO</text>`);
    out.push(`<path class="eixo" d="M${L} ${ly + 72}h34"/><text x="${L + 44}" y="${ly + 76}" font-size="11">HOJE</text>`);
    out.push(`<path class="dm" d="M${L + 150} ${ly + 66}l6 6l-6 6l-6 -6z"/><text x="${L + 164}" y="${ly + 76}" font-size="11">CHECKPOINT (cheio = feito)</text>`);
    out.push(`<g transform="translate(${L + 400} ${ly + 63})"><path style="fill:#0c3a78;stroke:#eaf4ff;stroke-width:1.2" d="M0 0h9l5 5v11h-14z"/><path style="fill:none;stroke:#eaf4ff;stroke-width:1.2;stroke-linecap:round" d="M9 0v5h5M3 9h8M3 13h6"/></g><text x="${L + 424}" y="${ly + 76}" font-size="11">OBSERVAÇÕES (clique para abrir)</text>`);
    const selecionado = atual ? "Marque planos e checkpoints clicando neles · passe o mouse sobre uma ação para ver os detalhes." : "Clique num blueprint para abrir o blueprint dele.";
    const maxSel = Math.max(1, Math.floor((cx - L - 18) / 7.7));
    out.push(`<text class="sel mid" data-max="${maxSel}" x="${L}" y="${ly + 100}" font-size="11">${esc(corta(selecionado, maxSel))}</text>`);
    out.push(`<rect class="ln" style="stroke-width:1.6" x="${cx}" y="${cy2}" width="${cw}" height="${ch}"/>`);
    out.push(`<path class="ln m" d="M${cx} ${cy2 + 36}H${cx + cw}M${cx} ${cy2 + 66}H${cx + cw}M${cx + 150} ${cy2 + 66}V${cy2 + ch}M${cx + 250} ${cy2 + 66}V${cy2 + ch}M${cx + 340} ${cy2 + 66}V${cy2 + ch}"/>`);
    out.push(`<text class="mid cap" x="${cx + 8}" y="${cy2 + 11}" font-size="8">Empresa</text><text x="${cx + 8}" y="${cy2 + 28}" font-size="16" font-weight="700" class="cap">${esc(corta(D.empresa, 28))}</text>`);
    out.push(`<text class="mid cap" x="${cx + 8}" y="${cy2 + 47}" font-size="8">Desenho</text><text x="${cx + 8}" y="${cy2 + 61}" font-size="12" font-weight="700" class="cap">${esc(corta(desenho, 38))}</text>`);
    [["Data", hoje.split("-").reverse().join("/"), 0], ["Rev.", D.rev, 150], ["Folha", `${folha}/${total}`, 250], ["Dados", D.exemplo ? "EXEMPLO" : "REAIS", 340]].forEach(([k, v, off]) => {
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
    const info = `MARCA · ${pct}% CONSTRUÍDA (${e.feitas} de ${e.total} peças)`;
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
    out.push(marcaFolha(pecasWorkspace(workspaceAtual, EMB), { centro: H / 2, maxLado: MINIMA * 0.55, progresso: geralTodos, titulo: D.empresa || "Empresa", fundo: true })); // a marca do workspace acompanha o progresso dos seus blueprints como uma marca d'água de fundo
    out.push(`<text x="${L}" y="${IL + 36}" font-size="18" font-weight="700" class="cap">${esc(D.empresa)} · ${esc(D.titulo)}</text>`);
    out.push(`<text class="mid cap" x="${L}" y="${IL + 56}" font-size="11">Blueprints · cada um abre o seu próprio blueprint</text>`);
    out.push(`<text class="mid cap" x="${L}" y="${yMapa + 6}" font-size="10">Clique para abrir o blueprint · segure e arraste para reordenar</text>`);
    const legendaY = yMapa + 30;
    out.push(`<text class="mid cap" x="${L}" y="${legendaY}" font-size="8">BARRA = AÇÃO · PREENCHIDO = FEITO · VAZIO = FALTA</text>`);
    out.push(`<rect class="ln" x="${L + 270}" y="${legendaY - 8}" width="22" height="8"/><rect class="hach" x="${L + 271}" y="${legendaY - 7}" width="20" height="6"/><text class="mid cap" x="${L + 298}" y="${legendaY}" font-size="8">CONCLUÍDA</text>`);
    out.push(`<rect class="ln" style="stroke-width:1.8" x="${L + 360}" y="${legendaY - 8}" width="22" height="8"/><rect class="prog" x="${L + 361}" y="${legendaY - 7}" width="10" height="6"/><text class="mid cap" x="${L + 388}" y="${legendaY}" font-size="8">EM ANDAMENTO</text>`);
    out.push(`<rect class="ln m" style="stroke-dasharray:6 4" x="${L + 470}" y="${legendaY - 8}" width="22" height="8"/><text class="mid cap" x="${L + 498}" y="${legendaY}" font-size="8">A FAZER</text>`);
    mapa.forEach(({ p, pi, ok }, i) => {
      const col = i % cols, row = Math.floor(i / cols);
      const tx = L + col * (tw + GAP), ty = yMapa + 48 + row * (TH + GAP);
      const fs = acoesDe(p), r = resumo(fs);
      const nv = nova("t:" + p.id);
      const artW = Math.min(116, Math.max(76, Math.round(tw * 0.25))), artX = tx + 12, artY = ty + 26, artH = TH - 52;
      const textoX = artX + artW + 14, textoW = tx + tw - 16 - textoX;
      const nome = corta(p.nome || "SEM NOME", Math.max(8, Math.floor(textoW / 10.2)));
      const descricao = corta(p.descricao || "", Math.max(12, Math.floor(textoW / 7.5)));
      const info = !p.acoes.length ? "SEM AÇÕES AINDA" : ok ? `✓ CONCLUÍDO · ${p.acoes.length} AÇÕES` : r.futuro ? "PLANEJADO" : `${r.concl}/${p.acoes.length} AÇÕES · ${Math.round(r.geral * 100)}% · FALTAM ${fs.reduce((t, f) => t + f.it[1] - f.it[0], 0)} ITENS`;
      const aria = `Blueprint ${pi + 1}, ${p.nome}: ${info}. Abrir o blueprint. Segure e arraste para reordenar`;
      out.push(`<g data-goto="${esc(p.id)}" class="alvo${ok ? " ok" : ""}" tabindex="0" role="button" aria-label="${esc(aria)}">`);
      out.push(`<rect class="moldura" x="${tx}" y="${ty}" width="${tw}" height="${TH}"/>`);
      out.push(`<g ${anima(nv, "surge")}>`);
      out.push(arteCartao(p.emblema?.pecas?.length ? p.emblema.pecas : EMB?.CONSTRUCAO, artX, artY, artW, artH, r.geral));
      out.push(`<text x="${textoX}" y="${ty + 44}" font-size="15" font-weight="700" class="cap">${esc(nome)}</text>`);
      out.push(`<text class="mid cap" x="${textoX}" y="${ty + 70}" font-size="11">${esc(descricao)}</text>`);
      out.push(`<text class="mid" x="${textoX}" y="${ty + 95}" font-size="10">${esc(corta(info, Math.floor(textoW / 7)))}</text>`);
      if (fs.length) {
        // como numa folha de projeto: a barra é o blueprint inteiro (hachurado = feito, vazio = falta) e, embaixo, a linha de cota
        // com as datas de início e fim, o "hoje" e os checkpoints em losango (cheio = feito)
        const x0 = textoX, x1 = tx + tw - 16, yb = ty + 106, yc = ty + 130;
        const ini = Math.min(...fs.map((f) => f.a)), fim = Math.max(...fs.map((f) => f.b + 1));
        const px = (d) => x0 + ((d - ini) / (fim - ini || 1)) * (x1 - x0), fmt = (d) => deDia(d).slice(8) + "/" + deDia(d).slice(5, 7);
        // uma célula por ação, juntas por situação (concluídas | em andamento | a fazer), com um divisor mais largo entre os grupos; a largura é a quantidade de itens
        const ordem = { concluido: 0, andamento: 1, planejado: 2 }, cel = [...fs].sort((u, v) => ordem[u.status] - ordem[v.status] || u.a - v.a);
        const vao = (i) => (i && cel[i].status !== cel[i - 1].status ? 7 : 2), peso = (f) => Math.max(1, f.it[1]);
        const total = cel.reduce((t, f) => t + peso(f), 0), util = x1 - x0 - cel.reduce((t, _, i) => t + (i ? vao(i) : 0), 0);
        let cx = x0;
        cel.forEach((f, i) => {
          if (i) { cx += vao(i); if (vao(i) > 2) out.push(`<path class="eixo" style="stroke-dasharray:none" d="M${cx - 3.5} ${yb - 3}V${yb + 15}"/>`); }
          const w = util * peso(f) / total, estilo = f.status === "concluido" ? "" : f.status === "andamento" ? ' style="stroke-width:1.8"' : ' style="stroke-dasharray:6 4"';
          const miolo = f.status === "concluido" ? `<rect class="hach" x="${cx + 1}" y="${yb + 1}" width="${Math.max(0, w - 2)}" height="10"/>` : f.pct > 0 ? `<rect class="prog" x="${cx + 1}" y="${yb + 1}" width="${Math.max(0, (w - 2) * f.pct)}" height="10"/>` : "";
          out.push(`<g><title>${esc(`${pi + 1}.${f.idx + 1} ${f.nome || "SEM NOME"} · ${f.status === "concluido" ? "concluída" : f.status === "andamento" ? `em andamento, ${Math.round(f.pct * 100)}% feito` : "a fazer"}`)}</title><rect class="ln${f.status === "planejado" ? " m" : ""}"${estilo} x="${cx}" y="${yb}" width="${w}" height="12"/>${miolo}</g>`);
          cx += w;
        });
        out.push(`<path class="ln l" d="M${x0} ${yc}H${x1}M${x0} ${yc - 4}V${yc + 4}M${x1} ${yc - 4}V${yc + 4}"/>`);
        fs.forEach((f) => f.checkpoints.forEach((m) => { const d = dia(m.data); if (d >= ini && d <= fim) out.push(`<path class="dm${m.feito ? " ok" : ""}" d="M${px(d)} ${yc - 5}l4 5l-4 5l-4 -5z"/>`); }));
        if (hojeD >= ini && hojeD <= fim) out.push(`<path class="eixo" style="stroke-dasharray:none" d="M${px(hojeD)} ${yc - 7}V${yc + 7}"/>`);
        out.push(`<text class="mid" x="${x0}" y="${ty + 146}" font-size="8">${fmt(ini)}</text><text class="mid" x="${x1}" y="${ty + 146}" font-size="8" text-anchor="end">${fmt(fim - 1)}</text>`);
      }
      const ag = agenteAqui();
      if (ag?.blueprintId === p.id) out.push(roboSvg(tx + tw - 30, ty + 12, ag));
      out.push(`</g></g>`);
      if (nv) n++;
    });
    const novoIndice = mapa.length, novaColuna = novoIndice % cols, novaLinha = Math.floor(novoIndice / cols);
    const nx = L + novaColuna * (tw + GAP), ny = yMapa + 48 + novaLinha * (TH + GAP);
    const artW = Math.min(108, Math.max(76, Math.round(tw * 0.24))), artX = nx + 12, artY = ny + 26, artH = TH - 52;
    const textoX = artX + artW + 14, plusX = nx + tw - 56, textoW = plusX - 12 - textoX;
    out.push(`<g data-adicionar-blueprint tabindex="0" role="button" aria-label="Criar novo blueprint" class="alvo criar"><rect class="moldura" x="${nx}" y="${ny}" width="${tw}" height="${TH}"/>${arteCartao(EMB?.CONSTRUCAO, artX, artY, artW, artH)}<text class="cap" x="${textoX}" y="${ny + 68}" font-size="12">${esc(corta("NOVO BLUEPRINT", Math.floor(textoW / 7.2)))}</text><text class="mid cap" x="${textoX}" y="${ny + 94}" font-size="10">${esc(corta("ADICIONAR BLUEPRINT", Math.floor(textoW / 6)))}</text><rect class="mais-borda" x="${plusX}" y="${ny + 58}" width="36" height="36"/><path class="mais-novo" d="M${plusX + 10} ${ny + 76}h16M${plusX + 18} ${ny + 68}v16"/></g>`);
    if (falhou) out.push(`<text class="aviso cap" x="${L}" y="${yMapa + 40}" font-size="12" font-weight="700">Servidor indisponível · abra o The Blueprint (beta) (blueprint.exe) e use http://127.0.0.1:3000</text>`);
    rodape(out, H, 1, "Lista de blueprints");
    return folhaSvg(H, out, `Lista de blueprints da ${D.empresa}`);
  };

  // ---- blueprint de um blueprint -----------------------------------------------------------------
  const folhaBlueprint = (b) => {
    const numero = b.pi + 2;
    const H = alturaProj(b), out = [];
    const { p, pi, y: yb } = b;
    const agP = agenteAqui()?.blueprintId === p.id ? agenteAgora : null;
    let roboPosto = false;
    moldura(out, H);
    out.push(marcaFolha(p.emblema?.pecas, { centro: H / 2, progresso: b.geral, titulo: p.nome || "Blueprint", blueprint: p.id })); // a marca deste blueprint, sempre no centro da janela, construída conforme o progresso
    out.push(`<text x="${L}" y="${IL + 36}" font-size="18" font-weight="700" class="cap">${esc(D.empresa)} · ${esc(D.titulo)}</text>`);
    out.push(`<text class="mid cap" x="${L}" y="${IL + 56}" font-size="11">Folha ${numero} de ${total} · Blueprint ${pad(pi + 1)} · ${esc(corta(p.nome || "Sem nome", 60))}</text>`);
    regua(out, H);
    const nh = nova("h:" + p.id);
    const nomeH = corta(p.nome || "Sem nome", 48), descH = corta(p.descricao || "", 70);
    out.push(`<g data-proj="${esc(p.id)}">`);
    out.push(`<g data-toggle="${esc(p.id)}" class="alvo" tabindex="0" role="button" aria-expanded="${b.aberto}" aria-label="${esc(`${p.nome}: ${b.aberto ? "recolher" : "expandir"} ações`)}"><rect class="cab" x="${L - 8}" y="${yb - 8}" width="${R - L + 16}" height="86"/>`);
    out.push(`<g ${anima(nh, "surge")}><circle class="ln" cx="${L + 14}" cy="${yb + 20}" r="14"/><text x="${L + 14}" y="${yb + 25}" font-size="13" font-weight="700" text-anchor="middle">${pad(pi + 1)}</text>`);
    out.push(`<text x="${L + 40}" y="${yb + 19}" font-size="17" font-weight="700" class="cap">${esc(nomeH)}</text><text class="mid cap" x="${L + 40}" y="${yb + 38}" font-size="11">${esc(descH)}</text></g>`);
    // medidor logo depois do nome, longe da linha de eixo "hoje"
    const px = Math.min(L + 40 + Math.max(nomeH.length * 11.6 + (p.obs?.trim() ? 30 : 0), descH.length * 7.2) + 64, R - 330);
    const contagem = `${b.concl}/${p.acoes.length} ações concluídas`;
    out.push(`<g ${anima(nh, "surge")}>` + (!p.acoes.length
      ? `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">Situação</text><text x="${px}" y="${yb + 32}" font-size="12" font-weight="700">SEM AÇÕES AINDA</text>`
      : b.ok
        ? `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">Situação · ${contagem}</text><text x="${px}" y="${yb + 32}" font-size="12" font-weight="700">✓ CONCLUÍDO · 100%</text>`
        : b.futuro
          ? `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">Situação</text><text x="${px}" y="${yb + 32}" font-size="12" font-weight="700">PLANEJADO</text>`
          : `<text class="mid cap" x="${px}" y="${yb + 14}" font-size="9">Progresso geral · ${contagem}</text><rect class="ln" x="${px}" y="${yb + 21}" width="190" height="10"/><rect class="prog" x="${px}" y="${yb + 21}" width="${190 * b.geral}" height="10"/><text x="${px + 200}" y="${yb + 31}" font-size="12" font-weight="700">${Math.round(b.geral * 100)}%</text>`) + `</g>`);
    out.push(`<path ${anima(nh, "desenha", "ln m")} pathLength="1" d="M${L} ${yb + 58}H${p.acoes.length ? R - 144 : R}"/>`);
    // botão grande com texto: some a dúvida do que fazer e dá um alvo fácil de acertar
    if (p.acoes.length) out.push(`<rect class="botao" x="${R - 136}" y="${yb + 44}" width="132" height="30"/><text class="botao-t cap" x="${R - 70}" y="${yb + 63}" font-size="11" text-anchor="middle">${b.aberto ? "− Recolher" : `+ Abrir ${p.acoes.length} ação${p.acoes.length > 1 ? "s" : ""}`}</text>`);
    out.push(`</g>`);
    // nota do blueprint: ao lado do nome, abre as observações na gaveta lateral
    if (p.obs?.trim()) out.push(notaSvg(L + 40 + nomeH.length * 11.6 + 10, yb + 3, `OBSERVAÇÃO DO BLUEPRINT: ${umaLinha(p.obs, 200)}`, `${p.id}|`));
    // links do blueprint: em até duas linhas, alinhados à direita, na ordem cadastrada
    const lks = p.links.filter((l) => urlSegura(l.url));
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
        chips.push(`<a class="lk" href="${esc(urlSegura(l.url))}" target="_blank" rel="noopener noreferrer" aria-label="${esc("Abrir " + rotuloDe(l) + " em nova aba")}"><rect fill="transparent" x="${x}" y="${ry}" width="${w}" height="18"/><rect class="foco" x="${x}" y="${ry}" width="${w}" height="18"/><text class="cap" x="${x + 6}" y="${ry + 13}" font-size="11">${esc(txt)}</text></a>`);
        x += w + 8;
      });
    });
    if (chips.length || livre) out.push(`<g ${anima(nh, "surge")}>${chips.join("")}${livre ? `<text class="mid" x="${R - 6}" y="${yb + 55}" font-size="10" text-anchor="end">+${livre} LINK${livre > 1 ? "S" : ""} · VEJA EM EDITAR</text>` : ""}</g>`);
    if (nh) n++;
    if (!p.acoes.length) out.push(`<text class="mid cap" x="${L + 40}" y="${yb + PH + 8}" font-size="11">Sem ações · abra Editar para adicionar</text>`);

    if (!b.acoes.length && p.acoes.length) out.push(`<text class="mid" x="${L}" y="${yb + PH + 30}" font-size="12">Nenhuma ação neste período.</text>`);
    b.acoes.forEach((f) => {
      const nf = nova((b.aberto ? "f:" : "c:") + f.id);
      const dias = Math.round(f.b - f.a) + 1;
      let w = Math.max(f.x1 - f.x0, 6);
      const extra = (f.tf[1] ? ` · ${f.tf[0]}/${f.tf[1]} PLANOS` : "") + (f.mf[1] ? ` · ${f.mf[0]}/${f.mf[1]} CHECKPOINTS` : "") + (f.obs?.trim() ? " · COM INSTRUÇÕES" : "");
      const abrir = b.aberto ? "" : " · CLIQUE PARA ABRIR AQUI"; // com as ações recolhidas, clicar abre e leva até o item
      const info = `${f.rotulo.toUpperCase()} · ${ROTULO[f.status].toUpperCase()} · ${Math.round(f.pct * 100)}% · ${fmt(f.inicio)} → ${fmt(f.fim)} · ${dias} dias${extra}${abrir}`;
      const aria = `${f.rotulo}, ${ROTULO[f.status]}, ${Math.round(f.pct * 100)}%, ${fmt(f.inicio)} a ${fmt(f.fim)}${extra.replaceAll("·", ",").toLowerCase()}`;
      const checkpoints = (cy, r, g) => (f.checkpoints || []).filter((m) => dia(m.data) >= t0 && dia(m.data) < t1).map((m) => {
        const mxp = g ? g.x0 + ((dia(m.data) + 0.5 - f.a) / (f.b + 1 - f.a)) * (g.x1 - g.x0) : Math.min(R, Math.max(L, X(dia(m.data) + 0.5)));
        const mi = `CHECKPOINT: ${(m.nome || "SEM NOME").toUpperCase()} · ${fmt(m.data)} · ${m.feito ? "FEITO" : "PENDENTE"}${abrir}`;
        return `<g class="checkpoint" data-m="${esc(m.id)}" data-info="${esc(mi)}"><circle cx="${mxp}" cy="${cy}" r="${r + 5}" fill="transparent"/><path class="dm${m.feito ? " ok" : ""}" d="M${mxp} ${cy - r}l${r} ${r}l${-r} ${r}l${-r} ${-r}z"/></g>`;
      }).join("");
      const dados = `class="acao" tabindex="0" role="group" aria-label="${esc(aria)}" data-st="${f.status}" data-p="${esc(p.id)}" data-f="${esc(f.id)}" data-info="${esc(info)}"`;
      if (!b.aberto) {
        const by = yb + PH - 12 + f.linha * 24;
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
      const fy = yb + PH + b.rowY[f.linha], by = fy + 20, bh = 26;
      out.push(`<g ${dados}>`);
      out.push(`<rect x="${Math.min(g.x0, f.lx) - 6}" y="${fy}" width="${Math.max(w + 12, f.lw + 12)}" height="${RH - 30}" fill="transparent"/>`);
      out.push(`<text x="${f.lx}" y="${fy + 12}" font-size="11" class="cap">${esc(f.rotulo)}</text>`);
      out.push(`<rect ${anima(nf, "desenha", "caixa")} pathLength="1" x="${g.x0}" y="${by}" width="${w}" height="${bh}"/>`);
      if (f.status === "concluido") {
        const cx = Math.max(g.x0 + 12, g.x1 - 17), cy = by + bh / 2;
        out.push(`<rect ${anima(nf, "surge", "hach")} x="${g.x0 + 1}" y="${by + 1}" width="${w - 2}" height="${bh - 2}"/>`);
        if (w >= 100) out.push(`<g ${anima(nf, "surge")}><circle cx="${cx}" cy="${cy}" r="8" fill="#0c3a78" stroke="#eaf4ff"/><path class="ln" d="M${cx - 3.6} ${cy}l2.6 3l4.6-6"/></g>`);
      } else if (f.status === "andamento") {
        out.push(`<rect ${anima(nf, "surge", "prog")} x="${g.x0 + 1}" y="${by + 1}" width="${Math.max(0, (w - 2) * f.pct)}" height="${bh - 2}"/>`);
      }
      // porcentagem da ação: vem dos planos e checkpoints marcados (ou do valor digitado, se a ação não tiver nenhum)
      out.push(`<text ${anima(nf, "surge")} x="${Math.min(g.x1 + 20, R + 4)}" y="${by + 17}" font-size="12" font-weight="700">${Math.round(f.pct * 100)}%</text>`);
      if (f.it[1]) out.push(`<text class="mid" ${anima(nf, "surge")} x="${Math.min(g.x1 + 20, R + 4)}" y="${by + 31}" font-size="9">${f.it[0]}/${f.it[1]}</text>`);
      out.push(checkpoints(by + bh / 2, 7, g));
      out.push(`<rect class="realce" x="${g.x0 - 4}" y="${by - 4}" width="${w + 8}" height="${bh + 8}"/>`);
      // cota: duração entre as duas datas, com traços oblíquos nas pontas
      const cy = fy + 72;
      out.push(`<g ${anima(nf, "surge")}><path class="ln m" d="M${g.x0} ${by + bh + 2}V${cy + 6}M${g.x1} ${by + bh + 2}V${cy + 6}M${g.x0} ${cy}H${g.x1}M${g.x0 - 4} ${cy + 4}l8 -8M${g.x1 - 4} ${cy + 4}l8 -8"/>`);
      out.push(`<text class="mid" x="${(g.x0 + g.x1) / 2}" y="${cy - 5}" font-size="10" text-anchor="middle">${dias} DIAS</text>`);
      out.push(datasCota(g, cy));
      // planos e checkpoints da ação: caixas de marcar direto no blueprint
      if (f.planos.length || f.checkpoints.length) {
        const base = fy + 112, maxc = Math.floor((COLW - 28) / 7.3);
        const cab = (x, titulo, feitos, tot) => `<text class="mid cap" x="${x}" y="${base}" font-size="9">${titulo} · ${feitos}/${tot}</text><path class="ln l" d="M${x} ${base + 7}h${COLW - 20}"/>`;
        const atr = (tipo, it, info2) => `class="chk${it.feito ? " ok" : ""}" tabindex="0" role="checkbox" aria-checked="${!!it.feito}" aria-label="${esc(info2)}" data-chk="${esc(it.id)}" data-tipo="${tipo}" data-p="${esc(p.id)}" data-f="${esc(f.id)}" data-id="${esc(it.id)}" data-info="${esc(info2)}"`;
        const gl = [];
        let cx0 = f.bx;
        if (f.planos.length) {
          gl.push(cab(cx0, "Planos", f.tf[0], f.tf[1]));
          f.planos.forEach((t, i) => {
            const yy = base + 30 + i * 22, nome = t.texto || "sem texto", horas = t.horas ? ` ~${t.horas} h` : "";
            gl.push(`<g ${atr("t", t, `PLANO: ${nome} · ${t.feito ? "FEITO" : "PENDENTE"} · clique para marcar`)}><rect fill="transparent" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/><rect class="foco2" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/><rect class="cx${t.feito ? " ok" : ""}" x="${cx0}" y="${yy - 11}" width="13" height="13"/>${t.feito ? `<path class="cxm" d="M${cx0 + 3} ${yy - 4}l3 3l5-6"/>` : ""}<text class="tx" x="${cx0 + 22}" y="${yy}" font-size="11">${esc(corta(nome, maxc - horas.length) + horas)}</text></g>`);
            if (agP?.planoId === t.id) { gl.push(`<rect class="robo-linha${roboClasse(agP)}" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/>`, roboSvg(cx0 - 30, yy - 12, agP)); roboPosto = true; }
          });
          cx0 += COLW + 30;
        }
        if (f.checkpoints.length) {
          gl.push(cab(cx0, "Checkpoints", f.mf[0], f.mf[1]));
          f.checkpoints.forEach((m, i) => {
            const yy = base + 30 + i * 22, nome = m.nome || "sem nome", dt = dia(m.data) === dia(m.data) ? " · " + fmt(m.data) : "";
            gl.push(`<g ${atr("m", m, `CHECKPOINT: ${nome}${dt} · ${m.feito ? "FEITO" : "PENDENTE"} · clique para marcar`)}><rect fill="transparent" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/><rect class="foco2" x="${cx0 - 4}" y="${yy - 15}" width="${COLW - 12}" height="22"/><path class="dm${m.feito ? " ok" : ""}" d="M${cx0 + 6.5} ${yy - 12}l6.5 6.5l-6.5 6.5l-6.5 -6.5z"/><text class="tx" x="${cx0 + 22}" y="${yy}" font-size="11">${esc(corta(nome + dt, maxc))}</text></g>`);
          });
        }
        out.push(`<g ${anima(nf, "surge")}>${gl.join("")}</g>`);
      }
      if (f.prototipos?.length) out.push(protoSvg(f.lx + (f.rotulo.length * CHAR) + 36, fy - 1, `PROTÓTIPO${f.prototipos.length > 1 ? "S" : ""} DA AÇÃO ${f.rotulo.toUpperCase()}: ${f.prototipos.map((x) => x.titulo.toUpperCase()).join(" · ")} · clique para ver`, `${p.id}|${f.id}`));
      out.push(notaSvg(f.lx + (f.rotulo.length * CHAR) + 10, fy - 1, `INSTRUÇÕES DA AÇÃO ${f.rotulo.toUpperCase()}: ${f.obs?.trim() ? umaLinha(f.obs, 200) : "ainda sem instruções · clique para escrever"}`, `${p.id}|${f.id}`));
      out.push(`</g>`);
      if (nf) n++;
    });
    if (agP && !roboPosto) out.push(roboSvg(R - 166, yb + 50, agP)); // sem o plano à vista: o robô fica no cabeçalho do blueprint
    out.push(`</g>`);
    rodape(out, H, numero, p.nome || "Sem nome");
    return folhaSvg(H, out, `Blueprint ${p.nome}`);
  };

  host.innerHTML = bloco ? folhaBlueprint(bloco) : capa();
  posicionaMarca();
  host.dataset.rotulo = bloco ? `FOLHA ${bloco.pi + 2}/${total}` : "BLUEPRINTS";
  const alvo = [...host.querySelectorAll(".acao")].find((g) => g.getAttribute("aria-label") === selecionada);
  if (alvo) marcar(alvo);
  if (desenrolarNaProxima) { desenrolarNaProxima = false; desenrolar(); sugereF11(bloco ? "blueprint" : "lista"); }
  montaRobo();
  if (roboRolar) { roboRolar = false; requestAnimationFrame(() => host.querySelector("[data-robo]")?.scrollIntoView({ block: "center", behavior: reduz() ? "auto" : "smooth" })); }

  // busca de blueprints sobre a lista
  $("busca-n").textContent = q ? (mapa.length ? `${mapa.length} resultado${mapa.length > 1 ? "s" : ""}` : "nenhum resultado") : "";
  requestAnimationFrame(atualizaRolagem);
}

