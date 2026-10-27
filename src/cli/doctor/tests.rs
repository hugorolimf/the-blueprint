//! Os cenários inteiros do `blueprint doctor` (tudo certo, plugin ausente, app fechado) e os ajudantes dos testes de cada conferência.
use super::*;
use crate::atualizador::VERSAO;
use crate::idioma::{com, Idioma};
use std::io::{Read, Write};
use std::path::Path;

/// Uma pasta temporária vazia, só deste teste (nunca o %APPDATA% real).
pub(super) fn temp(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-doctor-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Um banco em WAL (como o do app) com a tabela `pastas` da migração real e uma pasta ligada.
pub(super) fn banco_com_pasta(pasta_app: &Path, cwd: &str, ws: &str, bp: &str) -> PathBuf {
    let arquivo = pasta_app.join("blueprint.db");
    let conn = rusqlite::Connection::open(&arquivo).unwrap();
    conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(())).unwrap();
    conn.execute_batch(include_str!("../../dados/migracoes/003_pastas.sql")).unwrap();
    crate::dados::pastas::vincular(&conn, cwd, ws, bp).unwrap();
    drop(conn);
    arquivo
}

/// Um app falso que responde uma vez ao `GET /api/atualizacao` com a versão dada.
pub(super) fn servidor_falso(versao: &str) -> String {
    let ouvinte = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://127.0.0.1:{}", ouvinte.local_addr().unwrap().port());
    let corpo = format!(r#"{{"atual":"{versao}","fase":"parada"}}"#);
    std::thread::spawn(move || {
        let (mut s, _) = ouvinte.accept().unwrap();
        let mut buf = [0u8; 2048];
        let _ = s.read(&mut buf);
        let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}", corpo.len());
    });
    base
}

/// Um endereço em que nada escuta (o app fechado).
pub(super) fn porta_fechada() -> String {
    let ouvinte = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    format!("http://127.0.0.1:{}", ouvinte.local_addr().unwrap().port())
}

/// Tudo instalado numa pasta temporária: app (banco e porta), Claude com o plugin, Codex com o MCP e a pasta atual ligada.
fn ambiente(nome: &str, base: String) -> Ambiente {
    let raiz = temp(nome);
    let (pasta_app, claude, codex, cwd) = (raiz.join("Blueprint"), raiz.join(".claude"), raiz.join(".codex"), raiz.join("projeto"));
    for p in [&pasta_app, &claude.join("plugins"), &codex, &cwd] {
        std::fs::create_dir_all(p).unwrap();
    }
    banco_com_pasta(&pasta_app, &cwd.display().to_string(), "ws1", "bp1");
    let porta = base.rsplit(':').next().unwrap();
    std::fs::write(pasta_app.join("porta"), porta).unwrap();
    std::fs::write(claude.join("plugins/installed_plugins.json"), r#"{"plugins":{"blueprint@blueprint-local":[{"scope":"user"}]}}"#).unwrap();
    std::fs::write(codex.join("config.toml"), "[mcp_servers.blueprint]\ncommand = 'blueprint.exe'\n").unwrap();
    Ambiente { base, pasta_app, claude_json: raiz.join(".claude.json"), claude, codex, cwd, instancia: true }
}

/// Todos os arquivos (com tamanho) abaixo de `raiz`, para provar que o doctor não grava nada.
fn retrato(raiz: &Path) -> Vec<(PathBuf, u64)> {
    let mut lista = Vec::new();
    for e in std::fs::read_dir(raiz).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            lista.extend(retrato(&p));
        } else {
            lista.push((p.clone(), e.metadata().unwrap().len()));
        }
    }
    lista.sort();
    lista
}

fn estados(itens: &[Item]) -> Vec<Estado> {
    itens.iter().map(|i| i.estado).collect()
}

#[test]
fn tudo_certo_e_tudo_ok_sem_gravar_nada() {
    let amb = ambiente("tudo-certo", servidor_falso(VERSAO));
    let raiz = amb.pasta_app.parent().unwrap().to_path_buf();
    let antes = retrato(&raiz);
    let itens = conferir(&amb);
    assert_eq!(estados(&itens), vec![Estado::Ok; 6], "{itens:#?}");
    assert_eq!(retrato(&raiz), antes, "o doctor criou ou mudou arquivos");
    com(Idioma::En, || assert!(texto(&itens).ends_with("Nothing to fix.")));
}

#[test]
fn plugin_ausente_falha_so_nesse_item() {
    let amb = ambiente("plugin-ausente", servidor_falso(VERSAO));
    std::fs::remove_file(amb.claude.join("plugins/installed_plugins.json")).unwrap();
    let itens = conferir(&amb);
    let falhas: Vec<&Item> = itens.iter().filter(|i| i.estado == Estado::Falha).collect();
    assert_eq!(falhas.len(), 1, "{itens:#?}");
    assert!(falhas[0].texto.contains("blueprint registrar"));
}

#[test]
fn app_fechado_falha_no_app_e_ainda_confere_o_resto() {
    let mut amb = ambiente("app-fechado", porta_fechada());
    amb.instancia = false;
    let itens = conferir(&amb);
    assert_eq!(estados(&itens), vec![Estado::Falha, Estado::Ok, Estado::Ok, Estado::Ok, Estado::Ok, Estado::Ok], "{itens:#?}");
    com(Idioma::En, || {
        let t = texto(&itens);
        assert!(t.contains("FAIL") && t.contains("1 item(s) to fix"), "{t}");
    });
}

#[test]
fn o_relatorio_tem_uma_linha_por_item_no_idioma_da_pessoa() {
    let itens = vec![Item::novo("A".into(), Estado::Ok, "x".into()), Item::novo("B".into(), Estado::Aviso, "y".into())];
    let pt = com(Idioma::Pt, || texto(&itens));
    assert!(pt.contains("  OK    A: x") && pt.contains("  AVISO B: y") && pt.ends_with("Nada a corrigir."), "{pt}");
    let en = com(Idioma::En, || texto(&itens));
    assert!(en.contains("  WARN  B: y") && en.starts_with("The Blueprint doctor"), "{en}");
}
