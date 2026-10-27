//! Linha de comando. A janela nativa (sem argumentos) entra na ação 3.5.
mod atualizar;
pub(crate) mod claude;
mod completar;
mod doctor;
mod restaurar;
use crate::dados::arquivo::{exportar_arquivo, importar_arquivo};
use crate::http::{self, Config};
use std::path::Path;

pub(crate) const USO: &str = concat!(
    "uso:\n",
    "  blueprint                                 abre o app (janela, bandeja e servidor); os dados ficam em %APPDATA%\\Blueprint\n",
    "  blueprint blueprint://convite?u=<endereço>&s=<segredo>  o link de convite da equipe, entregue pelo navegador (o instalador registra o protocolo)\n",
    "  blueprint dev                            o app para desenvolver: interface lida da pasta do projeto, F12 e recarga ao salvar (feche o app antes)\n",
    "  blueprint serve                          liga o servidor (HOST, PORT, BLUEPRINT_DB, DADOS_ARQ, PROTOTIPOS_DIR, MCP_REDE)\n",
    "  blueprint serve --hospedar [--novo-link-dono]  servidor de equipe atrás de um proxy HTTPS (BLUEPRINT_URL_PUBLICA; veja docs/hospedar.md)\n",
    "  blueprint --mcp                             servidor MCP por stdio, para o agente (claude mcp add blueprint -- blueprint --mcp)\n",
    "  blueprint watch                             acompanha avisos do blueprint desta pasta (BLUEPRINT_URL)\n",
    "  blueprint claude [args]                    abre o Claude Code com o plugin; ativa o agente se a pasta estiver ligada\n",
    "  blueprint importar <data.json> <banco.db>   copia o data.json para o banco (o arquivo não é alterado)\n",
    "  blueprint exportar <banco.db> <saida.json>  grava o banco como data.json\n",
    "  blueprint exportar <banco.db> <pasta> --markdown  grava um índice e um arquivo Markdown por ação de cada blueprint (só leitura)\n",
    "  blueprint hook sessao|statusline|status|notificacao  comandos do Claude Code registrados pelo plugin\n",
    "  blueprint hook sessao|edicao|subagente|parada|fim --equipe  os hooks do plugin do convidado (blueprint conectar)\n",
    "  blueprint hook statusline-config           imprime o trecho de settings.json que liga a status line (opt-in)\n",
    "  blueprint atualizar [--verificar|--instalar]  procura e baixa a versão nova (--verificar só consulta; --instalar instala a baixada: é o seu clique em Atualizar)\n",
    "  blueprint restaurar [<número>|<arquivo>]  lista as cópias do banco (uma automática por dia, as 7 últimas) ou volta a uma; feche o app antes
",
    "  blueprint doctor                          confere o app, a porta, o banco, o plugin do Claude Code, o MCP do Codex e a pasta, e diz o que fazer (só lê)
",
    "  blueprint completions <powershell|bash>    imprime o script que completa os comandos no terminal\n",
    "  blueprint --version                       mostra a versão\n",
    "  blueprint registrar [--remover]           instala (ou remove) o plugin no Claude Code (skill, agente e MCP) e o MCP no Codex; o instalador faz isto\n",
    "  blueprint conectar <endereço> <chave>      liga o seu Claude (e o Codex) a um workspace compartilhado com você; rode de novo se o endereço mudar\n",
    "  blueprint conectar --remover              desfaz o conectar",
);

