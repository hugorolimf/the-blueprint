//! `POST /api/mcp/iniciar`: o usuário clicou em "Iniciar com o Claude" numa ação (contrato em docs/claude-na-folha.md). Abre um terminal NOVO na pasta do
//! blueprint rodando `blueprint claude "<prompt>"`: a sessão nova começa a ação. O prompt vai como argumento separado (nada de linha de comando montada por
//! texto) e nunca pelo monitor/canal. A rota é só loopback (rotas.rs) e só confirmada pelo usuário na página; aqui nada grava nos dados.
use super::acoes;
use super::hooks::{texto, travado};
use super::status::lista;
use crate::blueprint::numeros_das_acoes;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

/// O texto que vai ao prompt vem dos dados (qualquer agente propõe nomes): sem controle, sem `;` (o Windows Terminal o lê como separador de comandos) e curto.
pub(super) fn limpo(t: &str) -> String {
    t.chars().filter(|c| !c.is_control()).map(|c| if c == ';' { ',' } else { c }).take(80).collect()
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.chars().count() <= 100 && !id.chars().any(char::is_control)
}

fn recusa(status: u16, erro: &str, codigo: &str) -> (u16, Value) {
    (status, json!({"erro": erro, "codigo": codigo}))
}

/// `lancar` abre o terminal na pasta (1º argumento) rodando o exe (2º) com `claude` e o prompt (3º) e devolve qual abriu ("wt" ou "cmd"); os testes passam um falso.
pub(super) fn iniciar(dados: &Mutex<Dados>, corpo: &Value, lancar: &dyn Fn(&Path, &Path, &str) -> Result<&'static str, String>) -> Result<(u16, Value), ErroHttp> {
    let (ws, bp, acao_id) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"), texto(corpo, "acaoId"));
    if ![ws, bp, acao_id].iter().all(|i| id_valido(i)) {
        return Err(falha(400, "Informe workspaceId, blueprintId e acaoId."));
    }
    let (pasta, prompt) = {
        let mut d = travado(dados);
        let tudo = d.ler_tudo(&Contexto::local())?;
        let n = acoes::numero_do_blueprint(tudo, ws, bp);
        let estrutura = acoes::blueprint(tudo, ws, bp, None)?; // 404 de workspace ou blueprint que não existe
        let p = &estrutura["blueprint"];
        let acoes = lista(&p["acoes"]);
        let i = acoes.iter().position(|a| texto(a, "id") == acao_id).ok_or_else(|| falha(404, "Ação não encontrada neste blueprint."))?;
        let Some(pasta) = d.pasta_do_blueprint(&Contexto::local(), ws, bp)?.filter(|c| Path::new(c).is_dir()) else {
            return Ok(recusa(409, "Este blueprint não tem uma pasta ligada (ou ela não existe mais).", "sem-pasta"));
        };
        let prompt = format!(
            "Execute a ação {} «{}» do blueprint «{}» (workspace_id {ws}, blueprint_id {bp}, acao_id {acao_id}): leia com ler_blueprint, registre o progresso com registrar_progresso e conclua cada plano com marcar_concluido e a evidência.",
            numeros_das_acoes(p, n)[i],
            limpo(texto(&acoes[i], "nome")),
            limpo(texto(p, "nome"))
        );
        (pasta, prompt)
    };
    let exe = std::env::current_exe().map_err(|e| falha(500, format!("não sei onde este programa está: {e}")))?;
    Ok(match lancar(Path::new(&pasta), &exe, &prompt) {
        Ok(terminal) => (200, json!({"ok": true, "pasta": pasta, "terminal": terminal})),
        Err(e) => recusa(501, &e, "sem-terminal"),
    })
}

/// O lançador de verdade: Windows Terminal (`wt -d <pasta> <exe> claude <prompt>`) se existir no PATH; senão um console novo, direto no processo (sem passar pelo
/// cmd.exe, que leria `&` e `|` dos nomes). Não espera o filho. shortcut: só Windows (o app é só Windows); em outro sistema as flags não existem e abre sem elas.
pub(super) fn abrir_terminal(pasta: &Path, exe: &Path, prompt: &str) -> Result<&'static str, String> {
    if let Some(wt) = crate::registrar::procurar("wt") {
        let mut c = Command::new(wt);
        c.arg("-d").arg(pasta).arg(exe).arg("claude").arg(prompt);
        if sem_janela(&mut c).spawn().is_ok() {
            return Ok("wt");
        }
    }
    let mut c = Command::new(exe);
    c.arg("claude").arg(prompt).current_dir(pasta);
    novo_console(&mut c).spawn().map(|_| "cmd").map_err(|e| format!("não consegui abrir um terminal: {e}"))
}

#[cfg(windows)]
fn sem_janela(c: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0800_0000) // CREATE_NO_WINDOW: o wt abre a janela dele; o app não prende console
}
#[cfg(windows)]
fn novo_console(c: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0000_0010) // CREATE_NEW_CONSOLE
}
#[cfg(not(windows))]
fn sem_janela(c: &mut Command) -> &mut Command {
    c
}
#[cfg(not(windows))]
fn novo_console(c: &mut Command) -> &mut Command {
    c
}
