// Movimento calmo das figuras: um laço de quadros por figura, só enquanto ela está à vista, mais as duas curvas de seguir um alvo.
(function (L) {
  const mediaCalmo = () => (typeof matchMedia === "function" ? matchMedia("(prefers-reduced-motion: reduce)") : { matches: false });
  L.calmo = () => mediaCalmo().matches;

  // seguir exponencial: tau em segundos, calmo salta direto ao alvo
  L.segue = (valor, alvo, dt, tau, calmo) => (calmo ? alvo : valor + (alvo - valor) * (1 - Math.exp(-dt / tau)));

  // mola com passos pequenos (estado = { x, v }); devolve true enquanto ainda se move
  L.mola = (estado, alvo, dt, rigidez, amortecimento, calmo) => {
    if (calmo) { estado.x = alvo; estado.v = 0; return false; }
    for (let i = 0; i < 4; i++) {
      estado.v += ((alvo - estado.x) * rigidez - estado.v * amortecimento) * dt / 4;
      estado.x += estado.v * dt / 4;
    }
    if (Math.abs(alvo - estado.x) < 0.005 && Math.abs(estado.v) < 0.005) { estado.x = alvo; estado.v = 0; return false; }
    return true;
  };

  // passo(dt, calmo) devolve true para continuar. O laço para fora da tela (lê a ÚLTIMA entrada do observador) e quando o passo diz que acabou;
  // acordar() o retoma depois de uma mudança de alvo. Sem IntersectionObserver (testes) considera a figura à vista.
  L.laco = (palco, passo) => {
    let quadro = 0, ultimo = 0, visivel = typeof IntersectionObserver !== "function";
    const volta = (agora) => {
      const dt = ultimo ? Math.min((agora - ultimo) / 1000, 1 / 30) : 1 / 60;
      ultimo = agora;
      quadro = visivel && passo(dt, L.calmo()) ? requestAnimationFrame(volta) : 0;
    };
    const acordar = () => { if (!quadro && visivel) { ultimo = 0; quadro = requestAnimationFrame(volta); } };
    if (!visivel) {
      new IntersectionObserver((entradas) => { visivel = entradas[entradas.length - 1].isIntersecting; if (visivel) acordar(); }, { rootMargin: "120px 0px" }).observe(palco);
    } else acordar();
    return { acordar };
  };
})(L);
