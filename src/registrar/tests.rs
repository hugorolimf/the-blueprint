use super::*;
use std::fs;

#[test]
fn comando_do_claude_vale_em_qualquer_pasta_e_aponta_para_o_exe() {
    let args = montar(AGENTES[0].registrar, Path::new("C:\\Apps\\Blueprint\\blueprint.exe"));
    assert_eq!(args, ["mcp", "add", "--scope", "user", "blueprint", "--", "C:\\Apps\\Blueprint\\blueprint.exe", "--mcp"]);
}

#[test]
fn comando_do_codex_e_a_remocao_usam_so_o_nome_blueprint() {
    let exe = Path::new("/x/blueprint");
    assert_eq!(montar(AGENTES[1].registrar, exe), ["mcp", "add", "blueprint", "--", "/x/blueprint", "--mcp"]);
    assert_eq!(montar(AGENTES[1].remover, exe), ["mcp", "remove", "blueprint"]);
    assert_eq!(montar(AGENTES[0].remover, exe), ["mcp", "remove", "--scope", "user", "blueprint"]);
}

#[test]
fn procura_a_cli_na_ordem_das_pastas() {
    let base = std::env::temp_dir().join(format!("blueprint-registrar-{}", std::process::id()));
    let (a, b) = (base.join("a"), base.join("b"));
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let nome = if cfg!(windows) { "agente.cmd" } else { "agente" };
    fs::write(b.join(nome), "").unwrap();
    assert_eq!(procurar_em("agente", &[a.clone(), b.clone()]), Some(b.join(nome)));
    assert_eq!(procurar_em("outro", &[a, b]), None);
    fs::remove_dir_all(base).unwrap();
}
