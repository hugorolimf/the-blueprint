// Pesquisa na folha (DOM): a lista de blueprints do workspace e o blueprint aberto. A correspondência está em pesquisa-logica.js; o campo, em pesquisa-campo.js.
// Na lista, o render() filtra os cartões e cada um diz onde achou. No blueprint NADA é redesenhado a cada tecla: os itens que não casam esmaecem
// (fill/stroke-opacity, que não cria camada por item; a classe `pesquisando` na folha esmaece tudo e só os que casam levam classe: `pesq-achou`, ou `pesq-tem` na ação que só casa por um item), e o trecho achado leva um retângulo atrás do texto.
// Só quando o achado está numa seção fechada a folha é redesenhada, uma vez, para abri-la; ao limpar, a seção volta a ficar como estava.
const PESQ_MAX_REALCES = 200; // textos destacados por vez (o excedente fica sem o retângulo, mas continua aceso)
const pesqAbertas = { blueprints: new Set(), projetos: new Set() }; // seções que a pesquisa abriu
let pesqAtual = -1; // posição do resultado em foco (Enter / Shift+Enter)

// ---- lista de blueprints: devolve os cartões que casam e, para cada um, o texto "onde achou"
function pesqLista(todos, consultaBruta) {
  const termos = pesqTermos(consultaBruta), onde = new Map();
  if (!termos.length) return { mapa: todos, onde };
  const mapa = todos.filter((x) => {
    const r = pesqBlueprint(x.p, termos);
    if (!pesqCasaBlueprint(r)) return false;
    const partes = [];
    if (r.proprio && !r.n.total) partes.push(tr("nome ou descrição"));
    if (r.n.acoes) partes.push(trn(r.n.acoes, ["{n} ação", "{n} ações"]));
    if (r.n.planos) partes.push(trn(r.n.planos, ["{n} plano", "{n} planos"]));
    if (r.n.checkpoints) partes.push(trn(r.n.checkpoints, ["{n} checkpoint", "{n} checkpoints"]));
    if (r.proprio && r.n.total) partes.unshift(tr("nome ou descrição"));
    onde.set(x.p.id, tr("ACHOU: {onde}", { onde: partes.join(" · ").toUpperCase() }));
    return true;
  });
  return { mapa, onde };
}

// ---- retângulos atrás do trecho achado (texto SVG: getExtentOfChar dá a caixa de cada letra). Primeiro LÊ todas as caixas (sem nada sujo no layout), depois escreve.
function pesqMede(textos, termos) {
  const feitos = [];
  for (const el of textos) {
    if (feitos.length >= PESQ_MAX_REALCES) break;
    const n = el.getNumberOfChars(), rects = [];
    for (const [i, j] of pesqTrechos(el.textContent, termos)) {
      if (j > n) continue;
      let inicio = i;
      for (let k = i + 1; k <= j; k++) {
        const a = el.getExtentOfChar(inicio), b = el.getExtentOfChar(k - 1);
        if (k === j || el.getExtentOfChar(k).y !== a.y) {
          rects.push([a.x - 1.5, a.y, b.x + b.width - a.x + 3, a.height]);
          inicio = k;
        }
      }
    }
    if (rects.length) feitos.push([el, rects]);
  }
  return feitos;
}
const SVGNS = "http://www.w3.org/2000/svg";
function pesqEscreve(feitos, svg) {
  if (svg) { // blueprint: um só <path> logo depois do <defs>, atrás de tudo (um nó no lugar de centenas)
    const d = feitos.flatMap(([, rs]) => rs.map(([x, y, w, h]) => `M${x} ${y}h${w}v${h}h${-w}z`)).join("");
    if (!d) return;
    const path = document.createElementNS(SVGNS, "path");
    path.setAttribute("class", "pesq-realce"); path.setAttribute("d", d);
    (svg.querySelector(":scope > defs") || svg.firstChild)?.after(path);
    return;
  }
  for (const [el, rects] of feitos) for (const [x, y, w, h] of rects) { // lista: poucos cartões, um retângulo antes de cada texto
    const r = document.createElementNS(SVGNS, "rect");
    r.setAttribute("class", "pesq-realce");
    r.setAttribute("x", x); r.setAttribute("y", y); r.setAttribute("width", w); r.setAttribute("height", h);
    el.before(r);
  }
}
let pesqMarcados = [], pesqResultados = []; // elementos com classe de pesquisa; os que casam, na ordem da folha
const pesqLimpa = (svg) => {
  svg.querySelector(":scope > .pesq-realce")?.remove();
  svg.classList.remove("pesquisando");
  for (const e of pesqMarcados) e.classList.remove("pesq-achou", "pesq-tem", "pesq-atual");
  pesqMarcados = []; pesqResultados = [];
};

// ---- seções que precisam estar abertas para o achado aparecer (e a volta ao limpar)
function pesqAbre(p, r) {
  let mudou = false;
  if (!p.projetos?.length) {
    if (r.acoes.length && abertos[p.id] !== true) { abertos[p.id] = true; pesqAbertas.blueprints.add(p.id); mudou = true; }
    return mudou;
  }
  for (const a of r.acoes) {
    const f = p.acoes.find((x) => x.id === a.id), chave = chaveProjeto(p, projetoDaAcao(p, f));
    if (!projetosAbertos.has(chave)) { projetosAbertos.add(chave); pesqAbertas.projetos.add(chave); mudou = true; } // sem guardaProjetosAbertos: é só enquanto se pesquisa
  }
  return mudou;
}
function pesqFecha() {
  const mudou = pesqAbertas.blueprints.size + pesqAbertas.projetos.size > 0;
  pesqAbertas.blueprints.forEach((id) => { abertos[id] = false; });
  pesqAbertas.projetos.forEach((c) => projetosAbertos.delete(c));
  pesqAbertas.blueprints.clear(); pesqAbertas.projetos.clear();
  return mudou;
}

