// Túnel falso para o e2e do Compartilhar com o binário real (scripts/compartilhar-e2e.mjs --rust): faz o papel do cloudflared.
// O app o roda no lugar do cloudflared quando BLUEPRINT_TUNEL_COMANDO aponta para ele (o e2e cria um .cmd que chama este arquivo).
// Acha o destino como o cloudflared ("--url http://127.0.0.1:<porta dos convidados>", ou a primeira URL http nos argumentos; reserva:
// TUNEL_FALSO_ALVO), escuta numa porta livre e repassa cada pedido como a Cloudflare faria: Host e Origin do endereço público,
// X-Forwarded-Proto: https e X-Forwarded-For. Imprime o endereço público no formato do cloudflared (na saída de erro e na padrão) e grava
// {publica, local} em TUNEL_FALSO_INFO, para o e2e falar com a porta local no lugar do domínio público. Sai quando o processo pai sai.
import { randomBytes } from "node:crypto";
import { writeFileSync } from "node:fs";
import { createServer, request } from "node:http";

const args = process.argv.slice(2), i = args.indexOf("--url");
const alvo = new URL(i >= 0 ? args[i + 1] : args.find((a) => /^https?:\/\//.test(a)) || process.env.TUNEL_FALSO_ALVO);
const publica = process.env.TUNEL_FALSO_PUBLICA || `https://teste-${randomBytes(4).toString("hex")}.trycloudflare.com`, hostPublico = new URL(publica).host;

const srv = createServer((req, res) => {
  let local = "";
  try { local = `http://${req.headers.host}`; } catch {}
  const cab = { ...req.headers, host: hostPublico, "x-forwarded-proto": "https", "x-forwarded-for": "127.0.0.1" };
  if (cab.origin === local) cab.origin = publica;
  if (cab.referer) cab.referer = cab.referer.replace(local, publica);
  const ida = request({ hostname: alvo.hostname, port: alvo.port, path: req.url, method: req.method, headers: cab }, (volta) => {
    const h = { ...volta.headers };
    if (h["set-cookie"]) h["set-cookie"] = h["set-cookie"].map((c) => c.replace(/;\s*Secure/i, "")); // o teste fala http com o túnel
    res.writeHead(volta.statusCode, h);
    volta.pipe(res);
  });
  ida.on("error", () => { if (!res.headersSent) res.writeHead(502); res.end(); });
  req.pipe(ida);
});
srv.listen(0, "127.0.0.1", () => {
  const local = `http://127.0.0.1:${srv.address().port}`;
  if (process.env.TUNEL_FALSO_INFO) writeFileSync(process.env.TUNEL_FALSO_INFO, JSON.stringify({ publica, local, alvo: alvo.href }));
  const caixa = `INF |  Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):\nINF |  ${publica}  |\n`;
  process.stderr.write(caixa);
  process.stdout.write(caixa);
});
// o app desliga o túnel matando o processo; num .cmd ele mata o cmd.exe e não este node: então sai quando o pai some
const pai = process.ppid;
setInterval(() => { try { process.kill(pai, 0); } catch { process.exit(0); } }, 300).unref();
