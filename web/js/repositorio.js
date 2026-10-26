// Carregado só ao precisar: o card "Repositório GitHub" em Editar, o diálogo Configurar projeto e o aviso do canto da tela (contrato em docs/repositorio-github.md).
// Tudo o que roda algo (verificar o acesso, abrir o Claude) é do app local e só por clique; o repositório vem sempre dos dados do blueprint, nunca de um campo daqui.
async function post(recurso, corpo) {
  const r = await fetch(`/api/repositorio/${recurso}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
  return { status: r.status, j: await r.json().catch(() => ({})) };
}
async function pastasLigadas() {
  const r = await fetch("/api/mcp/pastas", { cache: "no-store" });
  return r.ok ? (await r.json()).pastas || [] : [];
}
const ws = () => workspaceAtual?.id || "sm";
// O servidor lê o repositório dos DADOS e a gravação do campo é adiada (rede.js: `timer`): a verificação espera ela chegar, senão responderia "sem-repositorio".
async function gravado(ms = 5000) {
  const fim = Date.now() + ms;
  while (typeof timer !== "undefined" && timer && Date.now() < fim) await new Promise((r) => setTimeout(r, 100));
}
export const verificarAcesso = async (blueprintId) => { await gravado(); return (await post("verificar", { workspaceId: ws(), blueprintId })).j; };

function guardaPasta(v) { try { if (v) localStorage.setItem("repositorio-pasta", v); return localStorage.getItem("repositorio-pasta") || ""; } catch { return v || ""; } }

// a resposta de POST /api/repositorio/configurar vira uma frase
function mensagemConfigurar(status, j) {
  if (status === 200 && j.ok) return tr("Terminal aberto: o Claude vai preparar o projeto em {destino}.", { destino: j.destino });
  if (j.codigo === "pasta-invalida") return tr("Escolha uma pasta que já existe (o caminho completo, sem “;”).");
  if (j.codigo === "sem-repositorio" || j.codigo === "url-invalida") return tr("Este blueprint não tem um repositório válido. Confira o endereço em Editar.");
  if (j.codigo === "sem-terminal" || status === 501) return tr("Não consegui abrir o terminal. Motivo: {erro}", { erro: j.erro || status });
  return j.erro || tr("Não foi possível configurar o projeto (erro {status}).", { status });
}

// o diálogo: mostra a URL e o destino, pede a pasta e o terminal; nada roda antes do clique em Configurar
function perguntar(repo) {
  return new Promise((resolve) => {
    const d = document.createElement("dialog");
    d.className = "confirmar escolher-sessao";
    d.setAttribute("role", "alertdialog");
    d.setAttribute("aria-label", tr("Configurar projeto"));
    d.innerHTML = `<div class="cu-topo"><h3>${tr("Configurar projeto")}</h3></div><p class="cu-corpo">${tr("O Claude vai clonar {url} numa subpasta chamada {nome} da pasta que você escolher, instalar o que o projeto pede e ligar a pasta a este blueprint. Gasta tokens.", { url: esc(repo.url), nome: esc(repo.nome) })}</p><label class="f">${tr("Pasta onde guardar")}<input data-pasta type="text" maxlength="500" placeholder="${tr("C:\\caminho\\da\\pasta")}" value="${esc(guardaPasta(""))}"></label><div class="cu-opcoes"><label><input type="radio" name="terminal" value="app" checked> <span>${tr("Terminal do app")}</span> <small>${tr("abre o Claude aqui dentro, numa aba do painel")}</small></label><label><input type="radio" name="terminal" value="externo"> <span>${tr("Terminal do Windows")}</span> <small>${tr("abre um terminal novo")}</small></label></div><p class="erro" data-erro hidden></p><div class="confirmar-acoes"><button class="btn" type="button" data-r="0">${tr("Cancelar")}</button><button class="btn perigo" type="button" data-r="1">${tr("Configurar")}</button></div>`;
    const campo = d.querySelector("[data-pasta]"), erro = d.querySelector("[data-erro]");
    d.addEventListener("click", (e) => { if (e.target === d) d.close("0"); });
    d.querySelector('[data-r="0"]').onclick = () => d.close("0");
    d.querySelector('[data-r="1"]').onclick = () => {
      if (!repositorioPastaOk(campo.value)) { erro.textContent = tr("Escolha uma pasta que já existe (o caminho completo, sem “;”)."); erro.hidden = false; campo.focus(); return; }
      d.close("1");
    };
    d.addEventListener("close", () => {
      const pasta = campo.value.trim(), terminal = d.querySelector('input[name="terminal"]:checked')?.value || "app";
      d.remove();
      resolve(d.returnValue === "1" ? { pasta, terminal } : null);
    });
    document.body.append(d);
    d.showModal();
    campo.focus();
  });
}

// "Configurar projeto": confirma e pede ao app para abrir o Claude (POST /api/repositorio/configurar); devolve true se abriu
export async function configurarProjeto(p, aviso) {
  const repo = repositorioValido(p.repositorio?.url);
  if (!repo) return false;
  const escolha = await perguntar(repo);
  if (!escolha) return false;
  guardaPasta(escolha.pasta);
  aviso(tr("Abrindo o terminal…"));
  try {
    const { status, j } = await post("configurar", { workspaceId: ws(), blueprintId: p.id, pasta: escolha.pasta, terminal: escolha.terminal, titulo: tr("Configurar · {nome}", { nome: repo.nome }) });
    aviso(mensagemConfigurar(status, j), status !== 200);
    if (status === 200 && j.sessao) terminalAbrirSessao(j.sessao);
    return status === 200;
  } catch { aviso(tr("Não consegui falar com o app para abrir o terminal. Ele está aberto?"), true); return false; }
}

// Janela da equipe (app instalado): o pedido vai ao app por IPC e ele pergunta numa janela do Windows antes de abrir o Claude (src/janela/projeto_local.rs)
export function configurarPeloApp(p, aviso) {
  window.ipc.postMessage(repositorioMensagemIpc("configurar", p));
  repositorioAgoraNao.gravar(p.id); // já pediu: o aviso do canto não volta a cada vez
  aviso(tr("Confirme na janela do Windows: o Claude vai preparar o projeto."));
}

// o card de Editar > Dados do blueprint
export async function editorRepositorio(caixa, p) {
  if (!caixa?.isConnected) return;
  const leitura = poderesDaSessao(sessao).soLeitura, local = !hospedado(), convidado = repositorioConvidadoNoApp();
  const repo = repositorioValido(p.repositorio?.url);
  const status = (txt, ruim) => { const el = caixa.querySelector("[data-status]"); if (el) { el.textContent = txt; el.classList.toggle("erro", !!ruim); } };
  caixa.innerHTML = `<h3>${tr("Repositório GitHub")}<small>${repo ? esc(repo.dono) : ""}</small></h3>
    ${leitura ? (repo ? `<p><a href="${esc(repo.url)}" target="_blank" rel="noopener noreferrer">${esc(repo.url)}</a></p>` : `<p class="vazio">${tr("Este blueprint não tem um repositório.")}</p>`)
      : `<label class="f">${tr("Endereço do repositório")}<input data-repo type="url" maxlength="200" placeholder="https://github.com/dono/repositório" value="${esc(p.repositorio?.url || "")}"></label><p class="erro" data-erro hidden>${tr("Endereço inválido: use https://github.com/dono/repositório")}</p><div class="acoes"><button class="btn mini" data-ra="conectar" type="button">${p.repositorio ? tr("Trocar") : tr("Conectar GitHub")}</button>${p.repositorio ? `<button class="btn mini" data-ra="desconectar" type="button">${tr("Desconectar")}</button>` : ""}</div>`}
    <p role="status" data-status></p>
    ${repo && local ? `<div class="acoes"><button class="btn mini" data-ra="verificar" type="button">${tr("Verificar o acesso")}</button><span data-configurar></span></div>` : ""}
    ${repo && convidado ? `<div class="acoes"><button class="btn mini cheio" data-ra="configurar-app" type="button">${tr("Configurar projeto")}</button></div>` : ""}
    ${repo && !local && !convidado ? `<p class="dica">${tr("Configurar o projeto numa pasta só funciona no The Blueprint instalado nesta máquina.")}</p>` : ""}
    <p class="dica">${tr("Quem tem acesso ao repositório no GitHub vê Configurar projeto aqui. O acesso é do GitHub: o app não guarda senha nem token.")}</p>`;
  const campo = caixa.querySelector("[data-repo]");
  const salvar = (valor) => { if (valor) p.repositorio = { url: valor }; else delete p.repositorio; mudou(); editorRepositorio(caixa, p); };
  caixa.querySelector('[data-ra="conectar"]')?.addEventListener("click", () => {
    const novo = repositorioValido(campo.value);
    const ruim = campo.value.trim() !== "" && !novo;
    caixa.querySelector("[data-erro]").hidden = !ruim;
    if (!ruim) salvar(novo?.url || "");
  });
  campo?.addEventListener("keydown", (e) => { if (e.key === "Enter") caixa.querySelector('[data-ra="conectar"]').click(); });
  caixa.querySelector('[data-ra="desconectar"]')?.addEventListener("click", () => salvar(""));
  const verificar = async () => {
    status(tr("Verificando…"));
    try {
      const j = await verificarAcesso(p.id);
      status(repositorioFrase(j.estado), j.estado !== "com-acesso");
      const slot = caixa.querySelector("[data-configurar]");
      if (!slot) return;
      slot.replaceChildren();
      const semPasta = repositorioSemPasta(p, await pastasLigadas(), ws());
      if (j.estado === "com-acesso" && semPasta) {
        const b = document.createElement("button");
        b.className = "btn mini cheio"; b.type = "button"; b.textContent = tr("Configurar projeto");
        b.onclick = () => configurarProjeto(p, (t, ruim) => status(t, ruim));
        slot.append(b);
      } else if (j.estado === "sem-acesso" && repo) {
        const a = document.createElement("a");
        a.href = `${repo.url}/settings/access`; a.target = "_blank"; a.rel = "noopener noreferrer"; a.textContent = tr("Colaboradores no GitHub (só o dono do repositório abre)");
        slot.append(a);
      }
    } catch { status(tr("Não consegui falar com o app para verificar o acesso."), true); }
  };
  caixa.querySelector('[data-ra="verificar"]')?.addEventListener("click", verificar);
  caixa.querySelector('[data-ra="configurar-app"]')?.addEventListener("click", () => configurarPeloApp(p, (t, ruim) => status(t, ruim)));
  if (repo && local) verificar();
}

// o aviso do canto da tela, ao abrir um blueprint com repositório e sem pasta nesta máquina (só aparece se a pessoa tem acesso)
export async function avisarRepositorio(blueprintId, convidado = false) {
  document.getElementById("repo-aviso")?.remove();
  const p = D?.blueprints?.find((x) => x.id === blueprintId);
  if (!p || repositorioAgoraNao.ler(p.id)) return;
  let j = { nome: repositorioValido(p.repositorio?.url)?.nome || "" };
  if (!convidado) { // app local: só avisa quem tem acesso e ainda não tem a pasta. Na janela da equipe o app não responde (o IPC é de ida só): o aviso vem uma vez, e pedir ou "Agora não" o silenciam
    let pastas;
    try { pastas = await pastasLigadas(); } catch { return; }
    if (!repositorioSemPasta(p, pastas, ws())) return;
    try { j = await verificarAcesso(p.id); } catch { return; }
    if (j.estado !== "com-acesso") return; // sem acesso a pessoa não precisa de aviso
  }
  if (tela !== p.id) return; // saiu do blueprint enquanto esperava
  const caixa = document.createElement("aside");
  caixa.id = "repo-aviso";
  caixa.setAttribute("aria-label", tr("Configurar projeto"));
  caixa.style.cssText = "position:fixed;bottom:16px;right:16px;z-index:10000;max-width:420px;background:var(--papel,#fff);color:var(--tinta,#222);padding:16px;border:1px solid #999;border-radius:8px;box-shadow:0 4px 20px #0003";
  caixa.innerHTML = `<p>${tr("Este blueprint tem o repositório {nome} e você ainda não tem a pasta dele nesta máquina.", { nome: esc(j.nome || "") })}</p><p role="status" data-status></p><div class="acoes"><button class="btn mini cheio" data-r="1" type="button">${tr("Configurar projeto")}</button><button class="btn mini" data-r="0" type="button">${tr("Agora não")}</button></div>`;
  const status = (t, ruim) => { const el = caixa.querySelector("[data-status]"); el.textContent = t; el.classList.toggle("erro", !!ruim); };
  caixa.querySelector('[data-r="0"]').onclick = () => { repositorioAgoraNao.gravar(p.id); caixa.remove(); };
  caixa.querySelector('[data-r="1"]').onclick = async () => {
    if (convidado) { configurarPeloApp(p, status); setTimeout(() => caixa.remove(), 2500); return; }
    if (await configurarProjeto(p, status)) caixa.remove();
  };
  document.body.append(caixa);
}
