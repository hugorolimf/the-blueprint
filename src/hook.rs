//! `blueprint hook <evento>`: os comandos que o Claude Code chama (hooks de comando, status line e o comando /blueprint:status).
//! Todos perguntam ao app aberto e nunca atrapalham a sessão: sem o app, ou com qualquer erro, não imprimem nada e saem com 0.
//! (No plugin local as edições não passam por aqui: o hook `http` fala direto com o app. No do convidado, `--equipe`, passam: edicao,
//! subagente, parada e fim vão ao endereço do `blueprint conectar` com a chave.)
//!   sessao           SessionStart: injeta o blueprint e a ação atual no contexto do agente
//!   statusline       a linha curta da status line (lê o JSON da sessão na entrada padrão)
//!   status           o estado completo, em texto, da pasta atual (usado por /blueprint:status)
//!   statusline-config  imprime o trecho de settings.json que liga a status line (opt-in; o plugin não entrega status line)
use crate::mcp::cliente::requisitar;
use crate::mcp::stdio::base;
use serde_json::{json, Value};
use std::io::Read;

/// O que o Claude Code espera de um SessionStart: `additionalContext` entra no contexto do agente sem aparecer como mensagem.
fn saida_do_claude(resposta: &Value) -> Option<Value> {
    let contexto = resposta.get("contexto")?.as_str()?;
    Some(json!({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": contexto}}))
}

/// Chama uma rota de hook do app; só aceita 200 com JSON.
fn perguntar(base: &str, rota: &str, corpo: &str) -> Option<Value> {
    let (status, resposta) = requisitar(base, "POST", rota, Some(corpo)).ok()?;
    if status != 200 { None } else { serde_json::from_str(&resposta).ok() }
}

fn sessao(base: &str, entrada: &str) -> Option<Value> {
    saida_do_claude(&perguntar(base, "hook/sessao", entrada)?)
}

/// O Claude emite o OSC pelo próprio terminal; controles do evento nunca viram escapes.
fn notificacao(entrada: &str) -> Option<Value> {
    let evento: Value = serde_json::from_str(entrada).ok()?;
    let mensagem: String = evento.get("message")?.as_str()?.chars()
        .filter(|c| !c.is_control()).take(180).collect();
    if mensagem.trim().is_empty() { return None; }
    Some(json!({"terminalSequence": format!("\u{1b}]9;The Blueprint · {mensagem}\u{7}")}))
}

/// A pasta de uma entrada de status line: `workspace.current_dir`, senão `cwd`.
fn pasta_da_entrada(entrada: &Value) -> Option<&str> {
    entrada["workspace"]["current_dir"].as_str().or_else(|| entrada["cwd"].as_str())
}

fn statusline(base: &str, entrada: &str) -> Option<String> {
    let e: Value = serde_json::from_str(entrada).ok()?;
    let corpo = json!({"session_id": e["session_id"], "cwd": pasta_da_entrada(&e)?}).to_string();
    perguntar(base, "hook/status", &corpo)?.get("linha")?.as_str().map(String::from)
}

fn status(base: &str, pasta: &str) -> String {
    let corpo = json!({"cwd": pasta}).to_string();
    match requisitar(base, "POST", "hook/status", Some(&corpo)) {
        Err(_) => crate::tr!("The Blueprint is not open. Open the app and try again.", "O The Blueprint não está aberto. Abra o app e tente de novo."),
        Ok((200, resposta)) => match serde_json::from_str::<Value>(&resposta).ok().and_then(|r| r["texto"].as_str().map(String::from)) {
            Some(t) => t,
            None => crate::tr!("This folder is not linked to any blueprint. Ask the agent: \"link this folder to blueprint X\" (tool vincular_pasta).", "Esta pasta não está ligada a nenhum blueprint. Peça ao agente: \"ligue esta pasta ao blueprint X\" (ferramenta vincular_pasta)."),
        },
        Ok((codigo, _)) => crate::tr!("The Blueprint refused the request ({codigo}).", "O The Blueprint recusou o pedido ({codigo})."),
    }
}

/// O trecho que o usuário cola no settings.json; o caminho é o deste programa, com `/` para não depender do shell.
pub(crate) fn trecho_da_status_line(exe: &str) -> Value {
    json!({"statusLine": {"type": "command", "command": format!("\"{}\" hook statusline", exe.replace('\\', "/")), "refreshInterval": 30}})
}

fn entrada_padrao() -> String {
    let mut entrada = String::new();
    let _ = std::io::stdin().read_to_string(&mut entrada);
    // só repassa um objeto JSON; qualquer outra coisa vira {} e o app decide que não há o que ligar
    serde_json::from_str::<Value>(&entrada).ok().filter(Value::is_object).map_or_else(|| "{}".to_string(), |v| v.to_string())
}

/// O corpo que vai ao app. No plugin do convidado (`--equipe`) o app é remoto e não conhece as pastas desta máquina: o corpo leva o
/// blueprint da linha `Blueprint:` do CLAUDE.md/AGENTS.md da pasta, e pasta sem a linha não manda nada (None).
fn corpo_do_hook(entrada: String, equipe: bool) -> Option<String> {
    if !equipe {
        return Some(entrada);
    }
    let mut v: Value = serde_json::from_str(&entrada).ok()?;
    let (ws, bp) = crate::cli::claude::blueprint_da_pasta(std::path::Path::new(v.get("cwd")?.as_str()?))?;
    (v["workspaceId"], v["blueprintId"]) = (json!(ws), json!(bp));
    Some(v.to_string())
}

/// Os hooks que só repassam o evento (no plugin local eles são hooks `http` direto no app; no do convidado o endereço muda, então passam
/// por aqui). Só o Stop devolve algo ao Claude: o lembrete de registrar o trabalho feito fora do plano.
fn repassar(base: &str, evento: &str, corpo: &str) -> Option<Value> {
    let resposta = perguntar(base, &format!("hook/{evento}"), corpo)?;
    (evento == "parada" && resposta.get("hookSpecificOutput").is_some()).then_some(resposta)
}

/// Devolve o código de saída (sempre 0).
pub fn executar(evento: &str) -> i32 {
    let equipe = crate::mcp::equipe::ligado();
    let saida = match evento {
        "sessao" => corpo_do_hook(entrada_padrao(), equipe).and_then(|c| sessao(&base(), &c)).map(|v| v.to_string()),
        "edicao" | "subagente" | "parada" | "fim" => corpo_do_hook(entrada_padrao(), equipe).and_then(|c| repassar(&base(), evento, &c)).map(|v| v.to_string()),
        "notificacao" => notificacao(&entrada_padrao()).map(|v| v.to_string()),
        "statusline" => statusline(&base(), &entrada_padrao()),
        "status" => Some(status(&base(), &std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default())),
        "statusline-config" => std::env::current_exe().ok().map(|e| serde_json::to_string_pretty(&trecho_da_status_line(&e.display().to_string())).unwrap_or_default()),
        _ => None,
    };
    if let Some(s) = saida {
        println!("{s}");
    }
    0
}

/// Executa a status line anterior com o mesmo JSON e conserva sua saída.
pub fn combinar(anterior: &str) -> i32 {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let entrada = entrada_padrao();
    let shell = if cfg!(windows) {
        std::env::var_os("CLAUDE_CODE_GIT_BASH_PATH").map(std::path::PathBuf::from).or_else(|| {
            ["C:/Program Files/Git/bin/bash.exe", "C:/Program Files (x86)/Git/bin/bash.exe"].into_iter().map(std::path::PathBuf::from).find(|p| p.is_file())
        }).unwrap_or_else(|| "bash.exe".into())
    } else { "sh".into() };
    let mut comando = Command::new(shell);
    let mut anterior_saida = String::new();
    if let Ok(mut filho) = comando.args(["-c", anterior]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn() {
        if let Some(mut stdin) = filho.stdin.take() { let _ = stdin.write_all(entrada.as_bytes()); }
        if let Ok(saida) = filho.wait_with_output() { anterior_saida = String::from_utf8_lossy(&saida.stdout).into_owned(); }
    }
    print!("{anterior_saida}");
    // a status line anterior pode já chamar o blueprint (por exemplo uma cópia antiga dele): não repete a linha
    if let Some(linha) = statusline(&base(), &entrada).filter(|l| !l.is_empty() && !anterior_saida.contains(l.as_str())) { println!(" {linha}"); }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notificacao_de_espera_e_segura_e_independe_do_app() {
        let aviso = notificacao(r#"{"notification_type":"permission_prompt","message":"Preciso da sua resposta"}"#).unwrap();
        assert_eq!(aviso["terminalSequence"], "\u{1b}]9;The Blueprint · Preciso da sua resposta\u{7}");
        let maliciosa = notificacao(&json!({"message": "oi\u{7}\u{1b}]52;c;segredo\u{1b}\\"}).to_string()).unwrap();
        let sequencia = maliciosa["terminalSequence"].as_str().unwrap();
        assert_eq!(sequencia.chars().filter(|c| *c == '\u{1b}').count(), 1);
        assert_eq!(sequencia.chars().filter(|c| *c == '\u{7}').count(), 1);
        assert!(notificacao("{}").is_none());
        assert!(notificacao(r#"{"message":""}"#).is_none());
    }

    #[test]
    fn a_saida_tem_a_forma_que_o_claude_le() {
        let s = saida_do_claude(&json!({"vinculado": true, "contexto": "oi"})).unwrap();
        assert_eq!(s["hookSpecificOutput"]["hookEventName"], "SessionStart");
        assert_eq!(s["hookSpecificOutput"]["additionalContext"], "oi");
        assert!(saida_do_claude(&json!({"vinculado": false})).is_none());
    }

    #[test]
    fn sem_o_app_aberto_nao_imprime_e_nao_falha() {
        // porta que ninguém usa: a conexão é recusada na hora
        assert!(sessao("http://127.0.0.1:9", "{}").is_none());
        assert!(statusline("http://127.0.0.1:9", r#"{"cwd":"/p"}"#).is_none());
        assert!(status("http://127.0.0.1:9", "/p").contains("não está aberto"));
    }

    #[test]
    fn a_status_line_acha_a_pasta_no_json_da_sessao_e_ignora_lixo() {
        assert_eq!(pasta_da_entrada(&json!({"workspace": {"current_dir": "/a"}, "cwd": "/b"})), Some("/a"));
        assert_eq!(pasta_da_entrada(&json!({"cwd": "/b"})), Some("/b"));
        assert_eq!(pasta_da_entrada(&json!({})), None);
        assert!(statusline("http://127.0.0.1:9", "isto não é json").is_none());
    }

    /// Um app falso que atende um pedido: devolve (o pedido bruto, que a thread lê) e responde `resposta`.
    fn app_falso(resposta: &'static str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let ouvinte = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", ouvinte.local_addr().unwrap());
        let t = std::thread::spawn(move || {
            let (mut c, _) = ouvinte.accept().unwrap();
            let mut bruto = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = c.read(&mut buf).unwrap();
                bruto.extend_from_slice(&buf[..n]);
                let texto = String::from_utf8_lossy(&bruto).to_string();
                if let Some((cab, corpo)) = texto.split_once("\r\n\r\n") {
                    let tamanho = cab.lines().find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap())).unwrap_or(0);
                    if corpo.len() >= tamanho || n == 0 {
                        break;
                    }
                }
            }
            write!(c, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resposta}", resposta.len()).unwrap();
            String::from_utf8_lossy(&bruto).to_string()
        });
        (base, t)
    }

    #[test]
    fn o_hook_do_convidado_leva_o_blueprint_da_pasta_e_so_o_stop_responde() {
        let pasta = std::env::temp_dir().join(format!("blueprint-hook-equipe-{}", std::process::id()));
        std::fs::create_dir_all(pasta.join("sub")).unwrap();
        std::fs::write(pasta.join("CLAUDE.md"), "# x\nBlueprint: workspace_id `w2`, blueprint_id `p9`\n").unwrap();
        let entrada = json!({"session_id": "s", "cwd": pasta.join("sub").display().to_string(), "tool_input": {"file_path": "a.rs"}}).to_string();
        let corpo: Value = serde_json::from_str(&corpo_do_hook(entrada.clone(), true).unwrap()).unwrap();
        assert_eq!((corpo["workspaceId"].as_str(), corpo["blueprintId"].as_str()), (Some("w2"), Some("p9")), "a linha da pasta acima vale");
        assert_eq!(corpo_do_hook(entrada.clone(), false), Some(entrada), "no plugin local o corpo vai como veio");
        assert_eq!(corpo_do_hook(json!({"cwd": std::env::temp_dir().join("nada-aqui-blueprint").display().to_string()}).to_string(), true), None, "pasta sem a linha não manda nada");
        let (base, t) = app_falso("{}");
        assert!(repassar(&base, "edicao", &corpo.to_string()).is_none(), "a edição não imprime nada");
        let pedido = t.join().unwrap();
        assert!(pedido.starts_with("POST /api/mcp/hook/edicao ") && pedido.contains("\"blueprintId\":\"p9\""), "{pedido}");
        let (base, t) = app_falso(r#"{"hookSpecificOutput":{"hookEventName":"Stop","additionalContext":"registrar_trabalho"}}"#);
        assert_eq!(repassar(&base, "parada", "{}").unwrap()["hookSpecificOutput"]["additionalContext"], "registrar_trabalho");
        t.join().unwrap();
        assert!(repassar("http://127.0.0.1:9", "edicao", "{}").is_none(), "app fora do ar: silêncio");
        std::fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn o_trecho_da_status_line_usa_barras_e_aspas_no_caminho() {
        let t = trecho_da_status_line("C:\\Meus Apps\\blueprint.exe");
        assert_eq!(t["statusLine"]["command"], "\"C:/Meus Apps/blueprint.exe\" hook statusline");
        assert_eq!(t["statusLine"]["type"], "command");
    }
}
