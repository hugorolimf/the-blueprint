// ---- uma tela de terminal: carrega o xterm.js sob demanda, liga a tela ao SSE da sessão e manda as teclas por POST ----
// O xterm.js (~290 KB) só chega na primeira vez que o terminal abre: import() do arquivo embutido em web/vendor/xterm. Ele é UMD; importado
// como módulo (sem `module` nem `define`), se põe em globalThis.Terminal. Quem usa: terminal.js (o painel). Lógica pura: terminal-logica.js.
let termCarregado = null;
function termCarregar() {
  termCarregado ||= (async () => {
    const css = document.createElement("link");
    css.rel = "stylesheet";
    css.href = "/web/vendor/xterm/xterm.css";
    document.head.append(css);
    await import("/web/vendor/xterm/xterm.js");
    await import("/web/vendor/xterm/addon-fit.js");
    return { Terminal: globalThis.Terminal, FitAddon: globalThis.FitAddon.FitAddon };
  })().catch((e) => { termCarregado = null; throw e; });
  return termCarregado;
}

const termPost = (rota, corpo) => fetch(`/api/terminal/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });

// links só para a web; a janela do app manda window.open para o navegador do sistema (src/janela)
function termAbrirLink(url) {
  if (/^https?:\/\//i.test(url)) window.open(url, "_blank", "noopener");
}

// Cria a tela da sessão dentro de `el`. `aoTecla(acao)` recebe os atalhos do painel (termTecla); `aoFim(codigo)` avisa que o processo saiu.
async function termVista(sessao, el, { fonte, leitor, aoTecla, aoFim }) {
  const { Terminal, FitAddon } = await termCarregar();
  const css = getComputedStyle(document.documentElement), tok = (n) => css.getPropertyValue(`--${n}`).trim();
  const term = new Terminal({
    theme: termTema({ paper: tok("paper"), ink: tok("ink"), mid: tok("mid"), lo: tok("lo"), erro: tok("erro") }),
    fontFamily: tok("mono") || "Consolas, monospace", fontSize: termFonteValida(fonte), lineHeight: 1.15,
    cursorBlink: false, // piscar é trabalho em repouso (um relógio por tela); o cursor fica fixo
    minimumContrastRatio: 4.5, screenReaderMode: !!leitor, scrollback: 5000, allowProposedApi: false,
    linkHandler: { activate: (_, uri) => termAbrirLink(uri) }, // links OSC 8 que o Claude escreve
  });
  const fit = new FitAddon();
  term.loadAddon(fit);
  term.open(el);
  term.textarea?.setAttribute("aria-label", tr("Terminal da sessão {titulo}. F6 sai do terminal.", { titulo: sessao.titulo || sessao.id }));
  term.registerLinkProvider({ // URLs em texto puro (sem OSC 8)
    provideLinks(y, devolver) {
      const linha = term.buffer.active.getLine(y - 1)?.translateToString(true) || "";
      devolver(termLinks(linha).map((l) => ({ range: { start: { x: l.inicio + 1, y }, end: { x: l.fim, y } }, text: l.url, activate: () => termAbrirLink(l.url) })));
    },
  });
  const escrever = termEnviador((dados) => termPost("escrever", { id: sessao.id, dados }));
  term.onData(escrever);
  term.onBinary(escrever);
  term.attachCustomKeyEventHandler((e) => {
    const acao = termTecla(e, term.hasSelection());
    if (!acao || acao === "colar") return !acao; // colar: o navegador cola e o xterm recebe o evento
    e.preventDefault();
    if (acao === "copiar") {
      navigator.clipboard?.writeText(term.getSelection()).catch(() => {});
      term.clearSelection();
    } else aoTecla?.(acao);
    return false;
  });
  let enviado = "";
  const ajustar = () => {
    if (!el.isConnected || !el.offsetWidth) return;
    try { fit.fit(); } catch {}
    const k = `${term.cols}x${term.rows}`;
    if (k !== enviado) { enviado = k; termPost("tamanho", { id: sessao.id, colunas: term.cols, linhas: term.rows }).catch(() => {}); }
  };
  let quadro = 0;
  const vigia = new ResizeObserver(() => { cancelAnimationFrame(quadro); quadro = requestAnimationFrame(ajustar); });
  vigia.observe(el);
  const sse = new EventSource(`/api/terminal/fluxo?id=${encodeURIComponent(sessao.id)}`);
  sse.addEventListener("historico", (e) => { term.reset(); term.write(JSON.parse(e.data)); }); // a cada (re)conexão: a tela recomeça do histórico
  sse.onmessage = (e) => term.write(JSON.parse(e.data));
  sse.addEventListener("fim", (e) => {
    sse.close();
    const { codigo } = JSON.parse(e.data);
    term.write(`\r\n\x1b[2m[${tr("a sessão terminou")}]\x1b[0m\r\n`);
    aoFim?.(codigo);
  });
  sse.onerror = () => { if (sse.readyState === EventSource.CLOSED) aoFim?.(null); }; // 404: a sessão saiu do servidor
  return {
    term,
    ajustar,
    focar: () => term.focus(),
    fonte(n) { term.options.fontSize = termFonteValida(n); ajustar(); },
    leitor(ligado) { term.options.screenReaderMode = !!ligado; },
    destruir() { vigia.disconnect(); cancelAnimationFrame(quadro); sse.close(); term.dispose(); el.replaceChildren(); },
  };
}
