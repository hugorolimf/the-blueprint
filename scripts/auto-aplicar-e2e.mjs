// Aplicar sozinho, de ponta a ponta: o agente envia (propor_blueprint e registrar_trabalho pelo `blueprint --mcp`, como o Claude faria) e, com a página aberta e o interruptor
// ligado, a proposta e o registro entram no blueprint SEM clique, com aviso e Desfazer; remoção continua pedindo o clique; com o interruptor desligado nada entra; e quem não
// é dono no modo equipe nunca aplica sozinho. Roda em pt-BR e em en-US. Servidor próprio (banco de exemplo, porta livre, cópia do .exe); não toca no app do usuário.
// Uso: node scripts/auto-aplicar-e2e.mjs [exe] [pasta-com-web]   (precisa do Microsoft Edge; serve web/ da pasta dada, por padrão a do repositório)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";
import { espera } from "./lib-e2e-edge.mjs";

const exeOrigem = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const assets = resolve(process.argv[3] || raiz);
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const TEXTOS = { "pt-BR": { aplicado: /Aplicado/, registro: /Registro adicionado/, desfeito: /Desfeito/, ligado: /ligado/, desligado: /desligado/, nota: /precisa da sua confirmação/ }, "en-US": { aplicado: /Applied/, registro: /Record added|Registro adicionado/, desfeito: /Undone/, ligado: /: on/, desligado: /: off/, nota: /needs your confirmation/ } };

