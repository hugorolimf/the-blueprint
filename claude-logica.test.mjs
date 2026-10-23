// A lógica pura do Claude na folha (web/js/claude-logica.js, script clássico carregado num contexto isolado, como o navegador faz):
// principal, variante por papel, pose por estado, lugar de cada figura, limite com o contador "+N" e a ordem lado a lado. O DOM fica em claude-camada.js.
//   node claude-logica.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";
import { poseDoEstado, MAX_SUBAGENTES, ESCALA, PAPEIS, POSES, QUADROS, SEQUENCIA } from "./agentes-claude.mjs";

const ctx = vm.createContext({});
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "claude-logica.js"), "utf8")
  + "\nthis.api = { claudeEstadoDoEvento, claudePrincipal, claudePapelDe, claudeFigura, claudeChave, claudeChaveSvg, claudeQuadrosUnicos, claudeVisiveis, claudeLugar, claudeItens, claudeLarguraDe, claudeLarguraItem, claudePosicao, claudeProximaAcao, claudeAgenteEspera, claudeItemEspera, CLAUDE_MAX_SUB, CLAUDE_ESCALA_SUB, CLAUDE_PAPEIS, CLAUDE_FOLGA };", ctx);
const A = ctx.api;
const J = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes

const ag = (agente, extra = {}) => ({ agente, blueprintId: "b1", planoId: "", estado: "escrevendo", em: 1000, ...extra });

// as constantes copiadas valem o mesmo que no desenho
assert.equal(A.CLAUDE_MAX_SUB, MAX_SUBAGENTES);
assert.equal(A.CLAUDE_ESCALA_SUB, ESCALA.subagente);
assert.deepEqual(J(A.CLAUDE_PAPEIS), PAPEIS);

// ---- o principal: o agente sem papel; se todos têm papel, o mais antigo; empate pelo nome (não troca a cada aviso)
assert.equal(A.claudePrincipal([]), null);
assert.equal(A.claudePrincipal([ag("x", { papel: "revisor", em: 5 }), ag("claude", { em: 9 }), ag("y", { papel: "construtor", em: 1 })]).agente, "claude");
assert.equal(A.claudePrincipal([ag("novo", { papel: "revisor", em: 50 }), ag("velho", { papel: "explorador", em: 10 })]).agente, "velho", "todos com papel: o mais antigo");
assert.equal(A.claudePrincipal([ag("b"), ag("a", { em: 99 })]).agente, "a", "dois sem papel: o nome decide, não o horário");

// ---- variante por papel: o principal nunca carrega ferramenta; subagente = o papel dele (ou construtor se vier sem papel válido)
const p = ag("claude");
assert.equal(A.claudePapelDe(p, p), null);
for (const papel of PAPEIS) assert.equal(A.claudePapelDe(ag("s", { papel }), p), papel);
assert.equal(A.claudePapelDe(ag("s"), p), "construtor");
assert.equal(A.claudePapelDe(ag("s", { papel: "mago" }), p), "construtor");
assert.equal(A.claudePapelDe(ag("claude", { papel: "revisor" }), ag("claude", { papel: "revisor" })), "revisor", "outro objeto não é o principal");
assert.equal(A.claudeLarguraDe(null), 48);
assert.equal(A.claudeLarguraDe("revisor"), A.claudeLarguraDe(null), "todos os agentes têm o mesmo tamanho (o Claude e os subagentes)");

