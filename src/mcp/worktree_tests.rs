//! Testes da resolução worktree -> pasta principal (`worktree.rs`) e do uso dela no vínculo pasta -> blueprint.
use super::hooks::{alvo, desvincular, sessao, travado, vincular};
use super::hooks_tests::{dados, mcp};
use super::worktree::{comum_de, gitdir_de, limpar, pasta_de, pasta_principal, principal_ou};
use crate::contexto::Contexto;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Uma pasta temporária só deste teste, apagada ao sair.
struct Temp(PathBuf);

impl Temp {
    fn nova() -> Temp {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!("blueprint-wt-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&p).unwrap();
        Temp(p)
    }
    fn caminho(&self, relativo: &str) -> PathBuf {
        self.0.join(relativo)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `/` e minúsculas, como o vínculo compara.
fn normal(p: impl AsRef<Path>) -> String {
    p.as_ref().to_string_lossy().replace('\\', "/").to_lowercase()
}

/// Monta à mão o que o git grava: o repositório principal em `repo/` com a worktree `nome` em `onde` (o `.git` dela é um arquivo apontando para
/// `repo/.git/worktrees/<nome>`, que tem `commondir`). `gitdir` é a linha escrita no arquivo `.git` da worktree.
fn worktree_falsa(t: &Temp, onde: &str, nome: &str, gitdir: impl Fn(&Path) -> String, com_commondir: bool) -> (PathBuf, PathBuf) {
    let repo = t.caminho("repo");
    let admin = repo.join(".git/worktrees").join(nome);
    fs::create_dir_all(&admin).unwrap();
    if com_commondir {
        fs::write(admin.join("commondir"), "../..\n").unwrap();
    }
    let wt = t.caminho(onde);
    fs::create_dir_all(wt.join("src/mod")).unwrap();
    fs::write(wt.join(".git"), format!("gitdir: {}\n", gitdir(&admin))).unwrap();
    (repo, wt)
}

fn barras(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

#[test]
fn limpar_junta_barras_e_resolve_pontos() {
    assert_eq!(limpar("C:\\Users\\Hugo\\repo\\.git\\worktrees\\wt\\..\\.."), "C:/Users/Hugo/repo/.git");
    assert_eq!(limpar("C:/a/./b//c/"), "C:/a/b/c");
    assert_eq!(limpar("/home/x/../y"), "/home/y");
    assert_eq!(limpar("C:/.."), "C:", "a letra da unidade não sobe");
    assert_eq!(limpar("\\\\servidor\\pasta\\x"), "//servidor/pasta/x");
}

#[test]
fn le_o_gitdir_em_qualquer_forma() {
    assert_eq!(gitdir_de("C:/p/wt", "gitdir: C:/repo/.git/worktrees/wt\n").as_deref(), Some("C:/repo/.git/worktrees/wt"));
    assert_eq!(gitdir_de("C:/p/wt", "gitdir: C:\\repo\\.git\\worktrees\\wt\r\n").as_deref(), Some("C:/repo/.git/worktrees/wt"));
    assert_eq!(gitdir_de("C:/p/wt", "gitdir: ../../repo/.git/worktrees/wt").as_deref(), Some("C:/repo/.git/worktrees/wt"), "relativo à pasta do .git");
    assert_eq!(gitdir_de("/p/wt", "gitdir: /repo/.git/worktrees/wt").as_deref(), Some("/repo/.git/worktrees/wt"));
    assert_eq!(gitdir_de("C:/p", "isto não é um .git"), None);
    assert_eq!(gitdir_de("C:/p", "gitdir:   \n"), None);
}

#[test]
fn acha_o_git_principal_pelo_commondir_ou_pelo_nome() {
    let admin = "C:/repo/.git/worktrees/wt";
    assert_eq!(comum_de(admin, Some("../..\n")).as_deref(), Some("C:/repo/.git"));
    assert_eq!(comum_de(admin, Some("C:\\repo\\.git")).as_deref(), Some("C:/repo/.git"));
    assert_eq!(comum_de(admin, None).as_deref(), Some("C:/repo/.git"), "sem commondir, .git/worktrees/<nome> ainda diz");
    assert_eq!(comum_de("C:/repo/.git/modules/sub", None), None, "submódulo não é worktree");
    assert_eq!(pasta_de("C:/repo/.git").as_deref(), Some("C:/repo"));
    assert_eq!(pasta_de("C:/repo.git"), None, "repositório bare não tem pasta de trabalho");
    assert_eq!(pasta_de("/.git"), None);
}

#[test]
fn a_worktree_dentro_da_pasta_do_repositorio_resolve_para_a_principal() {
    let t = Temp::nova();
    let (repo, wt) = worktree_falsa(&t, "repo/.claude/worktrees/feature-x", "feature-x", |a| barras(a), true);
    for cwd in [wt.clone(), wt.join("src/mod")] {
        let (nativo, com_barras) = (cwd.to_string_lossy().to_string(), barras(&cwd));
        for c in [nativo, com_barras] {
            assert_eq!(pasta_principal(&c).map(normal), Some(normal(&repo)), "{c}");
        }
    }
}

#[test]
fn a_worktree_fora_do_repositorio_com_gitdir_em_barras_invertidas_tambem() {
    let t = Temp::nova();
    let (repo, wt) = worktree_falsa(&t, "irma-feature", "irma-feature", |a| a.to_string_lossy().replace('/', "\\"), true);
    assert_eq!(pasta_principal(&wt.to_string_lossy()).map(normal), Some(normal(&repo)));
    assert_eq!(principal_ou(&barras(&wt.join("src"))).to_lowercase(), normal(&repo));
}

#[test]
fn gitdir_relativo_e_sem_commondir_tambem_resolvem() {
    let t = Temp::nova();
    let (repo, wt) = worktree_falsa(&t, "irma-b", "irma-b", |_| "../repo/.git/worktrees/irma-b".to_string(), false);
    assert_eq!(pasta_principal(&wt.to_string_lossy()).map(normal), Some(normal(&repo)));
}

#[test]
fn o_que_nao_e_worktree_fica_como_esta() {
    let t = Temp::nova();
    let repo = t.caminho("repo");
    fs::create_dir_all(repo.join(".git")).unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    assert_eq!(pasta_principal(&repo.to_string_lossy()), None, "a pasta principal já é a principal");
    assert_eq!(pasta_principal(&repo.join("src").to_string_lossy()), None);
    let sub = t.caminho("repo/lib/sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join(".git"), "gitdir: ../../.git/modules/sub\n").unwrap();
    assert_eq!(pasta_principal(&sub.to_string_lossy()), None, "submódulo");
    let solta = t.caminho("solta/x");
    fs::create_dir_all(&solta).unwrap();
    for c in [solta.to_string_lossy().to_string(), "C:\\nao\\existe\\mesmo".to_string(), String::new(), "C:".to_string()] {
        assert_eq!(pasta_principal(&c), None, "{c}");
        assert_eq!(principal_ou(&c), c);
    }
    fs::write(t.caminho("solta/.git"), "lixo\n").unwrap();
    assert_eq!(pasta_principal(&solta.to_string_lossy()), None, "um .git ilegível não vira vínculo");
}

#[test]
fn o_vinculo_da_pasta_principal_vale_na_worktree_e_o_proprio_vinculo_vence() {
    let t = Temp::nova();
    let (repo, wt) = worktree_falsa(&t, "irma", "irma", |a| barras(a), true);
    let (m, d) = (mcp(), dados());
    let (cwd_wt, cwd_repo) = (wt.to_string_lossy().to_string(), repo.to_string_lossy().to_string());
    assert_eq!(alvo(&m, &d, &json!({"cwd": cwd_wt})).unwrap(), None);
    vincular(&d, &json!({"cwd": cwd_repo, "workspaceId": "sm", "blueprintId": "p1"})).unwrap();
    assert_eq!(alvo(&m, &d, &json!({"cwd": cwd_wt})).unwrap(), Some(("sm".into(), "p1".into())));
    let r = sessao(&m, &d, &json!({"session_id": "s1", "cwd": wt.join("src").to_string_lossy()})).unwrap();
    assert_eq!((r["vinculado"].clone(), r["blueprintId"].clone()), (json!(true), json!("p1")));
    // a sessão da worktree conta na pasta principal (a que o canal e o diálogo de iniciar usam)
    let lista = m.sessoes_do_blueprint("sm", "p1");
    assert_eq!(lista["sessoes"][0]["pasta"].as_str().map(normal), Some(normal(&repo)), "{lista}");
}

#[test]
fn vincular_e_desvincular_numa_worktree_gravam_na_pasta_principal() {
    let t = Temp::nova();
    let (repo, wt) = worktree_falsa(&t, "irma", "irma", |a| barras(a), true);
    let (m, d) = (mcp(), dados());
    vincular(&d, &json!({"cwd": wt.to_string_lossy(), "workspaceId": "sm", "blueprintId": "p1"})).unwrap();
    let pastas = travado(&d).listar_pastas(&Contexto::local()).unwrap();
    assert_eq!(pastas.as_array().unwrap().len(), 1);
    assert_eq!(pastas[0]["cwd"].as_str().map(str::to_string), Some(normal(&repo)), "gravou a principal, não a worktree");
    assert_eq!(travado(&d).blueprint_da_pasta(&Contexto::local(), &repo.to_string_lossy()).unwrap(), Some(("sm".into(), "p1".into())));
    assert_eq!(alvo(&m, &d, &json!({"cwd": wt.to_string_lossy()})).unwrap(), Some(("sm".into(), "p1".into())));
    desvincular(&d, &json!({"cwd": wt.to_string_lossy()})).unwrap();
    assert!(travado(&d).listar_pastas(&Contexto::local()).unwrap().as_array().unwrap().is_empty());
}

#[test]
fn pasta_solta_numa_worktree_avisa_a_principal_uma_vez() {
    let t = Temp::nova();
    let (repo, wt) = worktree_falsa(&t, "irma", "irma", |a| barras(a), true);
    let (m, d) = (mcp(), dados());
    sessao(&m, &d, &json!({"session_id": "a", "cwd": wt.to_string_lossy()})).unwrap();
    sessao(&m, &d, &json!({"session_id": "b", "cwd": repo.to_string_lossy()})).unwrap();
    let itens = m.itens();
    assert_eq!(itens.len(), 1, "a worktree e a principal são o mesmo aviso");
    assert_eq!(itens[0]["cwd"].as_str().map(normal), Some(normal(&repo)));
}
