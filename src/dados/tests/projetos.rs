//! Projetos dentro do blueprint (`blueprints[].projetos` e `acoes[].projetoId`) vão na coluna `extra`: sem migração e sem perda,
//! com ou sem projetos, e dados antigos (sem esses campos) voltam exatamente iguais.
use super::exemplo;
use crate::contexto::Contexto;
use crate::dados::Dados;
use serde_json::{json, Value};

fn volta(entrada: &Value) -> Value {
    let (mut d, ctx) = (Dados::em_memoria().unwrap(), Contexto::local());
    d.gravar_tudo(&ctx, entrada).unwrap();
    (*d.ler_tudo(&ctx).unwrap()).clone()
}

fn acao(id: &str, extra: Value) -> Value {
    let mut a = json!({"id": id, "nome": id, "inicio": "2026-11-01", "fim": "2026-11-30", "status": "planejado", "progresso": 0, "obs": "",
        "planos": [{"id": format!("{id}-t"), "texto": "Fazer", "feito": false}], "checkpoints": []});
    a.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    a
}

#[test]
fn projetos_e_projeto_id_voltam_sem_perda() {
    let blueprint = |b: &str, a: &str| json!({"id": b, "nome": "B", "descricao": "d", "obs": "", "links": [],
        "projetos": [{"id": "pr1", "nome": "Interface"}, {"id": "pr2", "nome": "Servidor"}],
        "acoes": [acao(&format!("{a}1"), json!({"projetoId": "pr1"})), acao(&format!("{a}2"), json!({"projetoId": "pr2"})), acao(&format!("{a}3"), json!({})), acao(&format!("{a}4"), json!({"projetoId": "apagado"}))]});
    let entrada = json!({"empresa": "E", "blueprints": [blueprint("b1", "a")], "workspaces": [{"id": "w1", "nome": "W", "blueprints": [blueprint("b2", "x")]}]});
    assert_eq!(volta(&entrada), entrada);
}

#[test]
fn sem_projetos_nada_aparece_nem_some() {
    // dado antigo: o blueprint não tem `projetos`, nenhuma ação tem `projetoId`, e a volta não inventa os campos
    let exemplo = exemplo();
    let saida = volta(&exemplo);
    assert_eq!(saida, exemplo);
    assert!(!saida.to_string().contains("projetoId") && !saida.to_string().contains("\"projetos\""));
}

#[test]
fn projeto_vazio_e_lista_vazia_tambem_voltam() {
    let entrada = json!({"empresa": "E", "blueprints": [{"id": "b1", "nome": "B", "projetos": [], "acoes": [acao("a1", json!({"projetoId": ""})), acao("a2", json!({}))]}]});
    assert_eq!(volta(&entrada), entrada);
}
