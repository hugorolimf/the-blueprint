//! A situação do acesso e o "Configurar projeto": o repositório vem dos dados, as recusas não abrem terminal e o prompt vai inteiro num argumento só.
use super::acesso::Acesso;
use super::rotas::{configurar, situacao};
use crate::contexto::Contexto;
use crate::dados::Dados;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

fn dados(repositorio: Value) -> Mutex<Dados> {
    let mut d = Dados::em_memoria().unwrap();
    let mut p = json!({"id": "p1", "nome": "Portal; novo", "acoes": []});
    if !repositorio.is_null() {
        p["repositorio"] = repositorio;
    }
    d.gravar_tudo(&Contexto::local(), &json!({"empresa": "Eu", "blueprints": [p]})).unwrap();
    Mutex::new(d)
}

fn pasta() -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-repo-{}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn corpo(pasta: &str) -> Value {
    json!({"workspaceId": "sm", "blueprintId": "p1", "pasta": pasta})
}

fn nunca(_: &Path, _: &Path, _: &str) -> Result<&'static str, String> {
    panic!("não podia abrir terminal")
}

#[test]
fn a_situacao_segue_o_que_esta_guardado() {
    let com = |_: &str| Acesso::Com;
    let nao_chama = |_: &str| -> Acesso { panic!("não podia ir ao git") };
    assert_eq!(situacao(None, &nao_chama), json!({"estado": "sem-repositorio"}));
    assert_eq!(situacao(Some("http://github.com/a/b"), &nao_chama), json!({"estado": "url-invalida"}), "URL ruim nunca chega ao git");
    assert_eq!(situacao(Some("https://github.com/a/b.git"), &com), json!({"estado": "com-acesso", "url": "https://github.com/a/b", "nome": "b"}));
    assert_eq!(situacao(Some("https://github.com/a/b"), &|_: &str| Acesso::Sem)["estado"], "sem-acesso");
    assert_eq!(situacao(Some("https://github.com/a/b"), &|_: &str| Acesso::GitAusente)["estado"], "git-ausente");
}

#[test]
fn configurar_abre_o_terminal_na_pasta_com_o_prompt_como_um_argumento() {
    let d = dados(json!({"url": "https://github.com/dono/repo.git"}));
    let pasta = pasta();
    let chamadas: RefCell<Vec<(PathBuf, PathBuf, String)>> = RefCell::new(vec![]);
    let falso = |p: &Path, e: &Path, a: &str| {
        chamadas.borrow_mut().push((p.into(), e.into(), a.into()));
        Ok("app")
    };
    let (status, r) = configurar(&d, &corpo(pasta.to_str().unwrap()), &falso).unwrap();
    assert_eq!(status, 200, "{r}");
    assert_eq!((r["ok"].clone(), r["terminal"].clone()), (json!(true), json!("app")));
    assert_eq!(PathBuf::from(r["destino"].as_str().unwrap()), pasta.join("repo"));
    let c = chamadas.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!((c[0].0.as_path(), &c[0].1), (pasta.as_path(), &std::env::current_exe().unwrap()));
    assert!(c[0].2.contains("git clone -- https://github.com/dono/repo "), "{}", c[0].2);
    assert!(c[0].2.contains("«Portal, novo»"), "o ; do nome vira , para o wt: {}", c[0].2);
    assert!(!c[0].2.contains(';'));
}

#[test]
fn as_recusas_nao_abrem_terminal() {
    let sem = dados(Value::Null);
    let (s, r) = configurar(&sem, &corpo(pasta().to_str().unwrap()), &nunca).unwrap();
    assert_eq!((s, r["codigo"].clone()), (409, json!("sem-repositorio")));
    let ruim = dados(json!({"url": "https://evil.com/a/b"}));
    assert_eq!(configurar(&ruim, &corpo(pasta().to_str().unwrap()), &nunca).unwrap().1["codigo"], "url-invalida", "a URL guardada é revalidada");
    let d = dados(json!({"url": "https://github.com/dono/repo"}));
    for p in ["", "   ", "relativa/pasta", "C:/pasta/que/nao/existe/mais-123", "a;b", "C:/x\ny"] {
        let (s, r) = configurar(&d, &corpo(p), &nunca).unwrap();
        assert_eq!((s, r["codigo"].clone()), (400, json!("pasta-invalida")), "pasta {p:?}");
    }
    let arquivo = pasta().join("arquivo.txt");
    std::fs::write(&arquivo, "x").unwrap();
    assert_eq!(configurar(&d, &corpo(arquivo.to_str().unwrap()), &nunca).unwrap().0, 400, "um arquivo não é pasta");
    assert_eq!(configurar(&d, &json!({"pasta": "C:/"}), &nunca).unwrap_err().status, 400, "sem ids");
    assert_eq!(configurar(&d, &json!({"workspaceId": "sm", "blueprintId": "nao", "pasta": "C:/"}), &nunca).unwrap_err().status, 404);
    // a URL do corpo é ignorada: só vale a dos dados
    let (_, r) = configurar(&sem, &json!({"workspaceId": "sm", "blueprintId": "p1", "pasta": pasta(), "url": "https://github.com/a/b", "repositorio": {"url": "https://github.com/a/b"}}), &nunca).unwrap();
    assert_eq!(r["codigo"], "sem-repositorio");
}

#[test]
fn sem_terminal_devolve_501() {
    let d = dados(json!({"url": "https://github.com/dono/repo"}));
    let (s, r) = configurar(&d, &corpo(pasta().to_str().unwrap()), &|_: &Path, _: &Path, _: &str| Err("nada abriu".to_string())).unwrap();
    assert_eq!((s, r["codigo"].clone(), r["erro"].clone()), (501, json!("sem-terminal"), json!("nada abriu")));
}

#[test]
fn ler_blueprint_devolve_o_repositorio_guardado() {
    let d = dados(json!({"url": "https://github.com/dono/repo"}));
    let mut g = d.lock().unwrap();
    let tudo = g.ler_tudo(&Contexto::local()).unwrap();
    let e = crate::mcp::acoes::blueprint_lido(tudo, "sm", "p1", None, "ativas", "").unwrap();
    assert_eq!(e["blueprint"]["repositorio"], json!({"url": "https://github.com/dono/repo"}));
    let sem = dados(Value::Null);
    let mut g2 = sem.lock().unwrap();
    let tudo2 = g2.ler_tudo(&Contexto::local()).unwrap();
    assert!(crate::mcp::acoes::blueprint_lido(tudo2, "sm", "p1", None, "ativas", "").unwrap()["blueprint"].get("repositorio").is_none());
}
