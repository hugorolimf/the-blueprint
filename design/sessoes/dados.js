// Os dados de exemplo do protótipo: nada aqui é real e nada é gravado.
const WORKSPACES = [
  { id: "bp", nome: "The Blueprint", pasta: "~/Work/the-blueprint", blueprint: "Interface nativa", modelo: "Opus 5.5",
    regras: ["Bash(cargo test *)", "Edit em ui/src/**"], acoes: [["2.3", "Tela de sessões"], ["2.4", "Pedido de permissão na janela"], ["3.1", "Revisar o instalador"]] },
  { id: "pad", nome: "Site da padaria", pasta: "~/Work/padaria-site", blueprint: "Loja no ar", modelo: "Sonnet 5.5",
    regras: ["Bash(npm test)"], acoes: [["1.2", "Página de encomendas"], ["1.3", "Fotos dos produtos"]] },
  { id: "est", nome: "Estudos", pasta: "~/Documents/estudos", blueprint: "Rust neste semestre", modelo: "Sonnet 5.5",
    regras: [], acoes: [["1.1", "Exercícios de ownership"]] },
];

const SESSOES = [
  { id: "ab12f0", ws: "bp", acao: "2.4", nome: "Pedido de permissão na janela", estado: "pede", quando: "agora", contexto: 38, itens: [
    { t: "voce", x: "Inicia a ação 2.4: o pedido de permissão aparece na janela, com o comando real." },
    { t: "claude", x: "Vou ver como o servidor responde aos hooks hoje e depois escrevo o teste." },
    { t: "ferr", nome: "Read", alvo: "src/mcp/hooks_estado.rs" },
    { t: "sub", papel: "explorador", titulo: "Explorador: onde a janela lê os agentes", resumo: "2 leituras, terminou", itens: [
      { t: "ferr", nome: "Grep", alvo: "api/mcp/agentes em ui/src" }, { t: "ferr", nome: "Read", alvo: "ui/src/agentes.rs" },
      { t: "claude", x: "A janela lê GET /api/mcp/agentes em ui/src/rede.rs." } ] },
    { t: "ferr", nome: "Edit", alvo: "ui/src/telas/agente.rs" },
    { t: "pedido", ferramenta: "Bash", comando: "cargo test -p blueprint-ui permissao", motivo: "Quer rodar os testes do pedido de permissão." },
  ], roteiro: [{ ferr: ["Bash", "cargo test: 14 passaram"] }, { fala: "Os testes passaram. O cartão de permissão já mostra o comando real e some se o pedido for retirado. Registrei o progresso na ação 2.4." }] },
  { id: "c47d21", ws: "bp", acao: "2.3", nome: "Tela de sessões", estado: "trabalhando", quando: "há 2 min", contexto: 21, itens: [
    { t: "voce", x: "Inicia a ação 2.3 e desenha a lista de sessões em ui/src/telas." },
    { t: "ferr", nome: "Read", alvo: "ui/src/telas/mod.rs" },
    { t: "claude", x: "Vou criar a lista agrupada pelo estado de cada sessão." },
  ], roteiro: [{ ferr: ["Write", "ui/src/telas/sessoes.rs"] }, { ferr: ["Bash", "cargo check: sem erros"] }, { fala: "A lista está pronta e agrupa pelo estado. Quer que eu ligue o clique à conversa agora?" }] },
  { id: "e90b33", ws: "bp", acao: "3.1", nome: "Revisar o instalador", estado: "pronta", quando: "há 25 min", contexto: 54, itens: [
    { t: "voce", x: "Revisa o instalador e me diz o que falta para o Linux." },
    { t: "sub", papel: "revisor", titulo: "Revisor: instalador", resumo: "5 leituras, terminou", itens: [{ t: "ferr", nome: "Read", alvo: "docs/install.md" }, { t: "claude", x: "Falta detectar o claude no PATH do Linux." }] },
    { t: "claude", x: "Faltam duas coisas: detectar o claude no Linux e registrar o plugin sem o instalador do Windows. Faço as duas?" },
  ], roteiro: [] },
  { id: "1fd9a7", ws: "bp", acao: "2.1", nome: "Trocar a porta fixa dos hooks", estado: "encerrada", quando: "ontem", contexto: 62, itens: [
    { t: "voce", x: "Troca a porta 3000 fixa dos hooks pela porta real." },
    { t: "claude", x: "Feito, com evidência na ação 2.1: os hooks usam a porta real, e os testes passaram." },
    { t: "sis", x: "Sessão encerrada. A conversa ficou guardada e pode ser retomada." },
  ], roteiro: [] },
  { id: "77c0be", ws: "bp", acao: null, nome: "Olhando o changelog", estado: "fora", quando: "há 4 min", contexto: null, itens: [
    { t: "sis", x: "Sessão aberta no seu terminal. Daqui, o app sabe só o que os hooks e o MCP avisam." },
    { t: "ferr", nome: "Edit", alvo: "CHANGELOG.md" },
  ], roteiro: [] },
  { id: "b3e118", ws: "pad", acao: "1.2", nome: "Página de encomendas", estado: "pede", quando: "há 1 min", contexto: 17, itens: [
    { t: "voce", x: "Inicia a ação 1.2: página de encomendas com formulário." },
    { t: "claude", x: "Preciso instalar a biblioteca de validação antes do formulário." },
    { t: "pedido", ferramenta: "Bash", comando: "npm install zod", motivo: "Quer instalar um pacote novo." },
  ], roteiro: [{ ferr: ["Bash", "npm install zod: pronto"] }, { ferr: ["Write", "src/encomendas.html"] }, { fala: "A página está pronta. Falta você dizer o telefone que vai aparecer nela." }] },
];

const TEXTO_ESTADO = { pede: "Pede você", trabalhando: "Trabalhando", pronta: "Sua vez", encerrada: "Encerrada", fora: "No terminal" };
const GRUPOS = [["pede", "Pedem você"], ["trabalhando", "Trabalhando"], ["pronta", "Sua vez"], ["fora", "Fora do app"], ["encerrada", "Encerradas"]];
const POSE = { pede: "esperando", trabalhando: "trabalhando", pronta: "parado", encerrada: "concluido", fora: "parado" };
