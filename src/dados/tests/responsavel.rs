//! O responsável do plano (`planos[].responsavel = {id, agente?}`, docs/colaboracao.md): vai em `extra`, sincroniza como qualquer campo do plano, só aceita uma pessoa
//! do workspace do plano e o formato certo, e dado antigo (sem o campo) abre igual.
use crate::contexto::{Contexto, Papel};
use crate::dados::{membros, Autor, Dados, Falha, Op};
use crate::erro::Resultado;
use serde_json::{json, Map, Value};

fn dados() -> Dados {
    let mut d = Dados::em_memoria().unwrap();
    let base = json!({"empresa": "E", "blueprints": [], "workspaces": [
        {"id": "w2", "nome": "Time", "blueprints": [{"id": "b2", "nome": "B", "acoes": [{"id": "a2", "nome": "A", "planos": [{"id": "p2", "texto": "Fazer"}]}]}]},
        {"id": "w3", "nome": "Outro", "blueprints": []}]});
    d.gravar_tudo(&Contexto::local(), &base).unwrap();
    d.transacao(|t| {
        membros::criar(t, "ana", "Ana")?;
        membros::somar_acesso(t, "ana", "w2", "", Papel::Colaborador, None)?;
        membros::criar(t, "gil", "Gil")?;
        membros::somar_acesso(t, "gil", "w3", "", Papel::Colaborador, None)
    })
    .unwrap();
    d
}

fn eu() -> Autor {
    Autor { id: "ana".into(), nome: "Ana".into() }
}

fn obj(v: Value) -> Map<String, Value> {
    v.as_object().unwrap().clone()
}

fn atribuir(d: &mut Dados, versao: u64, responsavel: Value, antes: Value) -> Result<crate::dados::Saida, Falha> {
    let op = Op::Alterar { tipo: "plano".into(), id: "p2".into(), campos: obj(json!({"responsavel": responsavel})), antes: obj(json!({"responsavel": antes})) };
    d.aplicar_operacoes(&eu(), versao, 0, &[op], &|_: bool, _: &str, _: Option<&str>| -> Resultado<()> { Ok(()) })
}

fn plano(d: &mut Dados) -> Value {
    d.ler_tudo(&Contexto::local()).unwrap()["workspaces"][0]["blueprints"][0]["acoes"][0]["planos"][0].clone()
}

#[test]
fn atribuir_trocar_e_tirar_o_responsavel_pelas_operacoes() {
    let mut d = dados();
    let original = plano(&mut d);
    atribuir(&mut d, 101, json!({"id": "ana"}), Value::Null).unwrap();
    assert_eq!(plano(&mut d)["responsavel"], json!({"id": "ana"}));
    atribuir(&mut d, 102, json!({"id": "ana", "agente": true}), json!({"id": "ana"})).unwrap();
    assert_eq!(plano(&mut d)["responsavel"], json!({"id": "ana", "agente": true}), "o agente da pessoa");
    atribuir(&mut d, 103, Value::Null, json!({"id": "ana", "agente": true})).unwrap();
    assert_eq!(plano(&mut d), original, "tirar o responsável volta ao dado de antes, sem sobra");
}

#[test]
fn so_aceita_pessoa_do_workspace_e_o_formato_certo() {
    let mut d = dados();
    for (i, ruim) in [json!({"id": "gil"}), json!({"id": "fantasma"}), json!({"id": ""}), json!({}), json!({"id": "ana", "papel": "dono"}), json!({"id": "ana", "agente": "sim"}), json!("ana"), json!(["ana"]), json!({"id": "a".repeat(65)})].into_iter().enumerate() {
        let r = atribuir(&mut d, 200 + i as u64, ruim.clone(), Value::Null);
        assert!(matches!(r, Err(Falha::Invalida { .. })), "devia recusar {ruim}: {r:?}");
    }
    assert!(plano(&mut d).get("responsavel").is_none(), "o lote inválido não muda nada");
}

#[test]
fn dados_antigos_sem_o_campo_abrem_iguais_e_o_campo_volta_na_ida_e_volta() {
    let mut d = dados();
    assert!(plano(&mut d).get("responsavel").is_none());
    let mut e = Dados::em_memoria().unwrap();
    let entrada = json!({"empresa": "E", "blueprints": [{"id": "b", "nome": "B", "acoes": [{"id": "a", "nome": "A", "planos": [{"id": "p", "texto": "T", "responsavel": {"id": "ana", "agente": true}}]}]}]});
    e.gravar_tudo(&Contexto::local(), &entrada).unwrap();
    assert_eq!(e.ler_tudo(&Contexto::local()).unwrap().clone(), entrada);
}

#[test]
fn criar_um_plano_ja_com_responsavel_tambem_e_conferido() {
    let mut d = dados();
    let criar = |id: &str, r: Value| Op::Criar { tipo: "plano".into(), id: id.into(), pai: Some("a2".into()), posicao: 1, valor: obj(json!({"texto": "Novo", "responsavel": r})) };
    let livre = |_: bool, _: &str, _: Option<&str>| -> Resultado<()> { Ok(()) };
    assert!(d.aplicar_operacoes(&eu(), 300, 0, &[criar("p3", json!({"id": "gil"}))], &livre).is_err());
    d.aplicar_operacoes(&eu(), 301, 0, &[criar("p4", json!({"id": "ana"}))], &livre).unwrap();
    let tudo = d.ler_tudo(&Contexto::local()).unwrap();
    assert_eq!(tudo["workspaces"][0]["blueprints"][0]["acoes"][0]["planos"][1]["responsavel"], json!({"id": "ana"}));
}

#[test]
fn ler_blueprint_traz_o_responsavel_e_quem_e_voce_so_para_membros() {
    let mut d = dados();
    atribuir(&mut d, 400, json!({"id": "ana", "agente": true}), Value::Null).unwrap();
    let tudo = d.ler_tudo(&Contexto::local()).unwrap().clone();
    let lido = crate::mcp::acoes::blueprint_lido(&tudo, "w2", "b2", None, "tudo", "").unwrap();
    assert_eq!(lido["blueprint"]["acoes"][0]["planos"][0]["responsavel"], json!({"id": "ana", "agente": true}));
    let acao = crate::mcp::acoes::blueprint_lido(&tudo, "w2", "b2", Some("a2"), "tudo", "").unwrap();
    assert_eq!(acao["acao"]["planos"][0]["responsavel"], json!({"id": "ana", "agente": true}));
}
