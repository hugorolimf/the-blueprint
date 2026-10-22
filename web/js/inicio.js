render();
if (!nomeUsuario) desenrolar(document.querySelector(".login-folha"), document.querySelector(".login-folha"));
carregar();
mcpConsulta();
mcpAssina();
agentesBusca();
fetch("/api/mcp/info").then((r) => r.json()).then((j) => { mcpInfo = j; if (!hub.hidden) montaMcp(); }).catch(() => {});
