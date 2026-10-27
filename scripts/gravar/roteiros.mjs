// Os roteiros: cada um é uma sequência de passos que a "mão" faz no app de exemplo enquanto o gravador filma. Passos que dependem de um elemento
// que não existe são pulados com um aviso, para um roteiro não derrubar a série. Tudo no tema do app, sem emoji. Cada roteiro roda em inglês e em pt-BR:
// o Edge abre com --lang, o app escolhe o idioma sozinho e os dados de exemplo vêm no mesmo idioma (data.exemplo.en.json ou data.exemplo.json).
import { dormir, raiz } from "./nucleo.mjs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const ws = "sm";
const L = (A, pt, en) => (A.idioma.startsWith("pt") ? pt : en);
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
async function abrirFolha(A, c) { await A.ir("#/", 2200); await A.js(`location.hash = "#/p/${c.bp}"`); await dormir(3200); }
const esc = async (A) => { await A.cdp("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 }); await dormir(700); };

export const ROTEIROS = {
  "01-visao-geral": { gif: { largura: 640, fps: 8 }, // menor: a visão geral muda muito de tela e passaria de 3 MB
    descricao: "Da lista de workspaces até a folha do blueprint, por projetos e ações", async rodar(A) {
    const c = await contexto(A);
    await A.ir("#/", 2600);
    await A.gravar(async () => {
      await dormir(1400);
      await tentar("abrir workspace", () => clicarTexto(A, L(A, "Estúdio Aurora", "Aurora Studio"))); await dormir(1800);
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
      await tentar("menu da ação", async () => { await A.clicarEm(`.acao[data-f="${c.acao.id}"]`, "right"); await dormir(1800); await A.mover(A.mouse.x + 30, A.mouse.y + 60, 10); await dormir(1200); await esc(A); });
      await tentar("menu do plano", async () => { await A.clicarEm(".chk[data-p][data-f][data-id]", "right"); await dormir(1800); await esc(A); });
      await tentar("editar", async () => { await A.js(`editar(true)`); await dormir(2200); await rolar(A, 300, 12); await dormir(900); await A.js(`editar(false)`); await dormir(1200); });
    }); } },
  "03-proposta-do-agente": { gif: true, descricao: "O agente propõe, o usuário revisa e aplica no Planejar", async rodar(A) {
    const c = await contexto(A); await abrirFolha(A, c);
    await A.js(`try { localStorage.setItem("blueprint.autoAplicar.${ws}:${c.bp}", "0"); } catch {}`); // este vídeo mostra a revisão e o aplicar à mão: o aplicar sozinho fica desligado
    const d = (e, r, v) => L(A, `Escopo: ${e[0]}\nResultado esperado: ${r[0]}\nValidação: ${v[0]}`, `Scope: ${e[1]}\nExpected result: ${r[1]}\nValidation: ${v[1]}`);
    await A.gravar(async () => {
      await dormir(1500);
      await A.post("/api/mcp/proposta", { workspaceId: ws, blueprintId: c.bp, mensagem: L(A, "Proposta do agente: uma ação para a revisão final, com dois planos e um checkpoint.", "Agent proposal: one action for the final review, with two plans and one checkpoint."),
        acoes: [{ nome: L(A, "Revisão final", "Final review"), inicio: "2026-12-14", fim: "2026-12-15", status: "planejado", observacao: L(A, "Revisar o que foi entregue e fechar as pendências.", "Review what was delivered and close the open items."),
          planos: [{ texto: L(A, "Revisar os textos da entrega", "Review the delivery texts"), horas: 1.5, descricao: d(["ler os textos", "read the texts"], ["lista de ajustes", "a list of adjustments"], ["lista revisada", "the list reviewed"]) },
                   { texto: L(A, "Corrigir as pendências da revisão", "Fix the review findings"), horas: 2, descricao: d(["aplicar os ajustes", "apply the adjustments"], ["entrega limpa", "a clean delivery"], ["nenhuma pendência", "no open items"]) }],
          checkpoints: [{ nome: L(A, "Entrega revisada", "Delivery reviewed"), data: "2026-12-15" }] }] });
      await dormir(2200);
      // o aviso do agente (notificacao.js) aparece sobre a folha: o botão "Abrir o plano" dele leva ao Planejar; sem aviso, o botão da barra
      await tentar("abrir Planejar", async () => { try { await A.clicarEm(".notif-ir"); } catch { await A.clicarEm("#b-ia"); } }); await dormir(2400);
      await A.mover(640, 420, 14); await dormir(1400);
      await tentar("aplicar", () => A.clicarEm("#agente-tela [data-pact=aplicar]")); await dormir(2600);
    }); } },
  "04-claude-na-folha": { gif: true, descricao: "O Claude anda pelos planos enquanto trabalha e conclui com evidência", async rodar(A) {
    const c = await contexto(A); await abrirFolha(A, c);
    // uma ação com dois planos por fazer: o "concluir" tem de virar um check novo na tela, não cair num plano que o exemplo já traz feito
    const acao = c.acoes.find((a) => a.planos?.filter((p) => !p.feito).length >= 2) || c.acao;
    await tentar("abrir ação", () => A.js(`abertos[${JSON.stringify(c.bp)}] = true; render()`));
    await tentar("rolar até a ação", () => A.js(`document.querySelector('.acao[data-f="${acao.id}"]')?.scrollIntoView({ block: "start" })`)); await dormir(900);
    const [p1, p2] = acao.planos.filter((p) => !p.feito);
    const ev = (tipo, corpo) => A.evento(tipo, { workspaceId: ws, blueprintId: c.bp, acaoId: acao.id, ...corpo });
    await A.gravar(async () => {
      await dormir(1400);
      await ev("progresso", { agente: "Claude", estado: "lendo", texto: L(A, "Lendo o plano antes de começar", "Reading the plan before starting"), planoId: p1.id }); await dormir(2600);
      await ev("progresso", { agente: "Claude", estado: "escrevendo", texto: L(A, "Escrevendo a implementação", "Writing the implementation"), planoId: p1.id, onde: ["src/dados/mod.rs"] }); await dormir(2600);
      await ev("progresso", { agente: "Revisor", papel: "revisor", estado: "testando", texto: L(A, "Rodando os testes do plano", "Running the plan's tests"), planoId: p1.id }); await dormir(2600);
      if (p2) { await ev("progresso", { agente: "Claude", estado: "escrevendo", texto: L(A, "Passando ao próximo plano", "Moving on to the next plan"), planoId: p2.id }); await dormir(3200); }
      await ev("concluir", { agente: "Claude", texto: L(A, "Concluí com evidência", "Done, with evidence"), planoId: p1.id, evidencia: "cargo test: 12 passed" }); await dormir(3200);
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
    const pagina = L(A, "claude-previa.html", "claude-previa.en.html");
    await A.cdp("Page.navigate", { url: pathToFileURL(join(raiz, "referencias", pagina)).href }); await dormir(2500);
    await A.gravar(async () => { await A.mover(640, 300, 10); await dormir(6500); await rolar(A, 500, 16); await dormir(3500); await rolar(A, -500, 16); await dormir(1200); }); } },
  "07-roadmap": { gif: true, descricao: "A tela Roadmap: temas, estados, filtros e busca", async rodar(A) {
    await A.ir("#/", 2600);
    await A.gravar(async () => {
      await dormir(1200);
      await tentar("abrir o Roadmap", () => A.clicarEm("#b-roadmap")); await dormir(3200);
      await tentar("filtrar Em seguida", () => A.clicarEm('#roadmap-tela .rm-filtros [data-estado="proximo"]')); await dormir(1800);
      await tentar("filtrar Entregue", () => A.clicarEm('#roadmap-tela .rm-filtros [data-estado="entregue"]')); await dormir(1600);
      await tentar("limpar", () => A.clicarEm('#roadmap-tela .rm-filtros [data-estado=""]')); await dormir(900);
      await tentar("abrir um cartão", () => A.clicarEm('#roadmap-tela .rm-card[data-id="codex"]')); await dormir(1800);
      await tentar("buscar", async () => { await A.clicarEm("#roadmap-tela .rm-busca input"); await A.cdp("Input.insertText", { text: "claude" }); await dormir(2000); });
      await rolar(A, 400, 14); await dormir(1200);
    }); } },
};
