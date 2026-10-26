use super::*;

#[test]
fn todo_subcomando_esta_no_uso_e_o_script_os_lista() {
    for s in SUBCOMANDOS.iter().filter(|s| **s != "completions") {
        assert!(crate::cli::USO.contains(&format!("blueprint {s}")) || crate::cli::USO.contains(s), "{s} não está em USO");
    }
    for shell in ["bash", "powershell"] {
        let texto = script(shell).unwrap();
        assert!(SUBCOMANDOS.iter().all(|s| texto.contains(s)), "{shell} perdeu um subcomando");
    }
}

#[test]
fn shell_desconhecido_nao_gera_script() {
    assert!(script("fish").is_none());
}
