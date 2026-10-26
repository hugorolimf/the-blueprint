use super::recebido;
use crate::contexto::{Acesso, Contexto, Papel};
use crate::dados::Autor;
use crate::sincronia::presenca::INTERVALO;
use crate::sincronia::{Onde, Sincronia};
use serde_json::{json, Value};
use std::time::Duration;

fn pessoa(id: &str) -> Autor {
    Autor { id: id.into(), nome: id.to_uppercase() }
}

fn membro(id: &str) -> Contexto {
    Contexto::membro(id, &id.to_uppercase(), vec![Acesso::novo("w1", "", Papel::Colaborador)])
}

/// A última lista de presença recebida.
fn ultima_lista(texto: &str) -> Value {
    let linha = texto.split("event: presenca\ndata: ").last().unwrap().lines().next().unwrap();
    serde_json::from_str(linha).unwrap()
}

#[test]
fn entra_ao_abrir_atualiza_e_sai_ao_fechar() {
    let s = Sincronia::nova(1, true);
    let (rx, _g) = s.assinar("w1", None, &membro("ana"), Some("c-a"));
    assert_eq!(ultima_lista(&recebido(&rx))["pessoas"][0]["id"], "ana");
    let (_rx_b, g_b) = s.assinar("w1", None, &membro("bruno"), Some("c-b"));
    std::thread::sleep(INTERVALO + Duration::from_millis(100)); // a segunda lista sai no fim do intervalo
    assert_eq!(ultima_lista(&recebido(&rx))["pessoas"].as_array().unwrap().len(), 2);
    s.atualizar_presenca(&pessoa("bruno"), "c-b", Onde { blueprint_id: "b1".into(), acao_id: "a1".into(), editando: Some(json!({"tipo": "plano", "id": "p1"})) });
    std::thread::sleep(INTERVALO + Duration::from_millis(100));
    let lista = ultima_lista(&recebido(&rx));
    let bruno = lista["pessoas"].as_array().unwrap().iter().find(|p| p["id"] == "bruno").unwrap().clone();
    assert_eq!(bruno, json!({"id": "bruno", "nome": "BRUNO", "cliente": "c-b", "blueprintId": "b1", "acaoId": "a1", "editando": {"tipo": "plano", "id": "p1"}}));
    drop(g_b);
    std::thread::sleep(INTERVALO + Duration::from_millis(100));
    assert_eq!(ultima_lista(&recebido(&rx))["pessoas"].as_array().unwrap().len(), 1);
}

#[test]
fn sem_presenca_no_modo_local_nada_e_enviado() {
    let s = Sincronia::nova(1, false);
    let (rx, _g) = s.assinar("w1", None, &membro("ana"), Some("c-a"));
    s.atualizar_presenca(&pessoa("ana"), "c-a", Onde::default());
    assert_eq!(recebido(&rx), "");
}

#[test]
fn varias_mudancas_no_intervalo_viram_uma_lista_so() {
    let s = Sincronia::nova(1, true);
    let (rx, _g) = s.assinar("w1", None, &membro("ana"), Some("c-a"));
    recebido(&rx);
    for i in 0..20 {
        s.atualizar_presenca(&pessoa("ana"), "c-a", Onde { acao_id: format!("a{i}"), ..Onde::default() });
    }
    std::thread::sleep(INTERVALO + Duration::from_millis(150));
    let texto = recebido(&rx);
    assert!(texto.matches("event: presenca").count() <= 2, "{texto}");
    assert_eq!(ultima_lista(&texto)["pessoas"][0]["acaoId"], "a19");
}
