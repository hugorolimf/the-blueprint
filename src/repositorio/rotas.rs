//! `/api/repositorio/*`: só esta máquina (Host e Origin em loopback, como o terminal, porque `configurar` abre processo) e JSON em todo POST.
//! O repositório vem sempre dos DADOS do blueprint, nunca do corpo do pedido, e passa por `repositorio_valido` antes de qualquer uso.
//! `configurar` só vem do clique confirmado no diálogo Configurar projeto (web/js/repositorio.js).
use super::acesso::{self, Acesso};
use super::{prompt, repositorio_valido};
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::http::Estado;
use crate::terminal::rotas::{numero, texto};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub fn rota(estado: &Estado, pedido: &Pedido) -> Resposta {
    if !crate::mcp::permissao::so_loopback(&estado.mcp, pedido) {
        return Resposta::texto(403, "text/plain", crate::tr!("the repository setup only works on this machine", "a configuração do repositório só funciona nesta máquina"));
    }
    executar(estado, pedido).unwrap_or_else(|e| e.resposta())
}

fn executar(estado: &Estado, pedido: &Pedido) -> Result<Resposta, ErroHttp> {
    if pedido.metodo != "POST" {
        return Ok(Resposta::texto(405, "text/plain", crate::tr!("method not allowed", "método não permitido")));
    }
    if !pedido.eh_json() {
        return Ok(Resposta::texto(415, "text/plain", crate::tr!("send application/json", "envie application/json")));
    }
    let corpo: Value = serde_json::from_slice(&pedido.corpo).map_err(|e| falha(400, crate::tr!("invalid JSON: {e}", "JSON inválido: {e}")).codigo("json-invalido"))?;
    Ok(match pedido.caminho.trim_start_matches("/api/repositorio/") {
        "verificar" => {
            let guardado = guardado(&estado.dados, &corpo)?.0;
            Resposta::json(200, &situacao(guardado.as_deref(), &acesso::conferir))
        }
        "configurar" => {
            let (colunas, linhas) = (numero(&corpo, "colunas", 100), numero(&corpo, "linhas", 30));
            let info = crate::terminal::Info {
                titulo: texto(&corpo, "titulo").chars().filter(|c| !c.is_control()).take(80).collect(),
                workspace_id: texto(&corpo, "workspaceId").into(),
                blueprint_id: texto(&corpo, "blueprintId").into(),
                acao_id: String::new(),
            };
            let aberta = RefCell::new(None);
            let lancar = |pasta: &Path, exe: &Path, prompt: &str| -> Result<&'static str, String> {
                if texto(&corpo, "terminal") == "externo" {
                    return crate::mcp::iniciar::abrir_terminal(pasta, exe, prompt);
                }
                *aberta.borrow_mut() = Some(crate::terminal::rotas::abrir_no_app(estado, info.clone(), (colunas, linhas), pasta, exe, prompt)?);
                Ok("app")
            };
            let (status, mut resposta) = configurar(&estado.dados, &corpo, &lancar)?;
            if let Some(s) = aberta.take() {
                resposta["sessao"] = s;
            }
            Resposta::json(status, &resposta)
        }
        _ => Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado")),
    })
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.chars().count() <= 100 && !id.chars().any(char::is_control)
}

/// A URL guardada no blueprint (se houver) e o nome dele, lidos dos dados. 404 se o workspace ou o blueprint não existe.
fn guardado(dados: &Mutex<Dados>, corpo: &Value) -> Result<(Option<String>, String, String, String), ErroHttp> {
    let (ws, bp) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"));
    if !id_valido(ws) || !id_valido(bp) {
        return Err(falha(400, crate::tr!("Send workspaceId and blueprintId.", "Informe workspaceId e blueprintId.")).codigo("faltam-ids"));
    }
    let mut d = dados.lock().unwrap_or_else(|e| e.into_inner()); // solta antes de qualquer processo: o git pode demorar
    let tudo = d.ler_tudo(&Contexto::local())?;
    let estrutura = crate::mcp::acoes::blueprint(tudo, ws, bp, None)?;
    let p = &estrutura["blueprint"];
    Ok((p["repositorio"]["url"].as_str().map(String::from), crate::mcp::iniciar::limpo(p["nome"].as_str().unwrap_or("")), ws.into(), bp.into()))
}

/// O estado do acesso à vista da página (docs/repositorio-github.md); `conferir` é o `git ls-remote` de verdade, ou um falso nos testes.
pub(crate) fn situacao(guardado: Option<&str>, conferir: &dyn Fn(&str) -> Acesso) -> Value {
    let Some(texto) = guardado else { return json!({"estado": "sem-repositorio"}) };
    let Some(repo) = repositorio_valido(texto) else { return json!({"estado": "url-invalida"}) };
    json!({"estado": conferir(&repo.url).codigo(), "url": repo.url, "nome": repo.nome})
}

fn recusa(status: u16, erro: String, codigo: &str) -> (u16, Value) {
    (status, json!({"erro": erro, "codigo": codigo}))
}

/// A pasta de destino: absoluta, existente, sem controle nem `;` (o Windows Terminal lê `;` como separador de comandos).
fn pasta_valida(p: &str) -> Option<PathBuf> {
    let ok = !p.trim().is_empty() && p.chars().count() <= 500 && !p.chars().any(|c| c.is_control() || c == ';');
    Some(PathBuf::from(p.trim())).filter(|c| ok && c.is_absolute() && c.is_dir())
}

/// `lancar` abre o terminal na pasta (1º argumento) rodando o exe (2º) com `claude` e o prompt (3º); os testes passam um falso.
pub(crate) fn configurar(dados: &Mutex<Dados>, corpo: &Value, lancar: &dyn Fn(&Path, &Path, &str) -> Result<&'static str, String>) -> Result<(u16, Value), ErroHttp> {
    let (url, nome_bp, ws, bp) = guardado(dados, corpo)?;
    let Some(url) = url else { return Ok(recusa(409, crate::tr!("This blueprint has no GitHub repository.", "Este blueprint não tem um repositório do GitHub."), "sem-repositorio")) };
    let Some(repo) = repositorio_valido(&url) else { return Ok(recusa(409, crate::tr!("The repository address saved in this blueprint is not valid.", "O endereço do repositório guardado neste blueprint não é válido."), "url-invalida")) };
    let Some(pasta) = pasta_valida(texto(corpo, "pasta")) else {
        return Ok(recusa(400, crate::tr!("Choose a folder that already exists (a full path, without `;`).", "Escolha uma pasta que já existe (o caminho completo, sem `;`)."), "pasta-invalida"));
    };
    let destino = pasta.join(&repo.nome);
    let exe = std::env::current_exe().map_err(|e| falha(500, crate::tr!("cannot tell where this program is: {e}", "não sei onde este programa está: {e}")))?;
    let pedido = prompt::configurar(&repo, &destino, &nome_bp, &ws, &bp);
    Ok(match lancar(&pasta, &exe, &pedido) {
        Ok(terminal) => (200, json!({"ok": true, "pasta": pasta.to_string_lossy(), "destino": destino.to_string_lossy(), "terminal": terminal})),
        Err(e) => recusa(501, e, "sem-terminal"),
    })
}
