i18nTraduzDom(); // os textos que já estão no HTML da página, no idioma ativo (i18n.js)
{ const s = $("idioma"), g = lerIdiomaGuardado() || "auto"; if (s) { s.value = ["auto", "en", "pt-BR"].includes(g) ? g : "auto"; s.addEventListener("change", () => definirIdioma(s.value)); } } // o seletor de idioma da entrada
render();
if (!nomeUsuario) desenrolar(document.querySelector(".login-folha"), document.querySelector(".login-folha"));
sessaoGuarda(() => { // sessao.js: em /convite, ou no modo hospedado sem sessão, mostra a entrada e não carrega; no modo local segue como sempre
carregar();
if (!sessao.visitante) { // visitante não tem o MCP (403 em /api/mcp/*): nada de consulta repetida que só falharia
mcpConsulta();
mcpAssina();
agentesBusca();
}
atualizacaoBusca();
equipesBusca(); // equipes-locais.js: as equipes a que este app já entrou, na tela de workspaces
fetch("/api/mcp/info").then((r) => r.json()).then((j) => { mcpInfo = j; if (!hub.hidden) montaMcp(); }).catch(() => {});
});
