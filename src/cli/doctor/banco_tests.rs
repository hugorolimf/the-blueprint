use super::*;
use crate::cli::doctor::tests::{banco_com_pasta, temp};

#[test]
fn sem_banco_manda_abrir_o_app() {
    let (item, arquivo) = banco(&temp("banco-nenhum"));
    assert_eq!(item.estado, Estado::Falha);
    assert!(arquivo.is_none());
}

#[test]
fn banco_bom_passa_e_nao_e_alterado() {
    let p = temp("banco bom #1"); // espaço e # no caminho: a URI do modo imutável precisa escapar
    let arquivo = banco_com_pasta(&p, "C:/Projetos/Loja", "ws1", "bp1");
    let antes = std::fs::read(&arquivo).unwrap();
    let (item, achado) = banco(&p);
    assert_eq!(item.estado, Estado::Ok, "{}", item.texto);
    assert_eq!(achado.as_deref(), Some(arquivo.as_path()));
    assert_eq!(std::fs::read(&arquivo).unwrap(), antes, "o doctor gravou no banco");
}

#[test]
fn banco_que_nao_e_sqlite_falha_com_o_restaurar() {
    let p = temp("banco-ruim");
    std::fs::write(p.join("blueprint.db"), b"isto nao e um banco sqlite, so texto para o teste falhar ao abrir").unwrap();
    let (item, arquivo) = banco(&p);
    assert_eq!(item.estado, Estado::Falha);
    assert!(item.texto.contains("blueprint restaurar"), "{}", item.texto);
    assert!(arquivo.is_none());
}

#[test]
fn pasta_ligada_no_app_vale_para_subpastas() {
    let p = temp("pasta-ligada");
    let arquivo = banco_com_pasta(&p, "C:/Projetos/Loja", "ws1", "bp1");
    let item = pasta(Some(&arquivo), Path::new("C:\\Projetos\\Loja\\.claude\\worktrees\\a1"));
    assert_eq!(item.estado, Estado::Ok);
    assert!(item.texto.contains("ws1") && item.texto.contains("bp1"), "{}", item.texto);
}

#[test]
fn pasta_com_a_linha_no_claude_md_e_ok_e_sem_nada_e_aviso() {
    let p = temp("pasta-linha");
    let arquivo = banco_com_pasta(&p, "C:/Outro", "ws1", "bp1");
    assert_eq!(pasta(Some(&arquivo), &p).estado, Estado::Aviso);
    assert_eq!(pasta(None, &p).estado, Estado::Aviso);
    std::fs::write(p.join("AGENTS.md"), "# Projeto\n\nBlueprint: workspace_id ws9, blueprint_id bp9\n").unwrap();
    let item = pasta(None, &p);
    assert_eq!(item.estado, Estado::Ok);
    assert!(item.texto.contains("AGENTS.md") && item.texto.contains("ws9"), "{}", item.texto);
}
