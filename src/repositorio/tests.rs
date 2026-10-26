//! A regra da URL, o prompt e a conferência do acesso (com um git falso: nenhum teste fala com a rede).
use super::acesso::{conferir_com, Acesso};
use super::prompt::configurar;
use super::{repositorio_valido, Repo};
use std::path::Path;
use std::time::Duration;

/// Os mesmos casos do teste de navegador (repositorio.test.mjs): as duas implementações da regra têm de concordar.
fn casos() -> serde_json::Value {
    serde_json::from_str(include_str!("../../referencias/repositorio-urls.json")).unwrap()
}

#[test]
fn aceita_so_o_endereco_https_do_github_e_guarda_a_forma_limpa() {
    for c in casos()["aceitas"].as_array().unwrap() {
        let entrada = c["entrada"].as_str().unwrap();
        let r = repositorio_valido(entrada).unwrap_or_else(|| panic!("devia aceitar {entrada:?}"));
        assert_eq!(r.url, c["url"].as_str().unwrap());
    }
    let r = repositorio_valido("https://github.com/dono/repo.git").unwrap();
    assert_eq!(r, Repo { url: "https://github.com/dono/repo".into(), dono: "dono".into(), nome: "repo".into() });
}

#[test]
fn recusa_o_que_nao_e_um_repositorio_do_github_em_https() {
    for ruim in casos()["recusadas"].as_array().unwrap() {
        let ruim = ruim.as_str().unwrap();
        assert!(repositorio_valido(ruim).is_none(), "devia recusar {ruim:?}");
    }
}

fn repo() -> Repo {
    repositorio_valido("https://github.com/dono/repo").unwrap()
}

#[test]
fn o_prompt_e_o_modelo_fixo_e_nao_leva_ponto_e_virgula() {
    let p = configurar(&repo(), Path::new("C:\\Projetos\\repo"), "Portal", "sm", "p1");
    assert!(p.contains("git clone -- https://github.com/dono/repo \"C:\\Projetos\\repo\""), "{p}");
    assert!(p.contains("workspace_id sm, blueprint_id p1"), "{p}");
    assert!(p.contains("vincular_pasta") && p.contains("git pull --ff-only"), "{p}");
    assert!(p.contains("«Portal»") && p.contains("README"), "{p}");
    assert!(!p.contains(';'), "o Windows Terminal lê ; como separador de comandos: {p}");
    assert!(!p.contains('\n'), "um argumento só, numa linha");
}

#[cfg(windows)]
fn git_falso(nome: &str, corpo: &str) -> String {
    let dir = std::env::temp_dir().join(format!("blueprint-git-falso-{}-{nome}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let arq = dir.join("git.cmd");
    std::fs::write(&arq, format!("@echo off\r\n{corpo}\r\n")).unwrap();
    arq.to_string_lossy().into_owned()
}

#[test]
fn sem_git_no_caminho_o_estado_e_git_ausente() {
    assert_eq!(conferir_com("blueprint-git-que-nao-existe-123", "https://github.com/dono/repo", Duration::from_secs(2)), Acesso::GitAusente);
}

#[cfg(windows)]
#[test]
fn o_git_decide_com_acesso_sem_acesso_e_sem_resposta() {
    let url = "https://github.com/dono/repo";
    assert_eq!(conferir_com(&git_falso("ok", "exit /b 0"), url, Duration::from_secs(30)), Acesso::Com);
    assert_eq!(conferir_com(&git_falso("sem", "exit /b 128"), url, Duration::from_secs(30)), Acesso::Sem);
    // o git recebe ls-remote --exit-code -- <url> HEAD e nunca pergunta nada: o falso só diz sim se os argumentos forem esses
    let certo = r#"if "%1"=="ls-remote" if "%2"=="--exit-code" if "%3"=="--" if "%4"=="https://github.com/dono/repo" if "%5"=="HEAD" if "%GIT_TERMINAL_PROMPT%"=="0" if "%GCM_INTERACTIVE%"=="never" exit /b 0
exit /b 9"#;
    assert_eq!(conferir_com(&git_falso("args", certo), url, Duration::from_secs(30)), Acesso::Com);
    let travado = ":a
goto a"; // laço do próprio cmd: sem processo-neto que sobreviva ao kill
    let t = std::time::Instant::now();
    assert_eq!(conferir_com(&git_falso("trava", travado), url, Duration::from_millis(400)), Acesso::SemResposta);
    assert!(t.elapsed() < Duration::from_secs(10), "o tempo limite derruba o processo: {:?}", t.elapsed());
}

#[test]
fn os_codigos_dos_estados_sao_estaveis() {
    let todos = [(Acesso::Com, "com-acesso"), (Acesso::Sem, "sem-acesso"), (Acesso::GitAusente, "git-ausente"), (Acesso::SemResposta, "sem-resposta")];
    for (a, c) in todos {
        assert_eq!(a.codigo(), c);
    }
}
