// A régua usa dias UTC fracionários; a duração real é medida entre os avisos do plano.
const HORAS_PADRAO = 1.5;
function horasDaAcao(acao) {
  let inicio = dia(acao.inicio) * 24;
  return (acao.planos || []).map((plano) => {
    const valor = Number(plano.horas), previsto = Number.isFinite(valor) && valor > 0 ? valor : HORAS_PADRAO;
    const de = Date.parse(plano.agente?.inicioEm), ate = Date.parse(plano.agente?.feitoEm);
    const real = Number.isFinite(de) && Number.isFinite(ate) && ate >= de ? (ate - de) / 3600000 : null;
    const faixa = { id: plano.id, inicio, fim: inicio + previsto, previsto, real };
    inicio = faixa.fim;
    return faixa;
  });
}
function marcasDeHoras(inicio, fim) {
  if (!Number.isFinite(inicio) || !Number.isFinite(fim) || fim <= inicio) return [];
  const marcas = [];
  for (let h = Math.ceil(inicio); h < fim; h++) marcas.push({ hora: h, dia: Math.floor(h / 24), horaDoDia: ((h % 24) + 24) % 24 });
  return marcas;
}
