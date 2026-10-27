// O editor manual da apresentação com IA (web/js/apresentacao-ia-ops.js, -esquema.js, -salvar.js e os corpos de pedido de -logica.js): as operações, o Desfazer, a união com o que o
// servidor trouxe, o lote do salvador e os pedidos de gerar e de ajustar. Lógica pura, sem DOM: os scripts clássicos rodam num contexto isolado, como no navegador.
//   node apresentacao-ia-editar.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const ctx = vm.createContext({ tr: (t, p = {}) => t.replace(/\{(\w+)\}/g, (_, k) => p[k] ?? ""), setTimeout, clearTimeout });
for (const f of ["logica", "ops", "esquema", "salvar"]) vm.runInContext(readFileSync(new URL(`web/js/apresentacao-ia-${f}.js`, import.meta.url), "utf8"), ctx, { filename: f });
vm.runInContext("this.api = { apOpsAplica, apOpsAplicaLista, apOpsJuntar, apOpsCorpos, apOpsReaplicar, apOpsFraseErro, apOpsFragmento, apOpsFragmentoSecao, apOpsDestino, apIaSalvador, apIaListas, apIaGeral, AP_IA_SECOES_FIGURA, AP_IA_NOVO, apIaCorpoTerminal, apIaCorpoTerminalAjuste, apIaCorpoPedidoAjuste, apIaTextoAjuste, apIaMensagemAjuste, apIaDescNivel, apIaLista };", ctx);
const api = ctx.api, j = (x) => JSON.parse(JSON.stringify(x)); // o vm tem os seus próprios Object e Array: compara por valor
const doc = () => ({
  conteudo: { titulo: "Plano", publico: "cliente", resumo: ["a", "b"], etapas: [{ id: "e1", titulo: "Um", figura: "capa", marcos: [] }, { id: "e2", titulo: "Dois", figura: null, marcos: [{ data: "2026-11-01", nome: "m" }] }], riscos: [] },
  figuras: [{ id: "capa", titulo: "Capa", chaves: [], codigo: "function (L) { return { svg: 'x' }; }" }, { id: "outra", titulo: "Outra", codigo: "function (L) { return 1; }" }],
});
const aplica = (d, ...ops) => { const r = api.apOpsAplicaLista(d, ops); assert.deepEqual(j(r.falhas), [], "nenhuma operação falhou"); return r; };

// definir, inserir, remover, mover: cada uma muda só o seu ponto e a inversa devolve o documento como era
{
  const d0 = doc();
  for (const op of [
    { op: "definir", caminho: "etapas.1.titulo", valor: "Novo" },
    { op: "definir", caminho: "subtitulo", valor: "novo campo" },
    { op: "inserir", caminho: "etapas", indice: 1, valor: { id: "e9", titulo: "Meio", marcos: [] } },
    { op: "inserir", caminho: "etapas.1.marcos", valor: { data: "2026-11-02", nome: "x" } },
    { op: "remover", caminho: "resumo.0" },
    { op: "remover", caminho: "etapas.0.figura" },
    { op: "mover", caminho: "etapas.0", para: 1 },
    { op: "figura_definir", figura: "capa", campo: "titulo", valor: "Capa nova" },
    { op: "figura_trecho", figura: "outra", de: "return 1", para: "return 2" },
    { op: "figura_remover", figura: "capa" },
  ]) {
    const antes = j(d0), r = aplica(d0, op);
    assert.notDeepEqual(j(r.doc), antes, `${op.op} muda o documento`);
    assert.deepEqual(j(d0), antes, "a original não é tocada");
    assert.deepEqual(j(aplica(r.doc, ...r.inversos).doc), antes, `a inversa de ${op.op} ${op.caminho || op.figura} devolve o que era`);
  }
  // um campo de figura que não existia: o Desfazer o deixa vazio (não há operação que apague a chave), e `secao` volta a "nenhuma" (null)
  const nova = aplica(d0, { op: "figura_definir", figura: "capa", campo: "secao", valor: "visao" });
  assert.equal(nova.doc.figuras[0].secao, "visao");
  assert.equal(aplica(nova.doc, ...nova.inversos).doc.figuras[0].secao, null);
  const r = aplica(d0, { op: "inserir", caminho: "etapas", valor: { id: "ultima" } });
  assert.equal(r.doc.conteudo.etapas[2].id, "ultima", "sem índice: no fim");
  assert.deepEqual(j(aplica(d0, { op: "mover", caminho: "etapas.0", para: 1 }).doc.conteudo.etapas.map((e) => e.id)), ["e2", "e1"]);
}

