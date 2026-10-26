//! `aplicar_operacoes`: os cenários de conflito do desenho (docs/colaboracao.md, seção 6), as operações de estrutura e o histórico.
use crate::contexto::Contexto;
use crate::dados::{Autor, Dados, Falha, Op};
use crate::erro::{Erro, Resultado};
use serde_json::{json, Map, Value};

fn base() -> Value {
    json!({"empresa": "E", "blueprints": [{"id": "b1", "nome": "B", "acoes": [
        {"id": "a1", "nome": "A1", "planos": [{"id": "p1", "texto": "velho", "descricao": "d"}, {"id": "p2", "texto": "dois"}], "checkpoints": []},
        {"id": "a9", "nome": "A9", "planos": [{"id": "p9", "texto": "nove"}]}]}],
        "workspaces": [{"id": "w2", "nome": "Outro", "blueprints": [{"id": "b2", "nome": "Dele", "acoes": [{"id": "a2", "nome": "A2", "planos": []}]}]}]})
}

fn dados() -> Dados {
    let mut d = Dados::em_memoria().unwrap();
    d.gravar_tudo(&Contexto::local(), &base()).unwrap();
    d
}

fn pessoa(id: &str) -> Autor {
    Autor { id: id.into(), nome: id.to_uppercase() }
}

fn obj(v: Value) -> Map<String, Value> {
    v.as_object().unwrap().clone()
}

fn alterar(tipo: &str, id: &str, campos: Value, antes: Value) -> Op {
    Op::Alterar { tipo: tipo.into(), id: id.into(), campos: obj(campos), antes: obj(antes) }
}

fn livre(_: bool, _: &str, _: Option<&str>) -> Resultado<()> {
    Ok(())
}

fn plano(d: &mut Dados, acao: usize, id: &str) -> Value {
    let tudo = d.ler_tudo(&Contexto::local()).unwrap();
    tudo["blueprints"][0]["acoes"][acao]["planos"].as_array().unwrap().iter().find(|p| p["id"] == id).cloned().unwrap_or(Value::Null)
}

#[test]
fn campos_diferentes_do_mesmo_plano_ficam_os_dois_sem_aviso() {
    let mut d = dados();
    let s = d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar("plano", "p1", json!({"texto": "da Ana"}), json!({"texto": "velho"}))], &livre).unwrap();
    assert!(s.conflitos.is_empty());
    let s = d.aplicar_operacoes(&pessoa("bruno"), 102, 0, &[alterar("plano", "p1", json!({"descricao": "do Bruno"}), json!({"descricao": "d"}))], &livre).unwrap();
    assert!(s.conflitos.is_empty() && s.ignoradas.is_empty());
    let p = plano(&mut d, 0, "p1");
    assert_eq!((p["texto"].as_str(), p["descricao"].as_str()), (Some("da Ana"), Some("do Bruno")));
}

#[test]
fn o_mesmo_campo_vale_a_ultima_gravacao_e_avisa_quem_perdeu() {
    let mut d = dados();
    d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar("plano", "p1", json!({"texto": "A"}), json!({"texto": "velho"}))], &livre).unwrap();
    let s = d.aplicar_operacoes(&pessoa("bruno"), 102, 0, &[alterar("plano", "p1", json!({"texto": "B"}), json!({"texto": "velho"}))], &livre).unwrap();
    assert_eq!(s.conflitos.len(), 1);
    let c = &s.conflitos[0];
    assert_eq!((&c["campo"], &c["deles"], &c["seu"], &c["quem"]), (&json!("texto"), &json!("A"), &json!("B"), &json!({"id": "ana", "nome": "ANA"})));
    assert!(c["em"].is_string());
    assert_eq!(plano(&mut d, 0, "p1")["texto"], "B");
}

