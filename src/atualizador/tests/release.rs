use crate::atualizador::release::escolher;
use serde_json::{json, Value};

fn release(tag: &str, draft: bool, pre: bool, com_sums: bool) -> Value {
    let mut assets = vec![json!({"name": format!("Blueprint-{}-instalador.exe", tag.trim_start_matches('v')), "size": 3_000_000, "browser_download_url": "https://github.com/x/y/releases/download/a/i.exe"})];
    if com_sums {
        assets.push(json!({"name": "SHA256SUMS", "size": 100, "browser_download_url": "https://github.com/x/y/releases/download/a/SHA256SUMS"}));
    }
    json!({"tag_name": tag, "draft": draft, "prerelease": pre, "body": "notas", "assets": assets})
}

fn escolhe(lista: &[Value], instalada: &str, so_estaveis: bool) -> Option<String> {
    escolher(&Value::Array(lista.to_vec()).to_string(), instalada, so_estaveis).unwrap().map(|r| r.versao)
}

#[test]
fn escolhe_a_mais_nova_entre_varias() {
    let lista = [release("v0.1.1", false, false, true), release("v0.2.0", false, false, true), release("v0.1.5", false, false, true)];
    assert_eq!(escolhe(&lista, "0.1.0", false).as_deref(), Some("0.2.0"));
    assert_eq!(escolhe(&lista, "0.2.0", false), None, "já está em dia");
}

#[test]
fn rascunho_e_release_incompleta_nao_contam() {
    assert_eq!(escolhe(&[release("v0.3.0", true, false, true)], "0.1.0", false), None, "rascunho");
    assert_eq!(escolhe(&[release("v0.3.0", false, false, false)], "0.1.0", false), None, "sem SHA256SUMS");
    assert_eq!(escolhe(&[release("v0.3.0", false, false, false), release("v0.2.0", false, false, true)], "0.1.0", false).as_deref(), Some("0.2.0"));
}

#[test]
fn pre_lancamento_do_github_vale_por_padrao_mas_nao_com_so_estaveis() {
    let lista = [release("v0.1.1", false, true, true)];
    assert_eq!(escolhe(&lista, "0.1.0", false).as_deref(), Some("0.1.1"), "o app é beta: a v0.1.x sai como pré-lançamento");
    assert_eq!(escolhe(&lista, "0.1.0", true), None);
}

#[test]
fn traz_as_notas_e_os_dois_assets() {
    let r = escolher(&Value::Array(vec![release("v0.2.0", false, false, true)]).to_string(), "0.1.0", false).unwrap().unwrap();
    assert_eq!((r.versao.as_str(), r.notas.as_str(), r.instalador.nome.as_str(), r.sha256sums.nome.as_str()), ("0.2.0", "notas", "Blueprint-0.2.0-instalador.exe", "SHA256SUMS"));
}

#[test]
fn resposta_que_nao_e_lista_e_erro() {
    assert!(escolher("{\"message\": \"Not Found\"}", "0.1.0", false).is_err());
    assert!(escolher("isto não é json", "0.1.0", false).is_err());
    assert_eq!(escolher("[]", "0.1.0", false).unwrap(), None);
}
