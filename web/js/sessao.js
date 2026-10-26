// ---- sessão: em que modo o servidor está e quem é a pessoa (docs/colaboracao.md, seções 3 a 5) ----------------------
// Modo local (o app de sempre): GET /api/sessao diz {"modo":"local"} (ou nem existe, num binário antigo) e nada muda para o usuário.
// Modo hospedado (blueprint serve --hospedar): a sessão diz quem entrou pelo link de convite e com que papel; sem sessão, a página mostra a
// entrada (convite.js) e não carrega os dados. Os painéis Pessoas, Minha chave e Acompanhar são módulos carregados por import() ao abrir.
let sessao = { modo: "local" };
const hospedado = () => sessao.modo === "servidor";
const PESO_PAPEL = { visitante: 0, supervisor: 1, colaborador: 2, dono: 3 };
const nomeDoPapel = (p) => ({ dono: tr("Dono"), colaborador: tr("Colaborador"), supervisor: tr("Supervisor"), visitante: tr("Visitante") })[p] || p;
const explicaPapel = (p) => ({
  dono: tr("Dono: vê e edita tudo neste workspace, convida e remove pessoas."),
  colaborador: tr("Colaborador: vê e edita os blueprints e pode conectar o próprio agente."),
  supervisor: tr("Supervisor: acompanha os blueprints e as pessoas, sem editar."),
  visitante: tr("Visitante: só vê este blueprint, sem conta e sem editar."),
})[p] || "";

// o maior papel que cobre o alvo: '*' > workspace > blueprint (seção 3); "" quando nada cobre
function papelNoEscopo(acessos, workspaceId, blueprintId = "") {
  let melhor = "";
  for (const a of acessos || []) {
    const cobre = a.workspaceId === "*" || (a.workspaceId === workspaceId && (!a.blueprintId || a.blueprintId === blueprintId));
    if (cobre && (PESO_PAPEL[a.papel] ?? -1) > (PESO_PAPEL[melhor] ?? -1)) melhor = a.papel;
  }
  return melhor;
}
// o que a pessoa pode fazer em algum lugar: decide quais botões aparecem (o servidor confere de novo em cada rota)
function poderesDaSessao(s) {
  if (s.modo !== "servidor") return { pessoas: false, chave: false, acompanhar: false, soLeitura: false };
  const papeis = (s.acessos || []).map((a) => a.papel);
  return {
    pessoas: !s.visitante && papeis.some((p) => p === "dono" || p === "supervisor"),
    chave: !s.visitante && !!s.membro,
    acompanhar: !s.visitante && papeis.some((p) => p === "dono" || p === "supervisor"),
    soLeitura: s.visitante || !papeis.some((p) => p === "dono" || p === "colaborador"),
  };
}

