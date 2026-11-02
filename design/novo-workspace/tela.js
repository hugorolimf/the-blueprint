// O comportamento do protótipo: FIGURA vem do build.mjs. Nada aqui grava nada.
const $ = (id) => document.getElementById(id);

function monta() {
  $("q-traco").innerHTML = FIGURA.traco;
  $("q-pronta").innerHTML = FIGURA.pronta;
  const f = $("figura");
  f.classList.add("montando");
  void f.offsetWidth;
  requestAnimationFrame(() => setTimeout(() => f.classList.remove("montando"), 240));
}
function carimbo() {
  const n = $("nome").value.trim(), d = $("descricao").value.trim();
  $("c-nome").textContent = n || "Sem nome";
  $("c-nome").classList.toggle("vazio", !n);
  $("c-desc").textContent = d || "—";
  if (n) { $("erro").hidden = true; $("nome").classList.remove("invalido"); }
}

$("nome").oninput = carimbo;
$("descricao").oninput = carimbo;
$("form").onsubmit = (e) => {
  e.preventDefault();
  const n = $("nome").value.trim();
  if (!n) { $("erro").hidden = false; $("nome").classList.add("invalido"); $("nome").focus(); return; }
  $("criado-nome").textContent = n;
  $("criado").hidden = false;
};
$("refazer").onclick = () => { $("criado").hidden = true; $("nome").focus(); };

// as réguas da folha: como as margens de uma prancha, letras em cima e números do lado
const regua = (el, n, passo, horizontal, marcas) => {
  for (let i = 0; i <= n; i++) {
    const t = document.createElement("i");
    t.style.cssText = horizontal ? `left:${i * passo}px;bottom:0;width:1px;height:${i % 5 ? 4 : 9}px` : `top:${i * passo}px;right:0;height:1px;width:${i % 5 ? 4 : 9}px`;
    el.append(t);
    if (i % 5 === 0 && i < n) {
      const r = document.createElement("b");
      r.textContent = marcas[i / 5];
      r.style.cssText = horizontal ? `left:${i * passo + 5}px;top:-2px` : `top:${i * passo + 3}px;left:0`;
      el.append(r);
    }
  }
};
regua($("regua-topo"), 35, 18, true, "ABCDEFGH");
regua($("regua-lado"), 35, 18, false, "12345678");

const ajusta = () => { $("tela").style.transform = `scale(${Math.min(innerWidth / 1280, innerHeight / 760)})`; };
addEventListener("resize", ajusta);
ajusta();
carimbo();
monta();
// para capturas: #Nome|descrição
const [n, d] = decodeURIComponent(location.hash.slice(1)).split("|");
if (n) { $("nome").value = n; $("descricao").value = d || ""; carimbo(); }