const pesqLugar = (el) => (el.matches?.(".acao") ? el.querySelector(":scope > text.cap") || el : el);
function pesqVai(el) {
  const r = pesqLugar(el).getBoundingClientRect();
  if (r.top >= 90 && r.bottom <= innerHeight - 40) return;
  scrollTo({ top: Math.max(0, scrollY + r.top - innerHeight * 0.35), behavior: reduz() ? "auto" : "smooth" });
}
function pesqContador() {
  const n = pesqResultados.length, termos = pesqTermos(consulta);
  $("busca-n").textContent = !termos.length ? "" : !n ? tr("nenhum resultado") : pesqAtual >= 0 ? `${pesqAtual + 1} / ${n}` : trn(n, ["{n} resultado", "{n} resultados"]);
  return n;
}
// Enter / Shift+Enter: vai ao resultado seguinte / anterior (só no blueprint)
function pesqVaiPara(passo) {
  const todos = pesqResultados;
  if (!todos.length) return;
  todos[pesqAtual]?.classList.remove("pesq-atual");
  pesqAtual = (pesqAtual + passo + todos.length) % todos.length;
  todos[pesqAtual].classList.add("pesq-atual");
  pesqVai(todos[pesqAtual]);
  pesqContador();
}

// ---- o blueprint aberto: aplica a consulta sem redesenhar (ou redesenha uma vez para abrir uma seção)
function pesquisaFolha() {
  const p = tela && D.blueprints.find((x) => x.id === tela), svg = host.querySelector("svg.desenho");
  if (!p || !svg) return;
  const termos = pesqTermos(consulta);
  pesqAtual = -1;
  if (!termos.length) {
    pesqLimpa(svg);
    if (pesqFecha()) render(); // as seções que a pesquisa abriu voltam ao que eram
    pesqContador();
    return;
  }
  const r = pesqBlueprint(p, termos, acoesVisiveis(p));
  if (pesqAbre(p, r)) { render(); return; } // o render() termina chamando pesquisaAposRender → pesquisaFolha, já com a seção aberta
  pesqLimpa(svg);
  const porAcao = new Map(r.acoes.map((a) => [a.id, a])), itens = new Map(r.acoes.flatMap((a) => [...a.planos, ...a.checkpoints].map((x) => [x.id, x.onde])));
  const marcas = [], textos = [], achou = [];
  for (const e of svg.querySelectorAll(".acao[data-f], .chk[data-id]")) { // na ordem da folha: cada ação e depois os itens dela
    if (e.classList.contains("chk")) {
      const onde = itens.get(e.dataset.id);
      if (!onde) continue;
      marcas.push([e, "pesq-achou"]); achou.push(e);
      if (onde.includes("nome")) { const t = e.querySelector("text.tx"); if (t) textos.push(t); }
    } else {
      const a = porAcao.get(e.dataset.f);
      if (!a) continue;
      marcas.push([e, a.propria ? "pesq-achou" : "pesq-tem"]);
      if (a.propria) { achou.push(e); if (a.propria.includes("nome")) { const t = e.querySelector(":scope > text.cap"); if (t) textos.push(t); } }
    }
  }
  const feitos = pesqMede(textos, termos); // ler antes de escrever
  for (const [e, c] of marcas) { e.classList.add(c); pesqMarcados.push(e); }
  svg.classList.add("pesquisando");
  pesqEscreve(feitos, svg);
  pesqResultados = achou;
  const n = pesqContador();
  if (n) requestAnimationFrame(() => pesqVai(achou[0])); // depois do scrollTo(0, 0) que a navegação faz logo após o render
}

// Chamado no fim do render(): a lista só destaca os nomes dos cartões; o blueprint aplica a consulta.
function pesquisaAposRender(mapa) {
  const campo = $("busca");
  campo.placeholder = tela ? tr("Pesquisar ações, planos e checkpoints…") : tr("Pesquisar blueprints, ações, planos…");
  campo.setAttribute("aria-label", tela ? tr("Pesquisar ações, planos e checkpoints deste blueprint") : tr("Pesquisar blueprints, ações, planos e checkpoints do workspace"));
  campo.title = tr("Atalhos: / ou Ctrl+K pesquisa · Enter vai ao próximo · Shift+Enter ao anterior · Esc limpa");
  const termos = pesqTermos(consulta);
  if (tela) return pesquisaFolha();
  pesqAtual = -1;
  $("busca-n").textContent = !termos.length ? "" : mapa.length ? trn(mapa.length, ["{n} resultado", "{n} resultados"]) : tr("nenhum resultado");
  if (termos.length) pesqEscreve(pesqMede([...host.querySelectorAll("[data-goto] > g > text.cap")], termos)); // o nome e a descrição do cartão
}

// Ao trocar de tela (hashchange, antes de `tela` mudar): da lista para um blueprint a consulta vai junto; em qualquer outro caso ela zera.
function pesquisaAoNavegar(destino) {
  if (!(tela === null && destino)) {
    consulta = ""; $("busca").value = "";
    pesqFecha();
  }
}
