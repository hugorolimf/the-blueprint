// ---- aplicar sozinho: a cola da página (regras em auto-aplicar-logica.js; tela em auto-aplicar-vista.js) ----
// mcp.js chama autoElegivel / autoAplicaProposta / autoAplicaRegistro quando um item do agente chega ao workspace aberto. A aplicação usa o MESMO caminho do clique
// (aplicarAberta em proposta.js; registroAdiciona e registroRetira em registros.js), então a gravação, o ack e a fila são os de sempre: aqui só se decide, se
// guarda um retrato do blueprint (para o Desfazer) e se registra o que entrou sozinho. Quem grava continua sendo a página.
const CHAVE_AUTO_RASTRO = "blueprint.autoAplicados";
const autoMem = new Map(); // a escolha do interruptor quando o localStorage não deixa guardar (vale até recarregar)
let autoLista = []; // o que entrou sozinho: { chave, ws, blueprintId, em, texto, tipo, ops?, antes?, desfeito? }; ops e antes só existem nesta sessão (o Desfazer é de 60 s)
try { autoLista = JSON.parse(localStorage.getItem(CHAVE_AUTO_RASTRO) || "[]").filter((e) => e && typeof e.chave === "string"); } catch { autoLista = []; }
const autoGuardaRastro = () => { try { localStorage.setItem(CHAVE_AUTO_RASTRO, JSON.stringify(autoRastroGuardavel(autoLista))); } catch {} };

// o interruptor deste blueprint neste workspace: { pode, ligado }. Local: ligado até o usuário desligar. Hospedado: desligado, e só o dono pode ligar.
function autoEstadoDe(blueprintId) {
  const ws = workspaceAtual?.id || "sm", k = autoChaveBlueprint(ws, blueprintId), host = typeof hospedado === "function" && hospedado();
  let guardado = autoMem.get(k) ?? null;
  try { guardado = localStorage.getItem(k) ?? guardado; } catch {}
  return autoEstado({ hospedado: host, papel: host ? papelNoEscopo(sessao.acessos, ws, blueprintId) : "", soLeitura: host && poderesDaSessao(sessao).soLeitura, guardado });
}
function autoDefine(blueprintId, ligado) {
  const k = autoChaveBlueprint(workspaceAtual?.id || "sm", blueprintId);
  autoMem.set(k, ligado ? "1" : "0");
  try { localStorage.setItem(k, ligado ? "1" : "0"); } catch {}
}

const autoJaAplicado = (i) => autoRastroTem(autoLista, autoChaveItem(i));
// o servidor mandou de novo algo que esta página já aplicou sozinha (o ack se perdeu): só confirma de novo, nunca aplica duas vezes
function autoAck(i) {
  const alvo = { workspaceId: i.workspaceId, blueprintId: i.blueprintId, numero: i.numero, criadaEm: i.criadaEm };
  const corpo = i.tipo === "registro" ? { ids: [], registros: [alvo] } : { ids: [], propostas: [alvo] };
  return fetch("/api/mcp/fila/ack", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) }).catch(() => {});
}
// o item entra sozinho? (interruptor ligado, nada de remover ou mover, blueprint existente). Quem está digitando no Planejar deste blueprint não tem a tela trocada: espera o clique.
function autoElegivel(i) {
  if (autoJaAplicado(i)) return false;
  const existe = D.blueprints.some((x) => x.id === i.blueprintId);
  if (!autoDecide(i, { ligado: existe && autoEstadoDe(i.blueprintId).ligado, existe }).aplica) return false;
  return !(!hub.hidden && edProj === i.blueprintId && typeof digitando === "function" && digitando());
}

// o que fica guardado de uma aplicação: o texto, as operações para o Desfazer e o retrato de antes
function autoRegistra(i, p, antes, texto) {
  const e = { chave: autoChaveItem(i), ws: i.workspaceId, blueprintId: p.id, em: Date.now(), texto, tipo: i.tipo, ops: autoOpsDe(antes, p), antes, desfeito: false };
  autoLista = autoRastroAdiciona(autoLista, e);
  autoGuardaRastro();
  aviso(texto);
  autoMostra(e); // o aviso com Desfazer e a lista no Planejar (auto-aplicar-vista.js)
  return e;
}

// A proposta que o agente enviou (já na fila desta página, por mcpProposta) entra no blueprint. Falso: nada mudou e ela espera o clique como sempre.
function autoAplicaProposta(i) {
  const p = D.blueprints.find((x) => x.id === i.blueprintId), c = p && conv(p.id), e = c && filaTem(c, i);
  if (!e || autoPrecisaConfirmar(e) || e.dadosBlueprint) return false;
  if (e.numero !== c.numero && !filaAbre(c, e.numero)) return false; // vira a aberta (a que o usuário estava vendo vai para a fila, com as edições dele)
  const antes = autoEnvolve(p);
  let r;
  try { r = aplicarAberta(p, c); } catch { autoRestaura(p, antes); return false; } // falhou no meio: o blueprint volta ao que era e a proposta fica esperando
  avisarMonitor("proposta-aplicada", p.id, { numero: i.numero, origem: "automatica" });
  autoRegistra(i, p, antes, r.historico);
  if (!hub.hidden) montaHub(); // o mcp.js refaz a folha e grava (mudarComSaida); o Planejar aberto mostra a próxima da fila
  return true;
}

// O registro do que o agente já fez (já em registrosFila, por registroRecebe) entra como entra no "Adicionar ao blueprint": o feito e a evidência viram o que sempre viram.
function autoAplicaRegistro(i) {
  const p = D.blueprints.find((x) => x.id === i.blueprintId), r = registrosFila.find((x) => registroChave(x) === `${i.blueprintId}|${i.numero}|${i.criadaEm}`);
  if (!p || !r || !r.acoes.some((a) => a.inc)) return false;
  const antes = autoEnvolve(p);
  let res;
  try { res = registroAdiciona(r); } catch { autoRestaura(p, antes); return false; }
  if (!res) return false;
  registroRetira(r); // sai da lista e da fila do servidor
  avisarMonitor("registro-adicionado", p.id, { numero: i.numero, origem: "automatica" });
  mudou(); // este caminho não passa pelo mudarComSaida do mcp.js
  autoRegistra(i, p, antes, textoRegistroAdicionado(res));
  if (!hub.hidden) montaHub();
  return true;
}

// Desfazer o que entrou sozinho, dentro de 60 s e uma vez: inverte só as operações da aplicação e só onde o valor ainda é o que ela deixou.
// O que mexeram depois (outra janela, o agente) fica, e a página diz isso em vez de adivinhar. O item não volta à fila do servidor.
function autoDesfaz(chave) {
  const e = autoLista.find((x) => x.chave === chave);
  if (!autoPodeDesfazer(e, Date.now())) return aviso(tr("O Desfazer vale só por 60 segundos, e uma vez."));
  const { feitas, conflitos } = autoDesfazOps(e.ops, e.antes, D);
  if (feitas) { e.desfeito = true; mudou(); if (!hub.hidden) montaHub(); }
  aviso(!feitas ? tr("Não dá para desfazer com segurança: o blueprint mudou depois. Edite à mão.") : conflitos ? tr("Desfeito em parte: {n} mudança(s) ficaram porque foram mexidas depois.", { n: conflitos }) : tr("Desfeito. O item não volta à fila: peça ao agente para enviar de novo, se ainda quiser."));
  autoMostra(e);
}
