// ---- apresentação com IA, editor manual: o esquema (dados, sem DOM): as seções do painel, os campos de cada uma e o item novo de cada lista ----
// Contrato em docs/interno/apresentacao-ia.md ("Data model" e "Manual editing"). O editor (apresentacao-ia-editor.js) desenha tudo a partir daqui, então um campo novo do
// contrato entra numa linha. Campo = [chave, tipo, obrigatório?, opções?]; tipos: texto, area, data, opcoes, figura (as figuras do documento). Rótulos por tr().
const AP_IA_SECOES_FIGURA = ["visao", "recebe", "etapas", "linha", "precisamos", "decisoes", "riscos", "acoes"]; // os ids de `figuras[].secao` do contrato
const AP_IA_SITUACOES = ["prevista", "andamento", "concluida"];
const AP_IA_NOVO = {
  resumo: () => "",
  entregas: () => ({ titulo: "", texto: "" }),
  etapas: (doc) => { const ids = new Set((doc.conteudo.etapas || []).map((e) => e.id)); let n = ids.size + 1; while (ids.has("e" + n)) n++; return { id: "e" + n, titulo: "", inicio: "", fim: "", texto: "", figura: null, marcos: [] }; },
  marcos: () => ({ data: "", nome: "" }),
  precisamosDeVoce: () => ({ titulo: "", texto: "", ate: null }),
  decisoes: () => ({ titulo: "", texto: "", estado: "aberta" }),
  riscos: () => ({ titulo: "", texto: "" }),
  acoes: () => ({ numero: "", nome: "", planos: [] }),
  planos: () => "",
  chaves: () => ({ marca: "raised", rotulo: "" }),
};

// as listas do `conteudo`: caminho, campos do item e como o item aparece no título do bloco; `sub` = listas dentro do item
const apIaListas = () => ({
  resumo: { caminho: "resumo", simples: true, titulo: tr("Resumo"), item: tr("Frase"), novo: tr("Adicionar frase") },
  entregas: { caminho: "entregas", titulo: tr("O que você recebe"), item: tr("Entrega"), novo: tr("Adicionar entrega"), campos: [["titulo", "texto"], ["texto", "area"]], marca: "titulo" },
  etapas: { caminho: "etapas", titulo: tr("Etapas"), item: tr("Etapa"), novo: tr("Adicionar etapa"), marca: "titulo",
    campos: [["titulo", "texto"], ["inicio", "data"], ["fim", "data"], ["situacao", "opcoes", false, AP_IA_SITUACOES], ["figura", "figura"], ["texto", "area"]],
    sub: [{ chave: "marcos", titulo: tr("Marcos"), item: tr("Marco"), novo: tr("Adicionar marco"), campos: [["data", "data"], ["nome", "texto"]], marca: "nome" }] },
  precisamosDeVoce: { caminho: "precisamosDeVoce", titulo: tr("O que precisamos de você"), item: tr("Pedido"), novo: tr("Adicionar pedido"), campos: [["titulo", "texto"], ["ate", "data"], ["texto", "area"]], marca: "titulo" },
  decisoes: { caminho: "decisoes", titulo: tr("Decisões"), item: tr("Decisão"), novo: tr("Adicionar decisão"), campos: [["titulo", "texto"], ["estado", "opcoes", false, ["aberta", "decidida"]], ["texto", "area"]], marca: "titulo" },
  riscos: { caminho: "riscos", titulo: tr("Riscos"), item: tr("Risco"), novo: tr("Adicionar risco"), campos: [["titulo", "texto"], ["texto", "area"]], marca: "titulo" },
  acoes: { caminho: "interno.acoes", interno: true, titulo: tr("Ações (só na visão interna)"), item: tr("Ação"), novo: tr("Adicionar ação"), campos: [["numero", "texto"], ["nome", "texto"]], marca: "nome",
    sub: [{ chave: "planos", titulo: tr("Planos"), item: tr("Plano"), novo: tr("Adicionar plano"), simples: true }] },
});

// os campos soltos da seção Geral
const apIaGeral = () => [
  ["titulo", "texto", true], ["subtitulo", "texto"], ["cliente", "texto"], ["data", "data"],
  ["idioma", "opcoes", false, ["pt-BR", "en"]], ["publico", "opcoes", false, AP_IA_PUBLICOS], ["nivel", "opcoes", false, AP_IA_NIVEIS.map((n) => n.id)],
];

// os campos de texto de cada figura (o `id` é a chave das referências: não se edita) e a `secao` (lista fechada)
const apIaCamposFigura = () => [["titulo", "texto"], ["secao", "secao"], ["dica", "texto"], ["leitura", "texto"], ["legenda", "area"], ["rotulo", "area"]];

const apIaRotulosCampo = () => ({
  titulo: tr("Título"), subtitulo: tr("Subtítulo"), cliente: tr("Cliente"), data: tr("Data"), idioma: tr("Idioma"), publico: tr("Público"), nivel: tr("Nível"),
  texto: tr("Texto"), inicio: tr("Início"), fim: tr("Fim"), situacao: tr("Situação"), figura: tr("Figura"), ate: tr("Até"), estado: tr("Estado"), nome: tr("Nome"), numero: tr("Número"),
  dica: tr("Dica"), leitura: tr("Leitura inicial"), legenda: tr("Legenda"), rotulo: tr("Descrição para leitor de tela"), secao: tr("Sessão"), marca: tr("Marca"),
});
const apIaRotulosOpcao = () => ({
  "pt-BR": tr("Português"), en: tr("Inglês"), cliente: tr("Cliente"), interno: tr("Interno"), rapido: tr("Rápido"), padrao: tr("Padrão"), completo: tr("Completo"),
  prevista: tr("Prevista"), andamento: tr("Em andamento"), concluida: tr("Concluída"), aberta: tr("Aberta"), decidida: tr("Decidida"),
  visao: tr("Visão geral"), recebe: tr("O que você recebe"), etapas: tr("Etapas"), linha: tr("Linha do tempo"), precisamos: tr("O que precisamos de você"), decisoes: tr("Decisões e riscos: decisões"), riscos: tr("Decisões e riscos: riscos"), acoes: tr("Ações"),
  lit: tr("Acesa"), raised: tr("Em relevo"), flat: tr("Plana"),
});
const AP_IA_MARCAS = ["lit", "raised", "flat"];
