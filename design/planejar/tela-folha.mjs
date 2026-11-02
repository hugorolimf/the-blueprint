// Tela 1: a folha com todas as ações fechadas. Abrir o plano é abrir o blueprint.
import { quadro } from "./svg.mjs";
import { W, X0, LINHA, escala, moldura, regua, cabecalhoProjeto, linhaAcao, separador } from "./folha.mjs";
import { pendentesDoPlano } from "./revisao.mjs";

export const copia = (p, c) => ({ workspace: c.workspace, blueprint: p.blueprint, folha: c.folha });

export function telaFolha(p, c) {
  const e = escala(p, c.hoje), n = p.projetos.reduce((s, pr) => s + 56 + pr.acoes.length * LINHA, 0), H = 170 + n + 120;
  const espera = pendentesDoPlano(p, c);
  const out = [moldura(H, { ...copia(p, c), botoes: [...(espera ? [`Aguardando você · ${espera}`] : []), "Planejar", "Compartilhar", "Apresentar"] }), regua(e, 138, H - 110)];
  let y = 182;
  p.projetos.forEach((pr, i) => {
    out.push(cabecalhoProjeto(i + 1, pr, p.blueprint, p, c, e, y));
    y += 32;
    for (const a of pr.acoes) {
      out.push(linhaAcao(e, a, p, c, y));
      y += LINHA;
      out.push(separador(y - LINHA / 2));
    }
    y += 24;
  });
  return quadro(`f${c.id}`, W, H, out.join(""), `A folha do blueprint ${p.blueprint}`);
}
