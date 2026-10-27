//! Backup automático diário (7 cópias) e a volta a uma cópia.
use super::exemplo;
use crate::contexto::Contexto;
use crate::dados::backups::{backup_diario, listar, pasta_backups, restaurar};
use crate::dados::Dados;
use std::path::PathBuf;

const DIA: i64 = 24 * 60 * 60 * 1000;

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-backups-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn banco_com_exemplo(p: &std::path::Path) -> PathBuf {
    let banco = p.join("blueprint.db");
    Dados::abrir(&banco).unwrap().gravar_tudo(&Contexto::local(), &exemplo()).unwrap();
    banco
}

#[test]
fn um_por_dia_e_so_os_7_mais_recentes() {
    let p = pasta("diario");
    let banco = banco_com_exemplo(&p);
    let auto = pasta_backups(&banco).join("automaticos");
    let inicio = 1_800_000_000_000;
    for dia in 0..9 {
        assert!(backup_diario(&banco, &auto, inicio + dia * DIA).unwrap().is_some(), "dia {dia}");
        assert!(backup_diario(&banco, &auto, inicio + dia * DIA + 23 * 60 * 60 * 1000).unwrap().is_none(), "23 h depois ainda não");
    }
    assert_eq!(std::fs::read_dir(&auto).unwrap().count(), 7);
    assert!(!auto.join(format!("auto-{inicio}.db")).exists(), "a mais antiga saiu");
    let _ = std::fs::remove_dir_all(&p);
}

#[test]
fn sem_banco_nao_copia_nada() {
    let p = pasta("sem-banco");
    assert!(backup_diario(&p.join("nao-existe.db"), &p.join("auto"), 0).unwrap().is_none());
    assert!(!p.join("auto").exists());
    let _ = std::fs::remove_dir_all(&p);
}

#[test]
fn banco_apagado_volta_da_copia_e_o_atual_e_guardado() {
    let p = pasta("restaurar");
    let banco = banco_com_exemplo(&p);
    let copia = backup_diario(&banco, &pasta_backups(&banco).join("automaticos"), 1_800_000_000_000).unwrap().unwrap();
    assert_eq!(listar(&banco), vec![copia.clone()]);
    // banco apagado: restaura sem nada para guardar
    std::fs::remove_file(&banco).unwrap();
    assert_eq!(restaurar(&banco, &copia, 1).unwrap(), None);
    assert_eq!(*Dados::abrir(&banco).unwrap().ler_tudo(&Contexto::local()).unwrap(), exemplo());
    // banco presente: o atual vai para backups antes de ser trocado
    let guardado = restaurar(&banco, &copia, 2).unwrap().expect("o banco atual foi guardado");
    assert!(guardado.starts_with(pasta_backups(&banco)) && guardado.is_file());
    assert_eq!(listar(&banco).len(), 2);
    let _ = std::fs::remove_dir_all(&p);
}

#[test]
fn copia_que_nao_e_banco_e_recusada_sem_tocar_no_atual() {
    let p = pasta("invalida");
    let banco = banco_com_exemplo(&p);
    let lixo = p.join("lixo.db");
    std::fs::write(&lixo, b"isto nao e um banco sqlite, so texto qualquer").unwrap();
    assert!(restaurar(&banco, &lixo, 1).is_err());
    assert_eq!(*Dados::abrir(&banco).unwrap().ler_tudo(&Contexto::local()).unwrap(), exemplo());
    let _ = std::fs::remove_dir_all(&p);
}
