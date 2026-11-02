// O formato genérico de plano do The Blueprint (blueprint.plano/1): uma pasta com `plano.json` e as figuras. Serve para qualquer projeto
// e qualquer agente: só `blueprint` e as ações são obrigatórios; figura, tabela, decisão e pergunta entram quando o projeto tiver.
// Aqui: ler a pasta, validar com mensagens que o agente consegue corrigir, e somar o que vai aparecer na folha.
import { existsSync, readFileSync, statSync, lstatSync } from "node:fs";
import { extname, join, normalize, isAbsolute } from "node:path";
import { compilarEstilo, resolver } from "./estilo.mjs";
import { acharCiclo } from "./agentes.mjs";

export const FORMATO = "blueprint.plano/1";
const LIMITE_FIGURA = 2 * 1024 * 1024, LIMITE_IMAGEM = 4 * 1024 * 1024;
const PROIBIDO_NO_SVG = /<script\b|<foreignObject\b|\son\w+\s*=|(?:href|src)\s*=\s*"(?!#)/i;

export const acoesDe = (p) => p.projetos.flatMap((pr) => pr.acoes);

/** Um plano pode vir só como texto; aqui ele ganha id, e é esse o plano que um agente recebe. */
export const normalizarPlanos = (a) => (a.planos ?? []).map((q, i) => (typeof q === "string" ? { id: `${a.id}.${i + 1}`, texto: q } : { id: `${a.id}.${i + 1}`, ...q }));

/** A lista de erros de um plano.json e das suas figuras. Vazia: pode salvar. */
export function validar(p, pasta) {
  const erros = [];
  if (p.formato !== FORMATO) erros.push(`formato: use "${FORMATO}"`);
  if (!p.blueprint) erros.push("blueprint: falta o nome");
  if (!Array.isArray(p.projetos) || !p.projetos.length) return [...erros, "projetos: é preciso ao menos um, com ações (o nome é opcional)"];
  const ids = new Set();
  for (const a of p.projetos.flatMap((pr) => pr.acoes ?? [])) {
    if (!a.id || !a.nome) erros.push(`ação ${a.id ?? "?"}: id e nome são obrigatórios`);
    if (ids.has(a.id)) erros.push(`ação ${a.id}: id repetido; os ids ligam uma versão do plano à seguinte`);
    ids.add(a.id);
    if ((a.de && !a.ate) || (a.ate && !a.de)) erros.push(`ação ${a.id}: use de e ate juntos, ou nenhum dos dois`);
    for (const d of a.detalhes ?? []) erros.push(...validarFigura(a.id, d, pasta));
  }
  const planos = p.projetos.flatMap((pr) => pr.acoes ?? []).flatMap(normalizarPlanos), idsPlanos = new Set();
  for (const q of planos) {
    if (!q.texto) erros.push(`plano ${q.id}: falta o texto`);
    if (idsPlanos.has(q.id)) erros.push(`plano ${q.id}: id repetido; cada plano é o trabalho de um agente`);
    idsPlanos.add(q.id);
  }
  for (const q of planos) for (const d of q.depende_de ?? []) if (!planos.some((o) => o.id === d)) erros.push(`plano ${q.id}: não existe o plano ${d}`);
  const ciclo = acharCiclo(planos);
  if (ciclo) erros.push(`planos em ciclo: ${ciclo.join(" → ")}; nenhum agente conseguiria começar`);
  for (const d of p.decisoes ?? []) for (const a of d.acoes ?? []) if (!ids.has(a)) erros.push(`decisão ${d.id}: não existe a ação ${a}`);
  return erros;
}

function validarFigura(acao, d, pasta) {
  const nome = `detalhe ${acao}-${d.id}`;
  if (!d.figura) return []; // detalhe só com título e legenda é válido
  if (isAbsolute(d.figura) || normalize(d.figura).startsWith("..")) return [`${nome}: a figura precisa estar dentro da pasta do plano`];
  const arq = join(pasta, d.figura);
  if (!existsSync(arq)) return [`${nome}: não achei ${d.figura}`];
  if (lstatSync(arq).isSymbolicLink()) return [`${nome}: link simbólico não é aceito`];
  const ext = extname(arq).toLowerCase();
  if (![".svg", ".png", ".jpg", ".jpeg", ".webp"].includes(ext)) return [`${nome}: use SVG, PNG, JPG ou WebP`];
  const limite = ext === ".svg" ? LIMITE_FIGURA : LIMITE_IMAGEM;
  if (statSync(arq).size > limite) return [`${nome}: passa de ${limite / 1024 / 1024} MB`];
  if (ext === ".svg" && PROIBIDO_NO_SVG.test(readFileSync(arq, "utf8"))) return [`${nome}: SVG com script, foreignObject, evento ou arquivo externo`];
  return [];
}

/** Lê a pasta do plano: devolve o plano com o SVG de cada figura e o estilo pronto no tema do app. */
export function lerPacote(pasta) {
  const p = JSON.parse(readFileSync(join(pasta, "plano.json"), "utf8"));
  const erros = validar(p, pasta);
  if (erros.length) throw new Error(`${pasta}:\n  ${erros.join("\n  ")}`);
  const { css, vars } = compilarEstilo(p.estilo ? readFileSync(join(pasta, p.estilo), "utf8") : "", p.tema);
  p.css = css;
  // a versão anterior, quando existe: é dela que sai o diff (no app, ela vem do banco)
  const ant = join(pasta, "anterior.json");
  if (existsSync(ant)) {
    p.anterior = JSON.parse(readFileSync(ant, "utf8"));
    for (const pr of p.anterior.projetos) for (const a of pr.acoes) a.planos = normalizarPlanos(a);
  }
  for (const pr of p.projetos) for (const a of pr.acoes) a.planos = normalizarPlanos(a);
  for (const pr of p.projetos) for (const a of pr.acoes) for (const d of a.detalhes ?? []) {
    if (d.figura?.endsWith(".svg")) d.svg = resolver(readFileSync(join(pasta, d.figura), "utf8"), vars);
  }
  return p;
}

/** Os números da folha: quantos de cada peça o plano tem. */
export function contagem(p) {
  const acoes = acoesDe(p), soma = (f) => acoes.reduce((n, a) => n + (a[f]?.length ?? 0), 0);
  return { acoes: acoes.length, planos: soma("planos"), checkpoints: soma("checkpoints"), detalhes: soma("detalhes"), tabelas: soma("tabelas"), decisoes: p.decisoes?.length ?? 0, perguntas: p.perguntas?.length ?? 0 };
}
