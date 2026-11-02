// Revisar faz parte da folha: nada de formulário. Cada peça que pede revisão tem, na própria linha, um controle discreto de três partes e,
// depois de decidida, um carimbo como os de uma prancha. A observação só aparece quando você escolhe revisar. O agente só pede; o estado
// fica no Blueprint, nunca no plano.json, para ele não conseguir aprovar o próprio plano.
import { txt, ret, fio, largura, linhas, COR } from "./svg.mjs";
import { TEMA } from "./estilo.mjs";

export const SITUACAO = { pendente: ["Aguardando você", TEMA.ambar], aprovada: ["Aprovado", TEMA.verde], revisar: ["Em revisão", TEMA.ambar], reprovada: ["Reprovado", TEMA.vermelho] };

export const chaveDecisao = (d) => `d:${d.id}`;
export const chaveDetalhe = (a, d) => `det:${a.id}-${d.id}`;
export const estadoDe = (c, chave) => c.revisoes?.[chave] ?? { estado: "pendente" };

/** A situação escrita, com o pontinho na cor: nunca só a cor. */
export function situacao(x, y, estado, { fim = false, curta = false } = {}) {
  const [rot, cor] = SITUACAO[estado], t = curta && estado === "pendente" ? "Revisar" : rot;
  const larg = largura(t, 10), x0 = fim ? x - larg - 10 : x;
  return `<circle cx="${x0 + 3}" cy="${y - 3.5}" r="3" fill="${cor}"/>` + txt(x0 + 10, y, t, { tam: 10, c: cor });
}

/** O carimbo da decisão, de contorno duplo, como numa prancha aprovada. `x` é a borda direita. */
export function carimbo(x, y, estado, data) {
  const [rot, cor] = SITUACAO[estado], t = data ? `${rot} · ${data}` : rot, w = largura(t.toUpperCase(), 9, 0.1) + 14, x0 = x - w;
  return `<g transform="rotate(-1.2 ${x0 + w / 2} ${y})">` + ret(x0, y - 10, w, 18, { stroke: cor, sw: 1.2 }) + ret(x0 + 2, y - 8, w - 4, 14, { stroke: cor, sw: 0.5 }) + txt(x0 + 7, y + 2.5, t, { tam: 9, caixa: true, peso: 700, c: cor }) + `</g>`;
}

const ICONE = {
  aprovar: (x, y) => `M${x} ${y + 1}l3 3l5.5 -6.5`,
  revisar: (x, y) => `M${x} ${y + 5}l1 -3l5.5 -5.5l2 2l-5.5 5.5zM${x + 5} ${y - 5}l2 2`,
  reprovar: (x, y) => `M${x + 1} ${y - 3}l6.5 6.5m0 -6.5l-6.5 6.5`,
};

/** O controle de três partes, preso à borda direita em `xFim`. `meio`: «Revisar» numa peça comum, «Editar» num plano alterado. */
export function controleRevisao(xFim, y, { meio = "Revisar", alto = 22, ativo } = {}) {
  const partes = [["Aprovar", "aprovar"], [meio, "revisar"], ["Reprovar", "reprovar"]];
  const larg = partes.map(([t]) => largura(t, 10) + 30), total = larg.reduce((s, l) => s + l, 0);
  let x = xFim - total;
  const out = [ret(x, y, total, alto, { fill: "#0a2347", stroke: "medio" })];
  partes.forEach(([t, ic], i) => {
    if (i) out.push(fio(x, y, x, y + alto, { c: "medio", op: 0.7 }));
    const c = ativo === ic;
    if (c) out.push(ret(x, y, larg[i], alto, { fill: "tinta" }));
    out.push(`<path d="${ICONE[ic](x + 9, y + alto / 2)}" fill="none" stroke="${c ? COR.papel : COR.tinta}" stroke-width="1.2"/>`, txt(x + 24, y + alto / 2 + 3.5, t, { tam: 10, c: c ? "papel" : "tinta" }));
    x += larg[i];
  });
  return [out.join(""), total];
}

/** A observação: só existe na peça em revisão. Com `compondo`, é o campo onde você está escrevendo; depois, o seu recado preso à peça. */
export function notaRevisao(x, y, w, r) {
  const texto = r.observacao ?? "Diga o que mudar…", vazio = !r.observacao;
  const [t, h] = linhas(x + 12, y + 14, texto, { tam: 10.5, max: w - 24, passo: 15, maxLinhas: 3, c: vazio ? "medio" : "tinta" });
  const alt = h + 10;
  return [ret(x, y, w, alt, { fill: "#081d3c", stroke: TEMA.ambar, sw: 0.8, dash: r.compondo ? "3 2" : undefined }) + fio(x, y, x, y + alt, { c: TEMA.ambar, sw: 2.5 }) + t, alt];
}

/** A linha fina à esquerda da peça: a cor diz a situação, sem ocupar espaço. */
export const margemRevisao = (x, y, h, estado) => fio(x, y, x, y + h, { c: SITUACAO[estado][1], sw: 2 });

/** O canto direito de uma peça: o controle enquanto espera você, o carimbo depois. Devolve o svg. */
export function canto(xFim, y, r, op = {}) {
  if (r.estado === "pendente") return controleRevisao(xFim, y - 15, op)[0];
  return carimbo(xFim, y - 3, r.estado, r.quando);
}

/** Quantas peças da ação ainda esperam você: decisões, detalhes e a mudança da própria ação. */
export function pendentesDaAcao(a, p, c) {
  const dec = (p.decisoes ?? []).filter((d) => d.pede_aprovacao && d.acoes?.includes(a.id)).map(chaveDecisao);
  const det = (a.detalhes ?? []).filter((d) => d.pede_aprovacao).map((d) => chaveDetalhe(a, d));
  return [...dec, ...det].filter((k) => estadoDe(c, k).estado === "pendente").length + (c.comp?.alteradas?.[a.id] ? 1 : 0);
}

/** Quantas peças do plano inteiro esperam você; uma decisão ligada a duas ações conta uma vez só. */
export function pendentesDoPlano(p, c) {
  const dec = (p.decisoes ?? []).filter((d) => d.pede_aprovacao).map(chaveDecisao);
  const det = p.projetos.flatMap((pr) => pr.acoes).flatMap((a) => (a.detalhes ?? []).filter((d) => d.pede_aprovacao).map((d) => chaveDetalhe(a, d)));
  return [...dec, ...det].filter((k) => estadoDe(c, k).estado === "pendente").length + Object.keys(c.comp?.alteradas ?? {}).length;
}