// ---- pose por estado (o mapa mora em agentes-claude.mjs; aqui o encaixe com o agente e o evento da fila)
const pose = (a, andando = false) => J(A.claudeFigura(a, p, andando, poseDoEstado));
assert.equal(pose(ag("claude", { estado: "lendo" })).pose, "trabalhando");
assert.equal(pose(ag("claude", { estado: "testando" }), true).pose, "andando", "andando só enquanto trabalha");
assert.equal(pose(ag("claude", { estado: "aguardando" }), true).pose, "esperando");
assert.equal(pose(ag("claude", { estado: "bloqueado" })).pose, "bloqueado");
assert.equal(pose(ag("claude", { estado: "concluido" }), true).pose, "concluido");
assert.equal(pose(ag("claude", { estado: undefined })).pose, "parado");
assert.equal(pose(ag("claude", { estado: "ocioso" })).pose, "parado");
assert.deepEqual(pose(ag("s", { papel: "revisor", estado: "bloqueado" })), { papel: "revisor", pose: "bloqueado", chave: "revisor|bloqueado" });
// o último evento da fila (sem `estado`): o tipo vira o estado
assert.equal(A.claudeEstadoDoEvento({ tipo: "bloqueio" }), "bloqueado");
assert.equal(A.claudeEstadoDoEvento({ tipo: "concluir" }), "concluido");
assert.equal(A.claudeEstadoDoEvento({ tipo: "progresso" }), "escrevendo");
assert.equal(A.claudeEstadoDoEvento({ tipo: "proposta" }), undefined);
assert.equal(A.claudeEstadoDoEvento({ tipo: "bloqueio", estado: "lendo" }), "lendo", "o estado explícito vence");
// a chave do cache: andando tem 2 quadros, as outras poses um só
assert.equal(A.claudeChave(null, "trabalhando", 1), "claude|trabalhando");
assert.equal(A.claudeChave("construtor", "andando", 1), "construtor|andando|1");
assert.equal(A.claudeChave(null, "andando"), "claude|andando|0");
// quadros da animação: a chave do SVG leva o quadro em qualquer pose; a da pose não (trocar de quadro não é trocar de pose)
assert.equal(A.claudeChaveSvg(null, "parado"), "claude|parado|0");
assert.equal(A.claudeChaveSvg("revisor", "esperando", 2), "revisor|esperando|2");
assert.notEqual(A.claudeChaveSvg(null, "trabalhando", 0), A.claudeChaveSvg(null, "trabalhando", 1));
assert.equal(A.claudeChave(null, "trabalhando", 1), A.claudeChave(null, "trabalhando", 0), "a chave da pose ignora o quadro");
for (const pose of POSES) assert.deepEqual(J(A.claudeQuadrosUnicos(SEQUENCIA[pose])).sort(), Array.from({ length: QUADROS[pose] }, (_, i) => i), `${pose}: os quadros únicos da sequência são todos os quadros`);

// ---- lugar: no plano, se o plano está à vista; senão no cabeçalho do blueprint
const aVista = (id) => id === "t1" || id === "t2";
assert.equal(A.claudeLugar(ag("a", { planoId: "t1" }), aVista), "plano:t1");
assert.equal(A.claudeLugar(ag("a", { planoId: "t9" }), aVista), "cab:b1", "plano fora da folha (ação recolhida)");
assert.equal(A.claudeLugar(ag("a"), aVista), "cab:b1", "sem plano");
// do mais preciso ao mais geral: plano à vista > barra da ação (seção fechada) > cabeçalho da seção (projeto) da ação > cabeçalho do blueprint
const onde = { barra: (id) => id === "f1", secao: (id) => (id === "f1" || id === "f2" ? "b1|p2" : null) };
assert.equal(A.claudeLugar(ag("a", { planoId: "t1", acaoId: "f1" }), aVista, onde), "plano:t1", "o plano à vista vence");
assert.equal(A.claudeLugar(ag("a", { planoId: "t9", acaoId: "f1" }), aVista, onde), "barra:f1", "seção fechada: ao lado da barra da ação");
assert.equal(A.claudeLugar(ag("a", { planoId: "t9", acaoId: "f2" }), aVista, onde), "secao:b1|p2", "ação fora da folha: no cabeçalho do projeto dela, não do primeiro");
assert.equal(A.claudeLugar(ag("a", { acaoId: "f9" }), aVista, onde), "cab:b1", "ação desconhecida: cabeçalho do blueprint");
const doisProjetos = A.claudeItens([ag("a", { acaoId: "f1", em: 5 }), ag("b", { papel: "revisor", acaoId: "f2", em: 4 })], () => false, { barra: () => false, secao: onde.secao });
assert.deepEqual(J(doisProjetos.map((i) => [i.lugar, i.indice])), [["secao:b1|p2", 0], ["secao:b1|p2", 1]], "duas ações do mesmo projeto: lado a lado no cabeçalho dele, sem empilhar no mesmo ponto");

