i18nTraduzDom(); // os textos que já estão no HTML da página, no idioma ativo (i18n.js)
render();
if (!nomeUsuario) desenrolar(document.querySelector(".login-folha"), document.querySelector(".login-folha"));
carregar();
mcpConsulta();
mcpAssina();
agentesBusca();
atualizacaoBusca();
fetch("/api/mcp/info").then((r) => r.json()).then((j) => { mcpInfo = j; if (!hub.hidden) montaMcp(); }).catch(() => {});
