// Migração única (2026-10-21) do data.json para o vocabulário novo: companhias→workspaces, projetos→blueprints,
// fases→acoes, tarefas→planos, marcos→checkpoints. Só troca nomes de chaves; valores e o resto dos campos ficam iguais.
//   node scripts/migrar-vocabulario.mjs [data.json]   -> grava um backup ao lado e depois o arquivo migrado
import { readFile, rename, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

// troca as chaves mantendo a ordem dos campos
const renomeia = (obj, mapa) => Object.fromEntries(Object.entries(obj).map(([k, v]) => [mapa[k] || k, v]));

const acao = (f) => renomeia(f, { tarefas: "planos", marcos: "checkpoints" });
const blueprint = (p) => { const b = renomeia(p, { fases: "acoes" }); if (b.acoes) b.acoes = b.acoes.map(acao); return b; };
const workspace = (w) => { const n = renomeia(w, { projetos: "blueprints" }); if (n.blueprints) n.blueprints = n.blueprints.map(blueprint); return n; };

// as conversas guardadas (resto da tela Planejar antiga) têm propostas pela metade com os nomes antigos, em qualquer profundidade
const CHAT = { fases: "acoes", tarefas: "planos", marcos: "checkpoints", projetos: "blueprints", dadosProjeto: "dadosBlueprint", faseAlvo: "acaoAlvo" };
const renomeiaFundo = (v) => Array.isArray(v) ? v.map(renomeiaFundo) : v && typeof v === "object" ? Object.fromEntries(Object.entries(v).map(([k, x]) => [CHAT[k] || k, renomeiaFundo(x)])) : v;

export function migrar(dados) {
  if (!dados || !Array.isArray(dados.projetos)) throw new Error("formato inesperado: falta projetos[] (já migrado?)");
  const raiz = workspace(renomeia(dados, { companhias: "workspaces" }));
  if (raiz.workspaces) raiz.workspaces = raiz.workspaces.map(workspace);
  if (Array.isArray(raiz.chats)) raiz.chats = renomeiaFundo(raiz.chats);
  return raiz;
}

// para conferir que nada se perdeu: mesma contagem antes e depois
export function contar(d, nomes) {
  const n = { workspaces: 0, blueprints: 0, acoes: 0, planos: 0, checkpoints: 0, feitos: 0 };
  for (const w of [d, ...(d[nomes.workspaces] || [])]) {
    n.workspaces++;
    for (const p of w[nomes.blueprints] || []) {
      n.blueprints++;
      for (const f of p[nomes.acoes] || []) {
        n.acoes++;
        for (const t of f[nomes.planos] || []) { n.planos++; if (t.feito) n.feitos++; }
        for (const m of f[nomes.checkpoints] || []) { n.checkpoints++; if (m.feito) n.feitos++; }
      }
    }
  }
  return n;
}
export const ANTIGOS = { workspaces: "companhias", blueprints: "projetos", acoes: "fases", planos: "tarefas", checkpoints: "marcos" };
export const NOVOS = { workspaces: "workspaces", blueprints: "blueprints", acoes: "acoes", planos: "planos", checkpoints: "checkpoints" };

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const arq = process.argv[2] || fileURLToPath(new URL("../data.json", import.meta.url));
  const texto = await readFile(arq, "utf8"), antes = JSON.parse(texto), depois = migrar(antes);
  const a = contar(antes, ANTIGOS), b = contar(depois, NOVOS);
  if (JSON.stringify(a) !== JSON.stringify(b)) throw new Error(`contagens diferentes: ${JSON.stringify(a)} ≠ ${JSON.stringify(b)}`);
  const backup = arq.replace(/\.json$/, "") + `.backup-${new Date().toISOString().replace(/[:.]/g, "-")}-pre-vocabulario.json`;
  await writeFile(backup, texto);
  await writeFile(`${arq}.tmp`, JSON.stringify(depois, null, 2));
  await rename(`${arq}.tmp`, arq);
  console.log(`migrado: ${arq} · backup: ${backup} · ${JSON.stringify(b)}`);
}
