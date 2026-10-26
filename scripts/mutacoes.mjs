// Prova que os testes de verdade protegem as regras do CLAUDE.md: quebra cada regra de propósito no código Rust (uma de cada vez), roda `cargo test`
// e confere que ALGUM teste falha. No fim restaura os arquivos. Se uma mutação passar nos testes, a regra está sem teste: sai 1.
// Uso: node scripts/mutacoes.mjs [filtro]   (demora: um `cargo test` por mutação)
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const M = (regra, arquivo, de, para) => ({ regra, arquivo, de, para });
const MUTACOES = [
  M("MCP só em loopback: o Host é conferido", "src/mcp/permissao.rs", "!LOOPBACK.contains(&sem_porta(host)) || !LOOPBACK.contains(&mcp.host.as_str())", "false"),
  M("MCP só em loopback: a Origin é conferida", "src/mcp/permissao.rs", 'pedido.cabecalho("origin").is_none_or(|o| o == format!("http://{host}"))', "true"),
  M("MCP_REDE=1 é o único jeito de liberar a rede", "src/mcp/permissao.rs", "if mcp.rede {", "if true {"),
  M("iniciar a ação (abre processo) só em loopback, nem MCP_REDE=1 libera", "src/mcp/rotas.rs", 'if recurso == "iniciar" && !so_loopback(mcp, pedido) {', "if false {"),
  M("iniciar: ids sem caractere de controle e com tamanho máximo", "src/mcp/iniciar.rs", "!id.is_empty() && id.chars().count() <= 100 && !id.chars().any(char::is_control)", "true"),
  M("iniciar: o ';' dos nomes não chega ao terminal (separador do wt)", "src/mcp/iniciar.rs", "if c == ';' { ',' } else { c }", "c"),
  M("iniciar: pasta que sumiu do disco é recusada (409)", "src/mcp/iniciar.rs", ".filter(|c| Path::new(c).is_dir())", ""),
  M("subagente: só vira agente quem manda agent_id", "src/mcp/hooks.rs", "alvo(mcp, dados, corpo)?.filter(|_| sub)", "alvo(mcp, dados, corpo)?"),
  M("subagente: o hook de edição não herda o plano do Claude principal", "src/mcp/hooks.rs", 'l.iter().find(|a| a["agente"] == nome && a["blueprintId"] == bp)', 'l.iter().find(|a| a["blueprintId"] == bp)'),
  M("todo POST do MCP exige content-type JSON", "src/mcp/rotas.rs", "if !pedido.eh_json() {\n        return Ok(Resposta::texto(415", "if false {\n        return Ok(Resposta::texto(415"),
  M("protótipo sem allow-same-origin (sandbox)", "src/http/prototipos.rs", "sandbox allow-scripts;", "sandbox allow-scripts allow-same-origin;"),
  M("protótipo: só ids de 16 hexadecimais (sem ..)", "src/mcp/acoes.rs", 'id.len() == 16 && id.bytes().all(|c| matches!(c, b\'0\'..=b\'9\' | b\'a\'..=b\'f\'))', "true"),
  M("só sai do servidor o que a lista permite", "src/http/estaticos.rs", 'matches!(caminho, "/blueprint.html" | "/emblemas.mjs" | "/agentes-figura.mjs" | "/agentes-claude.mjs")', 'matches!(caminho, "/blueprint.html" | "/emblemas.mjs" | "/agentes-figura.mjs" | "/agentes-claude.mjs" | "/data.json")'),
  M("limite de corpo das rotas", "src/http/rotas.rs", "const LIMITE_PADRAO: usize = 256 * 1024;", "const LIMITE_PADRAO: usize = 256 * 1024 * 1024;"),
  M("PUT /api/dados exige blueprints[]", "src/http/api_dados.rs", "!valor.get(\"blueprints\").is_some_and(Value::is_array)", "false"),
  M("checkpoint já feito não é marcado de novo (409)", "src/mcp/acoes.rs", 'if verdade(checkpoint.get("feito")) {', "if false {"),
  M("plano já concluído não é marcado de novo (409)", "src/mcp/acoes.rs", 'if marca && verdade(plano.get("feito")) {', "if false {"),
  M("evento sem texto/evidência é recusado", "src/mcp/acoes.rs", "if texto.is_empty() {\n        return Err(falha(400, crate::tr!(\"texto is required\"", "if false {\n        return Err(falha(400, crate::tr!(\"texto is required\""),
  M("proposta nova substitui a pendente do mesmo alvo", "src/mcp/fila.rs", 'self.itens.retain(|i| !(i["tipo"] == "proposta"', 'self.itens.retain(|i| true || !(i["tipo"] == "proposta"'),
  M("atividade: estado e papel só os permitidos", "src/mcp/acoes.rs", "Some(Value::String(s)) if permitidos.contains(&s.as_str()) =>", "Some(Value::String(s)) if true =>"),
  M("atividade: onde tem tamanho máximo", "src/mcp/acoes.rs", "Some(Value::String(s)) if limpo(s, 120) =>", "Some(Value::String(s)) if true =>"),
  M("atividade: agente sem caracteres de controle", "src/mcp/acoes.rs", "Some(Value::String(s)) if limpo(s.trim(), 40) && !s.chars().any(char::is_control) =>", "Some(Value::String(s)) if true =>"),
  M("atividade: o agente sem sinal continua na lista (repouso, não some)", "src/mcp/atividade.rs", "let mut todos: Vec<&Registro> = self.registros.iter().collect();", "let mut todos: Vec<&Registro> = self.registros.iter().take(1).collect();"),
  M("atividade: o 33º agente substitui o mais antigo", "src/mcp/atividade.rs", "if self.registros.len() >= MAX_AGENTES {", "if false {"),
  M("proposta: fim não pode ser antes do início", "src/blueprint/validar.rs", "if dia_valido(&inicio) && dia_valido(&fim) && fim < inicio {", "if false {"),
  M("proposta: checkpoint dentro do período da ação", "src/blueprint/itens.rs", "else if periodo_ok && (data.as_str() < c.inicio || data.as_str() > c.fim) {", "else if false {"),
  M("proposta: nunca propor status concluído", "src/blueprint/validar.rs", 'let status_ok = status == "planejado" || status == "andamento";', "let status_ok = true;"),
  M("gravação do banco é atômica (transação)", "src/dados/mod.rs", "let tx = self.conn.transaction()?;\n        importar::importar(&tx, dados)?;\n        tx.commit()?;", "importar::importar(&self.conn, dados)?;"),
  M("só dono e colaborador editam", "src/contexto/mod.rs", "matches!(self.papel, Papel::Dono | Papel::Colaborador)", "true"),
  // modo hospedado (docs/colaboracao.md): rode só estas com `node scripts/mutacoes.mjs acesso:`
  M("acesso: só HTTPS pelo proxy (X-Forwarded-Proto)", "src/acesso/cabecalhos.rs", '!proto.is_some_and(|p| p.eq_ignore_ascii_case("https"))', "false"),
  M("acesso: só o par confiável (o proxy) diz que é HTTPS", "src/acesso/cabecalhos.rs", "if !par_confiavel(h, &pedido.ip) || !proto", "if !proto"),
  M("acesso: só o Host público", "src/acesso/cabecalhos.rs", 'if !pedido.cabecalho("host").is_some_and(|host| host.eq_ignore_ascii_case(&h.host_publico)) {', "if false {"),
  M("acesso: Origin pública em todo POST do navegador", "src/acesso/mod.rs", "&& !cabecalhos::origem_ok(h, pedido)", "&& false"),
  M("acesso: Referrer-Policy no-referrer", "src/acesso/cabecalhos.rs", '.com("referrer-policy", "no-referrer")', '.com("referrer-policy", "origin")'),
  M("acesso: rotas só locais não existem no servidor", "src/acesso/mod.rs", "if so_local(caminho) {", "if false {"),
  M("acesso: sem sessão só a página e o convite (401)", "src/acesso/mod.rs", "if !publica && !ctx.autenticado() {", "if false {"),
  M("acesso: só o hash do convite no banco", "src/acesso/convites.rs", "convites::criar(conn, &c, &segredo::hash(&valor))?;", "convites::criar(conn, &c, &valor)?;"),
  M("acesso: convite revogado, vencido ou esgotado não entra", "src/dados/convites.rs", "self.revogado_em.is_none() && self.expira_em.as_str() > agora && self.usos_max.is_none_or(|m| self.usos < m)", "true"),
  M("acesso: limite de tentativas por IP", "src/acesso/limite.rs", "if let Some(f) = ip_fila.filter(|f| f.len() >= POR_IP) {", "if let Some(f) = ip_fila.filter(|_| false) {"),
  M("acesso: cookie HttpOnly", "src/acesso/sessoes.rs", '"{COOKIE}={valor}; HttpOnly; SameSite=Lax;', '"{COOKIE}={valor}; SameSite=Lax;'),
  M("acesso: supervisor e visitante não gravam", "src/contexto/mod.rs", "Acao::Gravar => matches!(papel, Papel::Dono | Papel::Colaborador),", "Acao::Gravar => true,"),
  M("acesso: só o dono administra", "src/contexto/mod.rs", "Acao::Administrar => papel == Papel::Dono,", "Acao::Administrar => true,"),
  M("acesso: o acesso de um blueprint não alcança outro", "src/contexto/mod.rs", "(self.blueprint_id.is_empty() || Some(self.blueprint_id.as_str()) == blueprint)", "true"),
  M("acesso: GET /api/dados filtrado pelo papel", "src/http/api_dados.rs", '"GET" if !crate::acesso::filtro::le_tudo(ctx) => {', '"GET" if false => {'),
  M("acesso: PUT inteiro só do dono de todos (use-mudancas)", "src/http/api_dados.rs", '"PUT" if !crate::acesso::filtro::le_tudo(ctx) =>', '"PUT" if false =>'),
  M("acesso: o MCP confere o papel no modo hospedado", "src/mcp/rotas.rs", "} else if let Err(e) = exige_hospedado(dados, pedido, recurso, ctx) {", "} else if let Err(e) = Ok::<(), ErroHttp>(()) {"),
  M("acesso: a fila do MCP só mostra o que o papel lê", "src/mcp/rotas.rs", "mcp.itens().into_iter().filter(|i| pode_ler_item(ctx, i))", "mcp.itens().into_iter().filter(|_| true)"),
  M("acesso: visitante não lê os comentários do agente", "src/acesso/filtro.rs", 'agente.remove("comentarios");', 'agente.get("comentarios");'),
  M("acesso: nunca rebaixar o último dono", "src/acesso/rotas_pessoas.rs", "if papel != Papel::Dono && era_dono(t, membro, ws)? && membros::donos_do_workspace(t, ws, membro)? == 0 {", "if false {"),
  // da revisão de segurança (docs/seguranca-servidor.md): rode só estas com `node scripts/mutacoes.mjs "[telas]"`
  M("acesso: [telas] cookie Secure no modo hospedado", "src/acesso/sessoes.rs", "Path=/; Max-Age={max_age_s}; Secure\")", "Path=/; Max-Age={max_age_s}\")"),
  M("acesso: [telas] só o hash da sessão no banco", "src/acesso/sessoes.rs", "sessoes::criar(conn, &segredo::hash(&valor), &s,", "sessoes::criar(conn, &valor, &s,"),
  M("acesso: [telas] só o hash da chave de MCP no banco", "src/acesso/chaves.rs", "sessoes::gravar_chave(conn, &segredo::hash(&chave), membro_id,", "sessoes::gravar_chave(conn, &chave, membro_id,"),
  M("acesso: [telas] nunca remover o último dono", "src/acesso/rotas_pessoas.rs", "alvo_no_workspace(t, membro, ws)?;\n        if era_dono(t, membro, ws)?", "alvo_no_workspace(t, membro, ws)?;\n        if false && era_dono(t, membro, ws)?"),
  // sem mutação para `"chats" | "usuario" if ctx.visitante` (filtro.rs): o visitante nunca lê a raiz inteira, então o ramo `!le_raiz` já os tira
  // e a mutação seria equivalente (sobrevive sem mudar nada); visitante-supervisor-e2e.mjs confere a resposta no navegador.
  M("acesso: protótipo só para quem lê o blueprint","src/http/rotas.rs", 'if !ctx.local && caminho.starts_with("/prototipos/") && !prototipo_visivel(estado, caminho, ctx) {', "if false {"),
];