async function rodar(lang) {
  const T = TEXTOS[lang], t0 = Date.now();
  const [porta, depuracao] = [await portaLivre(), await portaLivre()];
  const url = `http://127.0.0.1:${porta}`, tmp = mkdtempSync(join(tmpdir(), "blueprint-auto-"));
  const exe = join(tmp, "blueprint.exe");
  copyFileSync(exeOrigem, exe); // uma cópia: outro `cargo` pode estar recompilando o de target/
  copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
  const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: assets, BLUEPRINT_SEM_ATUALIZACAO: "1" }, stdio: "ignore", windowsHide: true });
  const edge = spawn(edgeExe, ["--headless=new", `--lang=${lang}`, "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
  const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 500); };
  try {
    for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
    const bp = (await (await fetch(url + "/api/dados")).json()).blueprints.find((p) => p.acoes?.length);
    let alvo;
    for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
    const ws = new WebSocket(alvo.webSocketDebuggerUrl);
    await new Promise((r) => (ws.onopen = r));
    let n = 0; const pend = new Map(), erros = [];
    ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
    const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
    const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
    await cdp("Runtime.enable");

    // o agente chama uma ferramenta do MCP pelo `blueprint --mcp` (stdio); devolve { isError, texto }
    const chamar = async (nome, argumentos) => {
      const filho = spawn(exe, ["--mcp"], { env: { ...process.env, BLUEPRINT_URL: url }, stdio: ["pipe", "pipe", "ignore"], windowsHide: true });
      let buf = "", id = 0; const esp = new Map();
      filho.stdout.on("data", (d) => { buf += d; for (let i; (i = buf.indexOf("\n")) >= 0;) { const l = buf.slice(0, i).trim(); buf = buf.slice(i + 1); if (l) { const m = JSON.parse(l); esp.get(m.id)?.(m); } } });
      const rpc = (method, params) => new Promise((ok) => { esp.set(++id, ok); filho.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
      try {
        await rpc("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "auto-aplicar-e2e", version: "1" } });
        const r = await rpc("tools/call", { name: nome, arguments: { workspace_id: "sm", blueprint_id: bp.id, ...argumentos } });
        return { isError: !!(r.error || r.result?.isError), texto: r.result?.content?.map((x) => x.text).join("\n") ?? JSON.stringify(r.error) };
      } finally { filho.kill(); }
    };
    const dados = async () => (await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === bp.id);
    const acaoDe = async (nome) => (await dados()).acoes.find((f) => f.nome === nome);
    const fila = async (tipo) => (await (await fetch(url + "/api/mcp/fila")).json()).itens.filter((i) => i.tipo === tipo && i.blueprintId === bp.id);
    const aviso = () => js(`(() => { const e = document.getElementById("auto-aviso"); return e && !e.hidden ? { texto: e.querySelector(".aa-texto").textContent, desfazer: !e.querySelector(".aa-desfazer").hidden } : null; })()`);
    const proposta = (nome, extra = {}) => chamar("propor_blueprint", { acoes: [{ nome, inicio: "2027-03-01", fim: "2027-03-02", planos: [{ texto: "Escrever o roteiro", descricao: "Scope: o roteiro. Expected result: texto. Validation: revisão." }], checkpoints: [{ nome: "Roteiro aprovado", data: "2027-03-02" }] }], ...extra });

    await cdp("Page.navigate", { url: `${url}/#/p/${bp.id}` });
    await espera(() => js(`typeof autoElegivel === "function" && carregado && !!workspaceAtual`), "a página não carregou");
    const total = (await dados()).acoes.length;
    // o que o agente recebe quando algo é aplicado: QUAL item (número) e QUEM aplicou (usuario no clique, automatica pelo interruptor); lido do mesmo SSE que o monitor usa
    const avisosSse = [], ac = new AbortController();
    (async () => {
      try {
        const leitor = (await fetch(url + "/api/mcp/eventos", { signal: ac.signal })).body.getReader(), dec = new TextDecoder();
        let buf = "";
        for (;;) {
          const { done, value } = await leitor.read();
          if (done) break;
          buf += dec.decode(value, { stream: true });
          for (let i; (i = buf.indexOf("\n\n")) >= 0;) {
            const bloco = buf.slice(0, i); buf = buf.slice(i + 2);
            const nome = /^event: (.+)$/m.exec(bloco)?.[1], dados = /^data: (.+)$/m.exec(bloco)?.[1];
            if (dados && (nome === "proposta-aplicada" || nome === "registro-adicionado")) avisosSse.push({ nome, ...JSON.parse(dados) });
          }
        }
      } catch {}
    })();
    const avisouAgente = (nome, origem) => espera(async () => avisosSse.some((e) => e.nome === nome && e.origem === origem && Number.isInteger(e.numero)), `o aviso ao agente (${nome}) diz origem ${origem} e o número`, 8000);

    // 1) padrão local: ligado, e o interruptor aparece no Planejar
    assert.equal(await js(`autoEstadoDe(${JSON.stringify(bp.id)}).ligado`), true, "app local: ligado por padrão");
    await js(`document.getElementById("b-ia").click()`);
    await espera(() => js(`!!document.getElementById("ag-auto") && !document.getElementById("ag-auto").hidden`), "o interruptor aparece no cabeçalho do Planejar");
    assert.equal(await js(`document.getElementById("ag-auto").getAttribute("aria-pressed")`), "true");
    assert.match(await js(`document.getElementById("ag-auto").textContent`), T.ligado);
    await js(`document.getElementById("ag-fechar").click()`);
    await dormir(600);

    // 2) proposta: entra sozinha, com aviso e Desfazer, e sai da fila do servidor
    const r1 = await proposta("Ação enviada pelo agente");
    assert.equal(r1.isError, false, r1.texto);
    await espera(async () => !!(await acaoDe("Ação enviada pelo agente")), "a proposta entrou no blueprint sem clique", 8000);
    assert.equal((await dados()).acoes.length, total + 1);
    const a1 = await aviso();
    assert(a1, "o aviso Aplicado sozinho aparece");
    await avisouAgente("proposta-aplicada", "automatica");
    assert.match(a1.texto, T.aplicado);
    assert.equal(a1.desfazer, true, "o aviso traz o Desfazer");
    assert.equal((await fila("proposta")).length, 0, "a proposta saiu da fila do servidor (ack)");
    assert.equal(await js(`autoLista.length`), 1, "o rastro guarda o que entrou");

    // 3) Desfazer: a ação some e o aviso diz que foi desfeito
    await js(`document.querySelector("#auto-aviso .aa-desfazer").click()`);
    await espera(async () => !(await acaoDe("Ação enviada pelo agente")), "o Desfazer tirou a ação", 6000);
    assert.equal((await dados()).acoes.length, total);
    assert.match((await aviso()).texto, T.desfeito);
    assert.equal((await aviso()).desfazer, false, "só se desfaz uma vez");

    // 4) registro: entra sozinho como o "Adicionar ao blueprint" faz (feito e evidência)
    const EVID = "node teste.mjs: 5 passam, 0 falham";
    const r2 = await chamar("registrar_trabalho", { mensagem: "Corrigi fora do plano.", acoes: [{ nome: "Ajuste do agente", inicio: "2026-10-20", fim: "2026-10-23", planos: [{ texto: "Corrigir o botão", descricao: "Escopo: o botão. Validação: o teste passa.", feito: true, evidencia: EVID }] }] });
    assert.equal(r2.isError, false, r2.texto);
    await espera(async () => !!(await acaoDe("Ajuste do agente")), "o registro entrou sem clique", 8000);
    const f = await acaoDe("Ajuste do agente");
    assert.equal(f.planos[0].feito, true, "o plano feito ficou feito");
    assert(f.planos[0].agente?.comentarios.some((c) => c.texto === EVID && c.tipo === "relatorio"), "a evidência virou comentário do plano");
    assert.match((await aviso()).texto, T.registro);
    await avisouAgente("registro-adicionado", "automatica");
    assert.equal((await fila("registro")).length, 0, "o registro saiu da fila do servidor");

    // 5) remoção NÃO entra sozinha: espera o clique, com a nota, e nada muda nos dados
    const alvoAcao = (await dados()).acoes.find((a) => a.nome !== "Ajuste do agente");
    const antesRem = JSON.stringify(await dados());
    const r3 = await chamar("propor_blueprint", { acoes: [], remover: [{ acao_id: alvoAcao.id, motivo: "teste" }] });
    assert.equal(r3.isError, false, r3.texto);
    await espera(async () => (await fila("proposta")).length === 1, "a proposta com remoção chegou ao servidor");
    await dormir(1500);
    assert.equal((await fila("proposta")).length, 1, "a remoção continua na fila: espera o clique");
    assert.equal(JSON.stringify(await dados()), antesRem, "nada foi apagado");
    await js(`document.getElementById("b-ia").click()`);
    await espera(() => js(`!!document.querySelector("#cl-prop .aa-nota")`), "a nota de confirmação aparece na proposta");
    assert.match(await js(`document.querySelector("#cl-prop .aa-nota").textContent`), T.nota);
    assert.match(await js(`document.getElementById("cl-auto").textContent`), T.aplicado, "o Planejar lista o que entrou sozinho");
    await js(`document.querySelector("#cl-prop [data-pact=descartar]").click()`);
    await espera(async () => (await fila("proposta")).length === 0, "descartar tira a proposta da fila");

    // 6) interruptor desligado: a proposta comum espera o clique e nada entra
    await js(`document.getElementById("ag-auto").click()`);
    assert.equal(await js(`document.getElementById("ag-auto").getAttribute("aria-pressed")`), "false");
    assert.match(await js(`document.getElementById("ag-auto").textContent`), T.desligado);
    assert.equal(await js(`localStorage.getItem(autoChaveBlueprint("sm", ${JSON.stringify(bp.id)}))`), "0", "a escolha fica guardada");
    const antes6 = (await dados()).acoes.length;
    assert.equal((await proposta("Ação que espera o clique")).isError, false);
    await espera(async () => (await fila("proposta")).length === 1, "a proposta chegou");
    await dormir(1500);
    assert.equal((await fila("proposta")).length, 1, "desligado: continua na fila");
    assert.equal((await dados()).acoes.length, antes6, "desligado: nada entra");
    assert.match(await js(`document.getElementById("cl-prop").textContent`), /Ação que espera o clique/, "a proposta espera na tela Planejar");
    await js(`document.querySelector("#cl-prop [data-pact=descartar]").click()`);
    await espera(async () => (await fila("proposta")).length === 0, "descartada");
    // 6b) o clique do usuário também avisa o agente, com a origem "usuario" (a aprovação explícita)
    assert.equal((await proposta("Ação aplicada no clique")).isError, false);
    await espera(async () => (await fila("proposta")).length === 1, "a proposta do clique chegou");
    await js(`aplicarProposta()`);
    await avisouAgente("proposta-aplicada", "usuario");
    await js(`document.getElementById("ag-auto").click()`); // religa

    // 7) modo equipe: o padrão é desligado, só o dono pode ligar, e os outros papéis nunca aplicam sozinhos
    const estado = (s, guardado) => js(`(() => { const antes = sessao; sessao = ${JSON.stringify(s)}; const k = autoChaveBlueprint("sm", "x"); ${guardado ? `localStorage.setItem(k, "1")` : `localStorage.removeItem(k)`}; const r = autoEstadoDe("x"); sessao = antes; localStorage.removeItem(k); return r; })()`);
    const equipe = (papel) => ({ modo: "servidor", membro: { id: "m1", nome: "M" }, acessos: [{ workspaceId: "*", papel }] });
    assert.deepEqual(await estado(equipe("dono"), false), { pode: true, ligado: false }, "equipe: desligado por padrão");
    assert.deepEqual(await estado(equipe("dono"), true), { pode: true, ligado: true }, "o dono pode ligar");
    for (const papel of ["colaborador", "supervisor"]) assert.deepEqual(await estado(equipe(papel), true), { pode: false, ligado: false }, `${papel} nunca aplica sozinho`);
    assert.deepEqual(await estado({ modo: "servidor", visitante: true, membro: null, acessos: [{ workspaceId: "sm", papel: "visitante" }] }, true), { pode: false, ligado: false }, "visitante nunca aplica sozinho");

    assert.deepEqual(erros, [], "nenhum erro de script na página");
    ac.abort();
    console.log(`auto-aplicar-e2e (${lang}): ok em ${((Date.now() - t0) / 1000).toFixed(1)} s`);
  } finally { limpar(); }
}
for (const lang of ["pt-BR", "en-US"]) await rodar(lang);