// o que não cabe no documento falha com `codigo` e não leva as outras junto
{
  const r = api.apOpsAplicaLista(doc(), [{ op: "definir", caminho: "etapas.7.titulo", valor: "x" }, { op: "definir", caminho: "titulo", valor: "Novo título" }, { op: "figura_definir", figura: "nada", campo: "titulo", valor: "x" }, { op: "figura_trecho", figura: "outra", de: "n", para: "g" }, { op: "figura_trecho", figura: "outra", de: "inexistente", para: "g" }, { op: "figura_definir", figura: "capa", campo: "codigo", valor: "x" }, { op: "xx" }]);
  assert.deepEqual(j(r.falhas.map((f) => f.codigo)), ["caminho", "figura-desconhecida", "trecho-nao-unico", "trecho-nao-unico", "campo", "operacao"]);
  assert.equal(r.doc.conteudo.titulo, "Novo título", "a operação válida entrou mesmo com as outras falhando");
  const trecho = api.apOpsAplicaLista(doc(), [{ op: "figura_trecho", figura: "outra", de: "n", para: "g" }]).falhas[0];
  assert.equal(trecho.codigo, "trecho-nao-unico");
  assert.equal(api.apOpsAplicaLista(doc(), [{ op: "figura_trecho", figura: "capa", de: "function (L) { return { svg: 'x' }; }", para: "function (L) { return { svg: 'y' }; }" }]).doc.figuras[0].codigo.includes("'y'"), true, "o código inteiro troca como um trecho só");
}

// várias digitações no mesmo campo viram uma; o resto segue na ordem
assert.deepEqual(j(api.apOpsJuntar([{ op: "definir", caminho: "titulo", valor: "A" }, { op: "definir", caminho: "titulo", valor: "AB" }, { op: "definir", caminho: "cliente", valor: "x" }, { op: "definir", caminho: "titulo", valor: "ABC" }])),
  [{ op: "definir", caminho: "titulo", valor: "AB" }, { op: "definir", caminho: "cliente", valor: "x" }, { op: "definir", caminho: "titulo", valor: "ABC" }]);

// a troca da lista de figuras vai como `figuras` e quebra o lote em dois pedidos, na ordem; o autor vai junto
{
  const ids = { workspaceId: "w", blueprintId: "b", apresentacaoId: "ap1" };
  const corpos = j(api.apOpsCorpos(ids, [{ op: "definir", caminho: "titulo", valor: "x" }, { op: "__figuras", valor: [{ id: "f" }] }, { op: "definir", caminho: "cliente", valor: "y" }]));
  assert.equal(corpos.length, 3);
  assert.deepEqual(Object.keys(corpos[0]).sort(), ["apresentacaoId", "autor", "blueprintId", "operacoes", "workspaceId"]);
  assert.equal(corpos[0].autor, "usuario");
  assert.deepEqual(corpos[1].figuras, [{ id: "f" }]);
  assert.equal(corpos[1].operacoes, undefined);
  assert.equal(api.apOpsCorpos(ids, []).length, 0);
}

