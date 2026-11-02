// O HTML da nota: as classes bp-* deste protótipo (bp.css). Recebe o plano principal (qualquer pasta no formato) e os exemplos; todas as
// telas são desenhadas a partir deles, e o que é só demonstração está marcado aqui.
import { CAPA, CINCO, PECAS, APROVACAO, DIFF, AGENTES, ORQUESTRADOR, TEMPO_REAL, COMPATIVEL, MCP, TESTE, ETAPAS, PERGUNTAS } from "./plano.mjs";
import { fluxo } from "./fluxo.mjs";
import { telaFolha } from "./tela-folha.mjs";
import { telaAcao } from "./tela-acao.mjs";
import { telaDetalhe } from "./tela-detalhe.mjs";
import { telaProposta } from "./tela-proposta.mjs";
import { telaAgentes, figuraAgente } from "./tela-agentes.mjs";
import { telaEscrita } from "./tela-escrita.mjs";
import { escalonar, planosDe } from "./agentes.mjs";
import { compararPlanos, planosTravados } from "./comparar.mjs";
import { acoesDe, contagem } from "./pacote.mjs";
import { esc } from "./svg.mjs";
import { chaveDecisao, chaveDetalhe } from "./revisao.mjs";

const secao = (t, extra = "") => `<div class="bp-sec">${t}${extra ? `<i>${extra}</i>` : ""}</div>`;
const fig = (svg, leg) => `<div class="bp-quadro bp-tema">${svg}</div><p class="bp-leg">${leg}</p>`;
const tabela = (cab, linhas) => `<table><thead><tr>${cab.map((c) => `<th>${c}</th>`).join("")}</tr></thead><tbody>${linhas.map((l) => `<tr>${l.map((c, i) => `<td>${i ? c : `<b>${c}</b>`}</td>`).join("")}</tr>`).join("")}</tbody></table>`;

const JSON_MIN = `{
  "formato": "blueprint.plano/1",
  "blueprint": "Meu projeto",
  "projetos": [{ "acoes": [
    { "id": "1", "nome": "Primeira ação" }
  ] }]
}`;
const JSON_PLANO = `{ "id": "1.2.3",
  "texto": "Aviso por cima de qualquer tela",
  "area": "app/barra",
  "depende_de": ["1.1.3"] }`;

/** A ação com mais figuras é a que melhor mostra a ação aberta. */
const maisCheia = (p) => acoesDe(p).reduce((m, a) => ((a.detalhes?.length ?? 0) > (m.detalhes?.length ?? 0) ? a : m));

/** A execução de demonstração, calculada pela regra de verdade: o primeiro projeto iniciado, dois planos prontos e um pedindo ajuda. */
function execucaoDemo(p, comp, limite) {
  const pr = p.projetos.find((x) => x.acoes.some((a) => a.planos?.length)), l = pr.acoes.flatMap((a) => a.planos ?? []), g = pr.acoes.filter((a) => a.planos?.length);
  return escalonar(planosDe(p), { iniciados: l.map((q) => q.id), prontos: l.slice(0, 2).map((q) => q.id), bloqueados: g.length > 3 ? [g[3].planos[0]?.id].filter(Boolean) : [], travados: planosTravados(comp), limite });
}

/** Situações de demonstração, para as telas mostrarem cada estado da revisão. */
function revisoesDemo(p) {
  const r = {}, ok = { estado: "aprovada", quando: "01/11" }, dec = (p.decisoes ?? []).filter((d) => d.pede_aprovacao);
  if (dec[0]) r[chaveDecisao(dec[0])] = ok;
  if (dec[1]) r[chaveDecisao(dec[1])] = { estado: "revisar", quando: "01/11", observacao: "Quero ver outra opção antes de decidir." };
  if (dec[2]) r[chaveDecisao(dec[2])] = { estado: "pendente", compondo: true };
  if (dec[3]) r[chaveDecisao(dec[3])] = { estado: "reprovada", quando: "01/11" };
  for (const a of acoesDe(p)) {
    const det = (a.detalhes ?? []).filter((d) => d.pede_aprovacao);
    if (det[1]) r[chaveDetalhe(a, det[1])] = ok;
    if (det[2]) r[chaveDetalhe(a, det[2])] = { estado: "revisar", quando: "01/11" };
    if (det[3]) r[chaveDetalhe(a, det[3])] = { estado: "reprovada", quando: "01/11" };
  }
  return r;
}

