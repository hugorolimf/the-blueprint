// Gera exemplo/figuras.json: os metadados de cada figura (texto para o cartão) mais o código-fonte de exemplo/figuras/*.js como `codigo`.
// A impressão de cada figura é a do conteúdo que ela explica (aqui, o conteúdo inteiro), calculada pela mesma L.impressao da página.
// Uso: node apresentacao-ia/exemplo/gerar.mjs   (rode antes node apresentacao-ia/empacotar.mjs)
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const AQUI = dirname(fileURLToPath(import.meta.url));
const codigo = (nome) => readFileSync(join(AQUI, "figuras", nome + ".js"), "utf8").trim();
const conteudo = JSON.parse(readFileSync(join(AQUI, "conteudo.json"), "utf8"));
const contexto = vm.createContext({ Intl });
vm.runInContext(readFileSync(join(AQUI, "..", "dist", "biblioteca.js"), "utf8") + "\nglobalThis.__L = L;", contexto);

const figuras = [
  {
    id: "capa",
    titulo: "Fichário de cadastro",
    dica: "Passe o mouse para escolher uma ficha",
    leitura: "ficha 01 · pendente · 1 em aberto",
    chaves: [
      { marca: "lit", rotulo: "A ficha escolhida sobe e acende" },
      { marca: "raised", rotulo: "Quatorze fichas; a aba mostra a situação" },
    ],
    legenda: "Um fichário com um leitor no trilho da frente. Dados de exemplo: a ficha que sobe mostra, pela aba, se o cadastro está em dia, pendente ou atrasado.",
    rotulo: "Um fichário de quatorze fichas com um leitor que desliza e levanta a ficha escolhida",
    impressao: contexto.__L.impressao(conteudo),
    codigo: codigo("fichario"),
  },
];
writeFileSync(join(AQUI, "figuras.json"), JSON.stringify(figuras, null, 2) + "\n");
console.log("figuras.json:", figuras.length, "figura(s) para", conteudo.titulo);
