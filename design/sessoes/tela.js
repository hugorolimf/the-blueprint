// O comportamento do protótipo: simula os turnos do Claude com temporizadores. Nada aqui grava nada, e nenhum Claude de verdade roda.
const primeira = (ws) => doWs(ws).find((s) => s.estado === "pede") ?? doWs(ws).find((s) => s.estado !== "encerrada") ?? doWs(ws)[0];
const vista = { ws: "bp", sessao: primeira("bp").id, modo: "conversa", nova: "" };
const atual = () => SESSOES.find((s) => s.id === vista.sessao);
const relogios = new Map();

function avisa(texto) {
  $("aviso").textContent = texto;
  $("aviso").hidden = false;
  clearTimeout(avisa.t);
  avisa.t = setTimeout(() => { $("aviso").hidden = true; }, 5200);
}
const adia = (s, ms, f) => relogios.set(s.id, setTimeout(f, ms));
const para = (s) => { clearTimeout(relogios.get(s.id)); relogios.delete(s.id); };
const aberta = (s) => vista.modo === "conversa" && vista.sessao === s.id;

// um turno: anda pelo roteiro; acabou, a mensagem da fila vira o próximo turno, ou fica «Sua vez»
function anda(s) {
  const passo = s.roteiro.shift();
  if (!passo) {
    s.quando = "agora";
    const naFila = s.itens.filter((i) => i.fila);
    if (naFila.length) {
      naFila.forEach((i) => { i.fila = false; });
      s.roteiro = [{ fala: "Recebi sua mensagem e sigo por esse caminho." }];
      desenha();
      return adia(s, 700, () => anda(s));
    }
    s.estado = "pronta";
    return desenha();
  }
  if (passo.ferr) {
    s.itens.push({ t: "ferr", nome: passo.ferr[0], alvo: passo.ferr[1] });
    desenha();
    return adia(s, 900, () => anda(s));
  }
  const item = { t: "claude", x: "", vivo: true };
  s.itens.push(item);
  desenha();
  let n = 0;
  const letra = () => {
    n += 3;
    item.x = passo.fala.slice(0, n);
    if (aberta(s) && $("fala-viva")) { $("fala-viva").textContent = item.x; $("conversa").scrollTop = $("conversa").scrollHeight; }
    if (n < passo.fala.length) return adia(s, 24, letra);
    item.vivo = false;
    adia(s, 400, () => anda(s));
  };
  letra();
}

const pedidoAberto = (s) => s.itens.find((i) => i.t === "pedido" && !i.fim);

function decide(s, resposta) {
  const p = pedidoAberto(s);
  if (!p) return;
  p.fim = resposta;
  s.estado = "trabalhando";
  desenha();
  adia(s, 600, () => anda(s));
}

