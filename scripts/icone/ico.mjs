// Do SVG ao .ico: o Edge sem janela desenha cada tamanho em PNG com fundo transparente, e empacotarIco junta os PNGs num .ico
// (o Windows aceita PNG dentro do .ico desde o Vista). Usado por desenhar.mjs.
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

/** [{ tam, png }] → bytes de um .ico com uma imagem por tamanho, na ordem recebida. */
export function empacotarIco(pngs) {
  const cabeca = Buffer.alloc(6 + 16 * pngs.length);
  cabeca.writeUInt16LE(0, 0);
  cabeca.writeUInt16LE(1, 2); // 1 = ícone
  cabeca.writeUInt16LE(pngs.length, 4);
  let pos = cabeca.length;
  pngs.forEach(({ tam, png }, i) => {
    const e = 6 + 16 * i;
    cabeca.writeUInt8(tam >= 256 ? 0 : tam, e); // 0 quer dizer 256
    cabeca.writeUInt8(tam >= 256 ? 0 : tam, e + 1);
    cabeca.writeUInt16LE(1, e + 4); // planos
    cabeca.writeUInt16LE(32, e + 6); // bits por pixel
    cabeca.writeUInt32LE(png.length, e + 8);
    cabeca.writeUInt32LE(pos, e + 12);
    pos += png.length;
  });
  return Buffer.concat([cabeca, ...pngs.map((p) => p.png)]);
}

/** [[svg, tam]] → [{ tam, png }], cada SVG desenhado em tam × tam px sobre fundo transparente. */
export async function rasterizar(lista, { pixels = false } = {}) {
  const pasta = mkdtempSync(join(tmpdir(), "icone-"));
  const imgs = lista.map(([svg, tam], i) => `<img id="i${i}" width="${tam}" height="${tam}" style="position:absolute;left:0;top:${i * 260}px" src="data:image/svg+xml;base64,${Buffer.from(svg).toString("base64")}">`);
  const pagina = join(pasta, "icone.html");
  writeFileSync(pagina, `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:transparent}</style>${imgs.join("")}`);
  const porta = await new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
  const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
  if (!edgeExe) throw new Error("Edge não encontrado: ele desenha os PNGs do ícone");
  const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${porta}`, `--user-data-dir=${join(pasta, "perfil")}`, "about:blank"], { stdio: "ignore" });
  const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
  try {
    let alvo;
    for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((t) => t.type === "page"); } catch {} if (!alvo) await dormir(100); }
    if (!alvo) throw new Error("o Edge não abriu a porta de depuração");
    const ws = new WebSocket(alvo.webSocketDebuggerUrl);
    await new Promise((r) => (ws.onopen = r));
    let n = 0; const pend = new Map();
    ws.onmessage = (m) => { const d = JSON.parse(m.data); pend.get(d.id)?.(d); pend.delete(d.id); };
    const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
    await cdp("Emulation.setDeviceMetricsOverride", { width: 300, height: lista.length * 260, deviceScaleFactor: 1, mobile: false });
    await cdp("Emulation.setDefaultBackgroundColorOverride", { color: { r: 0, g: 0, b: 0, a: 0 } });
    await cdp("Page.navigate", { url: pathToFileURL(pagina).href });
    for (let i = 0; i < 50; i++) { // espera todas as imagens decodificarem
      const r = await cdp("Runtime.evaluate", { expression: "[...document.images].length > 0 && [...document.images].every((i) => i.complete && i.naturalWidth > 0)", returnByValue: true });
      if (r.result?.result?.value) break;
      await dormir(100);
    }
    await dormir(200);
    const saida = [];
    for (const [i, [, tam]] of lista.entries()) {
      const r = await cdp("Page.captureScreenshot", { format: "png", clip: { x: 0, y: i * 260, width: tam, height: tam, scale: 1 } });
      const imagem = { tam, png: Buffer.from(r.result.data, "base64") };
      if (pixels) {
        const p = await cdp("Runtime.evaluate", { expression: `(() => { const c = document.createElement('canvas'); c.width = c.height = ${tam}; const x = c.getContext('2d'); x.drawImage(document.getElementById('i${i}'), 0, 0, ${tam}, ${tam}); return Array.from(x.getImageData(0, 0, ${tam}, ${tam}).data); })()`, returnByValue: true });
        imagem.rgba = Buffer.from(p.result.result.value);
      }
      saida.push(imagem);
    }
    ws.close();
    return saida;
  } finally {
    edge.kill();
    await dormir(300);
    try { rmSync(pasta, { recursive: true, force: true }); } catch {} // o Edge pode ainda segurar o perfil por um instante
  }
}
