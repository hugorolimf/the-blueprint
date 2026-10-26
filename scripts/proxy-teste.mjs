// Um proxy reverso de teste no lugar do Caddy: recebe em http://127.0.0.1:<porta> e repassa ao `blueprint serve --hospedar` como o proxy
// de verdade faria depois de terminar o TLS: Host público, X-Forwarded-Proto: https e X-Forwarded-For com o IP de quem pediu. Repassa a
// resposta em fluxo (o SSE funciona). Só para testes: não tem TLS.
// Uso: node scripts/proxy-teste.mjs <alvo http://127.0.0.1:PORTA> <host público> [porta]   ou   import { ligarProxy } from "./proxy-teste.mjs"
import { createServer, request } from "node:http";

/** Liga o proxy; devolve { url, fechar }. */
export function ligarProxy({ alvo, host, porta = 0 }) {
  const destino = new URL(alvo);
  const servidor = createServer((req, res) => {
    const cabecalhos = { ...req.headers, host, "x-forwarded-proto": "https", "x-forwarded-for": req.socket.remoteAddress.replace(/^::ffff:/, "") };
    const ida = request({ hostname: destino.hostname, port: destino.port, path: req.url, method: req.method, headers: cabecalhos }, (volta) => {
      res.writeHead(volta.statusCode, volta.headers);
      volta.pipe(res);
    });
    ida.on("error", () => { res.writeHead(502); res.end("proxy: o servidor não respondeu"); });
    req.pipe(ida);
  });
  return new Promise((ok) => servidor.listen(porta, "127.0.0.1", () => ok({ url: `http://127.0.0.1:${servidor.address().port}`, fechar: () => servidor.close() })));
}

if (import.meta.url === `file:///${process.argv[1].replace(/\\/g, "/")}`) {
  const [alvo, host, porta] = process.argv.slice(2);
  if (!alvo || !host) { console.error("uso: node scripts/proxy-teste.mjs <alvo> <host público> [porta]"); process.exit(2); }
  const p = await ligarProxy({ alvo, host, porta: Number(porta) || 0 });
  console.log(`proxy de teste em ${p.url} -> ${alvo} (Host: ${host}, X-Forwarded-Proto: https)`);
}
