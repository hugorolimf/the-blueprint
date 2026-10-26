// ---- idiomas: o app abre no idioma do Windows (inglês quando não há tradução) e o usuário pode trocar ----
// O TEXTO-FONTE em português é a chave: tr("Aplicar") devolve o texto do catálogo do idioma ativo e, se ele não tiver tradução, o próprio português.
// Assim nada quebra enquanto as telas são convertidas aos poucos, e o pt-BR (o idioma-fonte) não precisa de catálogo.
// Catálogos: web/i18n/<idioma>-<área>.js, cada um com `Object.assign(I18N.catalogos["en"], { "Aplicar": "Apply" })`; só os do idioma ativo são carregados.
// Plural: trn(n, ["{n} plano", "{n} planos"]) escolhe pela regra do idioma ativo (Intl.PluralRules); no catálogo o valor é a lista [um, outros].
// Escolha: ?idioma=en na URL > escolha guardada (localStorage "idioma", ou "auto") > navigator.languages (o WebView2 usa o idioma do Windows) > inglês.
const I18N = {
  fonte: "pt-BR",
  suportados: { "pt-BR": "Português (Brasil)", en: "English" },
  areas: ["base", "folha", "avisos", "contexto", "planejar", "edicao", "resto", "roadmap"], // arquivos de web/i18n/<idioma>-<área>.js; só liste áreas que existem (cada uma é um pedido a mais)
  catalogos: { en: {} },
  ativo: "pt-BR",
};
function i18nNormaliza(l) {
  const x = String(l || "").toLowerCase();
  return x.startsWith("pt") ? "pt-BR" : x.startsWith("en") ? "en" : "";
}
function i18nEscolhe(entrada = {}) {
  const { url = (typeof location !== "undefined" ? location.search : ""), guardado = lerIdiomaGuardado(), sistema = typeof navigator !== "undefined" ? (navigator.languages?.length ? navigator.languages : [navigator.language]) : [] } = entrada;
  const pedido = i18nNormaliza((/[?&]idioma=([^&#]+)/.exec(url) || [])[1]) || (guardado && guardado !== "auto" ? i18nNormaliza(guardado) : "");
  if (pedido) return pedido;
  for (const l of sistema) { const n = i18nNormaliza(l); if (n) return n; }
  return "en"; // idioma do Windows sem tradução (francês, espanhol…): inglês
}
function lerIdiomaGuardado() { try { return localStorage.getItem("idioma") || ""; } catch { return ""; } }
function definirIdioma(codigo) { // "auto", "en" ou "pt-BR"; recarrega para a tela inteira nascer no idioma novo
  try { codigo === "auto" ? localStorage.removeItem("idioma") : localStorage.setItem("idioma", codigo); } catch {}
  location.reload();
}
function tr(fonte, p) {
  const cat = I18N.catalogos[I18N.ativo];
  let s = cat && Object.prototype.hasOwnProperty.call(cat, fonte) ? cat[fonte] : fonte;
  return p ? s.replace(/\{(\w+)\}/g, (m, k) => (k in p ? p[k] : m)) : s;
}
function trn(n, [um, varios], p) {
  const cat = I18N.catalogos[I18N.ativo], par = cat && Object.prototype.hasOwnProperty.call(cat, um) ? cat[um] : [um, varios];
  const valores = { n, ...p }; // a forma escolhida não passa por tr(): em inglês ela pode ser igual à chave, e a chave aponta para a lista
  return par[new Intl.PluralRules(I18N.ativo).select(n) === "one" ? 0 : 1].replace(/\{(\w+)\}/g, (m, k) => (k in valores ? valores[k] : m));
}
I18N.ativo = i18nEscolhe();
if (typeof document !== "undefined" && document.documentElement && document.write) {
  document.documentElement.lang = I18N.ativo;
  if (I18N.ativo !== I18N.fonte) // síncrono: os catálogos precisam existir antes de o próximo script rodar
    for (const area of I18N.areas) document.write(`<script src="/web/i18n/${I18N.ativo}-${area}.js"><\/script>`);
}
// textos que já estão no HTML da página (títulos, rótulos, aria-label, placeholder): trocados uma vez, por correspondência exata com o catálogo
function i18nTraduzDom(raiz = document.body) {
  if (I18N.ativo === I18N.fonte) return;
  const cat = I18N.catalogos[I18N.ativo], tem = (s) => Object.prototype.hasOwnProperty.call(cat, s);
  const andador = document.createTreeWalker(raiz, NodeFilter.SHOW_TEXT);
  for (let n = andador.nextNode(); n; n = andador.nextNode()) {
    const t = n.nodeValue.trim();
    if (t && tem(t)) n.nodeValue = n.nodeValue.replace(t, cat[t]);
  }
  for (const e of raiz.querySelectorAll("[aria-label],[title],[placeholder]"))
    for (const a of ["aria-label", "title", "placeholder"]) { const v = e.getAttribute(a); if (v && tem(v)) e.setAttribute(a, cat[v]); }
}
