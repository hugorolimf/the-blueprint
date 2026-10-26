//! `vincular_pasta` no plugin do convidado (`--equipe`, docs/app-no-projeto-compartilhado.md): o servidor remoto não conhece as pastas desta máquina,
//! então a ligação pasta e blueprint vai para um arquivo da própria pasta: a linha `Blueprint: workspace_id …, blueprint_id …` em `CLAUDE.local.md`,
//! que `blueprint hook --equipe` já lê, e `CLAUDE.local.md` entra em `.git/info/exclude` (a lista local do Git, que não é versionada). Assim o
//! repositório do projeto não muda nem ganha uma linha que só vale para quem está neste workspace.
use std::fs;
use std::path::Path;

pub const ARQUIVO: &str = "CLAUDE.local.md";

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Põe (ou troca) a linha do blueprint em `CLAUDE.local.md` e, se a pasta é um repositório Git, esconde o arquivo no `.git/info/exclude`.
pub fn vincular(pasta: &Path, workspace_id: &str, blueprint_id: &str) -> Result<String, String> {
    if !id_valido(workspace_id) || !id_valido(blueprint_id) {
        return Err(format!("Invalid ids: use the workspace_id and blueprint_id from listar_blueprints (got {workspace_id:?} and {blueprint_id:?})."));
    }
    let linha = format!("Blueprint: workspace_id `{workspace_id}`, blueprint_id `{blueprint_id}` (MCP `blueprint`).");
    let arquivo = pasta.join(ARQUIVO);
    let antes = fs::read_to_string(&arquivo).unwrap_or_default();
    let mut trocou = false;
    let mut linhas: Vec<String> = antes
        .lines()
        .map(|l| if crate::cli::claude::ids_da_linha(l).is_some() { trocou = true; linha.clone() } else { l.to_string() })
        .collect();
    if !trocou {
        linhas.push(linha);
    }
    fs::write(&arquivo, linhas.join("\n") + "\n").map_err(|e| format!("Cannot write {}: {e}", arquivo.display()))?;
    let escondido = esconder_no_git(pasta);
    Ok(format!(
        "Folder {} linked to the blueprint (line saved in {ARQUIVO}{}). The next Claude sessions in it start with the blueprint and the current action.",
        pasta.display(),
        if escondido { ", hidden from Git" } else { "" }
    ))
}

/// `.git/info/exclude` é a lista de ignorados só desta cópia. Uma pasta sem `.git` (ou um `.git` que é arquivo, como numa worktree) fica como está.
fn esconder_no_git(pasta: &Path) -> bool {
    let git = pasta.join(".git");
    if !git.is_dir() {
        return false;
    }
    let exclude = git.join("info").join("exclude");
    let atual = fs::read_to_string(&exclude).unwrap_or_default();
    if atual.lines().any(|l| l.trim() == ARQUIVO) {
        return true;
    }
    let separador = if atual.is_empty() || atual.ends_with('\n') { "" } else { "\n" };
    fs::create_dir_all(exclude.parent().unwrap_or(&git)).is_ok() && fs::write(&exclude, format!("{atual}{separador}{ARQUIVO}\n")).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::claude::blueprint_da_pasta;

    fn pasta(nome: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("blueprint-pasta-local-{}-{nome}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn liga_a_pasta_e_o_hook_do_convidado_a_acha_inclusive_numa_subpasta() {
        let p = pasta("liga");
        fs::create_dir_all(p.join(".git")).unwrap();
        fs::create_dir_all(p.join("src").join("fundo")).unwrap();
        let msg = vincular(&p, "yuzgakl", "mn9su6k").unwrap();
        assert!(msg.contains("hidden from Git"), "{msg}");
        assert_eq!(blueprint_da_pasta(&p), Some(("yuzgakl".into(), "mn9su6k".into())));
        assert_eq!(blueprint_da_pasta(&p.join("src").join("fundo")), Some(("yuzgakl".into(), "mn9su6k".into())), "uma sessão numa subpasta é do mesmo projeto");
        assert_eq!(fs::read_to_string(p.join(".git/info/exclude")).unwrap(), "CLAUDE.local.md\n");
    }

    #[test]
    fn ligar_de_novo_troca_a_linha_sem_duplicar_e_guarda_o_resto_do_arquivo() {
        let p = pasta("troca");
        fs::write(p.join(ARQUIVO), "# minhas notas\nBlueprint: workspace_id a, blueprint_id b\nmais uma\n").unwrap();
        vincular(&p, "w2", "p9").unwrap();
        vincular(&p, "w2", "p9").unwrap();
        let t = fs::read_to_string(p.join(ARQUIVO)).unwrap();
        assert_eq!(t.lines().filter(|l| l.starts_with("Blueprint:")).count(), 1, "{t}");
        assert!(t.contains("# minhas notas") && t.contains("mais uma") && t.contains("workspace_id `w2`, blueprint_id `p9`"), "{t}");
        assert_eq!(blueprint_da_pasta(&p), Some(("w2".into(), "p9".into())));
    }

    #[test]
    fn o_exclude_nao_duplica_nem_cola_na_ultima_linha() {
        let p = pasta("exclude");
        fs::create_dir_all(p.join(".git/info")).unwrap();
        fs::write(p.join(".git/info/exclude"), "# padrao\n*.tmp").unwrap(); // sem quebra de linha no fim
        vincular(&p, "w", "b").unwrap();
        vincular(&p, "w", "b").unwrap();
        assert_eq!(fs::read_to_string(p.join(".git/info/exclude")).unwrap(), "# padrao\n*.tmp\nCLAUDE.local.md\n");
    }

    #[test]
    fn sem_git_ou_com_git_de_worktree_so_escreve_o_arquivo() {
        let sem = pasta("sem-git");
        assert!(!vincular(&sem, "w", "b").unwrap().contains("hidden"));
        assert!(!sem.join(".git").exists());
        let wt = pasta("worktree");
        fs::write(wt.join(".git"), "gitdir: ../outro\n").unwrap(); // arquivo, não pasta
        assert!(!vincular(&wt, "w", "b").unwrap().contains("hidden"));
        assert_eq!(fs::read_to_string(wt.join(".git")).unwrap(), "gitdir: ../outro\n", "o .git de worktree não é tocado");
    }

    #[test]
    fn recusa_ids_que_nao_sao_ids() {
        let p = pasta("ids");
        for (w, b) in [("", "b"), ("w", ""), ("w w", "b"), ("w", "b`; rm"), ("w\n", "b"), ("../w", "b"), (&"x".repeat(65), "b")] {
            assert!(vincular(&p, w, b).is_err(), "{w:?} {b:?}");
        }
        assert!(!p.join(ARQUIVO).exists(), "nada foi escrito");
    }
}
