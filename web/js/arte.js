// "Arte do projeto" na criação do workspace: marca pronta, texto (sigla) ou peças coladas como texto (JSON), com prévia ao vivo.
const MARCAS_PRONTAS = [["blueprint", "Blueprint (padrão)"], ["moedas", "Moedas"], ["livros", "Livros"], ["chip", "Chip"], ["caminhao", "Caminhão"]];
const arteModo = () => $("workspace-arte-modo").value;
function pecasDoTexto(t) {
  const tam = Math.max(14, Math.min(80, Math.floor(250 / (t.length * 0.68)))); // a sigla cabe na placa de 300 de largura
  return [{ tipo: "laje", x: -150, y: -60, w: 300, d: 120, z: 0, h: 10, r: 6, borda: 1, tom: "mid" }, { tipo: "texto", x: 0, y: Math.round(tam * 0.35), z: 10, texto: t, tam, tom: "hi", ancora: "middle" }];
}
// devolve { icone, pecas }: pecas só existe quando a arte é do próprio workspace (texto ou JSON); lança Error com a mensagem para o usuário
function lerArte() {
  const modo = arteModo();
  if (modo === "marca") return { icone: $("workspace-arte-marca").value, pecas: null };
  if (!EMB) throw new Error("A prévia da arte ainda está carregando.");
  if (modo === "texto") {
    const t = $("workspace-arte-texto").value.trim().slice(0, 12);
    if (!t) throw new Error("Digite o texto da arte.");
    return { icone: "blueprint", pecas: pecasDoTexto(t) };
  }
  const bruto = $("workspace-arte-json").value.trim();
  if (!bruto) throw new Error("Cole as peças da arte em JSON.");
  let v;
  try { v = JSON.parse(bruto); } catch { throw new Error("O texto da arte não é um JSON válido."); }
  const pecas = EMB.sanitizaPecas(Array.isArray(v) ? v : v?.pecas);
  if (!pecas.length) throw new Error("Nenhuma peça válida na arte.");
  return { icone: "blueprint", pecas };
}
function atualizaArte() {
  const modo = arteModo();
  $("ws-arte-marca").hidden = modo !== "marca";
  $("ws-arte-texto").hidden = modo !== "texto";
  $("ws-arte-json").hidden = modo !== "json";
  let html;
  try {
    const a = lerArte();
    html = previaMarca(a.pecas || EMB?.MARCAS?.[a.icone] || EMB?.BLUEPRINT, 520, 380);
  } catch (e) { html = `<p class="vazio">${esc(e.message)}</p>`; }
  $("workspace-previa").innerHTML = html;
}
$("workspace-arte-marca").innerHTML = MARCAS_PRONTAS.map(([k, n]) => `<option value="${k}">${n}</option>`).join("");
["workspace-arte-modo", "workspace-arte-marca", "workspace-arte-texto", "workspace-arte-json"].forEach((id) => { $(id).oninput = atualizaArte; });
