// ---- o Claude na folha, parte PURA (sem DOM; testada em claude-logica.test.mjs): quem é o principal, a variante de cada agente, a pose,
// em que lugar da folha cada figura fica, o limite de figuras e a ordem lado a lado (juntas, sobrepostas, quando são muitas). Contrato: docs/claude-na-folha.md.
// Um agente aqui é { agente, papel?, estado?, blueprintId, planoId?, em (ms), tipo? } (o que agentesDoBlueprint devolve).
const CLAUDE_MAX_SUB = 7; // iguais a MAX_SUBAGENTES e ESCALA.subagente de agentes-claude.mjs (o teste confere)
const CLAUDE_ESCALA_SUB = 1; // todos os agentes têm o mesmo tamanho (decisão do usuário, 2026-10-22; era 0,7)
const CLAUDE_LARGURA = 48; // largura do Claude na folha, em px (o subagente tem a mesma)
const CLAUDE_PROPORCAO = 170 / 220; // altura / largura do desenho (enquadramento único das poses)
const CLAUDE_FOLGA = 4; // espaço entre figuras lado a lado
const CLAUDE_LINHA_MAX = 5 * CLAUDE_LARGURA + 4 * CLAUDE_FOLGA; // largura máxima do grupo (5 figuras lado a lado); passou disso, as figuras se juntam, sobrepostas
const CLAUDE_PAPEIS = ["explorador", "construtor", "revisor"];

// o evento da fila (sem `estado`) vira o estado que a figura entende
const claudeEstadoDoEvento = (a) => a?.estado || (a?.tipo === "bloqueio" ? "bloqueado" : a?.tipo === "concluir" ? "concluido" : a?.tipo === "progresso" ? "escrevendo" : undefined);

// principal = o agente sem papel ou, se todos têm papel, o mais antigo; no empate pelo nome (assim a figura não troca de lugar a cada aviso)
function claudePrincipal(ags) {
  if (!ags.length) return null;
  const semPapel = ags.filter((a) => !a.papel), univ = semPapel.length ? semPapel : ags;
  return [...univ].sort((a, b) => (semPapel.length ? 0 : (a.em || 0) - (b.em || 0)) || String(a.agente || "").localeCompare(String(b.agente || "")))[0];
}
const CLAUDE_SUB_SOME_MS = 10 * 1000; // subagente que concluiu some da folha depois disso; só o principal fica com "Concluído" até o usuário ver
// subagente = quem tem papel e não é o principal da lista (`em` pode ser ISO ou ms); concluído há CLAUDE_SUB_SOME_MS ou mais, ele sai
const claudeSubSomeEm = (a) => new Date(a.em).getTime() + CLAUDE_SUB_SOME_MS;
const claudeEhSub = (a, ags) => !!a.papel && a !== claudePrincipal(ags);
const claudeSubSumiu = (a, ags, agora = Date.now()) => a.estado === "concluido" && claudeEhSub(a, ags) && agora >= claudeSubSomeEm(a);
// papel da figura: o principal nunca carrega ferramenta (null); subagente sem papel válido vira construtor (suposição: é o desenho mais neutro)
const claudePapelDe = (a, principal) => (a === principal ? null : CLAUDE_PAPEIS.includes(a.papel) ? a.papel : "construtor");
const claudeLarguraDe = (papel) => Math.round(CLAUDE_LARGURA * (papel ? CLAUDE_ESCALA_SUB : 1));
const claudeAlturaDe = (papel) => Math.round(claudeLarguraDe(papel) * CLAUDE_PROPORCAO);
const claudeChave = (papel, pose, quadro = 0) => `${papel || "claude"}|${pose}${pose === "andando" ? "|" + (quadro ? 1 : 0) : ""}`;
// chave do SVG de UM quadro (cache): vale para qualquer pose; a chave da pose acima NÃO leva o quadro, para trocar de quadro não contar como trocar de pose
const claudeChaveSvg = (papel, pose, quadro = 0) => `${papel || "claude"}|${pose}|${quadro | 0}`;
// os quadros de uma sequência de animação sem repetir, na ordem em que aparecem
const claudeQuadrosUnicos = (sequencia) => [...new Set(sequencia)];
// pose do agente: `poseDoEstado` vem de agentes-claude.mjs (a página o carrega sob demanda)
function claudeFigura(a, principal, andando, poseDoEstado) {
  const papel = claudePapelDe(a, principal), pose = poseDoEstado(claudeEstadoDoEvento(a), andando);
  return { papel, pose, chave: claudeChave(papel, pose) };
}