// o servidor trouxe um documento novo (o agente mexeu em outro ponto): o que o usuário ainda não mandou vai por cima e nada que ele digitou se perde
{
  const servidor = doc();
  servidor.conteudo.etapas[1].titulo = "Dois (agente)"; // o agente mudou a etapa 2
  const pendentes = [{ op: "definir", caminho: "etapas.0.titulo", valor: "Um (usuário)" }, { op: "definir", caminho: "etapas.9.titulo", valor: "some" }];
  const r = api.apOpsReaplicar(servidor, pendentes);
  assert.equal(r.doc.conteudo.etapas[0].titulo, "Um (usuário)");
  assert.equal(r.doc.conteudo.etapas[1].titulo, "Dois (agente)");
  assert.equal(r.falhas.length, 1, "a operação que já não cabe é pulada, não derruba as outras");
}

// frases de erro do servidor e o fragmento da página
assert.equal(api.apOpsFraseErro(400, { codigo: "falta-titulo" }), "O título não pode ficar vazio.");
assert.equal(api.apOpsFraseErro(400, { codigo: "interno-no-cliente" }).includes("Ações"), true);
assert.match(api.apOpsFraseErro(422, { codigo: "trecho-ambiguo" }), /mais de uma vez/, "o código do servidor para um trecho repetido");
assert.match(api.apOpsFraseErro(404, { codigo: "figura-nao-encontrada" }), /figura já não existe/);
assert.equal(api.apOpsFraseErro(500, { erro: "boom" }), "boom");
assert.equal(api.apOpsFraseErro(502, {}), "Não foi possível salvar (erro 502).");
assert.equal(api.apOpsFragmento("etapas.2.titulo"), "etapas-3", "a página restaura #etapas-3");
assert.equal(api.apOpsFragmento("etapas"), "s-etapas");
assert.equal(api.apOpsFragmento("precisamosDeVoce.0.texto"), "s-voce");
assert.equal(api.apOpsFragmento("titulo"), "s-visao");
assert.equal(api.apOpsFragmentoSecao("linha"), "s-linha");
assert.equal(api.apOpsFragmentoSecao("recebe"), "s-entregas");
{ // o destino do postMessage "ap-ir": a seção (nomes que a página aceita) e, nas etapas, o número 1 a n
  const d = doc(); d.figuras[0].secao = "precisamos";
  assert.deepEqual(j(api.apOpsDestino("etapas.1.titulo", d)), { secao: "etapas", etapa: 2 });
  assert.deepEqual(j(api.apOpsDestino("etapas", d)), { secao: "etapas" });
  assert.deepEqual(j(api.apOpsDestino("precisamosDeVoce.0.texto", d)), { secao: "voce" });
  assert.deepEqual(j(api.apOpsDestino("riscos.0.titulo", d)), { secao: "decisoes" });
  assert.deepEqual(j(api.apOpsDestino("titulo", d)), { secao: "visao" });
  assert.deepEqual(j(api.apOpsDestino("figuras.capa.titulo", d)), { secao: "voce" }, "a figura vai para a sessão dela");
  assert.deepEqual(j(api.apOpsDestino("figuras.outra.titulo", d)), { secao: "etapas" }, "sem sessão, para o fim das etapas");
}

// o esquema do editor: cada lista do contrato tem caminho e campos; a lista de seções de figura é a do contrato; a seção Ações só existe para a visão interna
{
  const l = j(api.apIaListas());
  assert.deepEqual(Object.keys(l), ["resumo", "entregas", "etapas", "precisamosDeVoce", "decisoes", "riscos", "acoes"]);
  assert.equal(l.acoes.caminho, "interno.acoes");
  assert.equal(l.acoes.interno, true);
  assert.equal(l.etapas.sub[0].chave, "marcos");
  assert.deepEqual(j(api.AP_IA_SECOES_FIGURA), ["visao", "recebe", "etapas", "linha", "precisamos", "decisoes", "riscos", "acoes"]);
  assert.deepEqual(j(api.apIaGeral().map((c) => c[0])), ["titulo", "subtitulo", "cliente", "data", "idioma", "publico", "nivel"]);
  const novo = api.AP_IA_NOVO.etapas(doc());
  assert.equal(novo.id, "e3", "o id novo não repete");
  assert.deepEqual(j(api.AP_IA_NOVO.decisoes()), { titulo: "", texto: "", estado: "aberta" });
}

