use super::pasta;
use crate::atualizador::copiar_banco;
use rusqlite::Connection;

#[test]
fn copia_o_banco_inteiro_e_guarda_so_as_tres_mais_recentes() {
    let p = pasta("backup").join("backups");
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t(x TEXT); INSERT INTO t VALUES ('dado do usuário');").unwrap();
    let mut copias = Vec::new();
    for _ in 0..5 {
        copias.push(copiar_banco(&conn, &p, "0.2.0").unwrap());
        std::thread::sleep(std::time::Duration::from_millis(3));
    }
    let existentes: Vec<_> = copias.iter().filter(|c| c.exists()).collect();
    assert_eq!(existentes, copias[2..].iter().collect::<Vec<_>>(), "as 3 últimas ficam");
    let copia = Connection::open(copias.last().unwrap()).unwrap();
    assert_eq!(copia.query_row("SELECT x FROM t", [], |l| l.get::<_, String>(0)).unwrap(), "dado do usuário");
}

#[test]
fn o_nome_da_versao_nao_escapa_da_pasta() {
    let p = pasta("backup-nome").join("backups");
    let conn = Connection::open_in_memory().unwrap();
    let c = copiar_banco(&conn, &p, "..\\..\\fora").unwrap();
    assert_eq!(c.parent().unwrap(), p);
}
