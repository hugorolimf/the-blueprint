// Lista as DLLs que o blueprint.exe importa (lendo a tabela de importação do PE), para provar que só dependem do próprio Windows.
// Uso: node scripts/dlls-do-exe.mjs [caminho do blueprint.exe]
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
const exe = resolve(process.argv[2] || join(import.meta.dirname, "..", "target/release/blueprint.exe"));
const b = readFileSync(exe);
const pe = b.readUInt32LE(0x3c), nSecoes = b.readUInt16LE(pe + 6), tamOpt = b.readUInt16LE(pe + 20), opt = pe + 24;
const dir = b.readUInt32LE(opt + (b.readUInt16LE(opt) === 0x20b ? 112 : 96) + 8); // diretório de importação (índice 1)
const secoes = Array.from({ length: nSecoes }, (_, i) => { const s = opt + tamOpt + 40 * i; return { va: b.readUInt32LE(s + 12), tam: b.readUInt32LE(s + 8), bruto: b.readUInt32LE(s + 20) }; });
const rva = (r) => { const s = secoes.find((x) => r >= x.va && r < x.va + Math.max(x.tam, 1)); return s ? r - s.va + s.bruto : -1; };
const nomes = [];
for (let o = rva(dir); ; o += 20) { const nome = b.readUInt32LE(o + 12); if (!nome) break; const p = rva(nome); nomes.push(b.toString("latin1", p, b.indexOf(0, p)).toLowerCase()); }
nomes.sort();
const do_sistema = (n) => /^(api-ms-win-|kernel32|user32|gdi32|advapi32|shell32|ole32|oleaut32|ntdll|bcrypt|crypt32|comctl32|comdlg32|dwmapi|shlwapi|uxtheme|ws2_32|userenv|propsys|version|winmm|combase|rpcrt4|secur32|ncrypt|imm32|msvcrt|dxgi|d3d11|setupapi|cfgmgr32|shcore|netapi32|mswsock|iphlpapi|wintrust|win32u|windows\.ui)/.test(n);
console.log(nomes.join("\n"));
const fora = nomes.filter((n) => !do_sistema(n));
console.log(`\n${nomes.length} DLLs importadas; fora do Windows: ${fora.length ? fora.join(", ") : "nenhuma"}${nomes.some((n) => /^(vcruntime|msvcp|ucrtbase|concrt)/.test(n)) ? " (ATENÇÃO: depende do runtime do Visual C++)" : ""}`);
process.exitCode = fora.length || nomes.some((n) => /^(vcruntime|msvcp|concrt)/.test(n)) ? 1 : 0;
