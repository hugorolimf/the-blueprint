// Ponto único de abrir e fechar a folha com movimento (ação 1.13, docs/expandir-recolher.md): alternar, abrirOuFecharTudo, alternarProjeto e irPara passam por aqui.
// Guarda as posições, muda o estado, (ao recolher) esmaece o que sai, redesenha e faz o resto deslizar (organizarFolha) e o que entra aparecer.
// Sem animação (mesmo resultado final) com movimento reduzido, aba oculta ou sem folha na tela; um clique novo cancela as animações em curso.
const DUR_ABRIR = 350, MAX_SAEM = 120, MAX_ENTRAM = 60; // tetos de itens esmaecendo e entrando (o excedente muda sem animação)
let seqAbrir = 0;
const EASE_ABRIR = "cubic-bezier(.45,.05,.2,1)";
const dentroDaJanela = (r) => r.width && r.bottom >= 0 && r.top <= innerHeight;
// seletor do controle com foco (o redesenho troca o elemento): é onde o foco volta e o que fica parado na tela
function seletorDoFoco() {
  const a = document.activeElement;
  if (!a || !host.contains(a)) return null;
  const q = (attr, v) => `[${attr}="${CSS.escape(v)}"]`;
  if (a.dataset.toggle) return q("data-toggle", a.dataset.toggle);
  if (a.dataset.projeto) return q("data-projeto", a.dataset.projeto);
  if (a.dataset.chk) return q("data-chk", a.dataset.chk);
  return a.classList.contains("acao") && a.dataset.f ? `.acao${q("data-f", a.dataset.f)}` : null;
}
// planos e checkpoints que saem esmaecem, de baixo para cima (fill/stroke-opacity: opacity criaria uma camada por item)
function esmaecerSaida(svg, saem) {
  const fins = [];
  for (const el of svg.querySelectorAll(".chk[data-id]")) {
    if (fins.length >= MAX_SAEM) break;
    const r = el.getBoundingClientRect();
    if (dentroDaJanela(r) && saem(el)) fins.push(el.animate([{ fillOpacity: 1, strokeOpacity: 1 }, { fillOpacity: 0, strokeOpacity: 0 }], { duration: 150, delay: (1 - Math.max(r.top, 0) / innerHeight) * 60, fill: "both", easing: "ease-in" }).finished.catch(() => {}));
  }
  return Promise.all(fins);
}
// planos e checkpoints novos (não estavam na folha antes) aparecem de cima para baixo
function revelarEntrada(svg, antes) {
  const novos = [];
  for (const el of svg.querySelectorAll(".chk[data-id]")) {
    if (novos.length >= MAX_ENTRAM) break;
    if (!antes.has("i:" + el.dataset.id) && dentroDaJanela(el.getBoundingClientRect())) novos.push(el);
  }
  novos.forEach((el, i) => el.animate([{ fillOpacity: 0, strokeOpacity: 0 }, { fillOpacity: 1, strokeOpacity: 1 }], { duration: 220, delay: Math.min(i * 20, 240), easing: EASE_ABRIR, fill: "backwards" }));
}
// muda: altera o estado (síncrono); saem: quais planos/checkpoints esmaecem ao recolher (omitido = não há saída, é uma abertura);
// depois: roda logo após o redesenho, antes das animações (o que mede posições na folha final, como a rolagem de irPara)
async function animarAbrirFechar(muda, saem, depois) {
  const meu = ++seqAbrir, svg = host.querySelector("svg.desenho"), anima = !!svg && !reduz();
  if (anima) svg.getAnimations({ subtree: true }).forEach((a) => a.cancel()); // clique rápido: mede a folha parada, sem animação presa
  const antes = anima ? posicoesDaFolha() : null, foco = seletorDoFoco();
  const topo = foco && host.querySelector(foco)?.getBoundingClientRect().top;
  muda(); // o estado muda na hora: três cliques seguidos são três trocas, mesmo com o redesenho esperando o esmaecer
  if (anima && saem) await esmaecerSaida(svg, saem);
  if (meu !== seqAbrir) return; // outro clique chegou: ele cancela o esmaecer e redesenha com o estado já somado
  render();
  const novo = foco && host.querySelector(foco);
  if (novo) {
    novo.focus({ preventScroll: true });
    if (topo !== undefined) scrollBy({ top: novo.getBoundingClientRect().top - topo, behavior: "instant" }); // o controle clicado fica parado na tela
  }
  depois?.();
  if (antes) { organizarFolha(antes, undefined, DUR_ABRIR); revelarEntrada(host.querySelector("svg.desenho"), antes); } // antes de desenharBarras: mede a folha sem as animações dela
  desenharBarras();
}
