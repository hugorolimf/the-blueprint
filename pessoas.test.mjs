// Painéis de equipe (web/js/pessoas.js, pessoas-convites.js e chave-mcp.js, módulos ES): lista de pessoas, corpo e estado dos convites, comandos da chave.
// A tela inteira (abrir, criar, copiar uma vez, remover) é coberta por scripts/convite-tela-e2e.mjs; aqui ficam as regras puras.
//   node pessoas.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

// os módulos de web/js são ES num package.json sem "type": o Node os reconhece sozinho e só avisa; o aviso fica de fora da saída do teste
const avisar = process.emitWarning;
process.emitWarning = (w, ...r) => (/MODULE_TYPELESS/.test(JSON.stringify(r)) ? undefined : avisar.call(process, w, ...r));

// tr/trn do i18n.js no escopo global, como no navegador (os módulos os usam ao serem chamados)
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), { filename: "i18n.js" });
vm.runInThisContext(`I18N.ativo = "pt-BR"`);
const { haQuanto, linhasPessoas } = await import("./web/js/pessoas.js");
const { corpoDoConvite, estadoDoConvite } = await import("./web/js/pessoas-convites.js");
const { comandosDaChave } = await import("./web/js/chave-mcp.js");

// última atividade em texto curto
const agora = Date.parse("2027-01-15T12:00:00Z");
assert.equal(haQuanto(null, agora), "nunca");
assert.equal(haQuanto("2027-01-15T11:59:30Z", agora), "agora há pouco");
assert.equal(haQuanto("2027-01-15T11:30:00Z", agora), "há 30 min");
assert.equal(haQuanto("2027-01-15T07:00:00Z", agora), "há 5 h");
assert.equal(haQuanto("2027-01-14T11:00:00Z", agora), "há 1 dia");
assert.equal(haQuanto("2027-01-10T12:00:00Z", agora), "há 5 dias");

// lista: você primeiro, depois por papel e nome; o dono muda e remove os outros, nunca a própria linha
const membros = [
  { id: "m3", nome: "Carla", papel: "supervisor", blueprintId: "", ultimaAtividade: null, voce: false },
  { id: "m2", nome: "Bruno", papel: "colaborador", blueprintId: "b1", ultimaAtividade: "2027-01-15T11:30:00Z", voce: false },
  { id: "m1", nome: "Ana", papel: "dono", blueprintId: "", ultimaAtividade: "2027-01-15T11:59:00Z", voce: true },
  { id: "m4", nome: "Abel", papel: "colaborador", blueprintId: "", ultimaAtividade: null, voce: false },
];
const l = linhasPessoas(membros, true, agora);
assert.deepEqual(l.map((m) => m.nome), ["Ana", "Abel", "Bruno", "Carla"]);
assert.deepEqual(l.map((m) => m.podeMudar), [false, true, true, true], "o dono não mexe na própria linha");
assert.equal(l[2].escopo, "só um blueprint");
assert.equal(l[1].escopo, "workspace inteiro");
assert.equal(l[2].quando, "há 30 min");
assert.ok(linhasPessoas(membros, false, agora).every((m) => !m.podeMudar), "supervisor só lista");

// corpo do convite: valida como o servidor (seção 5) e manda só o que vale
assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "colaborador" }), { workspaceId: "w1", papel: "colaborador", validadeHoras: 168, usosMax: 1 }, "padrão: 7 dias, uma pessoa");
assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "supervisor", blueprintId: "b1", validadeHoras: "24", usos: "" }), { workspaceId: "w1", papel: "supervisor", validadeHoras: 24, usosMax: null, blueprintId: "b1" });
assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "visitante", blueprintId: "b1", abre: "apresentacao", usos: "1" }), { workspaceId: "w1", papel: "visitante", validadeHoras: 168, usosMax: null, blueprintId: "b1", abre: "apresentacao" }, "link público: sem limite de usos, com abre");
assert.equal(corpoDoConvite({ workspaceId: "w1", papel: "visitante", abre: "x", blueprintId: "b" }).abre, "folha", "abre desconhecido vira folha");
assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "visitante" }), { erro: "falta-blueprint" }, "link público precisa de um blueprint");
assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "admin" }), { erro: "papel-invalido" });
for (const v of [0, 721, "1.5", "abc"]) assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "colaborador", validadeHoras: v }), { erro: "validade-invalida" }, `validade ${v}`);
for (const u of ["0", "1001", "2.5"]) assert.deepEqual(corpoDoConvite({ workspaceId: "w1", papel: "colaborador", usos: u }), { erro: "usos-invalidos" }, `usos ${u}`);
assert.equal("blueprintId" in corpoDoConvite({ workspaceId: "w1", papel: "dono" }), false, "sem blueprint: o workspace inteiro");

// estado do convite
const c = (x) => ({ revogado: false, expiraEm: "2027-01-16T00:00:00Z", usosMax: 1, usos: 0, ...x });
assert.equal(estadoDoConvite(c({}), agora), "ativo");
assert.equal(estadoDoConvite(c({ revogado: true }), agora), "revogado");
assert.equal(estadoDoConvite(c({ expiraEm: "2027-01-15T11:00:00Z" }), agora), "vencido");
assert.equal(estadoDoConvite(c({ usos: 1 }), agora), "esgotado");
assert.equal(estadoDoConvite(c({ usosMax: null, usos: 50 }), agora), "ativo", "link público não esgota");

// comandos da chave (seção 4): a chave vai por --env, nunca como argumento solto do --mcp
const k = comandosDaChave("https://equipe.exemplo.com", "bpk_abc");
assert.equal(k.claude, "claude mcp add blueprint-equipe --env BLUEPRINT_URL=https://equipe.exemplo.com --env BLUEPRINT_CHAVE=bpk_abc -- blueprint --mcp");
assert.equal(k.codex, "codex mcp add blueprint-equipe --env BLUEPRINT_URL=https://equipe.exemplo.com --env BLUEPRINT_CHAVE=bpk_abc -- blueprint --mcp");

// inglês pelo catálogo
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "i18n", "en-pessoas.js"), "utf8"));
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "i18n", "en-resto.js"), "utf8"));
vm.runInThisContext(`I18N.ativo = "en"`);
assert.equal(haQuanto("2027-01-10T12:00:00Z", agora), "5 days ago");
assert.equal(haQuanto("2027-01-15T11:30:00Z", agora), "30 min ago");
assert.equal(linhasPessoas(membros, true, agora)[2].escopo, "one blueprint only");
console.log("pessoas: ok (última atividade, ordem e permissões da lista, corpo e estado dos convites, comandos da chave, inglês)");
