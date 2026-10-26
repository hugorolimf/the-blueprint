// ---- Repositório GitHub do blueprint: a lógica pura (contrato em docs/repositorio-github.md; o DOM fica em repositorio.js, carregado ao precisar) ----
// O dado é `blueprint.repositorio = { url }`. A regra da URL é a MESMA de src/repositorio/mod.rs (os dois leem referencias/repositorio-urls.json nos testes).
function repositorioValido(texto) {
  const t = String(texto ?? "").trim(), prefixo = "https://github.com/";
  if (!t.startsWith(prefixo)) return null;
  let r = t.slice(prefixo.length);
  if (r.endsWith("/")) r = r.slice(0, -1);
  if (r.endsWith(".git")) r = r.slice(0, -4);
  const i = r.indexOf("/");
  if (i < 0) return null;
  const dono = r.slice(0, i), nome = r.slice(i + 1);
  if (!/^[A-Za-z0-9-]{1,39}$/.test(dono) || dono.startsWith("-")) return null;
  if (!/^[A-Za-z0-9._-]{1,100}$/.test(nome) || nome === "." || nome === ".." || nome.startsWith("-")) return null;
  return { url: `${prefixo}${dono}/${nome}`, dono, nome };
}

// a frase de cada estado devolvido por POST /api/repositorio/verificar
function repositorioFrase(estado) {
  return ({
    "url-invalida": () => tr("O endereço guardado neste blueprint não é válido. Use https://github.com/dono/repositório."),
    "git-ausente": () => tr("O git não está instalado nesta máquina. Instale em git-scm.com e verifique de novo."),
    "com-acesso": () => tr("Você tem acesso a este repositório."),
    "sem-acesso": () => tr("Você não tem acesso a este repositório (ou ele não existe, ou o GitHub não respondeu). Peça ao dono para adicionar você como colaborador e verifique de novo."),
    "sem-resposta": () => tr("O GitHub não respondeu a tempo. Confira a conexão e verifique de novo."),
  }[estado] || (() => ""))();
}

// o blueprint tem repositório válido e esta máquina não tem pasta ligada a ele (pastas = a lista de GET /api/mcp/pastas)
function repositorioSemPasta(p, pastas, workspaceId) {
  return !!(p && repositorioValido(p.repositorio?.url) && !(pastas || []).some((x) => x.workspaceId === workspaceId && x.blueprintId === p.id));
}

// a pasta de destino: caminho completo do Windows, sem controle nem `;` (o servidor confere de novo e exige que ela exista)
function repositorioPastaOk(texto) {
  const t = String(texto ?? "").trim();
  return t.length > 0 && t.length <= 500 && /^(?:[A-Za-z]:[\\/]|\\\\)/.test(t) && !/[\u0000-\u001f\u007f;]/.test(t);
}

// o pedido que a janela da equipe manda ao app instalado (window.ipc, src/janela/projeto_local.rs): só ids, a URL do repositório e nomes; o app valida tudo de novo
function repositorioMensagemIpc(tipo, p, extra = {}) {
  return "projeto " + JSON.stringify({ tipo, workspaceId: workspaceAtual?.id || "sm", blueprintId: p.id, nome: p.nome || "", repositorio: p.repositorio?.url || "", ...extra });
}

// a janela da equipe do app instalado (modo hospedado + IPC) e uma pessoa que não é só visitante: ali o app local empresta o que roda nesta máquina
const repositorioConvidadoNoApp = () => hospedado() && !!window.ipc && !sessao.visitante;

// "Agora não" do aviso do canto da tela: lembrado por blueprint, só neste navegador
const repositorioAgoraNao = {
  ler: (id) => { try { return localStorage.getItem("repositorio-agora-nao:" + id) === "1"; } catch { return false; } },
  gravar: (id) => { try { localStorage.setItem("repositorio-agora-nao:" + id, "1"); } catch {} },
};

// chamado por render(): o aviso "Configurar projeto" só é considerado quando o blueprint aberto muda
let repositorioUltimo = "";
function repositorioAoAbrir() {
  const chave = (typeof tela === "string" ? tela : "") + "|" + (typeof workspaceAtual !== "undefined" && workspaceAtual ? workspaceAtual.id : "");
  if (chave === repositorioUltimo) return;
  repositorioUltimo = chave;
  const p = typeof tela === "string" && D?.blueprints?.find((x) => x.id === tela);
  const convidado = repositorioConvidadoNoApp();
  if (!p || (hospedado() && !convidado) || !repositorioValido(p.repositorio?.url)) { document.getElementById("repo-aviso")?.remove(); return; } // o servidor da equipe não tem as rotas locais: no navegador comum não há aviso
  import("/web/js/repositorio.js").then((m) => m.avisarRepositorio(p.id, convidado)).catch(() => {});
}