// quem aparece: o principal primeiro e depois os subagentes, quem avisou há menos tempo antes; passou de CLAUDE_MAX_SUB, `extras` conta os que ficam sob a pilha (não desenhados)
function claudeVisiveis(ags) {
  const principal = claudePrincipal(ags), resto = ags.filter((a) => a !== principal).sort((a, b) => (b.em || 0) - (a.em || 0));
  const lista = [principal, ...resto].filter(Boolean);
  return { principal, visiveis: lista.slice(0, 1 + CLAUDE_MAX_SUB), extras: Math.max(0, lista.length - 1 - CLAUDE_MAX_SUB) };
}
// Em que lugar da folha o agente fica, do mais preciso ao mais geral: na linha do plano dele (seção aberta); na barra da ação (seção fechada: `onde.barra(acaoId)`);
// no cabeçalho da seção (o projeto) da ação (`onde.secao(acaoId)` devolve a chave da seção, ou null; o lugar é por seção, para não empilhar tudo no mesmo ponto);
// senão no cabeçalho do blueprint (ou no cartão da lista). Sem `onde` valem só o plano e o cabeçalho.
const claudeLugar = (a, planoVisivel, onde = {}) => {
  if (a.planoId && planoVisivel(a.planoId)) return `plano:${a.planoId}`;
  if (a.acaoId && onde.barra?.(a.acaoId)) return `barra:${a.acaoId}`;
  const secao = a.acaoId && onde.secao?.(a.acaoId);
  return secao ? `secao:${secao}` : `cab:${a.blueprintId}`;
};

// os itens de um blueprint, cada um com o lugar e a posição lado a lado (indice 0 = o mais perto do lugar); os `extras` vão no último item (a dica dele os cita)
function claudeItens(ags, planoVisivel, onde = {}) {
  const { principal, visiveis, extras } = claudeVisiveis(ags), contagem = new Map(), itens = [];
  const poe = (lugar, resto) => { const indice = contagem.get(lugar) || 0; contagem.set(lugar, indice + 1); itens.push({ lugar, indice, ...resto }); };
  visiveis.forEach((a) => poe(claudeLugar(a, planoVisivel, onde), { a, papel: claudePapelDe(a, principal), principal: a === principal }));
  if (extras) itens[itens.length - 1].extras = extras;
  return itens;
}
const claudeLarguraItem = (it) => claudeLarguraDe(it.papel);

// onde a figura fica: `ancora` é a caixa do lugar (em px, na mesma régua) e `tipo` diz de que lado ela se alinha. As figuras se enfileiram para a esquerda do ponto de apoio.
// Devolve o canto de cima e da esquerda; os pés ficam em `base`. `larguras` = as larguras dos itens do lugar, na ordem de `indice`.
// Se o grupo passa de CLAUDE_LINHA_MAX, o passo entre as figuras encolhe (elas se juntam, sobrepostas) e o grupo não cresce mais.
function claudePosicao(tipo, ancora, indice, larguras) {
  const apoio = tipo === "plano" ? { dir: ancora.x - 6, base: ancora.y + ancora.h + 5 }
    : tipo === "cartao" ? { dir: ancora.x + ancora.w - 10, base: ancora.y + 4 }
    : { dir: ancora.x + ancora.w - 158, base: ancora.y + ancora.h - 8 }; // cabeçalho do blueprint: à esquerda dos medidores do canto
  const passos = (ate) => larguras.slice(1, ate + 1).reduce((s, l) => s + l + CLAUDE_FOLGA, 0), resto = passos(larguras.length);
  const k = resto > 0 ? Math.min(1, (CLAUDE_LINHA_MAX - larguras[0]) / resto) : 1; // fração do passo que sobra para as figuras caberem
  return { x: Math.round(apoio.dir - larguras[0] - k * passos(indice)), base: Math.round(apoio.base) };
}