export function painel(principal, exemplos, c) {
  const n = contagem(principal), comp = compararPlanos(principal.anterior, principal), alterada = Object.keys(comp.alteradas)[0];
  const a = acoesDe(principal).find((x) => x.id === alterada) ?? maisCheia(principal), d = maisCheia(principal).detalhes?.[0], aD = maisCheia(principal);
  const exec = execucaoDemo(principal, comp, 3);
  const ctx = (id, extra = {}) => ({ id, hoje: "2026-11-01", workspace: c.workspace, revisoes: revisoesDemo(principal), exec, comp, figura: figuraAgente, limite: 3, agente: "Orquestrador · 01/11", ...extra });
  return [
    `<div class="painel bp-nota">`,
    `<div class="bp-capa"><div class="bp-capa-texto"><div class="bp-capa-rot">${CAPA.rotulo}</div><div class="bp-capa-tit">${CAPA.titulo}</div><p>${CAPA.texto}</p></div><div class="bp-selo"><div><small>Formato</small><b>blueprint.plano/1</b></div><div><small>Exemplo aberto</small><b>${esc(principal.blueprint)}</b></div><div><small>Situação</small><b>Protótipo · esperando você</b></div></div></div>`,
    secao("A ideia"),
    `<ol class="bp-cinco">${CINCO.map(([t, x]) => `<li><b>${t}</b>${x.charAt(0).toUpperCase()}${x.slice(1)}</li>`).join("")}</ol>`,
    secao("Tela 1 · A folha é o plano", `${esc(principal.blueprint)} · ${n.acoes} ações · ${n.detalhes} detalhes · ${n.decisoes} decisões`),
    fig(telaFolha(principal, ctx("1")), "À esquerda, a ação e os seus números, como numa planilha; à direita, a régua. A barra enche conforme os agentes concluem os planos. Tudo aqui é desenhado a partir do plano, sem código próprio deste projeto. As datas e o andamento são de demonstração."),
    secao("Tela 2 · A ação aberta: é aqui que se decide", `${a.id} ${esc(a.nome)}, com a mudança como git diff`),
    fig(telaAcao(principal, a.id, ctx("2")), "O plano alterado aparece como diferença: vermelho saiu, verde entrou, com a palavra exata destacada. As decisões têm o controle na própria linha e, depois, o carimbo; a margem colorida diz a situação à distância. Os estados são de demonstração, e uma decisão está em «Revisar», com o campo aberto."),
    d ? secao("Tela 3 · A folha de detalhe", `${aD.id}-${d.id}`) : "",
    d ? fig(telaDetalhe(principal, aD.id, d.id, ctx("3")), "A figura manda: ocupa a largura da folha, e a revisão fica colada nela, na mesma linha. Embaixo, a legenda, as decisões da ação e a fita com os vizinhos.") : "",
    secao("Tela 4 · O que chegou", "só mostra: nenhuma decisão aqui"),
    fig(telaProposta(principal, ctx("4")), "A mesma folha, com o novo tracejado e o alterado marcado. Ao lado, o plano novo aparece só como resumo, com «Colocar»; o alterado leva ao git diff, dentro do plano. O cenário é de demonstração."),
    tabela(["", "Como funciona"], DIFF),
    secao("Tela 5 · O orquestrador e um agente por plano", "o mestre do Workspace; o check chama, o agente marca"),
    fig(telaAgentes(principal, ctx("5")), "Em cima, o orquestrador, com o que está fazendo agora e o campo para falar com ele. Embaixo, uma linha por plano com a caixinha marcada, e a figura do agente na pose da situação. A demonstração foi calculada pela regra de verdade (<code>agentes.mjs</code>, com testes). As áreas e as dependências deste exemplo são de demonstração."),
    tabela(["O orquestrador", "Como funciona"], ORQUESTRADOR),
    tabela(["Os agentes", "Como funciona"], AGENTES),
    secao("Tela 6 · O plano sendo escrito", "em tempo real; a tela está animada"),
    fig(telaEscrita(principal, ctx("6")), "As ações já estão na folha, «Aguardando plano». O orquestrador escreve uma de cada vez, e você acompanha: os planos sendo digitados, as decisões chegando e as figuras sendo desenhadas. A animação repete em ciclo e, na interface nativa, será o próprio plano chegando pelo MCP."),
    tabela(["", "Como funciona"], TEMPO_REAL),
    secao("Como você revisa o plano", "aprovar, revisar, editar, reprovar"),
    tabela(["", "Como funciona"], APROVACAO),
    secao("O formato", "o que a IA escreve"),
    `<div class="bp-duas"><div><div class="bp-rot">O menor plano válido</div><pre class="bp-codigo">${esc(JSON_MIN)}</pre></div><div><div class="bp-rot">Um plano com tudo</div><pre class="bp-codigo">${esc(JSON_PLANO)}</pre></div></div>`,
    tabela(["Peça", "Regra", "Onde aparece"], PECAS),
    secao("Como o plano entra"),
    fig(fluxo(), "O agente nunca desenha a interface: entrega o conteúdo no formato, peça por peça, e a aparência é do Blueprint."),
    tabela(["Ferramenta", "", "Para quê"], MCP),
    secao("Serve para qualquer projeto", "o mesmo código, três planos diferentes"),
    ...exemplos.map((x, i) => {
      const cx = compararPlanos(x.plano.anterior, x.plano), tudoNovo = !x.plano.anterior;
      return fig(telaAcao(x.plano, acoesDe(x.plano)[x.indice ?? 0].id, ctx(`e${i}`, { workspace: x.workspace, revisoes: revisoesDemo(x.plano), exec: execucaoDemo(x.plano, tudoNovo ? null : cx, 3), comp: tudoNovo ? { novas: [], alteradas: {}, iguais: [] } : cx })), `<b>${esc(x.plano.blueprint)}.</b> ${esc(x.plano.resumo)}`);
    }),
    tabela(["", "Como"], COMPATIVEL),
    secao("O que foi testado de verdade"),
    tabela(["", "Resultado"], TESTE.map(([k, v]) => [k, v === "RESULTADO_TESTE" ? c.teste : v])),
    secao("Etapas"),
    `<ol class="bp-etapas">${ETAPAS.map(([t, x]) => `<li><b>${t}</b><span>${x}</span></li>`).join("")}</ol>`,
    secao("Perguntas para você", "responda no chat, ex.: «P1: sim»"),
    tabela(["", "Pergunta", "Minha recomendação"], PERGUNTAS),
    `</div>`,
  ].join("\n");
}
