// Gera os arquivos de uma release que o updater do app enxerga: `Blueprint-<versão>-instalador.exe` e `SHA256SUMS`, em target/release-arquivos/.
// A versão vem do Cargo.toml (a mesma que o app mostra e compara); o instalador a recebe por ISCC /DVersao. O CI (.github/workflows/release.yml) roda este script.
// Uso: node scripts/gerar-release.mjs [--sem-build]      (--sem-build: usa o target/release/blueprint.exe que já existe; leva ~1 a 3 min com build)
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, copyFileSync } from "node:fs";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const versao = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync(join(raiz, "Cargo.toml"), "utf8"))?.[1];
if (!versao) throw new Error("não achei a versão no Cargo.toml");
const iscc = [process.env.ISCC, join(process.env.LOCALAPPDATA || "", "Programs/Inno Setup 6/ISCC.exe"), "C:/Program Files (x86)/Inno Setup 6/ISCC.exe", "ISCC"].find((c) => c && (c === "ISCC" || existsSync(c)));
const run = (cmd, args) => execFileSync(cmd, args, { cwd: raiz, stdio: "inherit" });

if (!process.argv.includes("--sem-build")) run("cargo", ["build", "--release", "--locked"]);
const nome = `Blueprint-${versao}-instalador.exe`, saida = join(raiz, "target", "release-arquivos");
rmSync(saida, { recursive: true, force: true });
mkdirSync(saida, { recursive: true });
run(iscc, [`/DVersao=${versao}`, "/Qp", join("instalador", "blueprint.iss")]);
copyFileSync(join(raiz, "target", "instalador", nome), join(saida, nome));
const hash = createHash("sha256").update(readFileSync(join(saida, nome))).digest("hex");
writeFileSync(join(saida, "SHA256SUMS"), `${hash}  ${nome}\n`); // o formato do `sha256sum`: é o que o updater lê
console.log(`\nversão ${versao}\n${nome}\nSHA-256 ${hash}\narquivos em ${saida}`);
