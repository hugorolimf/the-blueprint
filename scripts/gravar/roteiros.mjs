// Os roteiros: cada um é uma sequência de passos que a "mão" faz no app de exemplo enquanto o gravador filma. Passos que dependem de um elemento
// que não existe são pulados com um aviso, para um roteiro não derrubar a série. Tudo no tema do app, sem emoji.
import { dormir, raiz } from "./nucleo.mjs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const ws = "sm";
async function contexto(A) {
  const d = await (await fetch(A.url + "/api/dados")).json();
  const bp = d.blueprints.find((b) => b.acoes?.some((a) => a.planos?.length >= 2)) || d.blueprints[0];
  return { bp: bp.id, acoes: bp.acoes, acao: bp.acoes.find((a) => a.planos?.length >= 2) || bp.acoes[0] };
}
// clica no menor elemento cujo texto é exatamente `texto`
async function clicarTexto(A, texto, botao = "left") {
  const r = await A.js(`(() => { const t = ${JSON.stringify(texto.toLowerCase())}; let melhor = null, area = 1e12;
    for (const e of document.querySelectorAll("a,button,[role=button],[data-goto],div,span,h1,h2,h3,text,tspan")) { if ((e.textContent || "").trim().toLowerCase() !== t) continue; const b = e.getBoundingClientRect(); if (b.width < 2 || b.height < 2) continue; if (b.width * b.height < area) { area = b.width * b.height; melhor = b; } }
    return melhor && { x: melhor.left + melhor.width / 2, y: melhor.top + melhor.height / 2 }; })()`);
  if (!r) { console.log("   (pulei: não achei o texto", texto + ")"); return false; }
  await A.clicar(r.x, r.y, botao); return true;
}
const tentar = async (rotulo, f) => { try { await f(); } catch (e) { console.log(`   (pulei "${rotulo}":`, String(e.message).slice(0, 120) + ")"); } };
async function rolar(A, dy, passos = 10) { for (let i = 0; i < passos; i++) { await A.cdp("Input.dispatchMouseEvent", { type: "mouseWheel", x: A.mouse.x, y: A.mouse.y, deltaX: 0, deltaY: dy / passos }); await dormir(40); } }
async function abrirFolha(A, c) { await A.ir(`#/p/${c.bp}`, 3200); }

