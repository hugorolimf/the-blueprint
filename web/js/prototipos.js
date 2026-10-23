// ---- protótipos das ações: HTML autocontido que o agente anexa pelo MCP; roda num quadro isolado (sandbox, sem rede) ----
const protoTela = $("proto-tela");
let protoAlvo = null; // { pid, fid, id }
const acaoDe = (pid, fid) => D.blueprints.find((x) => x.id === pid)?.acoes.find((x) => x.id === fid) || null;
const protoUrl = (x) => `/prototipos/${x.id}.html?v=${encodeURIComponent(x.em || "")}`;
function mostraProto() {
  const f = acaoDe(protoAlvo.pid, protoAlvo.fid), x = f?.prototipos.find((y) => y.id === protoAlvo.id);
  if (!x) return fechaProtos();
  $("proto-acao").textContent = `Protótipo · ${f.nome || "acao"}`;
  $("proto-titulo").textContent = x.titulo;
  const lista = $("proto-lista");
  lista.hidden = f.prototipos.length < 2;
  lista.innerHTML = f.prototipos.map((y) => `<option value="${esc(y.id)}"${y.id === x.id ? " selected" : ""}>${esc(y.titulo)}</option>`).join("");
  $("proto-aba").href = protoUrl(x);
  const quadro = $("proto-quadro");
  if (quadro.dataset.url !== protoUrl(x)) { quadro.dataset.url = protoUrl(x); quadro.src = protoUrl(x); }
}
function abreProtos(pid, fid, id) {
  const f = acaoDe(pid, fid);
  if (!f?.prototipos?.length) return;
  protoAlvo = { pid, fid, id: (f.prototipos.find((y) => y.id === id) || f.prototipos.at(-1)).id };
  protoTela.hidden = false;
  mostraProto();
  $("proto-fechar").focus({ preventScroll: true });
}
function fechaProtos() {
  protoTela.hidden = true;
  protoAlvo = null;
  const quadro = $("proto-quadro");
  quadro.dataset.url = "";
  quadro.src = "about:blank";
}
async function excluirProto() {
  const f = protoAlvo && acaoDe(protoAlvo.pid, protoAlvo.fid), x = f?.prototipos.find((y) => y.id === protoAlvo.id);
  if (!x || !(await confirmar({ titulo: "Excluir protótipo?", texto: `O protótipo "${x.titulo}" será apagado. Não dá para desfazer.`, rotulo: "Excluir protótipo" }))) return;
  f.prototipos = f.prototipos.filter((y) => y.id !== x.id);
  try { await fetch("/api/mcp/prototipo/remover", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ prototipoId: x.id }) }); } catch {}
  mudou();
  if (f.prototipos.length) { protoAlvo.id = f.prototipos.at(-1).id; mostraProto(); } else fechaProtos();
}
$("proto-fechar").onclick = fechaProtos;
$("proto-excluir").onclick = excluirProto;
$("proto-lista").onchange = (e) => { if (protoAlvo) { protoAlvo.id = e.target.value; mostraProto(); } };
addEventListener("keydown", (e) => { if (e.key === "Escape" && !protoTela.hidden) { e.stopImmediatePropagation(); fechaProtos(); } }, true);