// os códigos estáveis do servidor (seção 5) viram texto no idioma da página; sem código conhecido, o texto que o servidor mandou
function textoDoErro(codigo, params = {}, reserva = "") {
  const p = params || {};
  const t = {
    "convite-invalido": tr("Este link de convite não vale mais: pode ter vencido, sido revogado ou já usado. Peça um novo a quem convidou você."),
    "muitas-tentativas": tr("Muitas tentativas. Espere {segundos} s e tente de novo.", { segundos: p.segundos ?? 60 }),
    "falta-nome": tr("Digite seu nome."),
    "nome-invalido": tr("Use um nome de até {max} caracteres, sem símbolos de controle.", { max: p.max ?? 60 }),
    "papel-invalido": tr("Papel inválido."),
    "validade-invalida": tr("Escolha uma validade entre 1 hora e 30 dias."),
    "usos-invalidos": tr("O número de usos vai de 1 a 1000."),
    "ultimo-dono": tr("O workspace precisa de pelo menos um dono."),
    "proibido": tr("Seu papel não permite esta ação."),
    "sem-sessao": tr("Sua sessão terminou. Entre de novo pelo link de convite."),
    "chave-invalida": tr("Chave de MCP inválida ou revogada."),
    "so-servidor": tr("Isto só existe no servidor da equipe."),
    "https-obrigatorio": tr("O servidor recusou o pedido: confira o endereço (HTTPS) e tente de novo."),
    "host-invalido": tr("O servidor recusou o pedido: confira o endereço (HTTPS) e tente de novo."),
    "origem-invalida": tr("O servidor recusou o pedido: confira o endereço (HTTPS) e tente de novo."),
  }[codigo];
  return t || reserva || tr("Não foi possível concluir. Tente novamente.");
}
// pedido às rotas de equipe: GET sem corpo, POST com JSON; erro vira Error com o texto traduzido e o código
async function equipeApi(rota, corpo) {
  const r = await fetch(rota, corpo === undefined ? { cache: "no-store" } : { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
  const j = await r.json().catch(() => ({}));
  if (r.status === 401 && j.codigo === "sem-sessao") semSessao();
  if (!r.ok) throw Object.assign(new Error(textoDoErro(j.codigo, j.params, j.erro)), { codigo: j.codigo || "", status: r.status, params: j.params || {} });
  return j;
}

async function lerSessao() {
  try {
    const r = await fetch("/api/sessao", { cache: "no-store" });
    if (r.status === 401) sessao = { modo: "servidor", membro: null, visitante: false, acessos: [], semSessao: true };
    else if (r.ok) { const j = await r.json(); if (j && j.modo === "servidor") sessao = { membro: null, visitante: false, acessos: [], ...j }; }
  } catch {} // sem resposta: segue como o app local (o carregar() já tenta de novo sozinho)
  return sessao;
}
// a porta de entrada da página (inicio.js): em /convite ou no modo hospedado sem sessão mostra a entrada e NÃO carrega os dados
function sessaoGuarda(iniciar) {
  if (location.pathname === "/convite") { entradaPeloConvite(); return; }
  lerSessao().then((s) => {
    if (s.semSessao) { semSessao(); return; }
    if (hospedado()) sessaoMostra();
    iniciar();
  });
}
// global (contrato com a SINCRONIA): um 401 em qualquer rota, ou o SSE avisando "sessao-encerrada", leva à entrada
function semSessao() {
  sessao = { modo: "servidor", membro: null, visitante: false, acessos: [], semSessao: true };
  mostrarEntradaConvite("sem-sessao");
}
function sessaoMostra() {
  const pode = poderesDaSessao(sessao);
  document.body.classList.add("hospedado");
  document.body.classList.toggle("so-leitura", pode.soLeitura);
  $("b-pessoas").hidden = $("b-pessoas-folha").hidden = !pode.pessoas; // na lista de workspaces e na barra da folha
  $("b-chave").hidden = !pode.chave;
  $("b-acompanhar").hidden = !pode.acompanhar;
  $("perfil-sair").textContent = tr("Sair");
  $("perfil-sair").onclick = sairDaSessao; // no modo local o botão continua "Trocar nome" (entrada.js)
  mostrarPerfil();
  const abre = sessao.visitante && (sessao.acessos || []).find((a) => a.abre === "apresentacao");
  if (abre) abrirApresentacaoQuandoCarregar(abre.blueprintId);
}
// shortcut: espera os dados com um relógio curto (no máximo 10 s, só na entrada do visitante); trocar por um evento de "dados prontos" se surgir
function abrirApresentacaoQuandoCarregar(blueprintId, tentativas = 100) {
  if (carregado) { abrirApresentacao(blueprintId || tela); return; }
  if (tentativas > 0) setTimeout(() => abrirApresentacaoQuandoCarregar(blueprintId, tentativas - 1), 100);
}
async function sairDaSessao() {
  try { await equipeApi("/api/sessao/sair", {}); } catch {}
  location.replace("/");
}
// os workspaces em que a pessoa tem um destes papéis (no workspace inteiro ou num blueprint dele)
function workspacesDaEquipe(papeis) {
  const ids = new Set((sessao.acessos || []).filter((a) => papeis.includes(a.papel)).map((a) => a.workspaceId));
  return workspacesDe(arquivo).filter((w) => ids.has("*") || ids.has(w.id));
}
const equipeModulos = {};
function abrirEquipe(modulo, funcao) {
  (equipeModulos[modulo] ||= import(`/web/js/${modulo}.js`))
    .then((m) => m[funcao]())
    .catch(() => { equipeModulos[modulo] = null; $("entrada-status").textContent = tr("Não foi possível abrir o painel."); });
}
$("b-pessoas").onclick = $("b-pessoas-folha").onclick = () => abrirEquipe("pessoas", "abrirPessoas");
$("b-chave").onclick = () => abrirEquipe("chave-mcp", "abrirChave");
$("b-acompanhar").onclick = () => abrirEquipe("supervisor", "abrirSupervisor");
$("equipe-fechar").onclick = () => $("equipe").close();
