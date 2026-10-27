// Textos fixos da página de apresentação, em pt-BR e en. A página nunca escreve texto que não esteja aqui ou em `conteudo`.
// `conteudo.idioma` escolhe o dicionário; idioma desconhecido cai no inglês. Chave sem tradução devolve a própria chave (nunca quebra).
(function (L) {
  const TEXTOS = {
    "pt-BR": {
      plano: "Plano de projeto", plano_interno: "Plano de projeto, visão interna", sim: "Simulação com dados de exemplo. O sistema final pode diferir.",
      menu: "Seções", pular: "Ir para o conteúdo", imprimir: "Imprimir",
      visao: "Visão geral", entregas: "O que você recebe", etapas: "Etapas", linha: "Linha do tempo",
      voce: "O que precisamos de você", decisoes: "Decisões e riscos", acoes: "Ações",
      riscos: "Riscos", decisoes_t: "Decisões", aberta: "a definir", decidida: "decidida", ate: "até",
      periodo: "Período", etapa: "Etapa", etapas_n: "etapas", dias: "dias", dia_de: "Dia {n} de {total}",
      marcos: "Marcos", fig: "Fig", figura_erro: "Esta figura não pôde ser desenhada.", codigo_nao_carregou: "O código desta figura não carregou.", figura_erro_det: "Detalhe",
      st_prevista: "prevista", st_andamento: "em andamento", st_concluida: "concluída",
      comeca: "começa em {d}", hoje: "hoje", nada: "Nada a informar nesta seção.",
      nav_dica: "Teclas: [ e ] trocam de seção, Home e End vão ao começo e ao fim.",
      gerado: "Gerado em {d}", nivel: "Nível", publico: "Público", cliente: "Cliente", interno: "Interno",
      nivel_rapido: "Rápido", nivel_padrao: "Padrão", nivel_completo: "Completo", nivel_epico: "Épico",
      inicio: "Início", fim: "Fim", progresso: "Dia do plano", sem_figura: "Sem figura nesta etapa.",
    },
    en: {
      plano: "Project plan", plano_interno: "Project plan, internal view", sim: "Simulation with sample data. The final system may differ.",
      menu: "Sections", pular: "Skip to content", imprimir: "Print",
      visao: "Overview", entregas: "What you receive", etapas: "Stages", linha: "Timeline",
      voce: "What we need from you", decisoes: "Decisions and risks", acoes: "Actions",
      riscos: "Risks", decisoes_t: "Decisions", aberta: "to be decided", decidida: "decided", ate: "by",
      periodo: "Period", etapa: "Stage", etapas_n: "stages", dias: "days", dia_de: "Day {n} of {total}",
      marcos: "Milestones", fig: "Fig", figura_erro: "This figure could not be drawn.", codigo_nao_carregou: "This figure's code did not load.", figura_erro_det: "Detail",
      st_prevista: "planned", st_andamento: "in progress", st_concluida: "done",
      comeca: "starts {d}", hoje: "today", nada: "Nothing to report in this section.",
      nav_dica: "Keys: [ and ] switch section, Home and End jump to the start and the end.",
      gerado: "Generated {d}", nivel: "Level", publico: "Audience", cliente: "Client", interno: "Internal",
      nivel_rapido: "Quick", nivel_padrao: "Standard", nivel_completo: "Complete", nivel_epico: "Epic",
      inicio: "Start", fim: "End", progresso: "Plan day", sem_figura: "No figure for this stage.",
    },
  };
  const dicionario = (idioma) => TEXTOS[idioma] || TEXTOS[String(idioma || "").startsWith("pt") ? "pt-BR" : "en"];
  // t("dia_de", "pt-BR", { n: 3, total: 20 }) -> "Dia 3 de 20"
  L.t = (chave, idioma, parametros) => {
    const texto = dicionario(idioma)[chave] ?? chave;
    return parametros ? texto.replace(/\{(\w+)\}/g, (_, nome) => (parametros[nome] ?? "")) : texto;
  };
  L.idiomaDe = (idioma) => (dicionario(idioma) === TEXTOS["pt-BR"] ? "pt-BR" : "en");
  // yyyy-mm-dd -> "26 out" / "Oct 26", em UTC para a data não andar com o fuso
  L.dataCurta = (iso, idioma) => {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(String(iso || ""))) return "";
    const d = new Date(iso + "T00:00:00Z");
    return new Intl.DateTimeFormat(L.idiomaDe(idioma), { day: "numeric", month: "short", timeZone: "UTC" }).format(d).replace(/\.$/, "");
  };
  L.TEXTOS = TEXTOS;
})(L);
