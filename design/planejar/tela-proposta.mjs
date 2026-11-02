// Tela 4: o que chegou, na própria folha. Só mostra: nenhuma decisão se toma aqui. O plano novo aparece resumido, para você colocar na folha;
// o plano alterado aponta para o git diff, dentro dele.
import { txt, ret, fio, botao, rotulo, etiqueta, canto, linhas, quadro } from "./svg.mjs";
import { W, X0, X1, LINHA, escala, moldura, regua, cabecalhoProjeto, linhaAcao, separador } from "./folha.mjs";
import { acoesDe } from "./pacote.mjs";
import { copia } from "./tela-folha.mjs";

const XP = 756, XN = X0 + 100;
const resumoNovo = (a) => [a.planos?.length && `${a.planos.length} ${a.planos.length > 1 ? "planos" : "plano"}`, a.detalhes?.length && `${a.detalhes.length} ${a.detalhes.length > 1 ? "detalhes" : "detalhe"}`, a.checkpoints?.length && `${a.checkpoints.length} checkpoint`].filter(Boolean).join(" · ");

function painel(p, c, y0, H) {
  const w = X1 - XP, x = XP + 16, wi = w - 32, out = [], comp = c.comp, novas = acoesDe(p).filter((a) => comp.novas.includes(a.id)), alt = acoesDe(p).filter((a) => comp.alteradas[a.id]);
  let y = y0 + 26;
  out.push(txt(x, y, `${c.agente}`, { tam: 10, caixa: true, c: "medio", trilha: 0.12, max: wi }));
  y += 22;
  const [t, th] = linhas(x, y, "O plano mudou", { tam: 14, peso: 700, max: wi, passo: 19 });
  out.push(t);
  y += th + 12;
  [[novas.length, novas.length === 1 ? "novo" : "novos"], [alt.length, alt.length === 1 ? "alterado" : "alterados"], [comp.iguais.length, comp.iguais.length === 1 ? "igual" : "iguais"]].forEach(([n, r], i) => out.push(txt(x + (i * wi) / 3, y + 12, n, { tam: 22, peso: 700 }), txt(x + (i * wi) / 3, y + 28, r, { tam: 10, c: "medio" })));
  y += 52;
  if (alt.length) {
    out.push(rotulo(x, y, "Planos alterados", wi));
    y += 22;
    for (const a of alt) {
      const r = comp.alteradas[a.id].resumo, d = [r.mais && `+${r.mais}`, r.menos && `−${r.menos}`, r.alterados && `~${r.alterados}`, r.detalhes && `+${r.detalhes} det.`].filter(Boolean).join("  ");
      out.push(txt(x, y, `${a.id} ${a.nome}`, { tam: 11.5, peso: 700, max: wi - 96 }), txt(x, y + 16, d, { tam: 10, c: "medio", max: wi - 96 }));
      out.push(botao(x + wi - botao(0, 0, "Ver diff", { mini: true })[1], y - 12, "Ver diff", { mini: true })[0]);
      y += 34;
    }
    y += 8;
  }
  if (novas.length) {
    out.push(rotulo(x, y, "Planos novos", wi));
    y += 22;
    for (const a of novas) {
      out.push(txt(x, y, `${a.id} ${a.nome}`, { tam: 11.5, peso: 700, max: wi - 96 }));
      out.push(txt(x, y + 16, resumoNovo(a) || "ainda sem planos", { tam: 10, c: "medio", max: wi - 96 }));
      out.push(botao(x + wi - botao(0, 0, "Colocar", { mini: true })[1], y - 12, "Colocar", { mini: true })[0]);
      y += 34;
    }
    y += 4;
  }
  return [ret(XP, y0, w, H - 110 - y0, { fill: "#082b5e", stroke: "tinta" }) + canto(XP, y0) + out.join(""), y + 30];
}

export function telaProposta(p, c) {
  const e = escala(p, c.hoje, 340, XP - 20), n = p.projetos.reduce((s, pr) => s + 56 + pr.acoes.length * LINHA, 0);
  const H = Math.max(170 + n + 120, painel(p, c, 112, 4000)[1] + 110);
  const out = [moldura(H, copia(p, c)), regua(e, 138, H - 110, { colunas: false }), painel(p, c, 112, H)[0]];
  let y = 182;
  p.projetos.forEach((pr, i) => {
    out.push(cabecalhoProjeto(i + 1, pr, p.blueprint, p, c, e, y, { colunas: false, xFim: XP - 20 }));
    y += 32;
    for (const a of pr.acoes) {
      const m = c.comp.novas.includes(a.id) ? "nova" : c.comp.alteradas[a.id] ? "alterada" : null;
      if (m) out.push(etiqueta(X0 + 2, y + 4, m === "nova" ? "Novo plano" : "Alterado", { cheia: m === "nova" })[0]);
      out.push(linhaAcao(e, a, p, c, y, { colunas: false, xNome: XN, marca: m }));
      y += LINHA;
      out.push(separador(y - LINHA / 2, XP - 20));
    }
    y += 24;
  });
  return quadro(`p${c.id}`, W, H, out.join(""), "O que chegou no plano, na própria folha");
}