/// Devolve o código de saída do processo.
pub fn executar(args: Vec<String>) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let resultado = match a.as_slice() {
        [] => return abrir_janela(false, None),
        ["dev"] => return abrir_janela(true, None),
        [link] if link.starts_with("blueprint:") => return abrir_janela(false, Some(link.to_string())), // o navegador entrega o convite ao app instalado
        ["--mcp"] => return crate::mcp::stdio::executar(),
        ["--mcp", "--equipe"] => {
            crate::mcp::equipe::ligar();
            return crate::mcp::stdio::executar();
        }
        ["hook", evento, "--equipe"] => {
            crate::mcp::equipe::ligar();
            return crate::hook::executar(evento);
        }
        ["conectar", "--remover"] => {
            let (texto, codigo) = crate::registrar::conectar::remover();
            println!("{texto}");
            return codigo;
        }
        ["conectar", endereco, chave] => {
            let (texto, codigo) = crate::registrar::conectar::executar(endereco, chave);
            println!("{texto}");
            return codigo;
        }
        ["watch"] => return crate::watch::executar(),
        ["doctor"] => return doctor::executar(),
        ["claude", ..] => return claude::executar(&args[1..]),
        #[cfg(windows)]
        ["atualizando", ..] => return crate::janela::atualizando::executar(&args[1..]), // interno: a janela de progresso que o app abre ao atualizar
        ["atualizar"] | ["atualizar", "--verificar" | "--instalar"] => return atualizar::executar(a.get(1).copied()),
        ["restaurar"] | ["restaurar", _] => return restaurar::executar(a.get(1).copied()),
        ["--version"] | ["-V"] => {
            println!("blueprint {}", crate::atualizador::VERSAO);
            return 0;
        }
        ["completions", shell] => match completar::script(shell) {
            Some(texto) => {
                print!("{texto}");
                return 0;
            }
            None => {
                eprintln!("shell desconhecido: use powershell ou bash");
                return 2;
            }
        },
        ["serve"] if std::env::var("BLUEPRINT_HOSPEDAR").as_deref() == Ok("1") => hospedar(false),
        ["serve"] => serve(),
        ["serve", "--hospedar"] => hospedar(false),
        ["serve", "--hospedar", "--novo-link-dono"] | ["serve", "--novo-link-dono", "--hospedar"] => hospedar(true),
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
        ["exportar", banco, pasta, "--markdown"] => {
            crate::dados::markdown::exportar_markdown(Path::new(banco), Path::new(pasta)).map(|n| format!("{n} arquivos Markdown em {pasta}"))
        }
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
fn abrir_janela(dev: bool, link: Option<String>) -> i32 {
    crate::janela::abrir(dev, link)
}

#[cfg(not(windows))]
fn abrir_janela(_dev: bool, _link: Option<String>) -> i32 {
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

/// `blueprint serve --hospedar`: o servidor de equipe (docs/hospedar.md). Imprime o link do dono uma vez (docs/colaboracao.md, seção 4).
fn hospedar(novo_link_dono: bool) -> Result<String, crate::erro::Erro> {
    let config = Config::hospedado(novo_link_dono).map_err(crate::erro::Erro::Formato)?;
    let url_publica = config.hospedagem.as_ref().map(|h| h.url_publica.clone()).unwrap_or_default();
    let servidor = http::iniciar(config)?;
    println!("{}", crate::tr!("The Blueprint (beta) hosted at {url_publica} (listening on {})", "The Blueprint (beta) hospedado em {url_publica} (escutando em {})", servidor.url));
    let link = crate::acesso::convites::link_dono(&mut servidor.estado.dados.lock().unwrap_or_else(|e| e.into_inner()), &url_publica, novo_link_dono, crate::tempo::agora_ms())?;
    match link {
        crate::acesso::convites::LinkDono::Novo(link) => {
            println!("{}", crate::tr!("Owner link (valid for 24 h, single use; it is printed only now):", "Link do dono (vale 24 h, uso único; só é impresso agora):"));
            println!("  {link}");
        }
        crate::acesso::convites::LinkDono::JaImpresso => println!("{}", crate::tr!("the owner link was already printed; for a new one use --novo-link-dono", "o link do dono já foi impresso; para um novo use --novo-link-dono")),
        crate::acesso::convites::LinkDono::TemDono => {}
    }
    servidor.aguardar();
    Ok("servidor encerrado".into())
}