const filtro = process.argv[2] || "";
const alvos = MUTACOES.filter((m) => m.regra.includes(filtro));
const originais = new Map();
const restaurar = () => { for (const [a, t] of originais) writeFileSync(join(raiz, a), t); };
process.on("exit", restaurar);
let sobreviveram = 0;
for (const m of alvos) {
  const caminho = join(raiz, m.arquivo);
  const texto = originais.get(m.arquivo) ?? readFileSync(caminho, "utf8");
  originais.set(m.arquivo, texto);
  const sem_crlf = texto.replace(/\r\n/g, "\n");
  if (!sem_crlf.includes(m.de)) { console.log(`?? ${m.regra}: o trecho não existe mais em ${m.arquivo}`); sobreviveram++; continue; }
  writeFileSync(caminho, sem_crlf.replace(m.de, () => m.para));
  const r = spawnSync("cargo", ["test", "--lib"], { cwd: raiz, encoding: "utf8", timeout: 600000, shell: process.platform === "win32" });
  writeFileSync(caminho, texto);
  const compilou = !/error(\[E\d+\])?:/.test(r.stderr || "") || /test result/.test((r.stdout || "") + (r.stderr || ""));
  const falhas = ((r.stdout || "") + (r.stderr || "")).split("\n").filter((l) => /^test .* FAILED$/.test(l)).map((l) => l.replace(/^test | \.\.\. FAILED$/g, ""));
  if (!compilou) { console.log(`?? ${m.regra}: a mutação nem compilou`); sobreviveram++; }
  else if (r.status === 0) { console.log(`SOBREVIVEU  ${m.regra}  (nenhum teste falhou)`); sobreviveram++; }
  else console.log(`morreu      ${m.regra}  <- ${falhas.slice(0, 2).join(", ") || "falha de teste"}${falhas.length > 2 ? ` (+${falhas.length - 2})` : ""}`);
}
restaurar();
console.log(sobreviveram ? `\n${sobreviveram} de ${alvos.length} mutações sobreviveram: falta teste` : `\ntodas as ${alvos.length} mutações foram pegas por algum teste`);
process.exitCode = sobreviveram ? 1 : 0;
