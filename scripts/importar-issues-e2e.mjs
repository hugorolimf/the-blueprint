// Importar issues do GitHub como proposta (tópico `importar` do guia), sem rede e sem agente: a saída do `gh issue list` gravada em
// scripts/fixtures/gh-issues.json é mapeada pela tabela do tópico (o que o agente faria) e enviada por propor_blueprint pelo `blueprint --mcp`.
// Confere que a proposta entra na fila numerada (uma não apaga a outra), que o formato inválido é recusado com erro claro e que nada grava antes do clique.
// Uso: node scripts/importar-issues-e2e.mjs [caminho do blueprint.exe]   (padrão: target/debug/blueprint.exe; ~3 s, sem gastar tokens)
import assert from "node:assert/strict";
import { readFile, rm } from "node:fs/promises";
import { join } from "node:path";
import { exePadrao, ligarServidor, mcpStdio, pastaDeDados, raiz } from "./lib-e2e-rust.mjs";

const REPO = "exemplo/loja";
const dia = (base, n) => new Date(base.getTime() + n * 864e5).toISOString().slice(0, 10);

/** A tabela do tópico `importar`: issue vira ação, item não marcado da lista de tarefas vira plano, marco vira projeto, rótulos vão na nota. */
function mapear(issues, repo, hoje = new Date()) {
  const marcos = [...new Set(issues.map((i) => i.milestone?.title).filter(Boolean))];
  const ordem = [...issues].sort((a, b) => (marcos.indexOf(a.milestone?.title) + 1 || 99) - (marcos.indexOf(b.milestone?.title) + 1 || 99) || a.number - b.number);
  const acoes = ordem.map((issue, n) => {
    const linhas = (issue.body || "").split("\n");
    const tarefas = linhas.map((l) => l.match(/^\s*- \[ \] (.+)$/)?.[1]).filter(Boolean).slice(0, 6);
    const resumo = linhas.filter((l) => !/^\s*- \[[ xX]\] /.test(l)).join(" ").replace(/\s+/g, " ").trim().slice(0, 600);
    const rotulos = issue.labels.map((l) => l.name).join(", ");
    const planos = (tarefas.length ? tarefas : [`Implement #${issue.number}`]).map((texto) => ({
      texto, horas: 2, descricao: `Scope: ${texto} (GitHub issue #${issue.number}). Expected result: done as the issue asks. Validation: the issue's own check.`,
    }));
    return {
      nome: issue.title, inicio: dia(hoje, 1 + 2 * n), fim: dia(hoje, 2 + 2 * n), status: "planejado", planos,
      observacao: `${resumo}${rotulos ? `\nLabels: ${rotulos}` : ""}\nGitHub issue ${repo}#${issue.number}`,
      ...(issue.milestone ? { projeto: issue.milestone.title } : {}),
    };
  });
  return { mensagem: `Import ${issues.length} open issues of ${repo}`, blueprint: { projetos: marcos.map((nome) => ({ nome })) }, acoes };
}

const exe = exePadrao();
const d = await pastaDeDados("blueprint-importar");
const alvo = { workspace_id: "sm", blueprint_id: "p1" };
let servidor;
try {
  const issues = JSON.parse(await readFile(join(raiz, "scripts/fixtures/gh-issues.json"), "utf8"));
  for (const i of issues) assert.ok(!("assignees" in i) && !("author" in i), "a saída gravada não traz responsável nem autor");
  const proposta = mapear(issues, REPO);
  servidor = await ligarServidor(exe, d);
  const mcp = mcpStdio(exe, d.env);
  await mcp.iniciar();

  const semProjetos = await mcp.chamar("propor_blueprint", { ...alvo, ...proposta, blueprint: undefined });
  assert.equal(semProjetos.erro, true);
  assert.match(semProjetos.texto, /project "Payments" does not exist.*blueprint\.projetos/s, semProjetos.texto);
  const comFeito = await mcp.chamar("propor_blueprint", { ...alvo, ...proposta, acoes: [{ ...proposta.acoes[0], planos: [{ ...proposta.acoes[0].planos[0], feito: true }] }] });
  assert.equal(comFeito.erro, true);
  assert.match(comFeito.texto, /`feito` is not allowed in a proposal/, comFeito.texto);
  console.log("ok: o formato inválido (marco sem projeto declarado, plano já feito) é recusado com erro claro");

  const r1 = await mcp.chamar("propor_blueprint", { ...alvo, ...proposta });
  assert.equal(r1.erro, false, r1.texto);
  const r2 = await mcp.chamar("propor_blueprint", { ...alvo, ...proposta, mensagem: "Import again" });
  assert.equal(r2.erro, false, r2.texto);
  mcp.fechar();

  const fila = (await (await fetch(d.url + "/api/mcp/fila")).json()).itens.filter((i) => i.tipo === "proposta");
  assert.deepEqual(fila.map((i) => i.numero), [1, 2], "fila numerada: a segunda importação não apaga a primeira");
  const acoes = fila[0].proposta.acoes;
  assert.deepEqual(acoes.map((a) => a.nome), ["Checkout with saved cards", "Refund flow", "Slow product search"], "na ordem dos marcos; sem marco no fim");
  assert.deepEqual(acoes[0].planos.map((p) => p.texto), ["Store the card token", "Show saved cards at checkout", "Delete a saved card"], "item já marcado não vira plano");
  assert.deepEqual(acoes[2].planos.map((p) => p.texto), ["Implement #15"], "issue sem lista de tarefas vira um plano só");
  assert.match(acoes[0].observacao, /Labels: feature, payments\nGitHub issue exemplo\/loja#12$/);
  assert.ok(acoes.every((a) => a.planos.every((p) => !p.feito)), "a ordem escrita na issue continua só texto: nada vem marcado");
  console.log("ok: 2 propostas na fila (nº 1 e 2), 3 ações, 6 planos, marco como projeto e a nota com a origem da issue");

  const dados = await (await fetch(d.url + "/api/dados")).json();
  assert.equal(dados.blueprints.find((b) => b.id === "p1").acoes.length, 1, "nada gravou antes do clique em Aplicar");
  console.log("ok: os dados continuam iguais até o usuário aplicar na tela Planejar");
} finally {
  servidor?.kill();
  await rm(d.dir, { recursive: true, force: true }).catch(() => {});
}
