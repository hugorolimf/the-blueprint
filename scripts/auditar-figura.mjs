// Auditoria de geometria do anatomy (kit/audit.mjs) sobre a figura do agente. A figura é uma lista de peças (dados), não um build com helpers que
// gravam formas, então este adaptador grava cada peça sólida (laje, cilindro, roda) como a forma que ela desenha e põe cada uma na ORDEM em que o
// desenhaEmblema a pinta; a auditoria então confere se essa ordem de pintura bate com a profundidade de verdade e se nada atravessa nada nem fica no ar.
// Uso: node scripts/auditar-figura.mjs [--papel construtor] [--estado escrevendo] [--posicao 0.5] [--progresso 0.6] [--todas]   (--todas: todas as poses; sai 1 se algo falhar)
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { desenhaEmblema, sanitizaPecas } from "../emblemas.mjs";
import { ESTADOS, PAPEIS, pecasDoAgente } from "../agentes-figura.mjs";

const skill = process.env.ANATOMY_DIR || join(process.env.USERPROFILE || process.env.HOME || "", ".agents", "skills", "anatomy");
const carrega = (arquivo) => import(pathToFileURL(resolve(skill, arquivo)).href);
const [A, k, G] = await Promise.all([carrega("kit/audit.mjs"), carrega("kit/iso-kit.mjs"), carrega("kit/lathe.mjs")]);
const arg = (nome, padrao) => { const i = process.argv.indexOf(`--${nome}`); return i > 0 ? process.argv[i + 1] : padrao; };

// quem é "dono" de quê: peças do mesmo conjunto podem se encostar e se encaixar (um parafuso na placa), peças de conjuntos diferentes não podem se atravessar
const DONOS = { base: "bancada", etiqueta: "bancada", torre: "bancada", mostrador: "bancada", agulha: "bancada", enchimento: "bancada", cartas: "bancada", frente: "bancada", cabo: "cabo",
  alvo: "carro", trava: "carro", carro: "carro", roda: "carro", raios: "carro", cubo: "carro", braco: "carro" };

function auditaPose(pose) {
  const pecas = pecasDoAgente(pose);
  const ps = sanitizaPecas(pecas, 400);
  const e = desenhaEmblema(pecas, { largura: 880, altura: 680, pad: 3, progresso: 1, sombra: false, limite: 400 });
  const pts = [];
  for (const p of ps) {
    if (p.tipo === "laje") pts.push(...k.boxCorners({ x: p.x, y: p.y, w: p.w, d: p.d }, p.z, p.z + p.h));
    else if (p.tipo === "cilindro" || p.tipo === "serrilha") pts.push(...k.boxCorners({ x: p.cx - p.r, y: p.cy - p.r, w: 2 * p.r, d: 2 * p.r }, p.z, p.z + p.h));
    else if (p.tipo === "anel") pts.push([p.cx - p.r, p.cy - p.r, p.z], [p.cx + p.r, p.cy + p.r, p.z]);
    else if (p.tipo === "linha") pts.push([p.x, p.y, p.z], [p.x2, p.y2, p.z]);
    else if (p.tipo === "risco") pts.push([p.x, p.y, p.z], [p.x2, p.y2, p.z2]);
    else if (p.tipo === "quadro") pts.push([p.x, p.y, p.z], [p.plano === "y" ? p.x2 : p.x, p.plano === "y" ? p.y : p.y2, p.z2]);
    else if (p.tipo === "pontos") p.pts.forEach((q) => pts.push(q));
    else if (p.tipo === "roda") pts.push(...(p.eixo === "y" ? [[p.cx - p.r, p.cy - p.w / 2, p.z - p.r], [p.cx + p.r, p.cy + p.w / 2, p.z + p.r]] : [[p.cx - p.w / 2, p.cy - p.r, p.z - p.r], [p.cx + p.w / 2, p.cy + p.r, p.z + p.r]]));
  }
  const P = k.fitProjection(pts, 880, 680, { pad: 3, azimuth: 45 }); // a mesma câmera do desenho
  const R = A.recorder(P, {});
  const posto = new Map(e.ordem.map((i, pos) => [i, pos]));
  let n = 0;
  ps.forEach((p, i) => {
    const dono = DONOS[pecas[i]?.grupo] ?? "bancada";
    const nome = `${pecas[i]?.grupo ?? "?"}.${p.tipo}${i}`;
    const meta = { name: nome, owner: dono };
    let forma = null, centro = null;
    if (p.tipo === "laje") { forma = A.slab({ x: p.x, y: p.y, w: p.w, d: p.d, r: p.r }, p.z, p.h, meta); centro = [p.x + p.w / 2, p.y + p.d / 2, p.z + p.h / 2]; }
    else if (p.tipo === "cilindro") { forma = A.cylinder(p.cx, p.cy, p.r, p.z, p.h, meta); centro = [p.cx, p.cy, p.z + p.h / 2]; }
    else if (p.tipo === "roda") { forma = A.disc(G.frameAlong([p.cx, p.cy, p.z], p.eixo === "y" ? [0, 1, 0] : [1, 0, 0]), -p.w / 2, p.w / 2, p.r, meta); centro = [p.cx, p.cy, p.z]; }
    if (!forma) return; // linhas, anéis, pontos e texto são marcas sobre as faces: quem confere é o lines.mjs
    if (nome.startsWith("base.laje") && p.w === 256) forma.ground = true; // a placa é o chão da bancada
    if (p.tipo === "cilindro" && p.z === 0) forma.ground = true; // os pés
    R.put({ at: centro, key: posto.get(i), name: nome, piece: dono, shapes: [R.solid(forma)] });
    n++;
  });
  const resultado = A.audit(R, P, {});
  return { n, resultado, falhas: A.failures(resultado), relatorio: A.report(resultado) };
}

const poses = process.argv.includes("--todas")
  ? PAPEIS.flatMap((papel) => ESTADOS.flatMap((estado) => [0, 0.5, 1].map((posicao) => ({ papel, estado, posicao, progresso: 0.6 }))))
  : [{ papel: arg("papel", "construtor"), estado: arg("estado", "escrevendo"), posicao: Number(arg("posicao", 0.5)), progresso: Number(arg("progresso", 0.6)) }];
let total = 0;
for (const pose of poses) {
  const r = auditaPose(pose);
  total += r.falhas;
  if (poses.length === 1) console.log(r.relatorio);
  else console.log(`${pose.papel}/${pose.estado}/${pose.posicao}: ${r.n} sólidos, ${r.falhas} falhas`);
}
console.log(total ? `auditoria: ${total} falhas` : "audit passed");
process.exitCode = total ? 1 : 0;
