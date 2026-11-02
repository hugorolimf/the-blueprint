// ---- o terminal do app, parte pura (sem DOM): tema a partir dos tokens, rótulo e estado da aba, links, atalhos e a fila de teclas ----
// Testada em terminal-logica.test.mjs. O painel fica em terminal.js e cada tela de terminal em terminal-sessao.js (decisão em docs/terminal-no-app.md).

// As cores do app (web/css/tokens.css) viram o tema do xterm. A paleta ANSI é clara de propósito: o fundo é o azul do blueprint, e o
// `minimumContrastRatio` do xterm ainda clareia o que o Claude pintar com pouco contraste (WCAG AA, 4,5:1).
function termTema(c = {}) {
  const papel = c.paper || "#0c3a78", tinta = c.ink || "#eaf4ff", meio = c.mid || "#9cc4ee", baixo = c.lo || "#5b93d0", erro = c.erro || "#ffb4a8";
  return {
    background: papel, foreground: tinta, cursor: tinta, cursorAccent: papel, selectionBackground: "rgba(156,196,238,0.38)", selectionInactiveBackground: "rgba(156,196,238,0.22)",
    black: "#0a2a58", red: erro, green: "#9fe3b4", yellow: "#ffd98a", blue: meio, magenta: "#e2b6ff", cyan: "#8de8f0", white: "#dbe9fb",
    brightBlack: baixo, brightRed: "#ffd0c8", brightGreen: "#c4f2d2", brightYellow: "#ffe9b8", brightBlue: "#c8e0fa", brightMagenta: "#f0d6ff", brightCyan: "#c2f4f8", brightWhite: "#ffffff",
  };
}

const TERM_FONTE = { min: 10, max: 28, padrao: 14 };
const termFonteValida = (n) => Math.min(TERM_FONTE.max, Math.max(TERM_FONTE.min, Math.round(Number(n) || TERM_FONTE.padrao)));

// O texto da aba e o estado dela: o título que a página mandou ao abrir; o estado vem dos hooks (o último aviso do agente nessa ação) e,
// sem aviso, diz só se o processo ainda roda. `estados` é o ESTADO_AG de agentes.js (rótulos traduzidos).
function termRotulo(sessao, agentes = [], estados = {}) {
  const texto = sessao.titulo || tr("Sessão {id}", { id: sessao.id });
  if (!sessao.viva) return { texto, estado: sessao.codigo == null ? tr("Terminou") : tr("Terminou (código {codigo})", { codigo: sessao.codigo }), viva: false };
  const dela = agentes.filter((a) => a.acaoId && a.acaoId === sessao.acaoId && (!sessao.blueprintId || a.blueprintId === sessao.blueprintId)).sort((a, b) => (Date.parse(b.em) || 0) - (Date.parse(a.em) || 0))[0];
  return { texto, estado: dela ? estados[dela.estado] || dela.estado : tr("Rodando"), viva: true };
}

// http(s) no texto de uma linha do terminal, sem a pontuação que fecha a frase; devolve {inicio, fim, url} com posições de 0 em diante
function termLinks(linha) {
  const achados = [];
  for (const m of String(linha).matchAll(/https?:\/\/[^\s"'<>`]+/g)) {
    const url = m[0].replace(/[.,;:!?)\]}]+$/, "");
    if (url.length > 8) achados.push({ inicio: m.index, fim: m.index + url.length, url });
  }
  return achados;
}

// O que uma tecla faz antes de chegar ao Claude. Esc e Ctrl+C são do Claude (interromper, voltar), então sair do terminal é F6 (o atalho
// padrão de trocar de painel); Ctrl+C só copia quando há seleção. Devolve o nome da ação ou "" (a tecla segue para o terminal).
function termTecla(e, temSelecao = false) {
  if (e.type && e.type !== "keydown") return "";
  const ctrl = e.ctrlKey && !e.altKey && !e.metaKey;
  if (e.key === "F6" && !ctrl) return e.shiftKey ? "sair-voltar" : "sair";
  if (ctrl && e.key === "PageDown") return "proxima";
  if (ctrl && e.key === "PageUp") return "anterior";
  if (ctrl && !e.shiftKey && (e.key === "=" || e.key === "+")) return "maior";
  if (ctrl && !e.shiftKey && e.key === "-") return "menor";
  if (ctrl && !e.shiftKey && e.key === "0") return "normal";
  if (ctrl && e.key === "`") return "painel";
  if (ctrl && (e.key === "c" || e.key === "C") && (e.shiftKey || temSelecao)) return "copiar";
  if (ctrl && (e.key === "v" || e.key === "V")) return "colar"; // deixa o navegador colar (o xterm recebe o evento de colar, com bracketed paste)
  return "";
}

// A fila das teclas: um POST por vez, na ordem; o que for digitado enquanto um POST anda vai junto no próximo. `enviar(texto)` devolve uma Promise.
function termEnviador(enviar) {
  let pendente = "", andando = false;
  const proximo = async () => {
    if (andando || !pendente) return;
    andando = true;
    const texto = pendente;
    pendente = "";
    try { await enviar(texto); } catch {} finally { andando = false; proximo(); }
  };
  return (texto) => { pendente += texto; proximo(); };
}
