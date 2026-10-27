// ---- apresentação com IA, editor manual: o salvador (sem DOM) ----
// Junta as operações do editor num lote, manda para POST /api/mcp/apresentacao/editar depois de uma pausa, guarda a pilha do Desfazer e diz o estado (salvo, pendente,
// salvando, erro). As rotas entram por `postar(corpo) -> { status, json }` e o relógio por `esperar(ms, fn) -> cancela`, para o teste rodar sem rede nem tempo.
// Uma operação recusada pelo servidor (400/404/413) sai do lote e o editor volta ao último salvo; um erro de rede deixa o lote para a próxima tentativa.
function apIaSalvador({ ids, postar, esperar, atraso = 700, aoEstado = () => {}, aoAvisos = () => {}, aoRecusa = () => {} }) {
  let pendentes = [], estado = "salvo", cancela = null, enviando = false, outraVez = false, ultimoEm = "";
  const desfazer = []; // { inversos, caminho, em }: um item por mudança do usuário (digitar no mesmo campo junta)
  const mudaEstado = (e) => { estado = e; aoEstado(e); };

  async function descarrega() {
    cancela?.(); cancela = null;
    if (enviando) { outraVez = true; return; }
    if (!pendentes.length) return;
    const lote = apOpsJuntar(pendentes);
    pendentes = [];
    enviando = true;
    mudaEstado("salvando");
    try {
      for (const corpo of apOpsCorpos(ids, lote)) {
        const r = await postar(corpo);
        if (r.status !== 200 && !r.json?.ok) { aoRecusa(r.status, r.json || {}); enviando = false; mudaEstado("erro"); return; }
        ultimoEm = r.json.editadoEm || ultimoEm;
        aoAvisos(Array.isArray(r.json.avisos) ? r.json.avisos : []);
      }
      enviando = false;
      mudaEstado(pendentes.length ? "pendente" : "salvo");
    } catch {
      pendentes = lote.concat(pendentes); // a rede caiu: o lote volta e segue na próxima tentativa
      enviando = false;
      mudaEstado("erro");
      return;
    }
    if (outraVez || pendentes.length) { outraVez = false; descarrega(); }
  }
  function agenda() { cancela?.(); cancela = esperar(atraso, descarrega); mudaEstado("pendente"); }

  return {
    // uma mudança do usuário: as operações e as que a desfazem (já na ordem de desfazer); `junta` = continua a digitação do mesmo campo
    mudar(ops, inversos, { caminho = "", junta = false, agora = Date.now() } = {}) {
      pendentes.push(...ops);
      const topo = desfazer[desfazer.length - 1];
      if (junta && topo && topo.caminho === caminho && agora - topo.em < 1500) topo.em = agora; // o inverso que vale é o do começo da digitação
      else { desfazer.push({ inversos, caminho, em: agora }); if (desfazer.length > 50) desfazer.shift(); }
      agenda();
    },
    // as operações que desfazem a última mudança (o editor as aplica à cópia local); elas também vão ao servidor
    desfazer() {
      const t = desfazer.pop();
      if (!t) return null;
      pendentes.push(...t.inversos);
      agenda();
      return t.inversos;
    },
    pode: () => desfazer.length > 0,
    descarregar: descarrega,
    pendentes: () => pendentes.slice(),
    ocupado: () => enviando || pendentes.length > 0,
    descartar() { pendentes = []; desfazer.length = 0; cancela?.(); cancela = null; mudaEstado("salvo"); },
    ultimoEm: () => ultimoEm,
    estado: () => estado,
  };
}
