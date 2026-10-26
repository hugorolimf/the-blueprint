// Um `tailscale` falso para os e2e do endereço fixo (scripts/endereco-fixo-e2e.mjs): `status --json` e `serve status --json` respondem como o de verdade e `funnel --https=<porta> <alvo>`
// sobe o túnel falso (scripts/tunel-falso.mjs) com o endereço FIXO https://maq.tail1234.ts.net (a mesma coisa que o Funnel faria: o endereço não muda entre uma vez e outra).
// O e2e cria um .cmd que chama este arquivo e aponta BLUEPRINT_TAILSCALE_COMANDO para ele. TAILSCALE_FALSO_ESTADO troca o BackendState (padrão Running).
import { spawn } from "node:child_process";
import { join } from "node:path";

const [cmd, ...resto] = process.argv.slice(2);
const maquina = "maq.tail1234.ts.net";
if (cmd === "status") {
  console.log(JSON.stringify({ BackendState: process.env.TAILSCALE_FALSO_ESTADO || "Running", Self: { DNSName: `${maquina}.` } }));
} else if (cmd === "serve") {
  console.log("{}");
} else if (cmd === "funnel") {
  const alvo = resto.find((a) => /^https?:\/\//.test(a));
  const filho = spawn(process.execPath, [join(import.meta.dirname, "tunel-falso.mjs"), "--url", alvo], { env: { ...process.env, TUNEL_FALSO_PUBLICA: `https://${maquina}` }, stdio: "inherit" });
  filho.on("exit", (c) => process.exit(c ?? 0));
} else {
  process.exit(1);
}
