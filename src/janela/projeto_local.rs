//! O que a janela da equipe pede ao app instalado (docs/app-no-projeto-compartilhado.md): `projeto <json>` pelo IPC do WebView2, só de página `https://`.
//! A página é do servidor do dono do workspace, então cada pedido é validado aqui, vira um prompt montado pelo APP, passa por uma pergunta nativa que mostra
//! o que vai rodar e só então abre o Claude do convidado (`blueprint claude --equipe`) numa pasta sob `%USERPROFILE%\Blueprint\Projetos`. Nenhum caminho,
//! comando ou texto livre da página chega ao terminal: só ids, a URL (github.com, `repositorio_valido`), um número e dois nomes limpos.
//!   configurar  clona o repositório do blueprint e liga a pasta a ele (o mesmo prompt de "Configurar projeto" do app local)
//!   iniciar     abre o Claude na pasta já ligada para executar uma ação
use crate::mcp::iniciar::{limpo, prompt_da_acao};
use crate::repositorio::{prompt, repositorio_valido, Repo};
use serde_json::Value;
use std::path::{Path, PathBuf};

const MAX_PEDIDO: usize = 4000;
/// A flag de `blueprint claude` que abre a sessão do convidado (cli/claude.rs).
const FLAG: &str = "--equipe";

#[derive(Debug, PartialEq, Eq)]
pub enum Pedido {
    Configurar { ws: String, bp: String, nome_bp: String, repo: Repo },
    Iniciar { ws: String, bp: String, nome_bp: String, repo: Repo, acao_id: String, numero: String, acao: String },
}