export const ROTEIROS = {
  "01-visao-geral": { gif: true, descricao: "Da lista de workspaces até a folha do blueprint, por projetos e ações", async rodar(A) {
    const c = await contexto(A);
    await A.ir("#/", 2600);
    await A.gravar(async () => {
      await dormir(1400);
      await tentar("abrir workspace", () => clicarTexto(A, "Estúdio Aurora")); await dormir(1800);
      await tentar("abrir blueprint", async () => { await A.js(`location.hash = "#/p/${c.bp}"`); await dormir(2600); });
      await A.mover(700, 420, 18); await dormir(900);
      await tentar("rolar", async () => { await rolar(A, 360, 14); await dormir(900); await rolar(A, -360, 14); });
      await tentar("abrir ação", () => A.clicarEm(`.acao[data-f="${c.acao.id}"]`)); await dormir(1800);
      await A.mover(1000, 300, 16); await dormir(1200);
    }); } },
  "02-menu-e-edicao": { gif: true, descricao: "Menu do botão direito e a tela Editar", async rodar(A) {
    const c = await contexto(A); await abrirFolha(A, c);
    await A.gravar(async () => {
      await A.mover(600, 360, 12); await dormir(700);
      await tentar("menu da ação", async () => { await A.clicarEm(`.acao[data-f="${c.acao.id}"]`, "right"); await dormir(1800); await A.mover(A.mouse.x + 30, A.mouse.y + 60, 10); await dormir(1200); await A.cdp("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 }); await dormir(700); });
      await tentar("menu do plano", async () => { await A.clicarEm(".chk[data-p][data-f][data-id]", "right"); await dormir(1800); await A.cdp("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 }); await dormir(600); });
      await tentar("editar", async () => { await A.js(`editar(true)`); await dormir(2200); await rolar(A, 300, 12); await dormir(900); await A.js(`editar(false)`); await dormir(1200); });
    }); } },
  "03-proposta-do-agente": { gif: true, descricao: "O agente propõe, o usuário revisa e aplica no Planejar", async rodar(A) {
    const c = await contexto(A); await abrirFolha(A, c);
    await A.gravar(async () => {
      await dormir(1500);
      await A.post("/api/mcp/proposta", { workspaceId: ws, blueprintId: c.bp, mensagem: "Proposta do agente: uma ação para a revisão final, com dois planos e um checkpoint.", acoes: [{ nome: "Revisão final", inicio: "2026-12-14", fim: "2026-12-15", status: "planejado", observacao: "Revisar o que foi entregue e fechar as pendências.", planos: [{ texto: "Revisar os textos da entrega", horas: 1.5, descricao: "Escopo: ler os textos.\nResultado esperado: lista de ajustes.\nValidação: lista revisada." }, { texto: "Corrigir as pendências da revisão", horas: 2, descricao: "Escopo: aplicar os ajustes.\nResultado esperado: entrega limpa.\nValidação: nenhuma pendência." }], checkpoints: [{ nome: "Entrega revisada", data: "2026-12-15" }] }] });
      await dormir(2200);
      await tentar("abrir Planejar", () => A.clicarEm("#b-ia")); await dormir(2400);
      await A.mover(640, 420, 14); await dormir(1400);
      await tentar("aplicar", () => A.clicarEm("#agente-tela [data-pact=aplicar]")); await dormir(2600);
    }); } },
  "04-claude-na-folha": { gif: true, descricao: "O Claude anda pelos planos enquanto trabalha e conclui com evidência", async rodar(A) {
    const c = await contexto(A); await abrirFolha(A, c);
    await tentar("abrir ação", () => A.js(`abertos[${JSON.stringify(c.bp)}] = true; render()`));
    const [p1, p2] = c.acao.planos;
    const ev = (tipo, corpo) => A.evento(tipo, { workspaceId: ws, blueprintId: c.bp, acaoId: c.acao.id, ...corpo });
    await A.gravar(async () => {
      await dormir(1400);
      await ev("progresso", { agente: "Claude", estado: "lendo", texto: "Lendo o plano antes de começar", planoId: p1.id }); await dormir(2600);
      await ev("progresso", { agente: "Claude", estado: "escrevendo", texto: "Escrevendo a implementação", planoId: p1.id, onde: ["src/dados/mod.rs"] }); await dormir(2600);
      await ev("progresso", { agente: "Revisor", papel: "revisor", estado: "testando", texto: "Rodando os testes do plano", planoId: p1.id }); await dormir(2600);
      if (p2) { await ev("progresso", { agente: "Claude", estado: "escrevendo", texto: "Passando ao próximo plano", planoId: p2.id }); await dormir(3200); }
      await ev("concluir", { agente: "Claude", texto: "Concluí com evidência", planoId: p1.id, evidencia: "cargo test: 12 passed" }); await dormir(3200);
    }); } },
  "05-animacoes": { gif: true, descricao: "Expandir e recolher, o movimento da folha", async rodar(A) {
    const c = await contexto(A); await abrirFolha(A, c);
    await A.gravar(async () => {
      await dormir(1200);
      for (let i = 0; i < 2; i++) {
        await tentar("abrir ou recolher", () => A.clicarEm("rect.botao")); await dormir(1800); // o botão grande do cabeçalho da seção alterna entre Abrir e Recolher
        await tentar("recolher ou abrir", () => A.clicarEm("rect.botao")); await dormir(1600);
      }
      await tentar("abrir ação", () => A.clicarEm(`.acao[data-f="${c.acao.id}"]`)); await dormir(1800);
      await tentar("fechar ação", () => A.clicarEm(`.acao[data-f="${c.acao.id}"]`)); await dormir(1400);
    }); } },
  "06-estilos-e-anatomy": { gif: true, descricao: "A marca e o Claude desenhados com o anatomy, em todas as poses", async rodar(A) {
    await A.cdp("Page.navigate", { url: pathToFileURL(join(raiz, "referencias", "claude-previa.html")).href }); await dormir(2500);
    await A.gravar(async () => { await A.mover(640, 300, 10); await dormir(6500); await rolar(A, 500, 16); await dormir(3500); await rolar(A, -500, 16); await dormir(1200); }); } },
};
