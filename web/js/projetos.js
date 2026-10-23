// ---- projetos dentro do blueprint (decisão de 2026-10-22): cada projeto é uma SEÇÃO da folha, igual a um blueprint -------------------------------
// Número (01, 02, 03), nome, descrição, progresso geral e botão Recolher/Abrir (cabecalho.js), com as ações do projeto logo abaixo: compactas quando a seção
// está fechada (nasce fechada) e com planos e checkpoints quando aberta. As seções seguem a data de início da primeira ação de cada projeto; o "Geral"
// (ações sem projeto ou com um projeto que não existe mais) vem por último. As ações são numeradas POR PROJETO (02.3 = projeto 2, ação 3); blueprint sem
// `projetos` é desenhado e numerado como sempre foi (1.5 = blueprint 1, ação 5). O que o usuário abre fica só neste navegador; não há timer nem animação.
const CHAVE_PROJETOS = "blueprint.projetosAbertos";
const projetosAbertos = (() => {
  try {
    const v = JSON.parse(localStorage.getItem(CHAVE_PROJETOS) || "[]");
    return new Set(Array.isArray(v) ? v.filter((x) => typeof x === "string").slice(0, 500) : []);
  } catch { return new Set(); }
})();
const guardaProjetosAbertos = () => { try { localStorage.setItem(CHAVE_PROJETOS, JSON.stringify([...projetosAbertos])); } catch {} };
const chaveProjeto = (p, projetoId) => `${p.id}|${projetoId || ""}`; // o Geral é o projeto de id vazio
// o id do projeto da ação, ou "" (Geral) se ela não tem projeto ou o projeto não existe mais
const projetoDaAcao = (p, f) => ((p.projetos || []).some((x) => x.id === f.projetoId) ? f.projetoId : "");

// abre o projeto da ação (para o robô e o clique no aviso do agente chegarem a ela)
function abrirProjetoDaAcao(p, acaoId) {
  const f = p.acoes.find((x) => x.id === acaoId);
  if (!f || !p.projetos?.length) return;
  const chave = chaveProjeto(p, projetoDaAcao(p, f));
  if (projetosAbertos.has(chave)) return;
  projetosAbertos.add(chave);
  guardaProjetosAbertos();
}

// Os projetos na ordem das seções: pela data de início da primeira ação de cada um (empate: a ordem em que foram criados); projeto sem ações vai depois dos
// que têm. O Geral não entra aqui (é sempre o último).
function projetosOrdenados(p) {
  const inicio = (id) => Math.min(Infinity, ...p.acoes.filter((f) => projetoDaAcao(p, f) === id).map((f) => dia(f.inicio)).filter(Number.isFinite));
  return (p.projetos || []).map((x, i) => ({ x, i, ini: inicio(x.id) })).sort((a, b) => (a.ini === b.ini ? a.i - b.i : a.ini - b.ini)).map((o) => o.x);
}
// O número da ação na folha: <projeto>.<posição da ação no projeto> quando o blueprint tem projetos (o Geral é o último projeto), senão <blueprint>.<posição>.
function numeroDaAcao(p, f, pi = D.blueprints.indexOf(p)) {
  if (!p.projetos?.length) return `${pi + 1}.${p.acoes.findIndex((x) => x.id === f.id) + 1}`;
  const id = projetoDaAcao(p, f), ordem = [...projetosOrdenados(p).map((x) => x.id), ""];
  return `${ordem.indexOf(id) + 1}.${p.acoes.filter((x) => projetoDaAcao(p, x) === id).findIndex((x) => x.id === f.id) + 1}`;
}

// Os grupos do blueprint (as seções): os projetos na ordem acima e, por último, o Geral se tiver ações; null se o blueprint não tem projetos.
// `visiveis`: as ações que a folha desenha (com o filtro e o período); `todas`: as do blueprint inteiro, de onde vêm o resumo e as datas da seção.
function gruposDoBlueprint(p, visiveis, todas) {
  if (!p.projetos?.length) return null;
  const ag = agenteAqui();
  if (roboRolar && ag?.blueprintId === p.id) abrirProjetoDaAcao(p, ag.acaoId);
  const doGrupo = (lista, id) => lista.filter((f) => projetoDaAcao(p, f) === id);
  const ids = [...projetosOrdenados(p).map((x) => ({ id: x.id, nome: x.nome || "Sem nome", descricao: x.descricao || "" })), { id: "", nome: "Geral", descricao: "" }];
  return ids.map((g, i) => ({ ...g, numero: i + 1 })).filter((g) => g.id || doGrupo(todas, "").length).map((g) => {
    const chave = chaveProjeto(p, g.id), seus = doGrupo(todas, g.id);
    return { ...g, chave, aberto: projetosAbertos.has(chave), acoes: doGrupo(visiveis, g.id), total: seus.length, ini: Math.min(...seus.map((f) => f.a)), fim: Math.max(...seus.map((f) => f.b)), ...resumo(seus) };
  });
}

// Posição das linhas de UMA seção (a de um projeto, ou o blueprint inteiro quando não há projetos): só as linhas das ações, como sempre foi.
// `op`: a seção aberta ou não, as alturas e como medir cada ação.
function disposicaoFolha(acoes, op) {
  const { aberto, RH, PH, FOLGA, de, ate } = op, alts = [];
  const n = linhas(acoes, de, ate);
  for (let i = 0; i < n; i++) alts.push(aberto ? RH : 24);
  acoes.forEach((f) => { if (aberto) alts[f.linha] = Math.max(alts[f.linha], f.alt); });
  const rowY = [];
  let corpo = 0;
  alts.forEach((h, i) => { rowY[i] = corpo; corpo += h; });
  return { acoes, rowY, alto: (!alts.length ? PH + 36 : PH + (aberto ? corpo : corpo + 10)) + FOLGA };
}

// "Expandir tudo" / "Recolher tudo": abre ou fecha todas as seções do blueprint
function abrirTodosProjetos(p, aberto) {
  if (!p?.projetos?.length) return;
  for (const id of [...p.projetos.map((x) => x.id), ""]) { const c = chaveProjeto(p, id); if (aberto) projetosAbertos.add(c); else projetosAbertos.delete(c); }
  guardaProjetosAbertos();
}
// alguma seção do blueprint está aberta? (decide se o menu oferece "Recolher" ou "Expandir")
const algumProjetoAberto = (p) => [...(p.projetos || []).map((x) => x.id), ""].some((id) => projetosAbertos.has(chaveProjeto(p, id)));
function alternarProjeto(chave) {
  const fechando = projetosAbertos.has(chave);
  const dele = (el) => { const p = D.blueprints.find((x) => x.id === el.dataset.p), f = p?.acoes.find((x) => x.id === el.dataset.f); return !!f && chaveProjeto(p, projetoDaAcao(p, f)) === chave; }; // só os planos desta seção esmaecem
  animarAbrirFechar(() => { if (!projetosAbertos.delete(chave)) projetosAbertos.add(chave); guardaProjetosAbertos(); }, fechando ? dele : undefined); // o foco volta ao cabeçalho (animarAbrirFechar)
}
host.addEventListener("click", (e) => { const t = e.target.closest("[data-projeto]"); if (t) alternarProjeto(t.dataset.projeto); });
host.addEventListener("keydown", (e) => {
  const t = (e.key === "Enter" || e.key === " ") && e.target.closest?.("[data-projeto]");
  if (t) { e.preventDefault(); alternarProjeto(t.dataset.projeto); }
});
