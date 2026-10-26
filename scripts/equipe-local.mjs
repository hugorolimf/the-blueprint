// Um servidor de equipe de teste nesta máquina, para experimentar a colaboração à mão: `blueprint serve --hospedar` com banco próprio em
// .dev/equipe (dados de data.exemplo.json na primeira vez; nunca o banco real) atrás do proxy de teste, que faz o papel do Caddy e troca a
// Origin. Abra o link do dono num navegador e os convites em outro perfil ou numa janela InPrivate (cada um guarda o próprio cookie).
// Uso: node scripts/equipe-local.mjs [blueprint.exe] [--porta 8443] [--novo-link-dono]   (Ctrl+C encerra; --limpar recomeça do zero)
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { portaLivre } from "./lib-e2e-rust.mjs";
import { ligarProxy } from "./proxy-teste.mjs";

const raiz = resolve(import.meta.dirname, "..");
const args = process.argv.slice(2);
const opcao = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : null; };
const exe = resolve(args.find((a) => a.endsWith(".exe")) || join(raiz, "target/debug/blueprint.exe"));
const portaProxy = Number(opcao("porta")) || 8443;
const PUBLICA = "https://equipe.teste";
const pasta = join(raiz, ".dev/equipe");
if (args.includes("--limpar")) rmSync(pasta, { recursive: true, force: true });
mkdirSync(pasta, { recursive: true });
if (!existsSync(join(pasta, "data.json"))) copyFileSync(join(raiz, "data.exemplo.json"), join(pasta, "data.json"));
if (!existsSync(exe)) { console.error(`não achei ${exe}; rode cargo build antes`); process.exit(1); }

const porta = await portaLivre();
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(pasta, "blueprint.db"), DADOS_ARQ: join(pasta, "data.json"), PROTOTIPOS_DIR: join(pasta, "prototipos"), BLUEPRINT_URL_PUBLICA: PUBLICA };
const proxy = await ligarProxy({ alvo: `http://127.0.0.1:${porta}`, host: "equipe.teste", porta: portaProxy, origem: PUBLICA });
const servidor = spawn(exe, ["serve", "--hospedar", ...(args.includes("--novo-link-dono") ? ["--novo-link-dono"] : [])], { env, stdio: ["ignore", "pipe", "pipe"], windowsHide: true });
// o servidor imprime os links com a origem pública; aqui eles saem já apontando para o proxy, prontos para abrir
const local = (b) => process.stdout.write(String(b).replaceAll(PUBLICA, proxy.url));
servidor.stdout.on("data", local);
servidor.stderr.on("data", local);
servidor.on("exit", (c) => { proxy.fechar(); process.exit(c ?? 0); });
process.on("SIGINT", () => servidor.kill());
console.log(`equipe de teste em ${proxy.url} (dados em .dev/equipe). Link do dono já usado? rode de novo com --novo-link-dono.`);
