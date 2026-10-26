//! `POST /api/mcp/iniciar`: o usuário clicou em "Iniciar com o Claude" numa ação (contrato em docs/claude-na-folha.md). Abre um terminal NOVO na pasta do
//! blueprint rodando `blueprint claude "<prompt>"`: a sessão nova começa a ação. O prompt vai como argumento separado (nada de linha de comando montada por
//! texto) e nunca pelo monitor/canal. A rota é só loopback (rotas.rs) e só confirmada pelo usuário na página; aqui nada grava nos dados.
use super::acoes;
use super::hooks::{texto, travado};
use super::status::lista;
use super::Mcp;
use crate::blueprint::numeros_das_acoes;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};
use std::cell::RefCell;
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
        return Err(falha(400, crate::tr!("Send workspaceId, blueprintId and acaoId.", "Informe workspaceId, blueprintId e acaoId.")).codigo("faltam-ids"));
    }
    let (pasta, prompt) = {
        let mut d = travado(dados);
        let tudo = d.ler_tudo(&Contexto::local())?;
        let n = acoes::numero_do_blueprint(tudo, ws, bp);
        let estrutura = acoes::blueprint(tudo, ws, bp, None)?; // 404 de workspace ou blueprint que não existe
        let p = &estrutura["blueprint"];
        let acoes = lista(&p["acoes"]);
        let i = acoes.iter().position(|a| texto(a, "id") == acao_id).ok_or_else(|| falha(404, crate::tr!("Action not found in this blueprint.", "Ação não encontrada neste blueprint.")).codigo("acao-nao-encontrada"))?;
        let Some(pasta) = d.pasta_do_blueprint(&Contexto::local(), ws, bp)?.filter(|c| Path::new(c).is_dir()) else {
            return Ok(recusa(409, &crate::tr!("This blueprint has no linked folder (or it no longer exists).", "Este blueprint não tem uma pasta ligada (ou ela não existe mais)."), "sem-pasta"));
        };
        // o pedido segue o idioma de quem clicou (o agente responde nele); os nomes das ferramentas não mudam
        let (numero, acao, nome_bp) = (&numeros_das_acoes(p, n)[i], limpo(texto(&acoes[i], "nome")), limpo(texto(p, "nome")));
        let prompt = crate::tr!(
            "Execute action {numero} «{acao}» of blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}, acao_id {acao_id}): read it with ler_blueprint, report progress with registrar_progresso and complete each plan with marcar_concluido and the evidence.",
            "Execute a ação {numero} «{acao}» do blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}, acao_id {acao_id}): leia com ler_blueprint, registre o progresso com registrar_progresso e conclua cada plano com marcar_concluido e a evidência."
        );
        (pasta, prompt)
    };
    let exe = std::env::current_exe().map_err(|e| falha(500, crate::tr!("cannot tell where this program is: {e}", "não sei onde este programa está: {e}")))?;
    Ok(match lancar(Path::new(&pasta), &exe, &prompt) {
        Ok(terminal) => (200, json!({"ok": true, "pasta": pasta, "terminal": terminal})),
        Err(e) => recusa(501, &e, "sem-terminal"),
    })
}

/// `sessaoId` no corpo: em vez de abrir um terminal, o pedido vai a uma sessão do Claude já aberta, pelo canal dela (docs/claude-na-folha.md, "Escolher a sessão"). Com
/// `semCanal: true` nada é entregue: a resposta só traz o `prompt` para a página copiar. A resposta sempre traz o `prompt`; os erros da entrega (sessão que caiu,
/// de outra pasta) saem com o status e o código deles, no lugar do `sem-terminal`.
pub(super) fn em_sessao(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<(u16, Value), ErroHttp> {
    let sessao = texto(corpo, "sessaoId");
    if !id_valido(sessao) {
        return Err(falha(400, crate::tr!("invalid sessaoId.", "sessaoId inválido.")).codigo("sessao-invalida"));
    }
    let (ws, bp, copiar) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"), corpo["semCanal"] == true);
    let (pedido, recusa_da_entrega) = (RefCell::new(String::new()), RefCell::new(None));
    let lancar = |_: &Path, _: &Path, prompt: &str| -> Result<&'static str, String> {
        *pedido.borrow_mut() = prompt.to_string();
        if copiar {
            return Ok("copiar");
        }
        mcp.entregar_pedido(ws, bp, sessao, prompt).map(|_| "canal").map_err(|e| {
            *recusa_da_entrega.borrow_mut() = Some(e);
            e.2.to_string()
        })
    };
    let (mut status, mut resposta) = iniciar(dados, corpo, &lancar)?;
    if let Some((s, codigo, frase)) = recusa_da_entrega.take() {
        (status, resposta) = recusa(s, frase, codigo);
    }
    if !pedido.borrow().is_empty() {
        resposta["prompt"] = json!(pedido.borrow().as_str());
    }
    Ok((status, resposta))
}

/// O lançador de verdade: Windows Terminal (`wt -d <pasta> <exe> claude <prompt> --canal`; `--canal` liga o canal da sessão, ver `cli/claude.rs`) se existir no PATH; senão um console novo, direto no processo (sem passar pelo
/// cmd.exe, que leria `&` e `|` dos nomes). Não espera o filho. shortcut: só Windows (o app é só Windows); em outro sistema as flags não existem e abre sem elas.
pub(super) fn abrir_terminal(pasta: &Path, exe: &Path, prompt: &str) -> Result<&'static str, String> {
    let mut falha_wt = String::new();
    if let Some(wt) = crate::registrar::procurar("wt") {
        let mut c = Command::new(&wt);
        c.arg("-d").arg(pasta).arg(exe).arg("claude").arg(prompt).arg("--canal");
        match sem_janela(&mut c).spawn() {
            Ok(_) => return Ok("wt"),
            Err(e) => falha_wt = format!("wt ({}): {e}; ", wt.display()),
        }
    }
    let mut c = Command::new(exe);
    c.arg("claude").arg(prompt).arg("--canal").current_dir(pasta);
    novo_console(&mut c).spawn().map(|_| "cmd").map_err(|e| format!("{falha_wt}console novo ({}) em {}: {e}", exe.display(), pasta.display()))
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
