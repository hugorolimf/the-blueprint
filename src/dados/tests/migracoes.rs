use crate::dados::{migracoes, Dados};
use rusqlite::Connection;

#[test]
fn cria_o_banco_do_zero_pelas_migracoes() {
    let mut conn = Connection::open_in_memory().unwrap();
    let aplicadas = migracoes::aplicar(&mut conn).unwrap();
    assert_eq!(aplicadas, vec!["001_inicial", "002_vocabulario", "003_pastas", "004_linha_do_tempo", "005_acesso", "006_historico", "007_tempos_planos", "008_fila_mcp"]);
    assert_eq!(
        migracoes::tabelas(&conn).unwrap(),
        vec!["acessos", "acoes", "atividade", "blueprints", "chaves_mcp", "checkpoints", "comentarios", "convites", "fila_mcp", "fila_mcp_contadores", "historico_campos", "linha_do_tempo", "membros", "migracoes", "pastas", "planos", "prototipos", "sessoes", "tempos_planos", "workspaces"]
    );
}

#[test]
fn aplicar_de_novo_nao_faz_nada() {
    let mut conn = Connection::open_in_memory().unwrap();
    migracoes::aplicar(&mut conn).unwrap();
    assert!(migracoes::aplicar(&mut conn).unwrap().is_empty());
}

#[test]
fn toda_linha_tem_revisao_e_atualizado_em() {
    let d = Dados::em_memoria().unwrap();
    for tabela in ["workspaces", "blueprints", "acoes", "planos", "checkpoints", "comentarios", "prototipos"] {
        let colunas: Vec<String> = d
            .conexao()
            .prepare(&format!("SELECT name FROM pragma_table_info('{tabela}')"))
            .unwrap()
            .query_map([], |l| l.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for esperada in ["id", "ordem", "revisao", "atualizado_em"] {
            assert!(colunas.iter().any(|c| c == esperada), "{tabela} sem {esperada}");
        }
    }
}

#[test]
fn o_banco_em_arquivo_usa_wal() {
    let caminho = std::env::temp_dir().join(format!("blueprint-wal-{}.db", std::process::id()));
    let d = Dados::abrir(&caminho).unwrap();
    let modo: String = d.conexao().query_row("PRAGMA journal_mode", [], |l| l.get(0)).unwrap();
    assert_eq!(modo, "wal");
    drop(d);
    for sufixo in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{sufixo}", caminho.display()));
    }
}
