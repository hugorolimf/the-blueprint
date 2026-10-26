use crate::dados::Op;
use crate::sincronia::operacoes::{ler, MAX_OPS};
use serde_json::json;

fn corpo(v: serde_json::Value) -> Vec<u8> {
    v.to_string().into_bytes()
}

#[test]
fn le_as_quatro_operacoes_e_tira_o_antes_do_que_vai_para_os_outros() {
    let l = ler(&corpo(json!({"base": 100, "cliente": "c-1", "ops": [
        {"op": "criar", "tipo": "plano", "id": "p8", "pai": "a1", "posicao": 3, "valor": {"texto": "Novo"}},
        {"op": "alterar", "tipo": "plano", "id": "p1", "campos": {"texto": "B"}, "antes": {"texto": "A"}},
        {"op": "mover", "tipo": "workspace", "id": "w2", "posicao": 0},
        {"op": "apagar", "tipo": "acao", "id": "a9"}]})))
    .unwrap();
    assert_eq!((l.base, l.cliente.as_str(), l.ops.len()), (Some(100), "c-1", 4));
    assert!(matches!(&l.ops[0], Op::Criar { posicao: 3, pai: Some(p), .. } if p == "a1"));
    assert!(matches!(&l.ops[2], Op::Mover { pai: None, posicao: 0, .. }));
    assert_eq!(l.brutas[1], json!({"op": "alterar", "tipo": "plano", "id": "p1", "campos": {"texto": "B"}}));
}

#[test]
fn recusa_o_malformado_com_o_indice() {
    let erro = |ops: serde_json::Value| ler(&corpo(json!({"ops": ops}))).unwrap_err();
    assert_eq!(erro(json!([{"op": "alterar", "tipo": "plano", "id": "p1", "campos": {}}])).indice, Some(0));
    assert_eq!(erro(json!([{"op": "apagar", "tipo": "plano", "id": "p1"}, {"op": "x", "tipo": "plano", "id": "p"}])).indice, Some(1));
    assert_eq!(erro(json!([{"op": "apagar", "tipo": "usuario", "id": "p1"}])).indice, Some(0));
    assert_eq!(erro(json!([{"op": "criar", "tipo": "plano", "id": "p1", "valor": {}}])).motivo, "pai inválido");
    assert_eq!(erro(json!([{"op": "criar", "tipo": "workspace", "id": "w", "pai": "x", "valor": {}}])).indice, Some(0));
    assert_eq!(erro(json!([{"op": "apagar", "tipo": "plano", "id": ""}])).indice, Some(0));
    assert_eq!(erro(json!(vec![json!({"op": "apagar", "tipo": "plano", "id": "p"}); MAX_OPS + 1])).indice, None);
    assert_eq!(ler(b"[]").unwrap_err().indice, None);
    assert_eq!(ler(&corpo(json!({"ops": [], "cliente": 3}))).unwrap_err().motivo, "cliente inválido");
}