// o salvador: junta as mudanças numa só chamada, avisa o estado, guarda o Desfazer, recusa vai a "erro" e a rede que cai devolve o lote
{
  const chamadas = [], estados = [], avisos = [], recusas = [], agendados = [];
  let resposta = { status: 200, json: { ok: true, editadoEm: "2026-10-26T10:00:00Z", avisos: ["w1"] } };
  const s = api.apIaSalvador({
    ids: { workspaceId: "w", blueprintId: "b", apresentacaoId: "ap1" }, atraso: 700,
    postar: async (corpo) => { chamadas.push(j(corpo)); if (resposta === "rede") throw new Error("rede"); return resposta; },
    esperar: (ms, fn) => { agendados.push(ms); return () => {}; },
    aoEstado: (e) => estados.push(e), aoAvisos: (l) => avisos.push(j(l)), aoRecusa: (st, jj) => recusas.push([st, j(jj)]),
  });
  const d = doc();
  const digita = (valor, agora) => { const r = api.apOpsAplicaLista(d, [{ op: "definir", caminho: "titulo", valor }]); s.mudar([{ op: "definir", caminho: "titulo", valor }], j(r.inversos), { caminho: "titulo", junta: true, agora }); };
  digita("A", 1000); digita("AB", 1200); digita("ABC", 1400);
  assert.deepEqual(j(agendados), [700, 700, 700], "cada mudança reagenda a pausa");
  assert.equal(s.estado(), "pendente");
  assert.equal(chamadas.length, 0, "nada vai ao servidor antes da pausa");
  s.mudar([{ op: "definir", caminho: "cliente", valor: "C" }], [{ op: "remover", caminho: "cliente" }], { caminho: "cliente", agora: 1500 });
  await s.descarregar();
  assert.equal(chamadas.length, 1, "um lote, uma chamada");
  assert.deepEqual(j(chamadas[0].operacoes), [{ op: "definir", caminho: "titulo", valor: "ABC" }, { op: "definir", caminho: "cliente", valor: "C" }], "as digitações do mesmo campo viram uma");
  assert.equal(chamadas[0].autor, "usuario");
  assert.deepEqual(j(estados), ["pendente", "pendente", "pendente", "pendente", "salvando", "salvo"]);
  assert.deepEqual(j(avisos), [["w1"]]);
  assert.equal(s.ultimoEm(), "2026-10-26T10:00:00Z");
  assert.equal(s.ocupado(), false);
  // Desfazer: a digitação "A", "AB", "ABC" foi UMA mudança (inverso do começo); depois dela veio a do cliente
  assert.equal(s.pode(), true);
  assert.deepEqual(j(s.desfazer()), [{ op: "remover", caminho: "cliente" }]);
  assert.deepEqual(j(s.desfazer()[0]), j(api.apOpsAplicaLista(doc(), [{ op: "definir", caminho: "titulo", valor: "A" }]).inversos[0]), "o inverso guardado é o do começo da digitação");
  assert.equal(s.desfazer(), null, "pilha vazia");
  await s.descarregar();
  assert.equal(chamadas.length, 2, "o desfazer também vai ao servidor");
  // recusa: o lote sai, o estado vira erro e o editor é avisado (ele volta ao último salvo)
  resposta = { status: 400, json: { erro: "x", codigo: "falta-titulo" } };
  s.mudar([{ op: "definir", caminho: "titulo", valor: "" }], [], { caminho: "titulo" });
  await s.descarregar();
  assert.equal(s.estado(), "erro");
  assert.deepEqual(recusas, [[400, { erro: "x", codigo: "falta-titulo" }]]);
  assert.equal(s.pendentes().length, 0);
  // a rede cai: o lote volta e segue na próxima tentativa
  resposta = "rede";
  s.mudar([{ op: "definir", caminho: "cliente", valor: "Z" }], [], { caminho: "cliente" });
  await s.descarregar();
  assert.equal(s.estado(), "erro");
  assert.equal(s.pendentes().length, 1, "nada se perde");
  resposta = { status: 200, json: { ok: true, editadoEm: "2026-10-26T11:00:00Z" } };
  await s.descarregar();
  assert.equal(s.estado(), "salvo");
  assert.equal(s.pendentes().length, 0);
}