// ---- limite: 1 principal + 4 subagentes; o resto vira "+N"; ordem: principal, depois quem avisou há menos tempo
const muitos = [ag("claude", { em: 100 }), ...[1, 2, 3, 4, 5, 6, 7].map((i) => ag(`sub${i}`, { papel: "construtor", em: 100 + i }))];
let v = A.claudeVisiveis(muitos);
assert.equal(v.visiveis.length, 5);
assert.equal(v.extras, 3);
assert.deepEqual(J(v.visiveis.map((a) => a.agente)), ["claude", "sub7", "sub6", "sub5", "sub4"], "os 4 mais recentes");
v = A.claudeVisiveis(muitos.slice(0, 5));
assert.deepEqual([v.visiveis.length, v.extras], [5, 0], "cinco cabem sem contador");
v = A.claudeVisiveis(muitos.slice(0, 6));
assert.deepEqual([v.visiveis.length, v.extras], [5, 1]);
assert.deepEqual(J(A.claudeVisiveis([])), { principal: null, visiveis: [], extras: 0 });

// ---- itens lado a lado: dois agentes no mesmo plano ganham índices 0 e 1 (principal primeiro); o "+N" entra no lugar do último
const dois = A.claudeItens([ag("sub", { papel: "revisor", planoId: "t1", em: 9 }), ag("claude", { planoId: "t1", em: 1 })], aVista);
assert.deepEqual(J(dois.map((i) => [i.a.agente, i.lugar, i.indice, i.papel])), [["claude", "plano:t1", 0, null], ["sub", "plano:t1", 1, "revisor"]]);
const espalhados = A.claudeItens([ag("claude", { planoId: "t1" }), ag("s1", { papel: "explorador", planoId: "t2", em: 5 }), ag("s2", { papel: "revisor", planoId: "t2", em: 4 }), ag("s3", { papel: "revisor", planoId: "t77", em: 3 })], aVista);
assert.deepEqual(J(espalhados.map((i) => [i.a.agente, i.lugar, i.indice])), [["claude", "plano:t1", 0], ["s1", "plano:t2", 0], ["s2", "plano:t2", 1], ["s3", "cab:b1", 0]]);
const cheio = A.claudeItens(muitos.map((a) => ({ ...a, planoId: "t1" })), aVista);
assert.equal(cheio.length, 6, "5 figuras + o contador");
assert.deepEqual(J([cheio[5].mais, cheio[5].lugar, cheio[5].indice]), [3, "plano:t1", 5], "o contador fica no lugar do último, depois dele");
assert.equal(A.claudeLarguraItem(cheio[5]), 26);
assert.equal(A.claudeLarguraItem(cheio[0]), 48);
assert.equal(A.claudeLarguraItem(cheio[1]), 48, "subagente com o mesmo tamanho do Claude");
assert.equal(A.claudeItens([], aVista).length, 0);

// ---- posição: as figuras se enfileiram para a esquerda do apoio, sem se sobrepor, com a mesma linha dos pés
const linha = { x: 200, y: 100, w: 300, h: 22 }; // a caixa do destaque do plano
const larg = [48, 48, 48];
const posicoes = [0, 1, 2].map((i) => J(A.claudePosicao("plano", linha, i, larg)));
assert.deepEqual(posicoes.map((q) => q.base), [127, 127, 127], "os pés um pouco abaixo da linha do plano (a figura fica de pé ao lado dela)");
assert.deepEqual(posicoes.map((q) => q.x), [146, 94, 42], "lado a lado, da direita para a esquerda");
for (let i = 1; i < 3; i++) assert(posicoes[i].x + larg[i] + A.CLAUDE_FOLGA <= posicoes[i - 1].x, "sem sobrepor a figura vizinha");
const cab = J(A.claudePosicao("cab", { x: 62, y: 80, w: 800, h: 86 }, 0, [48]));
assert(cab.x + 48 <= 62 + 800 - 100 && cab.base <= 80 + 86, "no cabeçalho a figura fica dentro da caixa, à esquerda do canto");
const cartao = J(A.claudePosicao("cartao", { x: 70, y: 200, w: 330, h: 152 }, 0, [48]));
assert(cartao.x + 48 <= 70 + 330, "no cartão da lista fica dentro da largura do cartão");

