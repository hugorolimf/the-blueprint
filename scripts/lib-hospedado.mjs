// Peças dos e2e do modo hospedado (servidor-e2e.mjs, convites-e2e.mjs): `blueprint serve --hospedar` com banco temporário atrás do proxy de
// teste, e pedidos como o navegador os faz pelo proxy (Origin pública, cookie). Nunca usa o banco real.
import { spawn } from "node:child_process";
import { dormir, http, pastaDeDados } from "./lib-e2e-rust.mjs";
import { ligarProxy } from "./proxy-teste.mjs";

export const PUBLICA = "https://equipe.teste";

/** Liga o servidor hospedado; devolve { proc, saida() } com o que ele imprimiu (o link do dono sai aqui, uma vez). */
export async function ligarHospedado(exe, d, args = []) {
  const env = { ...d.env, BLUEPRINT_URL_PUBLICA: PUBLICA, BLUEPRINT_HSTS: "1" };
  const proc = spawn(exe, ["serve", "--hospedar", ...args], { env, stdio: ["ignore", "pipe", "pipe"], windowsHide: true });
  let texto = "";
  proc.stdout.on("data", (b) => (texto += b));
  proc.stderr.on("data", (b) => (texto += b));
  for (let i = 0; i < 200; i++) {
    // pronto quando já respondeu e terminou de imprimir (o link do dono vem depois de escutar)
    const r = await http(d.url + "/").catch(() => null);
    if (r && /\(escutando em|\(listening on/.test(texto) && (i > 4 || /convite#|já foi impresso|already printed/.test(texto))) return { proc, saida: () => texto };
    await dormir(50);
  }
  proc.kill();
  throw new Error("o servidor hospedado não subiu:\n" + texto);
}

/** Pasta de dados, servidor e proxy; `fechar()` derruba tudo. */
export async function ambiente(exe, prefixo, dados) {
  const d = await pastaDeDados(prefixo);
  if (dados) await (await import("node:fs/promises")).writeFile(d.env.DADOS_ARQ, JSON.stringify(dados));
  const servidor = await ligarHospedado(exe, d);
  const proxy = await ligarProxy({ alvo: d.url, host: "equipe.teste" });
  return { d, servidor, proxy, fechar: () => { proxy.fechar(); servidor.proc.kill(); } };
}

/** Um pedido do navegador pelo proxy: Origin pública e, se houver, o cookie. */
export function navegador(proxy, metodo, caminho, { corpo, cookie, origem = PUBLICA } = {}) {
  const cabecalhos = { origin: origem };
  if (cookie) cabecalhos.cookie = `bp_sessao=${cookie}`;
  if (corpo !== undefined) cabecalhos["content-type"] = "application/json";
  return http(proxy.url + caminho, { metodo, cabecalhos, corpo: corpo === undefined ? undefined : JSON.stringify(corpo) });
}

/** O segredo do link `https://equipe.teste/convite#<segredo>`. */
export const segredoDe = (link) => link.slice(link.indexOf("#") + 1);

/** Entra pelo link (conferir + entrar) e devolve { cookie, destino }. */
export async function entrar(proxy, link, nome) {
  const segredo = segredoDe(link);
  const conf = await navegador(proxy, "POST", "/api/convite/conferir", { corpo: { segredo } });
  if (conf.status !== 200) throw new Error(`conferir: ${conf.status} ${conf.texto}`);
  const r = await rawEntrar(proxy, segredo, nome);
  return r;
}

async function rawEntrar(proxy, segredo, nome) {
  const { request } = await import("node:http");
  const u = new URL(proxy.url + "/api/convite/entrar");
  return new Promise((ok, falha) => {
    const req = request({ hostname: u.hostname, port: u.port, path: u.pathname, method: "POST", headers: { origin: PUBLICA, "content-type": "application/json" } }, (res) => {
      let t = "";
      res.on("data", (b) => (t += b));
      res.on("end", () => {
        const set = [].concat(res.headers["set-cookie"] || [])[0] || "";
        const cookie = (set.match(/bp_sessao=([^;]*)/) || [])[1];
        ok({ status: res.statusCode, cookie, setCookie: set, json: (() => { try { return JSON.parse(t); } catch { return null; } })() });
      });
    });
    req.on("error", falha);
    req.end(JSON.stringify(nome === undefined ? { segredo } : { segredo, nome }));
  });
}
