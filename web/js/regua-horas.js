// Altera somente a régua e as barras da folha existente; alternar não reconstrói o SVG.
let modoHoras = (() => { try { return localStorage.getItem("blueprint.escala") === "horas"; } catch { return false; } })();
let diaHoras = "";
const horaLocal = (ms) => { const d = new Date(ms); return dia(local(d)) * 24 + d.getHours() + d.getMinutes() / 60 + (d.getSeconds() * 1000 + d.getMilliseconds()) / 3600000; };
const horasNumero = (n) => Number(n.toFixed(3)).toLocaleString(I18N.ativo, { maximumFractionDigits: 3 });
function horasFaixa(inicio, fim, diaInicio, x, largura, y, altura, classe, plano, dica) {
  const a = Math.max(inicio, diaInicio), b = Math.min(fim, diaInicio + 24);
  if (!(b > a)) return "";
  const de = x + (a - diaInicio) / 24 * largura, w = Math.max(0.5, (b - a) / 24 * largura);
  return `<rect class="${classe}" data-horas-plano="${esc(plano)}" tabindex="0" role="img" aria-label="${esc(dica)}" x="${de}" y="${y}" width="${w}" height="${altura}"><title>${esc(dica)}</title></rect>`;
}
function horasAtualiza() {
  const ativo = !!(modoHoras && tela && carregado && workspaceAtual), p = ativo && D.blueprints.find((x) => x.id === tela);
  $("escala-dias").setAttribute("aria-pressed", String(!modoHoras));
  $("escala-horas").setAttribute("aria-pressed", String(modoHoras));
  $("escala-dia-label").hidden = !p;
  const svg = host.querySelector("svg.desenho"), regua = svg?.querySelector(".regua");
  if (!svg || !regua) return;
  svg.querySelectorAll(".horas-regua, .horas-acao").forEach((el) => el.remove());
  svg.querySelectorAll(".acao.horas-on").forEach((el) => el.classList.remove("horas-on"));
  if (!p) { regua.style.removeProperty("display"); if (!regua.getAttribute("style")) regua.removeAttribute("style"); return; }
  if (!ISO.test(diaHoras) || !Number.isFinite(dia(diaHoras))) diaHoras = local(new Date());
  $("escala-dia").value = diaHoras;
  regua.style.display = "none";
  const W = svg.viewBox.baseVal.width, H = svg.viewBox.baseVal.height, L = 70, R = W - 86, largura = R - L;
  const yRegua = 204, yFim = H - 166, inicio = dia(diaHoras) * 24, agora = horaLocal(Date.now());
  const X = (h) => L + (h - inicio) / 24 * largura;
  const marcas = marcasDeHoras(inicio, inicio + 24).map((m) => `<g class="horas-marca"><path class="ln m" d="M${X(m.hora)} ${yRegua - 6}V${yRegua}"/><text class="mid" x="${X(m.hora)}" y="${yRegua - 13}" font-size="9" text-anchor="middle">${String(m.horaDoDia).padStart(2, "0")}</text></g>`).join("");
  const agoraSvg = agora >= inicio && agora < inicio + 24 ? `<path class="eixo horas-agora" d="M${X(agora)} ${yRegua}V${yFim}"/><g class="hoje-chip"><rect x="${X(agora) - 46}" y="${yRegua + 8}" width="92" height="20" fill="#eaf4ff"/><text x="${X(agora)}" y="${yRegua + 22}" font-size="11" font-weight="700" text-anchor="middle" style="fill:#0c3a78">${tr("AGORA")}</text></g>` : "";
  regua.insertAdjacentHTML("afterend", `<g class="horas-regua"><path class="ln" d="M${L} ${yRegua}H${R}"/><text class="mid" x="${L}" y="${yRegua - 32}" font-size="11">${esc(fmt(diaHoras))}</text>${marcas}${agoraSvg}</g>`);
  const originais = new Map(p.acoes.map((a) => [a.id, a]));
  const visiveis = new Map(acoesVisiveis(p).map((a) => [a.id, a]));
  const trabalhando = new Set(agentesDoBlueprint(p.id).filter((a) => a.estado !== "concluido" && a.estado !== "ocioso" && a.planoId).map((a) => a.planoId));
  for (const el of svg.querySelectorAll(".acao[data-f]")) {
    const acao = originais.get(el.dataset.f), visivel = visiveis.get(el.dataset.f), caixa = el.querySelector(":scope > .caixa");
    if (!acao || !visivel || !caixa) continue;
    const y = Number(caixa.getAttribute("y")), altura = Number(caixa.getAttribute("height"));
    const ids = new Set(visivel.planos.map((t) => t.id)), faixas = horasDaAcao(acao), linhas = [];
    let primeiro = Infinity, ultimo = -Infinity;
    const inclui = (a, b) => { const de = Math.max(a, inicio), ate = Math.min(b, inicio + 24); if (ate > de) { primeiro = Math.min(primeiro, de); ultimo = Math.max(ultimo, ate); } };
    el.classList.add("horas-on");
    faixas.forEach((f, i) => {
      const t = acao.planos[i]; if (!ids.has(t.id)) return;
      const nome = t.texto || tr("sem texto"), previsto = tr("Previsto: {horas} h", { horas: horasNumero(f.previsto) });
      inclui(f.inicio, f.fim);
      linhas.push(horasFaixa(f.inicio, f.fim, inicio, L, largura, y + 1, Math.max(1, (altura - 2) / 2), "horas-previsto", t.id, `${nome} · ${previsto}`));
      const iniMs = Date.parse(t.agente?.inicioEm), fimRegistrado = Date.parse(t.agente?.feitoEm);
      const emCurso = !Number.isFinite(fimRegistrado) && trabalhando.has(t.id), fimMs = emCurso ? Date.now() : fimRegistrado;
      if (!Number.isFinite(iniMs) || !Number.isFinite(fimMs) || fimMs < iniMs) return;
      const duracao = (fimMs - iniMs) / 3600000, iniReal = horaLocal(iniMs), fimReal = iniReal + duracao;
      inclui(iniReal, fimReal);
      const real = emCurso ? tr("Em andamento: {horas} h", { horas: horasNumero(duracao) }) : tr("Real: {horas} h", { horas: horasNumero(duracao) });
      const desvio = f.real == null ? "" : `${f.real >= f.previsto ? "+" : ""}${horasNumero(f.real - f.previsto)} h`;
      linhas.push(horasFaixa(iniReal, fimReal, inicio, L, largura, y + altura / 2, Math.max(1, altura / 2 - 1), "horas-real", t.id, `${nome} · ${real}${desvio ? " · " + tr("Desvio") + ": " + desvio : ""}`));
      const ponto = Math.max(iniReal, inicio), espaco = (Math.min(fimReal, inicio + 24) - ponto) / 24 * largura;
      if (desvio && altura >= 20 && espaco >= 42) linhas.push(`<text class="horas-desvio" data-horas-delta="${esc(desvio)}" x="${X(ponto) + 3}" y="${y + altura - 3}" font-size="9">${esc(desvio)}</text>`);
    });
    if (primeiro < ultimo) {
      const de = X(primeiro), ate = X(ultimo);
      linhas.unshift(`<rect class="caixa horas-contorno" x="${de}" y="${y}" width="${Math.max(.5, ate - de)}" height="${altura}"/>`);
      if (altura >= 20) linhas.push(`<text x="${Math.min(ate + 20, R + 4)}" y="${y + 17}" font-size="12" font-weight="700">${Math.round(visivel.pct * 100)}%</text>${visivel.it[1] ? `<text class="mid" x="${Math.min(ate + 20, R + 4)}" y="${y + 31}" font-size="9">${visivel.it[0]}/${visivel.it[1]}</text>` : ""}`);
    }
    el.insertAdjacentHTML("beforeend", `<g class="horas-acao">${linhas.join("")}</g>`);
  }
}
function horasAlterna(ativo) {
  modoHoras = ativo;
  try { localStorage.setItem("blueprint.escala", ativo ? "horas" : "dias"); } catch {}
  horasAtualiza();
  if (typeof claudeCamadaAtualiza === "function") claudeCamadaAtualiza();
}
$("escala-dias").onclick = () => horasAlterna(false);
$("escala-horas").onclick = () => horasAlterna(true);
$("escala-dia").addEventListener("change", (e) => { diaHoras = e.target.value; horasAtualiza(); });
document.addEventListener("keydown", (e) => { if (tela && e.altKey && !e.ctrlKey && !e.metaKey && e.key.toLowerCase() === "h") { e.preventDefault(); horasAlterna(!modoHoras); } });
