// Gera ROADMAP.md (inglês), ROADMAP.pt-BR.md e web/js/roadmap-dados.js (o módulo que a tela Roadmap do app importa) a partir de roadmap.json, a fonte única.
//   node scripts/gerar-roadmap.mjs              grava os dois arquivos
//   node scripts/gerar-roadmap.mjs --conferir   sai com 1 se algum estiver diferente do que o JSON gera (roda no CI)
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const dados = JSON.parse(readFileSync(join(raiz, "roadmap.json"), "utf8"));
const ESTADOS = {
  entregue: { ordem: 0, en: "Shipped", pt: "Entregue" },
  agora: { ordem: 1, en: "Doing now", pt: "Fazendo agora" },
  proximo: { ordem: 2, en: "Next", pt: "Em seguida" },
  depois: { ordem: 3, en: "Later", pt: "Depois" },
};
const TEXTOS = {
  en: { titulo: "The Blueprint roadmap", outro: "[Leia em português](ROADMAP.pt-BR.md)", legenda: "How to read it", atualizado: "Updated", ideia: "Have an idea or a vote? [Open an issue](https://github.com/hugorolimf/the-blueprint/issues). Source of this page: `roadmap.json` (the app's Roadmap screen reads the same file)." },
  pt: { titulo: "Roadmap do The Blueprint", outro: "[Read in English](ROADMAP.md)", legenda: "Como ler", atualizado: "Atualizado", ideia: "Tem uma ideia ou um voto? [Abra uma issue](https://github.com/hugorolimf/the-blueprint/issues). Fonte desta página: `roadmap.json` (a tela Roadmap do app lê o mesmo arquivo)." },
};

export function gerar(idioma) {
  const t = TEXTOS[idioma], linhas = [];
  linhas.push(`# ${t.titulo}`, "", `${t.outro} · ${t.atualizado}: ${dados.atualizado}`, "", dados.resumo[idioma], "");
  linhas.push(`## ${t.legenda}`, "", Object.values(ESTADOS).map((e) => e[idioma]).join(" · "), "");
  for (const tema of dados.temas) {
    const itens = dados.itens.filter((i) => i.tema === tema.id).sort((a, b) => ESTADOS[a.estado].ordem - ESTADOS[b.estado].ordem);
    if (!itens.length) continue;
    linhas.push(`## ${tema[idioma]}`, "");
    for (const i of itens) {
      const x = i[idioma];
      linhas.push(`- *${ESTADOS[i.estado][idioma]}* · **${x.titulo}**: ${x.resumo}${x.detalhe ? ` ${x.detalhe}` : ""}`);
    }
    linhas.push("");
  }
  linhas.push("---", "", t.ideia, "");
  return linhas.join("\n");
}

const alvos = [["en", "ROADMAP.md"], ["pt", "ROADMAP.pt-BR.md"], ["js", "web/js/roadmap-dados.js"]];
const modulo = () => `// GERADO por scripts/gerar-roadmap.mjs a partir de roadmap.json: não edite à mão.
export const ROADMAP = ${JSON.stringify(dados)};
`;
if (import.meta.url === `file:///${process.argv[1].replace(/\\/g, "/")}`) {
  const conferir = process.argv.includes("--conferir");
  let erro = false;
  for (const [idioma, arquivo] of alvos) {
    const novo = idioma === "js" ? modulo() : gerar(idioma), caminho = join(raiz, arquivo);
    if (conferir) {
      if (!existsSync(caminho) || readFileSync(caminho, "utf8") !== novo) { console.error(`${arquivo} está diferente do que roadmap.json gera: rode node scripts/gerar-roadmap.mjs`); erro = true; }
    } else { writeFileSync(caminho, novo); console.log(`${arquivo}: ${novo.split("\n").length} linhas`); }
  }
  if (conferir && !erro) console.log("roadmap: os dois arquivos batem com roadmap.json");
  process.exit(erro ? 1 : 0);
}