#[test]
fn duas_janelas_da_mesma_pessoa_tambem_tem_o_aviso_com_ela_mesma() {
    let mut d = dados();
    d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar("plano", "p1", json!({"texto": "janela 1"}), json!({"texto": "velho"}))], &livre).unwrap();
    let s = d.aplicar_operacoes(&pessoa("ana"), 102, 0, &[alterar("plano", "p1", json!({"texto": "janela 2"}), json!({"texto": "velho"}))], &livre).unwrap();
    assert_eq!(s.conflitos[0]["quem"]["id"], "ana");
    assert_eq!(plano(&mut d, 0, "p1")["texto"], "janela 2");
}

#[test]
fn apagar_vence_editar_e_diz_quem_apagou() {
    let mut d = dados();
    d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[Op::Apagar { tipo: "acao".into(), id: "a9".into() }], &livre).unwrap();
    let s = d.aplicar_operacoes(&pessoa("bruno"), 102, 0, &[alterar("plano", "p9", json!({"texto": "editado"}), json!({"texto": "nove"}))], &livre).unwrap();
    assert!(s.aplicadas.is_empty());
    assert_eq!(s.ignoradas, vec![json!({"indice": 0, "codigo": "item-apagado", "tipo": "plano", "id": "p9", "quem": {"id": "ana", "nome": "ANA"}})]);
    assert_eq!(d.ler_tudo(&Contexto::local()).unwrap()["blueprints"][0]["acoes"].as_array().unwrap().len(), 1);
}

#[test]
fn vazio_normalizado_pela_pagina_nao_e_conflito() {
    let mut d = dados();
    // a ação não tem `obs` no banco; a página tinha `obs: ""` (normalizado)
    let s = d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar("acao", "a1", json!({"obs": "nota"}), json!({"obs": ""}))], &livre).unwrap();
    assert!(s.conflitos.is_empty());
}

#[test]
fn criar_mover_e_apagar_mantem_a_ordem_e_as_listas() {
    let mut d = dados();
    let ops = [
        Op::Criar { tipo: "plano".into(), id: "p3".into(), pai: Some("a1".into()), posicao: 1, valor: obj(json!({"texto": "meio", "feito": false})) },
        Op::Mover { tipo: "plano".into(), id: "p2".into(), pai: Some("a1".into()), posicao: 0 },
        Op::Criar { tipo: "checkpoint".into(), id: "c1".into(), pai: Some("a9".into()), posicao: 9, valor: obj(json!({"nome": "marco"})) },
        Op::Mover { tipo: "plano".into(), id: "p9".into(), pai: Some("a1".into()), posicao: 99 },
    ];
    let s = d.aplicar_operacoes(&pessoa("ana"), 101, 0, &ops, &livre).unwrap();
    assert_eq!(s.aplicadas.len(), 4);
    let tudo = d.ler_tudo(&Contexto::local()).unwrap().clone();
    let ids: Vec<&str> = tudo["blueprints"][0]["acoes"][0]["planos"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["p2", "p1", "p3", "p9"]);
    assert_eq!(tudo["blueprints"][0]["acoes"][1]["checkpoints"], json!([{"id": "c1", "nome": "marco"}]), "a lista nova aparece no JSON");
    assert_eq!(tudo["blueprints"][0]["acoes"][1]["planos"], json!([]));
    let s = d.aplicar_operacoes(&pessoa("ana"), 102, 0, &[Op::Criar { tipo: "plano".into(), id: "p1".into(), pai: Some("a1".into()), posicao: 0, valor: obj(json!({})) }], &livre).unwrap();
    assert_eq!(s.ignoradas[0]["codigo"], "id-existe");
}

