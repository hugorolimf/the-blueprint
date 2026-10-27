//! A fila persistida: o que entra volta igual ao reabrir o banco (itens, entradas das propostas, números por alvo), o ack tira do banco e o limite de 200 vale lá também.
use super::Mcp;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::path::PathBuf;

struct Banco(PathBuf);

impl Drop for Banco {
    fn drop(&mut self) {
        for sufixo in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{sufixo}", self.0.display()));
        }
    }
}

fn banco(nome: &str) -> Banco {
    let b = Banco(std::env::temp_dir().join(format!("blueprint-fila-{nome}-{}.db", std::process::id())));
    drop(Banco(b.0.clone())); // sobra de uma execução anterior
    b
}

/// O app abrindo (de novo) com este banco.
fn abrir(b: &Banco) -> Mcp {
    let m = Mcp::novo("127.0.0.1", false, "blueprint.exe", "http://127.0.0.1:3000", std::env::temp_dir());
    m.persistir_fila(&b.0).unwrap();
    m
}

fn item(tipo: &str, blueprint: &str) -> Map<String, Value> {
    json!({"tipo": tipo, "workspaceId": "sm", "blueprintId": blueprint, "texto": "x"}).as_object().unwrap().clone()
}

#[test]
fn enfileira_reabre_e_a_fila_volta_igual() {
    let b = banco("igual");
    let m = abrir(&b);
    m.empurra_proposta(item("proposta", "p1"), json!({"acoes": [{"id": "+a1"}]}), None);
    m.empurra_registro(item("registro", "p1"));
    m.empurra(item("concluido", "p1"));
    m.empurra_proposta(item("proposta", "p1"), json!({"n": 2}), None);
    let antes = m.itens();
    drop(m);

    let m = abrir(&b);
    assert_eq!(m.itens(), antes, "itens, ordem, números, criadaEm e ids iguais");
    assert_eq!(m.entrada_da_proposta("sm", "p1", 1).unwrap(), json!({"acoes": [{"id": "+a1"}]}), "o ajuste continua possível");
    // a numeração e os ids continuam de onde pararam
    assert_eq!(m.empurra_proposta(item("proposta", "p1"), json!({}), None)["numero"], 3);
    assert_eq!(m.empurra_registro(item("registro", "p1"))["numero"], 2);
    assert_eq!(m.itens().last().unwrap()["id"], 6);
}

#[test]
fn o_ack_tira_do_banco_e_o_item_aplicado_nao_volta() {
    let b = banco("ack");
    let m = abrir(&b);
    let p1 = m.empurra_proposta(item("proposta", "p1"), json!({}), None);
    let r1 = m.empurra_registro(item("registro", "p1"));
    let marca = m.empurra(item("concluido", "p1"));
    m.empurra_proposta(item("proposta", "p1"), json!({}), None);
    m.ack_proposta(&json!({"workspaceId": "sm", "blueprintId": "p1", "numero": 1, "criadaEm": p1["criadaEm"]}));
    m.ack_registro(&json!({"workspaceId": "sm", "blueprintId": "p1", "numero": 1, "criadaEm": r1["criadaEm"]}));
    m.ack(&HashSet::from([marca["id"].as_i64().unwrap()]));
    drop(m);

    let m = abrir(&b);
    let numeros: Vec<(String, i64)> = m.itens().iter().map(|i| (i["tipo"].as_str().unwrap().into(), i["numero"].as_i64().unwrap())).collect();
    assert_eq!(numeros, vec![("proposta".into(), 2)], "só a proposta nº 2, que a página não aplicou");
    // o ajuste troca a proposta no banco também: reabrir não traz a versão velha
    m.empurra_proposta(item("proposta", "p1"), json!({"v": "nova"}), Some(2));
    drop(m);
    let m = abrir(&b);
    assert_eq!(m.itens().len(), 1);
    assert_eq!(m.entrada_da_proposta("sm", "p1", 2).unwrap(), json!({"v": "nova"}));
    // número de proposta já usado nunca volta, mesmo com a fila vazia
    m.ack(&m.itens().iter().filter_map(|i| i["id"].as_i64()).collect());
    drop(m);
    assert_eq!(abrir(&b).empurra_proposta(item("proposta", "p1"), json!({}), None)["numero"], 3);
}

#[test]
fn o_limite_de_200_vale_no_banco() {
    let b = banco("limite");
    let m = abrir(&b);
    for _ in 0..205 {
        m.empurra(item("progresso", "p1"));
    }
    drop(m);
    let itens = abrir(&b).itens();
    assert_eq!((itens.len(), itens[0]["id"].as_i64()), (200, Some(6)));
}