// os pedidos: gerar no terminal do app, ajustar por conversa (sessão com ou sem canal) e as frases
{
  const base = { workspaceId: "w", blueprintId: "b" };
  assert.deepEqual(j(api.apIaCorpoTerminal({ ...base, publico: "cliente", nivel: "rapido", titulo: "Apresentação: X" })), { ...base, apresentar: { publico: "cliente", nivel: "rapido" }, titulo: "Apresentação: X" });
  assert.equal(api.apIaCorpoTerminal({ ...base, publico: "cliente", nivel: "epico" }), null, "Épico não sai da tela");
  assert.equal(api.apIaCorpoTerminal({ ...base, publico: "todos", nivel: "rapido" }), null);
  assert.equal(api.apIaTextoAjuste("  deixe   o texto\n curto  "), "deixe o texto curto");
  assert.equal(api.apIaTextoAjuste("x".repeat(3000)).length, 2000);
  assert.deepEqual(j(api.apIaCorpoTerminalAjuste({ ...base, apresentacaoId: "ap1", texto: "  mude  isto ", titulo: "T" })), { ...base, apresentacaoId: "ap1", ajuste: "mude isto", titulo: "T" });
  assert.equal(api.apIaCorpoTerminalAjuste({ ...base, apresentacaoId: "../x", texto: "a" }), null, "o id vira parte de uma URL: só o que o servidor gera");
  assert.equal(api.apIaCorpoTerminalAjuste({ ...base, apresentacaoId: "ap1", texto: "   " }), null, "pedido vazio não vai");
  assert.deepEqual(j(api.apIaCorpoPedidoAjuste({ ...base, apresentacaoId: "ap1", texto: "oi" })), { ...base, apresentacaoId: "ap1", texto: "oi" });
  assert.deepEqual(j(api.apIaCorpoPedidoAjuste({ ...base, apresentacaoId: "ap1", texto: "oi", sessao: { id: "s1", canal: true } })), { ...base, apresentacaoId: "ap1", texto: "oi", sessaoId: "s1" });
  assert.deepEqual(j(api.apIaCorpoPedidoAjuste({ ...base, apresentacaoId: "ap1", texto: "oi", sessao: { id: "s2", canal: false } })), { ...base, apresentacaoId: "ap1", texto: "oi", sessaoId: "s2", semCanal: true });
  assert.match(api.apIaMensagemAjuste(200, { ok: true }, "app"), /^Terminal do app aberto/);
  assert.match(api.apIaMensagemAjuste(200, { ok: true }, "sessao"), /^Pedido enviado à sessão/);
  assert.match(api.apIaMensagemAjuste(404, {}, "sessao"), /não existe mais/);
  assert.match(api.apIaMensagemAjuste(410, { codigo: "sessao-caiu" }, "sessao"), /não está mais conectada/);
  assert.equal(api.apIaLista([{ id: "ap1", titulo: "T", publico: "cliente", nivel: "rapido", em: "2026-10-26T10:00:00Z", editadoEm: "2026-10-26T12:00:00Z" }])[0].editadoEm, "2026-10-26T12:00:00Z");
  const d = j(api.apIaDescNivel());
  assert.match(d.rapido, /Uma figura da IA/);
  assert.match(d.padrao, /diferente por sessão/);
  assert.match(d.completo, /máquina densa/);
  assert.ok(!/original|torre|estrutura/i.test(Object.values(d).join(" ")), "nenhuma menção à figura original nem às torres");
}
console.log("apresentacao-ia-editar: ok");
