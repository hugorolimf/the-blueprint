// ---- ícones do menu do botão direito: traços de 16 px em currentColor (strings fixas e confiáveis; nada vem de dados do usuário) ----
const ICONES_CONTEXTO = {
  abrir: '<path d="M6 3h7v7M13 3L6 10M12 9v4H3V4h4"/>',
  lapis: '<path d="M3 13l1-3.5L11.5 2l2.5 2.5L6.5 12z M10 3.5l2.5 2.5"/>',
  alvo: '<circle cx="8" cy="8" r="5"/><circle cx="8" cy="8" r="1.5"/><path d="M8 1v2M8 13v2M1 8h2M13 8h2"/>',
  tela: '<rect x="2" y="3" width="12" height="8" rx="1"/><path d="M6 14h4M8 11v3"/>',
  doc: '<path d="M4 2h6l3 3v9H4z M10 2v3h3M6 8h5M6 11h5"/>',
  losango: '<path d="M8 2l6 6-6 6-6-6z"/>',
  externo: '<path d="M9 2h5v5M14 2L7 9M12 10v4H2V4h4"/>',
  check: '<path d="M3 8.5l3.5 3.5L13 4.5"/>',
  circulo: '<circle cx="8" cy="8" r="5.5"/>',
  lixo: '<path d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.7 9h5.6l.7-9M7 7v4M9 7v4"/>',
  voltar: '<path d="M13 8H3M7 4L3 8l4 4"/>',
  tela_cheia: '<path d="M2 6V2h4M10 2h4v4M14 10v4h-4M6 14H2v-4"/>',
  estrela: '<path d="M8 1.5l1.8 4.2 4.2.5-3.2 2.9.9 4.4L8 11.3 4.3 13.5l.9-4.4L2 6.2l4.2-.5z"/>',
  relogio: '<circle cx="8" cy="8" r="5.5"/><path d="M8 5v3l2 1.5"/>',
  calendario: '<rect x="2.5" y="3.5" width="11" height="10" rx="1"/><path d="M2.5 7h11M5.5 2v3M10.5 2v3"/>',
  play: '<path d="M5 3l8 5-8 5z"/>',
  mais: '<path d="M8 3v10M3 8h10"/>',
  recolher: '<path d="M4 10l4-4 4 4"/>',
  expandir: '<path d="M4 6l4 4 4-4"/>',
  entrar: '<path d="M6 3H3v10h3M7 8h7M11 5l3 3-3 3"/>',
};
/** O <svg> de um ícone (vazio se o nome não existe). */
function iconeContexto(nome) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 16 16");
  svg.setAttribute("aria-hidden", "true");
  svg.innerHTML = ICONES_CONTEXTO[nome] || "";
  return svg;
}
