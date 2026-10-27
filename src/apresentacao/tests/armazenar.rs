use super::{conteudo, figura, pasta};
use crate::apresentacao::armazenar::{alvo, gravar, id_valido, ler, listar, meta, remover};
use serde_json::json;

fn registro(id: &str, ws: &str, bp: &str, em: &str) -> serde_json::Value {
    json!({"id": id, "workspaceId": ws, "blueprintId": bp, "titulo": "T", "publico": "cliente", "nivel": "padrao", "em": em, "conteudo": conteudo(), "figuras": [figura("fichas")]})
}

#[test]
fn grava_le_lista_e_remove_so_com_id_de_formato_valido() {
    let p = pasta("armazenar");
    let (a, b, c) = ("0123456789abcdef", "fedcba9876543210", "aaaaaaaaaaaaaaaa");
    gravar(&p, a, &registro(a, "sm", "p1", "2026-10-26T10:00:00.000Z")).unwrap();
    gravar(&p, b, &registro(b, "sm", "p1", "2026-10-27T10:00:00.000Z")).unwrap();
    gravar(&p, c, &registro(c, "sm", "outro", "2026-10-28T10:00:00.000Z")).unwrap();
    assert_eq!(alvo(&ler(&p, a).unwrap()), ("sm", "p1"));
    let lista = listar(&p, "sm", "p1");
    assert_eq!(lista.iter().map(|m| m["id"].as_str().unwrap()).collect::<Vec<_>>(), [b, a], "só deste blueprint, da mais nova para a mais antiga");
    assert_eq!(lista[0], meta(&ler(&p, b).unwrap()));
    assert_eq!(lista[0]["figuras"], json!([{"id": "fichas", "impressao": "abc123"}]), "os metadados não levam o conteúdo nem o código");
    assert!(lista[0].get("conteudo").is_none());
    for ruim in ["../x", "..\\x", "0123", "0123456789abcdeg", ""] {
        assert!(!id_valido(ruim) && ler(&p, ruim).is_none(), "{ruim:?}");
    }
    gravar(&p, a, &registro(a, "sm", "p1", "2026-10-29T10:00:00.000Z")).unwrap();
    assert_eq!(ler(&p, a).unwrap()["em"], "2026-10-29T10:00:00.000Z", "gravar de novo substitui");
    remover(&p, a);
    remover(&p, "../../fora");
    assert!(ler(&p, a).is_none() && ler(&p, b).is_some());
    assert!(listar(&p.join("nao-existe"), "sm", "p1").is_empty());
    let _ = std::fs::remove_dir_all(&p);
}
