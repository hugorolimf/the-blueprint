// Como um plano entra: qualquer agente compõe a pasta (com o kit, ou com qualquer ferramenta que gere SVG ou imagem), o Blueprint
// valida e transforma em proposta na própria folha; você aprova e aplica. Salvar de novo mostra só o que mudou.
import { txt, ret, rotulo, canto, quadro, COR } from "./svg.mjs";

const W = 1100, H = 270;

function caixa(x, y, w, h, rot, ls, cheia = false) {
  const out = [ret(x, y, w, h, { fill: cheia ? "#14509b" : "#082b5e", stroke: cheia ? "tinta" : "baixo" }), canto(x, y), rotulo(x + 12, y + 26, rot)];
  ls.forEach((l, i) => out.push(txt(x + 12, y + 50 + i * 18, l, { tam: 11, c: i ? "medio" : "tinta" })));
  return out.join("");
}
const seta = (d, t, x, y, op = {}) => `<path d="${d}" fill="none" stroke="${COR.tinta}" stroke-width="1.2"${op.dash ? ` stroke-dasharray="${op.dash}"` : ""} marker-end="url(#fp-seta)"/>` + txt(x, y, t, { tam: 9.5, c: "medio", meio: true });

export function fluxo() {
  const out = [`<defs><marker id="fp-seta" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10Z" fill="${COR.tinta}"/></marker></defs>`];
  const Y = 30, A = 124, B = 188, G = 32, x = (i) => 16 + i * (B + G);
  out.push(caixa(x(0), Y, B, A, "Orquestrador", ["Claude Code, Codex…", "escreve o plano", "kit, ou qualquer SVG", "ou imagem"]));
  out.push(caixa(x(1), Y, B, A, "Plano no formato", ["ações e planos", "figuras SVG ou imagem", "decisões, perguntas"]));
  out.push(caixa(x(2), Y, B, A, "Blueprint valida", ["peça por peça", "erros na hora", "compara pelos ids"]));
  out.push(caixa(x(3), Y, B, A, "Na folha, ao vivo", ["ação «Aguardando plano»", "plano sendo escrito", "mudança como git diff"], true));
  out.push(caixa(x(4), Y, B - 4, A, "Você", ["revisa no plano", "marca a caixinha:", "o agente começa"]));
  for (let i = 0; i < 4; i++) out.push(seta(`M${x(i) + B} ${Y + A / 2}H${x(i + 1) - 1}`, "", 0, 0));
  out.push(seta(`M${x(3) + B / 2} ${Y + A + 2}C${x(3) + B / 2 - 20} ${H - 40} ${x(0) + 120} ${H - 40} ${x(0) + B / 2} ${Y + A + 2}`, "revisões, recados e respostas voltam ao orquestrador; ao concluir um plano, o agente marca a caixinha", 450, H - 12, { dash: "4 3" }));
  return quadro("fp", W, H, out.join(""), "Como um plano entra no Blueprint: agente, pasta do plano, validação, proposta na folha e aplicar");
}
