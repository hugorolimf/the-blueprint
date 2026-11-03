// Textos fixos da página de apresentação, em pt-BR e en. A página nunca escreve texto que não esteja aqui ou em `conteudo`.
// `conteudo.idioma` escolhe o dicionário; idioma desconhecido cai no inglês. Chave sem tradução devolve a própria chave (nunca quebra).
(function (L) {
  const TEXTOS = {
    "pt-BR": {
      plano: "Plano de projeto", plano_interno: "Plano de projeto, visão interna", sim: "Simulação com dados de exemplo. O sistema final pode diferir.",
      menu: "Seções", pular: "Ir para o conteúdo", imprimir: "Imprimir",
      visao: "Visão geral", entregas: "O que você recebe", etapas: "Etapas",
      voce: "O que precisamos de você", decisoes: "Decisões e riscos", acoes: "Ações",
      riscos: "Riscos", decisoes_t: "Decisões", aberta: "a definir", decidida: "decidida",
      etapa: "Etapa", etapas_n: "etapas",
      marcos: "Marcos", fig: "Fig", figura_erro: "Esta figura não pôde ser desenhada.", codigo_nao_carregou: "O código desta figura não carregou.", figura_erro_det: "Detalhe",
      st_prevista: "prevista", st_andamento: "em andamento", st_concluida: "concluída",
      nada: "Nada a informar nesta seção.",
      nav_dica: "Teclas: [ e ] trocam de seção, Home e End vão ao começo e ao fim.",
      ref_item: "Aponte um item do texto para vê-lo na figura.", ir_fig: "Ir para a figura {n}",
      nivel: "Nível", publico: "Público", cliente: "Cliente", interno: "Interno",
      nivel_rapido: "Rápido", nivel_padrao: "Padrão", nivel_completo: "Completo", nivel_epico: "Épico",
      sem_figura: "Sem figura nesta etapa.",
    },
    en: {
      plano: "Project plan", plano_interno: "Project plan, internal view", sim: "Simulation with sample data. The final system may differ.",
      menu: "Sections", pular: "Skip to content", imprimir: "Print",
      visao: "Overview", entregas: "What you receive", etapas: "Stages",
      voce: "What we need from you", decisoes: "Decisions and risks", acoes: "Actions",
      riscos: "Risks", decisoes_t: "Decisions", aberta: "to be decided", decidida: "decided",
      etapa: "Stage", etapas_n: "stages",
      marcos: "Milestones", fig: "Fig", figura_erro: "This figure could not be drawn.", codigo_nao_carregou: "This figure's code did not load.", figura_erro_det: "Detail",
      st_prevista: "planned", st_andamento: "in progress", st_concluida: "done",
      nada: "Nothing to report in this section.",
      nav_dica: "Keys: [ and ] switch section, Home and End jump to the start and the end.",
      ref_item: "Point at an item of the text to see it in the figure.", ir_fig: "Go to figure {n}",
      nivel: "Level", publico: "Audience", cliente: "Client", interno: "Internal",
      nivel_rapido: "Quick", nivel_padrao: "Standard", nivel_completo: "Complete", nivel_epico: "Epic",
      sem_figura: "No figure for this stage.",
    },
  };
  const dicionario = (idioma) => TEXTOS[idioma] || TEXTOS[String(idioma || "").startsWith("pt") ? "pt-BR" : "en"];
  // t("ir_fig", "pt-BR", { n: 3 }) -> "Ir para a figura 3"
  L.t = (chave, idioma, parametros) => {
    const texto = dicionario(idioma)[chave] ?? chave;
    return parametros ? texto.replace(/\{(\w+)\}/g, (_, nome) => (parametros[nome] ?? "")) : texto;
  };
  L.idiomaDe = (idioma) => (dicionario(idioma) === TEXTOS["pt-BR"] ? "pt-BR" : "en");
  L.TEXTOS = TEXTOS;
})(L);
