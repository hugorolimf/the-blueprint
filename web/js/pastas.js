// Carregado somente ao abrir o campo de pasta ou receber um aviso de sessão.
async function api(recurso, corpo) {
  const r = await fetch(`/api/mcp/${recurso}`, corpo ? { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) } : { cache: "no-store" });
  const d = await r.json();
  if (!r.ok) throw new Error(d.erro || tr("Erro {status}", { status: r.status }));
  return d;
}
export const vincularPasta = (cwd, workspaceId, blueprintId) => api("pasta/vincular", { cwd, workspaceId, blueprintId });

export async function editorPasta(caixa, workspaceId, blueprintId) {
  if (!caixa) return;
  try {
    const { pastas } = await api("pastas");
    if (!caixa.isConnected) return;
    caixa.replaceChildren();
    const titulo = document.createElement("h3"); titulo.textContent = tr("Pasta do projeto"); caixa.append(titulo);
    const lista = pastas.filter((p) => p.workspaceId === workspaceId && p.blueprintId === blueprintId);
    const status = document.createElement("p"); status.setAttribute("role", "status");
    for (const pasta of lista) {
      const linha = document.createElement("p"), input = document.createElement("input");
      input.value = pasta.cwd; input.maxLength = 500; input.setAttribute("aria-label", tr("Pasta ligada ao blueprint"));
      const trocar = document.createElement("button"); trocar.className = "btn mini"; trocar.textContent = tr("Trocar");
      const remover = document.createElement("button"); remover.className = "btn mini"; remover.textContent = tr("Remover");
      trocar.onclick = async () => {
        try {
          await vincularPasta(input.value.trim(), workspaceId, blueprintId);
          if (input.value.trim().replaceAll("\\", "/").replace(/\/$/, "").toLowerCase() !== pasta.cwd) await api("pasta/desvincular", { cwd: pasta.cwd });
          await editorPasta(caixa, workspaceId, blueprintId);
        } catch (e) { status.textContent = e.message; }
      };
      remover.onclick = async () => { try { await api("pasta/desvincular", { cwd: pasta.cwd }); await editorPasta(caixa, workspaceId, blueprintId); } catch (e) { status.textContent = e.message; } };
      linha.append(input, trocar, remover); caixa.append(linha);
    }
    const label = document.createElement("label"); label.className = "f"; label.textContent = lista.length ? tr("Ligar outra pasta") : tr("Pasta do projeto");
    const input = document.createElement("input"); input.placeholder = tr("C:\\caminho\\da\\pasta"); input.maxLength = 500; label.append(input);
    const ligar = document.createElement("button"); ligar.className = "btn mini"; ligar.textContent = tr("Ligar pasta");
    ligar.onclick = async () => { try { await vincularPasta(input.value.trim(), workspaceId, blueprintId); await editorPasta(caixa, workspaceId, blueprintId); } catch (e) { status.textContent = e.message; } };
    caixa.append(label, ligar, status);
  } catch (e) { caixa.textContent = tr("Não foi possível carregar as pastas: {erro}", { erro: e.message }); }
}

export async function avisarPasta(cwd) {
  const { workspaces } = await api("blueprints");
  let painel = document.getElementById("pastas-pendentes");
  if (!painel) {
    painel = document.createElement("aside"); painel.id = "pastas-pendentes"; painel.setAttribute("aria-label", tr("Pastas sem blueprint"));
    painel.style.cssText = "position:fixed;bottom:16px;right:16px;z-index:10000;max-width:440px;max-height:60vh;overflow:auto;background:var(--papel,#fff);color:var(--tinta,#222);padding:16px;border:1px solid #999;border-radius:8px;box-shadow:0 4px 20px #0003";
    document.body.append(painel);
  }
  const item = document.createElement("div"), texto = document.createElement("p"), seletor = document.createElement("select"), status = document.createElement("p");
  texto.textContent = tr("Pasta sem blueprint: {cwd}", { cwd }); status.setAttribute("role", "status"); seletor.setAttribute("aria-label", tr("Blueprint para {cwd}", { cwd }));
  const vazio = new Option(tr("Escolha o blueprint para ligar"), ""); seletor.append(vazio);
  for (const w of workspaces) for (const b of w.blueprints) seletor.append(new Option(`${w.nome} → ${b.nome}`, JSON.stringify([w.id, b.id])));
  seletor.onchange = async () => { if (!seletor.value) return; try { const [ws, bp] = JSON.parse(seletor.value); await vincularPasta(cwd, ws, bp); item.remove(); if (!painel.children.length) painel.remove(); } catch (e) { status.textContent = e.message; } };
  const fechar = document.createElement("button"); fechar.className = "btn mini"; fechar.textContent = tr("Agora não"); fechar.onclick = () => { item.remove(); if (!painel.children.length) painel.remove(); };
  item.append(texto, seletor, fechar, status); painel.append(item);
}