#[test]
fn criar_workspace_e_subarvore_e_apagar_grava_o_historico_de_cada_descendente() {
    let mut d = dados();
    let valor = obj(json!({"nome": "Novo", "blueprints": [{"id": "b7", "nome": "B7", "acoes": [{"id": "a7", "planos": [{"id": "p7", "texto": "x"}]}]}]}));
    d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[Op::Criar { tipo: "workspace".into(), id: "w7".into(), pai: None, posicao: 0, valor }], &livre).unwrap();
    let tudo = d.ler_tudo(&Contexto::local()).unwrap().clone();
    assert_eq!(tudo["workspaces"][0]["id"], "w7", "posição 0 entre os workspaces (o raiz continua sendo o arquivo)");
    assert_eq!(tudo["workspaces"][0]["blueprints"][0]["acoes"][0]["planos"][0]["texto"], "x");
    d.aplicar_operacoes(&pessoa("bruno"), 102, 0, &[Op::Apagar { tipo: "workspace".into(), id: "w7".into() }], &livre).unwrap();
    let h = d.historico(&Contexto::local(), "w7", None, None, 100).unwrap();
    let apagados: Vec<&str> = h.as_array().unwrap().iter().filter(|l| l["operacao"] == "apagar").map(|l| l["itemId"].as_str().unwrap()).collect();
    assert_eq!(apagados, ["w7", "b7", "a7", "p7"], "do mais recente ao mais antigo: o item por último");
}

#[test]
fn operacao_invalida_ou_proibida_derruba_o_lote_inteiro() {
    let mut d = dados();
    let ops = [alterar("plano", "p1", json!({"texto": "não fica"}), json!({})), alterar("acao", "a1", json!({"planos": []}), json!({}))];
    assert!(matches!(d.aplicar_operacoes(&pessoa("ana"), 101, 0, &ops, &livre), Err(Falha::Invalida { indice: 1, .. })));
    assert_eq!(plano(&mut d, 0, "p1")["texto"], "velho");
    let so_w2 = |_: bool, ws: &str, _: Option<&str>| if ws == "w2" { Ok(()) } else { Err(Erro::Proibido("não".into())) };
    let ops = [alterar("acao", "a2", json!({"nome": "ok"}), json!({})), alterar("plano", "p1", json!({"texto": "outro workspace"}), json!({}))];
    assert!(matches!(d.aplicar_operacoes(&pessoa("ana"), 101, 0, &ops, &so_w2), Err(Falha::Proibida { indice: 1, .. })));
    assert_eq!(d.ler_tudo(&Contexto::local()).unwrap()["workspaces"][0]["blueprints"][0]["acoes"][0]["nome"], "A2", "a primeira também voltou");
    assert!(matches!(d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[Op::Apagar { tipo: "workspace".into(), id: "sm".into() }], &livre), Err(Falha::Invalida { .. })));
}

#[test]
fn o_agente_com_comentarios_troca_por_inteiro_e_o_historico_corta_valores_grandes() {
    let mut d = dados();
    let agente = json!({"estado": "concluido", "comentarios": [{"texto": "feito", "em": "x", "tipo": "comentario", "sessaoId": ""}]});
    d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar("plano", "p1", json!({"agente": agente.clone(), "nota": "y".repeat(5000)}), json!({}))], &livre).unwrap();
    assert_eq!(plano(&mut d, 0, "p1")["agente"], agente);
    let h = d.historico(&Contexto::local(), "sm", Some("b1"), Some("p1"), 10).unwrap();
    let nota = h.as_array().unwrap().iter().find(|l| l["campo"] == "nota").unwrap();
    assert_eq!((nota["depois"].clone(), nota["membro"]["nome"].clone(), nota["versao"].clone()), (json!({"cortado": true}), json!("ANA"), json!(101)));
    // remover o campo (null) tira a chave
    d.aplicar_operacoes(&pessoa("ana"), 102, 0, &[alterar("plano", "p1", json!({"nota": null}), json!({}))], &livre).unwrap();
    assert!(plano(&mut d, 0, "p1").get("nota").is_none());
}

#[test]
fn mover_entre_workspaces_avisa_a_origem_e_o_destino() {
    let mut d = dados();
    let s = d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[Op::Mover { tipo: "acao".into(), id: "a2".into(), pai: Some("b1".into()), posicao: 0 }], &livre).unwrap();
    let onde: Vec<(&str, &str)> = s.aplicadas.iter().map(|a| (a.workspace_id.as_str(), a.blueprint_id.as_str())).collect();
    assert_eq!(onde, [("w2", "b2"), ("sm", "b1")]);
    assert_eq!(d.ler_tudo(&Contexto::local()).unwrap()["blueprints"][0]["acoes"][0]["id"], "a2");
}
