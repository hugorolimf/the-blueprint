// Grava os vídeos do app (MP4 e GIF) em videos/ (pasta fora do Git). Os dados são sempre os de exemplo, num servidor temporário.
//   node scripts/gravar/index.mjs                 grava todos os roteiros
//   node scripts/gravar/index.mjs 03 04           só os que começam com esses números
//   node scripts/gravar/index.mjs --lista         lista os roteiros
// Precisa do Microsoft Edge, do ffmpeg no PATH e de um blueprint.exe (BLUEPRINT_EXE; padrão target/debug/blueprint.exe).
import { join, resolve } from "node:path";
import { abrirAmbiente, codificar, raiz } from "./nucleo.mjs";
import { ROTEIROS } from "./roteiros.mjs";

const args = process.argv.slice(2);
if (args.includes("--lista")) { for (const [n, r] of Object.entries(ROTEIROS)) console.log(n.padEnd(26), r.descricao); process.exit(0); }
const exe = resolve(process.env.BLUEPRINT_EXE || join(raiz, "target/debug/blueprint.exe"));
const nomes = Object.keys(ROTEIROS).filter((n) => !args.length || args.some((a) => n.startsWith(a)));
let falhas = 0;
for (const nome of nomes) {
  const A = await abrirAmbiente({ exe });
  try {
    console.log(`gravando ${nome}…`);
    let quadros;
    const orig = A.gravar; // o roteiro chama A.gravar uma vez; guardamos o resultado para codificar
    A.gravar = async (f) => (quadros = await orig(f));
    await ROTEIROS[nome].rodar(A);
    const r = codificar(quadros, join(raiz, "videos", nome), { gif: ROTEIROS[nome].gif ? { largura: 800, fps: 12 } : null });
    console.log(`  ${r.quadros} quadros, ${r.duracao} s${A.erros.length ? " | erros no console: " + A.erros.length : ""}`);
  } catch (e) { falhas++; console.error(`  FALHOU: ${e.message}`); } finally { A.fechar(); }
}
process.exit(falhas ? 1 : 0);
