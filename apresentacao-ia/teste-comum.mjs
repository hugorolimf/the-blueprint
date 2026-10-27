// Ajuda dos testes: carrega dist/biblioteca.js num contexto isolado (sem navegador) e devolve L, o exemplo e as figuras com `codigo` já como função.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

export const AQUI = dirname(fileURLToPath(import.meta.url));
export const lerJson = (...p) => JSON.parse(readFileSync(join(AQUI, ...p), "utf8"));
export function carregar() {
  const contexto = vm.createContext({ Intl, console, setTimeout, clearTimeout, CSS: { escape: (s) => String(s) } });
  vm.runInContext(readFileSync(join(AQUI, "dist", "biblioteca.js"), "utf8") + "\nglobalThis.__L = L;", contexto);
  const L = contexto.__L;
  const comFuncao = (figuras) => figuras.map((f) => ({ ...f, codigo: vm.runInContext("(" + f.codigo + ")", contexto) }));
  return { L, comFuncao };
}
export const exemplo = () => ({ conteudo: lerJson("exemplo", "conteudo.json"), figuras: lerJson("exemplo", "figuras.json") });
