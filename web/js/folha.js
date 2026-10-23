// ---- a folha que se abre: o papel se revela de cima para baixo com um rolo na frente, e a marca "em construção" (anatomy) vai sendo montada atrás ----
// Serve ao aviso do agente e à tela Planejar, para as duas abrirem do mesmo jeito. Só anima ao abrir (não sobra timer nem laço depois);
// com movimento reduzido, ou com a janela oculta, a folha aparece pronta.
const FOLHA_MS = 640, MARCA_MS = 1500, MARCA_PASSOS = 14;
let folhaFundoEl = null;
const folhaTimers = new WeakMap();

// a cortina escura atrás da folha (um elemento à parte: o clip-path da folha cortaria uma sombra dela)
function folhaFundo(ligado) {
  if (!folhaFundoEl) {
    folhaFundoEl = document.createElement("div");
    folhaFundoEl.id = "folha-fundo";
    document.body.appendChild(folhaFundoEl);
  }
  folhaFundoEl.classList.toggle("on", !!ligado);
}
// A marca de construção montando: com o progresso de 0 a 1 as primeiras peças aparecem prontas e as outras só no traço, como num blueprint por fazer.
function folhaMarca(el) {
  let m = el.querySelector(":scope > .fa-marca");
  if (!m) { m = document.createElement("div"); m.className = "fa-marca"; m.setAttribute("aria-hidden", "true"); el.prepend(m); }
  clearTimeout(folhaTimers.get(el));
  const desenha = (p) => { m.innerHTML = previaMarca(EMB?.CONSTRUCAO, 560, 400, p); };
  const passo = (i) => {
    if (el.hidden) return;
    desenha(i / MARCA_PASSOS);
    if (i < MARCA_PASSOS) folhaTimers.set(el, setTimeout(() => passo(i + 1), MARCA_MS / MARCA_PASSOS));
  };
  const comeca = () => { if (reduz()) desenha(1); else passo(0); };
  if (EMB) comeca(); else emblemasProntos.then(comeca);
}
// `el` já visível. Devolve quando a folha terminou de abrir (promessa).
function folhaAbre(el, { fundo = true, marca = true } = {}) {
  if (fundo) folhaFundo(true);
  if (marca) folhaMarca(el);
  if (reduz()) return Promise.resolve();
  const alto = el.getBoundingClientRect().height;
  const rolo = document.createElement("div");
  rolo.className = "fa-rolo";
  rolo.setAttribute("aria-hidden", "true");
  el.appendChild(rolo);
  const facil = "cubic-bezier(.25,.7,.2,1)";
  el.animate([{ clipPath: "inset(0 0 100% 0)", opacity: 0.4 }, { clipPath: "inset(0 0 0 0)", opacity: 1 }], { duration: FOLHA_MS, easing: facil });
  const fim = rolo.animate([{ transform: "translateY(0)" }, { transform: `translateY(${Math.max(0, alto - 18)}px)` }], { duration: FOLHA_MS, easing: facil, fill: "forwards" });
  return fim.finished.then(() => rolo.remove(), () => rolo.remove());
}
// Fecha (a folha se enrola de volta) e depois chama `depois`; sem animação chama já.
function folhaFecha(el, depois, { fundo = true } = {}) {
  clearTimeout(folhaTimers.get(el));
  if (fundo) folhaFundo(false);
  const termina = () => { el.hidden = true; depois?.(); };
  if (reduz()) return termina();
  const a = el.animate([{ clipPath: "inset(0 0 0 0)", opacity: 1 }, { clipPath: "inset(0 0 100% 0)", opacity: 0.4 }], { duration: 280, easing: "cubic-bezier(.5,0,.8,.3)", fill: "forwards" });
  a.finished.then(() => { a.cancel(); termina(); }, termina);
}
