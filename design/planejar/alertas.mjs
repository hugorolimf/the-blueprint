// O exemplo de verdade: o plano de Alertas do SM PRO, gerado pelo próprio design/alertas/nota.mjs do repositório SM PRO, aberto como
// página do Blueprint. É o mesmo HTML que vai para o cofre; aqui ele só ganha a janela e o tema do app. É assim que o `--blueprint`
// dos geradores funcionaria: o gerador não muda, o Blueprint recebe as peças e desenha no tema dele.
// O repositório do SM PRO vem de SMPRO_DIR (padrão ~/Work/Sua Majestade/SM-PRO); sem ele, a nota diz o que falta.
import { existsSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";
import { pathToFileURL } from "node:url";
import { janela, marcarBlocos } from "./janela.mjs";

const SMPRO = process.env.SMPRO_DIR || join(homedir(), "Work/Sua Majestade/SM-PRO");

export async function alertasNoBlueprint() {
  const arquivo = join(SMPRO, "design/alertas/nota.mjs");
  if (!existsSync(arquivo)) return { html: `<p class="bp-leg">Sem o repositório do SM PRO em ${SMPRO}: o exemplo do plano de Alertas não foi gerado.</p>`, conta: {}, secoes: [] };
  const { painel } = await import(pathToFileURL(arquivo).href);
  const [pagina, secoes, conta] = marcarBlocos(painel());
  const figuras = (pagina.match(/<svg class="al-svg"/g) ?? []).length;
  const html = janela({
    rotulo: "Página do blueprint · Alertas e avisos",
    titulo: "Alertas e avisos: etapa 1",
    estado: `${figuras} figuras · ${secoes.length} seções`,
    botoes: [["Planejar"], ["Memória"], ["Grafo"]],
    workspace: "SM PRO",
    arvore: [
      ["Alertas e avisos", { sinal: "▾" }],
      ["Página do blueprint", { nivel: 1, ativo: true }],
      ["Planejar", { nivel: 1 }],
      ["Memória", { nivel: 1, num: "9" }],
      ["Decisões", { nivel: 1, num: "16" }],
      ["1.1 Servidor", { nivel: 1, sinal: "▸" }],
      ["1.2 App", { nivel: 1, sinal: "▸" }],
      ["1.3 Permanência", { nivel: 1, sinal: "▸" }],
      ["1.4 Saiu da rota", { nivel: 1, sinal: "▸" }],
      ["WhatsApp no servidor", { sinal: "▸" }],
      ["Relatórios", { sinal: "▸" }],
      ["Rotas para caminhão", { sinal: "▸" }],
    ],
    secoes,
    meta: `Página do blueprint · escrita pelo Claude · gerada por design/alertas/build.mjs · 31/10/2026 21:40 · ${figuras} figuras`,
    pagina,
  });
  return { html, conta, secoes, figuras };
}
