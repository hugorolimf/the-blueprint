//! Linha de comando. A janela nativa (sem argumentos) entra na ação 3.5.
mod atualizar;
mod claude;
use crate::dados::arquivo::{exportar_arquivo, importar_arquivo};
use crate::http::{self, Config};
use std::path::Path;

const USO: &str = concat!(
    "uso:\n",
    "  blueprint                                 abre o app (janela, bandeja e servidor); os dados ficam em %APPDATA%\\Blueprint\n",
    "  blueprint dev                              o app para desenvolver: interface lida da pasta do projeto, F12 e recarga ao salvar (feche o app antes)\n",
    "  blueprint serve                          liga o servidor (HOST, PORT, BLUEPRINT_DB, DADOS_ARQ, PROTOTIPOS_DIR, MCP_REDE)\n",
    "  blueprint --mcp                             servidor MCP por stdio, para o agente (claude mcp add blueprint -- blueprint --mcp)\n",
    "  blueprint watch                             acompanha avisos do blueprint desta pasta (BLUEPRINT_URL)\n",
    "  blueprint claude [args]                    abre o Claude Code com o plugin; ativa o agente se a pasta estiver ligada\n",
    "  blueprint importar <data.json> <banco.db>   copia o data.json para o banco (o arquivo não é alterado)\n",
    "  blueprint exportar <banco.db> <saida.json>  grava o banco como data.json\n",
    "  blueprint hook sessao|statusline|status|notificacao  comandos do Claude Code registrados pelo plugin\n",
    "  blueprint hook statusline-config           imprime o trecho de settings.json que liga a status line (opt-in)\n",
    "  blueprint atualizar [--verificar|--instalar]  procura e baixa a versão nova (--verificar só consulta; --instalar instala a baixada: é o seu clique em Atualizar)\n",
    "  blueprint --version                       mostra a versão\n",
    "  blueprint registrar [--remover]           instala (ou remove) o plugin no Claude Code (skill, agente e MCP) e o MCP no Codex; o instalador faz isto",
);

/// Devolve o código de saída do processo.
pub fn executar(args: Vec<String>) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let resultado = match a.as_slice() {
        [] => return abrir_janela(false),
        ["dev"] => return abrir_janela(true),
        ["--mcp"] => return crate::mcp::stdio::executar(),
        ["watch"] => return crate::watch::executar(),
        ["claude", ..] => return claude::executar(&args[1..]),
        ["atualizar"] | ["atualizar", "--verificar" | "--instalar"] => return atualizar::executar(a.get(1).copied()),
        ["--version"] | ["-V"] => {
            println!("blueprint {}", crate::atualizador::VERSAO);
            return 0;
        }
        ["serve"] => serve(),
        ["hook", evento] => return crate::hook::executar(evento),
        ["hook", "statusline", "--anterior", comando] => return crate::hook::combinar(comando),
        ["registrar", "--status-line"] | ["registrar", "--status-line", "--remover"] => {
            let (texto, codigo) = crate::registrar::statusline::executar(a.len() == 3);
            println!("{texto}");
            return codigo;
        }
        ["registrar"] | ["registrar", "--remover"] => {
            let (texto, codigo) = crate::registrar::executar(a.len() == 2);
            println!("{texto}");
            return codigo;
        }
        ["importar", json, banco] => importar_arquivo(Path::new(json), Path::new(banco)).map(|c| format!("importado: {c}")),
        ["exportar", banco, saida] => exportar_arquivo(Path::new(banco), Path::new(saida)).map(|_| format!("exportado para {saida}")),
        _ => {
            eprintln!("{USO}");
            return 2;
        }
    };
    match resultado {
        Ok(mensagem) => {
            println!("{mensagem}");
            0
        }
        Err(erro) => {
            eprintln!("erro: {erro}");
            1
        }
    }
}

/// Sem argumentos: o app com janela (Windows).
#[cfg(windows)]
fn abrir_janela(dev: bool) -> i32 {
    crate::janela::abrir(dev)
}

#[cfg(not(windows))]
fn abrir_janela(_dev: bool) -> i32 {
    eprintln!("{USO}");
    2
}

/// Liga o servidor e fica atendendo até o processo ser encerrado.
fn serve() -> Result<String, crate::erro::Erro> {
    let servidor = http::iniciar(Config::do_ambiente())?;
    println!("The Blueprint (beta) em {} · MCP: {} --mcp", servidor.url, servidor.estado.config.exe.display());
    servidor.aguardar();
    Ok("servidor encerrado".into())
}