/// O que o app vai perguntar e fazer: abrir `terminal_em` com `prompt` (criando `Projetos` antes, se `criar_projetos`).
#[derive(Debug, PartialEq, Eq)]
pub struct Plano {
    pub pergunta: String,
    pub terminal_em: PathBuf,
    pub prompt: String,
    pub criar_projetos: bool,
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn numero_valido(n: &str) -> bool {
    !n.is_empty() && n.len() <= 12 && n.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// O JSON do IPC (depois do prefixo `projeto `) vira um pedido, ou a frase do motivo da recusa.
pub fn interpretar(corpo: &str) -> Result<Pedido, String> {
    if corpo.len() > MAX_PEDIDO {
        return Err(crate::tr!("The request is too large.", "O pedido é grande demais."));
    }
    let v: Value = serde_json::from_str(corpo).map_err(|_| crate::tr!("The request is not valid.", "O pedido não é válido."))?;
    let texto = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("");
    let (ws, bp) = (texto("workspaceId"), texto("blueprintId"));
    if !id_valido(ws) || !id_valido(bp) {
        return Err(crate::tr!("The request has invalid ids.", "O pedido tem ids inválidos."));
    }
    let Some(repo) = repositorio_valido(texto("repositorio")) else {
        return Err(crate::tr!("This blueprint has no valid GitHub repository (https://github.com/owner/repository).", "Este blueprint não tem um repositório do GitHub válido (https://github.com/dono/repositório)."));
    };
    let (ws, bp, nome_bp) = (ws.to_string(), bp.to_string(), limpo(texto("nome")));
    match texto("tipo") {
        "configurar" => Ok(Pedido::Configurar { ws, bp, nome_bp, repo }),
        "iniciar" if id_valido(texto("acaoId")) && numero_valido(texto("numero")) => {
            Ok(Pedido::Iniciar { ws, bp, nome_bp, repo, acao_id: texto("acaoId").into(), numero: texto("numero").into(), acao: limpo(texto("acao")) })
        }
        "iniciar" => Err(crate::tr!("The request has an invalid action.", "O pedido tem uma ação inválida.")),
        _ => Err(crate::tr!("The request is not valid.", "O pedido não é válido.")),
    }
}

/// `%USERPROFILE%\Blueprint\Projetos` (BLUEPRINT_PROJETOS troca, para os testes).
pub fn projetos_padrao() -> PathBuf {
    if let Some(p) = std::env::var_os("BLUEPRINT_PROJETOS").filter(|p| !p.is_empty()) {
        return PathBuf::from(p);
    }
    PathBuf::from(std::env::var_os("USERPROFILE").unwrap_or_else(|| ".".into())).join("Blueprint").join("Projetos")
}

/// O Claude do convidado só vale para o servidor a que ele foi conectado: a janela da equipe tem de ser a do mesmo endereço.
pub fn conferir_conexao(conectado_a: Option<&str>, origem: &str) -> Result<(), String> {
    let origem = origem.trim_end_matches('/');
    match conectado_a.map(|u| u.trim_end_matches('/')) {
        None => Err(crate::tr!("Connect your Claude to this team first (the Connect my Claude button on the team page).", "Conecte o seu Claude a esta equipe primeiro (o botão Conectar meu Claude na página da equipe).")),
        Some(u) if u != origem => Err(crate::tr!("Your Claude is connected to another server ({u}). Connect it to this team first (the Connect my Claude button).", "O seu Claude está conectado a outro servidor ({u}). Conecte-o a esta equipe primeiro (o botão Conectar meu Claude).")),
        Some(_) => Ok(()),
    }
}

/// O blueprint da linha `Blueprint:` que está NESTA pasta (e só nela): um `CLAUDE.md` de uma pasta acima não vale como "já configurado".
fn ligada_aqui(pasta: &Path) -> Option<(String, String)> {
    ["CLAUDE.local.md", "CLAUDE.md", "AGENTS.md"].iter().find_map(|n| std::fs::read_to_string(pasta.join(n)).ok()?.lines().find_map(crate::cli::claude::ids_da_linha))
}

pub fn planejar(pedido: &Pedido, projetos: &Path) -> Result<Plano, String> {
    match pedido {
        Pedido::Configurar { ws, bp, nome_bp, repo } => {
            let destino = projetos.join(&repo.nome);
            let (url, d) = (&repo.url, destino.display());
            Ok(Plano {
                pergunta: crate::tr!(
                    "Set up the project «{nome_bp}»?\n\nClaude will clone {url} into\n{d}\n(or update it if it is already there), install what the project asks for and link the folder to this workspace. It spends tokens.",
                    "Configurar o projeto «{nome_bp}»?\n\nO Claude vai clonar {url} em\n{d}\n(ou atualizar, se já estiver lá), instalar o que o projeto pede e ligar a pasta a este workspace. Gasta tokens."
                ),
                terminal_em: projetos.to_path_buf(),
                prompt: prompt::configurar(repo, &destino, nome_bp, ws, bp),
                criar_projetos: true,
            })
        }
        Pedido::Iniciar { ws, bp, nome_bp, repo, acao_id, numero, acao } => {
            let destino = projetos.join(&repo.nome);
            if !destino.is_dir() || ligada_aqui(&destino).as_ref().map(|(w, b)| (w.as_str(), b.as_str())) != Some((ws.as_str(), bp.as_str())) {
                return Err(crate::tr!(
                    "This computer has no folder for «{nome_bp}» yet. Use Set up project first.",
                    "Este computador ainda não tem a pasta de «{nome_bp}». Use Configurar projeto primeiro."
                ));
            }
            let d = destino.display();
            Ok(Plano {
                pergunta: crate::tr!(
                    "Open Claude in\n{d}\nto run action {numero} «{acao}» of «{nome_bp}»? It spends tokens.",
                    "Abrir o Claude em\n{d}\npara executar a ação {numero} «{acao}» de «{nome_bp}»? Gasta tokens."
                ),
                terminal_em: destino,
                prompt: prompt_da_acao(numero, acao, nome_bp, ws, bp, acao_id),
                criar_projetos: false,
            })
        }
    }
}

/// O fluxo inteiro com as pontas injetadas (os testes não abrem janela nem terminal): valida, confere a conexão, planeja, pergunta e abre.
pub fn atender(
    corpo: &str, origem: &str, conectado_a: Option<&str>, projetos: &Path, perguntar: &dyn Fn(&str) -> bool, avisar: &dyn Fn(&str),
    abrir: &dyn Fn(&Path, &str) -> Result<&'static str, String>,
) {
    let resultado = interpretar(corpo).and_then(|p| conferir_conexao(conectado_a, origem).and_then(|()| planejar(&p, projetos)));
    let plano = match resultado {
        Ok(p) => p,
        Err(motivo) => return avisar(&motivo),
    };
    if !perguntar(&plano.pergunta) {
        return;
    }
    if plano.criar_projetos {
        if let Err(e) = std::fs::create_dir_all(projetos) {
            let d = projetos.display();
            return avisar(&crate::tr!("Could not create {d}: {e}", "Não consegui criar {d}: {e}"));
        }
    }
    if let Err(e) = abrir(&plano.terminal_em, &plano.prompt) {
        avisar(&crate::tr!("Could not open the terminal: {e}", "Não consegui abrir o terminal: {e}"));
    }
}

/// Chamado pelo IPC da janela da equipe (`origem` é a da página que mandou, sempre https).
pub fn pedir(origem: String, corpo: String) {
    std::thread::spawn(move || {
        let conectado = crate::mcp::equipe::ler_de(&crate::mcp::equipe::arquivo()).map(|c| c.url);
        let Ok(exe) = std::env::current_exe() else { return };
        atender(
            &corpo, &origem, conectado.as_deref(), &projetos_padrao(), &super::equipe::perguntar, &super::equipe::avisar,
            &|pasta, prompt| crate::mcp::iniciar::abrir_terminal_com(pasta, &exe, prompt, FLAG),
        );
    });
}

#[cfg(test)]
mod tests;