// ---- o Claude esperando: a próxima ação do blueprint, só da interface ----
const acao = (id, inicio, status, planos) => ({ id, nome: "Ação " + id, inicio, status, planos: planos.map((feito) => ({ feito })) });
const proxima = (acoes, ags = []) => A.claudeProximaAcao(J({ id: "b1", acoes }), ags)?.id ?? null;
assert.equal(proxima([acao("a", "2026-11-03", "planejado", [false]), acao("b", "2026-11-01", "planejado", [false])]), "b", "a primeira por data de início, não pela ordem do blueprint");
assert.equal(proxima([acao("a", "2026-11-01", "concluido", [true]), acao("b", "2026-11-02", "andamento", [true, false])]), "b", "ação concluída é pulada");
assert.equal(proxima([acao("a", "2026-11-01", "andamento", [true, true]), acao("b", "2026-11-02", "planejado", [false])]), "b", "ação sem plano pendente é pulada");
assert.equal(proxima([acao("a", "2026-11-01", "planejado", []), acao("b", "2026-11-02", "planejado", [false])]), "b", "ação sem nenhum plano é pulada");
assert.equal(proxima([acao("a", "2026-11-01", "andamento", [false]), acao("b", "2026-11-02", "planejado", [false])], [{ acaoId: "a", estado: "escrevendo" }]), "b", "ação com agente trabalhando é pulada");
assert.equal(proxima([acao("a", "2026-11-01", "andamento", [false]), acao("b", "2026-11-02", "planejado", [false])], [{ acaoId: "a", estado: "concluido" }]), "b", "o concluído ainda não visto também ocupa a ação");
assert.equal(proxima([acao("a", "2026-11-01", "andamento", [false])], [{ acaoId: "a", estado: "ocioso" }, { acaoId: "", estado: "escrevendo" }]), "a", "agente em repouso (ou sem ação) não ocupa");
assert.equal(proxima([acao("a", "2026-11-01", "planejado", [false]), acao("b", "2026-11-01", "planejado", [false])]), "a", "empate de data: a ordem do blueprint");
assert.equal(proxima([acao("a", "2026-11-01", "concluido", [true])]), null, "tudo concluído: ninguém espera");
assert.equal(proxima([]), null, "blueprint sem ações: ninguém espera");
assert.equal(A.claudeProximaAcao(null, []), null);
const esp = J(A.claudeAgenteEspera({ id: "b1" }, { id: "a", nome: "Tela" }, "2.4"));
assert.deepEqual([esp.estado, esp.espera, esp.acaoId, esp.blueprintId], ["aguardando", true, "a", "b1"]);
assert.equal(poseDoEstado(A.claudeEstadoDoEvento(esp)), "esperando", "o estado da espera dá a pose esperando");
assert(esp.texto.includes("2.4") && esp.texto.includes("Tela"));
const itensCab = J(A.claudeItens([ag("claude", { planoId: "" })], () => false)), p0 = { id: "b1" }, f0 = { id: "a" };
assert.equal(J(A.claudeItemEspera(p0, f0, "1.1", (id) => id === "a", itensCab)).lugar, "acao:a", "à vista e aberta: ao lado da barra da ação");
const emCab = J(A.claudeItemEspera(p0, f0, "1.1", () => false, itensCab));
assert.equal(J(A.claudeItemEspera(p0, f0, "1.1", () => false, [], () => "b1|p2")).lugar, "secao:b1|p2", "ação fora de vista: a espera fica no cabeçalho do projeto dela");
assert.deepEqual([emCab.lugar, emCab.indice, emCab.papel, emCab.principal], ["cab:b1", 1, null, false], "fora de vista: no cabeçalho, depois do agente que já está lá");
assert.equal(J(A.claudeItens([ag("claude")], () => false)).length, 1, "a espera não entra no limite nem no +N (é um item à parte)");

console.log("claude-logica: ok (principal, variante por papel, pose por estado, lugar, limite + \"+N\" e ordem lado a lado)");
