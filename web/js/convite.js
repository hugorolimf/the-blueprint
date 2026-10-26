// ---- entrada pelo link de convite (modo hospedado; docs/colaboracao.md, seção 4) ----------------------------------------
// O segredo vem no fragmento (/convite#<segredo>): o navegador nunca o manda ao servidor, ao log do proxy nem no Referer. A página o lê,
// apaga da barra com replaceState ANTES de qualquer pedido e o manda só no corpo de POST /api/convite/conferir e /entrar. No fim,
// location.replace(destino) troca a página sem deixar o convite no histórico. A mesma tela serve para "sem sessão" (semSessao()).
const segredoDoFragmento = (hash) => { const s = String(hash || "").replace(/^#/, "").trim(); return /^[A-Za-z0-9_-]{20,200}$/.test(s) ? s : ""; };
// só destinos da própria página ("/" para o dono do servidor, /#/c/<ws> ou /#/p/<bp>): uma resposta estranha nunca leva a outro site
const destinoSeguro = (d) => (/^\/#\/[cp]\/[^\s/?#]+$/.test(String(d || "")) ? d : "/");
// nome: 1 a 60 caracteres sem controle (o servidor confere de novo); devolve o código de erro ou ""
const problemaDoNome = (nome) => { const n = String(nome || "").trim(); return !n ? "falta-nome" : n.length > 60 || /[\u0000-\u001f\u007f]/.test(n) ? "nome-invalido" : ""; };

let conviteSegredo = "", conviteInfo = null;
function mostrarEntradaConvite(estado, extra = {}) {
  document.body.classList.add("na-convite");
  document.body.classList.remove("carregando");
  $("convite").hidden = false;
  const info = conviteInfo || {}, papel = info.papel || "", alvo = info.blueprint?.nome || info.workspace?.nome || "";
  const textos = {
    "conferindo": [tr("Convite"), tr("Conferindo o convite…"), ""],
    "sem-segredo": [tr("Convite"), tr("Abra o link completo"), tr("O link de convite termina com # e um código. Abra de novo o link inteiro que você recebeu, nesta janela.")],
    "invalido": [tr("Convite"), tr("Convite indisponível"), extra.erro || textoDoErro("convite-invalido")],
    "nome": [tr("Convite · {papel}", { papel: nomeDoPapel(papel) }), alvo ? tr("Você foi convidado para {nome}.", { nome: alvo }) : tr("Você foi convidado para este servidor."),tr("Como a equipe vai chamar você? O nome aparece para as outras pessoas; não é senha.")],
    "entrando": [tr("Convite"), tr("Entrando…"), ""],
    "sem-sessao": [tr("Servidor da equipe"), tr("Entre pelo link de convite"), tr("Este The Blueprint é de uma equipe. Para entrar, abra o link de convite que você recebeu; se não tem um, peça a quem administra o servidor.")],
  }[estado];
  $("convite-rot").textContent = textos[0];
  $("convite-titulo").textContent = textos[1];
  $("convite-texto").textContent = textos[2];
  $("convite-papel").textContent = estado === "nome" ? explicaPapel(papel) : "";
  $("convite-papel").hidden = estado !== "nome";
  $("convite-campos").hidden = estado !== "nome";
  $("convite-erro").textContent = extra.aviso || "";
  if (estado === "nome") $("convite-nome").focus();
  if (typeof EMB !== "undefined" && EMB && !$("convite-arte").firstChild) $("convite-arte").innerHTML = previaMarca(EMB.BLUEPRINT, 390, 280);
}
async function entradaPeloConvite() {
  conviteSegredo = segredoDoFragmento(location.hash);
  history.replaceState(null, "", "/convite"); // o segredo sai da barra e do histórico antes de qualquer pedido
  emblemasProntos.then(() => { if (!$("convite").hidden && EMB && !$("convite-arte").firstChild) $("convite-arte").innerHTML = previaMarca(EMB.BLUEPRINT, 390, 280); });
  if (!conviteSegredo) return mostrarEntradaConvite("sem-segredo");
  mostrarEntradaConvite("conferindo");
  try { conviteInfo = await equipeApi("/api/convite/conferir", { segredo: conviteSegredo }); }
  catch (e) { conviteSegredo = ""; return mostrarEntradaConvite("invalido", { erro: e.message }); }
  if (conviteInfo.pedeNome) return mostrarEntradaConvite("nome");
  entrarPeloConvite(); // visitante: sem nome, segue direto
}
async function entrarPeloConvite(nome) {
  const botao = $("convite-form").querySelector("button");
  botao.disabled = true;
  if (nome === undefined) mostrarEntradaConvite("entrando");
  try {
    const j = await equipeApi("/api/convite/entrar", nome === undefined ? { segredo: conviteSegredo } : { segredo: conviteSegredo, nome });
    conviteSegredo = "";
    location.replace(destinoSeguro(j.destino)); // URL limpa; o histórico não guarda o convite
  } catch (e) {
    if (e.codigo === "falta-nome" || e.codigo === "nome-invalido" || e.codigo === "muitas-tentativas") mostrarEntradaConvite("nome", { aviso: e.message });
    else { conviteSegredo = ""; mostrarEntradaConvite("invalido", { erro: e.message }); }
  } finally { botao.disabled = false; }
}
// outro link colado na mesma aba em /convite só muda o fragmento (a página não recarrega): confere o novo
addEventListener("hashchange", () => { if (location.pathname === "/convite" && segredoDoFragmento(location.hash)) { conviteInfo = null; entradaPeloConvite(); } });
$("convite-form").onsubmit = (e) => {
  e.preventDefault();
  const nome = $("convite-nome").value.trim(), problema = problemaDoNome(nome);
  if (problema) { $("convite-erro").textContent = textoDoErro(problema, { max: 60 }); $("convite-nome").focus(); return; }
  entrarPeloConvite(nome);
};