const acoes = {
  permitir: (s) => decide(s, "Você permitiu."),
  sempre: (s) => {
    const w = WORKSPACES.find((x) => x.id === s.ws), regra = `Bash(${pedidoAberto(s).comando})`;
    if (!w.regras.includes(regra)) w.regras.push(regra);
    decide(s, "Você permitiu sempre nesta pasta.");
    avisa("A regra entrou no Ambiente deste workspace.");
  },
  negar: (s) => { s.roteiro = [{ fala: "Sem problema, não vou rodar isso. Como prefere seguir?" }]; decide(s, "Você negou."); },
  interromper: (s) => {
    para(s);
    s.roteiro = [];
    const p = pedidoAberto(s);
    if (p) p.fim = "Pedido retirado: a sessão foi interrompida.";
    s.itens.forEach((i) => { if (i.t === "claude") i.vivo = false; });
    s.itens.push({ t: "sis", x: "Interrompida por você. A sessão continua aberta." });
    s.estado = "pronta";
    desenha();
  },
  encerrar: (s) => { s.itens.push({ t: "sis", x: "Sessão encerrada. A conversa ficou guardada e pode ser retomada." }); s.estado = "encerrada"; desenha(); },
  retomar: (s) => { s.itens.push({ t: "sis", x: "Sessão retomada de onde parou." }); s.estado = "pronta"; desenha(); },
  terminal: (s) => {
    para(s);
    s.roteiro = [];
    s.itens.push({ t: "sis", x: `Sessão enviada ao terminal: claude --resume ${s.id}` });
    s.estado = "fora";
    desenha();
    avisa("Abriria o terminal aqui. A sessão sai do app, porque não pode estar aberta em dois lugares.");
  },
  trazer: (s) => { s.itens.push({ t: "sis", x: "De volta ao app. Feche o terminal antes, senão as mensagens se misturam." }); s.estado = "pronta"; desenha(); },
  enviar: (s) => {
    const x = $("msg").value.trim();
    if (!x) return $("msg").focus();
    const ocupada = s.estado === "trabalhando" || s.estado === "pede";
    if (s.estado === "encerrada") s.itens.push({ t: "sis", x: "Sessão retomada de onde parou." });
    s.itens.push({ t: "voce", x, fila: ocupada });
    if (ocupada) return desenha();
    s.estado = "trabalhando";
    s.roteiro = [{ ferr: ["Read", "o que você citou"] }, { fala: "Entendi. Olhei o que você pediu e deixei pronto para a sua revisão." }];
    desenha();
    adia(s, 700, () => anda(s));
  },
};

function iniciar() {
  const w = WORKSPACES.find((x) => x.id === vista.ws), acao = w.acoes.find(([n]) => n === vista.nova);
  const texto = $("nova-texto").value.trim() || (acao ? `Inicia a ação ${acao[0]}.` : "Vamos conversar sobre este workspace.");
  const s = { id: Math.random().toString(16).slice(2, 8), ws: w.id, acao: acao?.[0] ?? null, nome: acao?.[1] ?? "Conversa solta", estado: "trabalhando", quando: "agora", contexto: 3,
    itens: [{ t: "voce", x: texto }], roteiro: [{ ferr: ["Read", "CLAUDE.md"] }, { fala: "Li o que há nesta pasta e já vou começar. Registro o progresso no blueprint enquanto trabalho." }] };
  SESSOES.unshift(s);
  vista.sessao = s.id;
  vista.modo = "conversa";
  desenha();
  adia(s, 700, () => anda(s));
}

$("tela").addEventListener("click", (e) => {
  const b = e.target.closest("button");
  if (!b) return;
  if (b.dataset.ws) { vista.ws = b.dataset.ws; vista.sessao = primeira(vista.ws)?.id; vista.modo = "conversa"; return desenha(); }
  if (b.dataset.sessao) { vista.sessao = b.dataset.sessao; vista.modo = "conversa"; return desenha(); }
  if (b.dataset.acao !== undefined) { vista.nova = b.dataset.acao; return desenha(); }
  if (b.dataset.tira !== undefined) { WORKSPACES.find((w) => w.id === vista.ws).regras.splice(+b.dataset.tira, 1); return desenha(); }
  const faz = b.dataset.faz;
  if (faz === "abrir-nova" || b.id === "nova") { vista.modo = "nova"; vista.nova = WORKSPACES.find((w) => w.id === vista.ws).acoes[0][0]; return desenha(); }
  if (b.id === "ambiente") { vista.modo = vista.modo === "ambiente" ? "conversa" : "ambiente"; return desenha(); }
  if (faz === "iniciar") return iniciar();
  if (faz === "cancelar") { vista.modo = "conversa"; return desenha(); }
  if (faz && atual()) acoes[faz](atual());
});
$("tela").addEventListener("keydown", (e) => {
  if (e.target.id === "msg" && e.key === "Enter" && !e.shiftKey) { e.preventDefault(); acoes.enviar(atual()); }
});

const ajusta = () => { $("tela").style.transform = `scale(${Math.min(innerWidth / 1280, innerHeight / 760)})`; };
addEventListener("resize", ajusta);
ajusta();
desenha();
const roda = SESSOES.find((s) => s.estado === "trabalhando");
if (roda) adia(roda, 2500, () => anda(roda));
