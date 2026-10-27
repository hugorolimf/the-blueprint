//! Os casos do antigo plano.test.mjs, um por um, mais os de mover e remover: as mesmas entradas devem ser aceitas e recusadas com os mesmos erros.
use super::{dia_valido, validar_proposta, Validacao};
use serde_json::{json, Value};

const HOJE: &str = "2026-10-21";

fn blueprint() -> Value {
    json!({"id": "p", "nome": "Portal", "descricao": "d", "obs": "o", "acoes": [
        {"id": "f1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "andamento", "obs": "obs",
         "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "x", "feito": true}], "checkpoints": [{"id": "m1", "nome": "Rota no ar", "data": "2026-11-20", "feito": false}]}]})
}

fn plano(n: u32) -> Value {
    json!({"texto": format!("Criar item {n}"), "descricao": "Escopo: x. Resultado: y. Validação: node x.test.mjs passa."})
}

fn com(mut base: Value, extra: Value) -> Value {
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn acao(extra: Value) -> Value {
    com(json!({"nome": "Ação nova", "inicio": "2026-12-01", "fim": "2026-12-31", "status": "planejado", "observacao": "", "planos": [plano(1), plano(2), plano(3)], "checkpoints": [{"nome": "Entrega validada", "data": "2026-12-30"}]}), extra)
}

fn ok(entrada: Value, p: Option<&Value>) -> Validacao {
    let r = validar_proposta(&entrada, p, HOJE);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    r
}

fn falha(entrada: Value, trecho: &str, p: Option<&Value>) {
    let r = validar_proposta(&entrada, p, HOJE);
    assert!(r.erros.iter().any(|e| e.contains(trecho)), "esperava erro com \"{trecho}\"; veio: {:?}", r.erros);
    assert!(r.proposta.is_none());
}

#[test]
fn as_datas_existem_no_calendario() {
    for (dia, vale) in [("2026-10-21", true), ("2024-02-29", true), ("2026-02-29", false), ("2026-02-30", false), ("2026-13-01", false), ("2026-00-10", false), ("01/12/2026", false), ("2026-1-1", false), ("", false), ("2000-02-29", true), ("1900-02-29", false)] {
        assert_eq!(dia_valido(dia), vale, "{dia}");
    }
}

#[test]
fn instrucoes_longas_cabem() {
    let longa = "Passo a passo. ".repeat(900);
    let r = ok(json!({"blueprint": {"nome": "N", "descricao": "d"}, "acoes": [acao(json!({"observacao": longa}))]}), None);
    assert_eq!(r.proposta.unwrap()["acoes"][0]["observacao"].as_str().unwrap().chars().count(), longa.trim().chars().count());
}

#[test]
fn proposta_valida_e_normalizada() {
    let r = ok(json!({"mensagem": " ok ", "acoes": [acao(json!({}))]}), Some(&blueprint()));
    let proposta = r.proposta.unwrap();
    assert_eq!(proposta["acoes"][0]["id"], "");
    assert_eq!(proposta["mensagem"], "ok");
    assert_eq!(r.avisos.len(), 1, "{:?}", r.avisos);
    assert!(r.avisos[0].contains("dura 31 dias"), "uma ação de 31 dias sem motivo avisa: {:?}", r.avisos);
}

#[test]
fn reaproveitar_acao_plano_e_checkpoint_existentes() {
    let p = blueprint();
    let edit = ok(json!({"acoes": [acao(json!({"id": "f1", "inicio": "2026-11-01", "fim": "2026-11-30",
        "planos": [com(plano(1), json!({"id": "t1"})), plano(2), plano(3)], "checkpoints": [{"id": "m1", "nome": "Rota no ar", "data": "2026-11-25"}]}))]}), Some(&p));
    let f = &edit.proposta.unwrap()["acoes"][0];
    assert_eq!(f["id"], "f1");
    assert_eq!(f["planos"][0]["id"], "t1");
    falha(json!({"acoes": [acao(json!({"id": "inventado"}))]}), "o id \"inventado\" não existe", Some(&p));
    falha(json!({"acoes": [acao(json!({"id": "f1", "inicio": "2026-11-01", "fim": "2026-11-30", "planos": [com(plano(1), json!({"id": "t9"}))]}))]}), "o id \"t9\" não existe nesta ação", Some(&p));
    falha(json!({"acoes": [acao(json!({"id": "f1", "inicio": "2026-11-01", "fim": "2026-11-30", "checkpoints": [{"id": "m9", "nome": "x", "data": "2026-11-10"}]}))]}), "o id \"m9\" não existe", Some(&p));
}

#[test]
fn formato_lista_de_acoes_datas_status_e_periodos() {
    let p = blueprint();
    falha(Value::Null, "precisa ser um objeto", Some(&p));
    falha(json!({}), "`acoes` é obrigatório", Some(&p));
    falha(json!({"acoes": []}), "`acoes` é obrigatório", Some(&p));
    falha(json!({"acoes": [acao(json!({"inicio": "01/12/2026"}))]}), "`inicio` deve ser uma data aaaa-mm-dd", Some(&p));
    falha(json!({"acoes": [acao(json!({"inicio": "2026-02-30"}))]}), "`inicio` deve ser uma data", Some(&p));
    falha(json!({"acoes": [acao(json!({"fim": "2026-11-01"}))]}), "anterior a `inicio`", Some(&p));
    falha(json!({"acoes": [acao(json!({"status": "concluido"}))]}), "status concluído não pode ser proposto", Some(&p));
    falha(json!({"acoes": [acao(json!({"nome": ""}))]}), "falta o nome", Some(&p));
    falha(json!({"acoes": (0..21).map(|_| acao(json!({}))).collect::<Vec<_>>()}), "Ações demais", Some(&p));
}

#[test]
fn planos_titulo_descricao_repeticao_e_quantidade() {
    let p = blueprint();
    let longo = "Criar um título que passa bastante dos quarenta e cinco caracteres";
    let r = ok(json!({"acoes": [acao(json!({"planos": [com(plano(1), json!({"texto": longo})), plano(2), plano(3)]}))]}), Some(&p));
    assert_eq!(r.proposta.unwrap()["acoes"][0]["planos"][0]["texto"], longo);
    falha(json!({"acoes": [acao(json!({"planos": [com(plano(1), json!({"descricao": ""})), plano(2), plano(3)]}))]}), "falta a descrição", Some(&p));
    falha(json!({"acoes": [acao(json!({"planos": [plano(1), plano(1), plano(3)]}))]}), "título repetido", Some(&p));
    let r = ok(json!({"acoes": [acao(json!({"planos": (0..13).map(plano).collect::<Vec<_>>()}))]}), Some(&p));
    assert_eq!(r.proposta.unwrap()["acoes"][0]["planos"].as_array().unwrap().len(), 13);
    let avisado = validar_proposta(&json!({"acoes": [acao(json!({"planos": [{"texto": "Pasta", "descricao": "só isso"}, plano(2)]}))]}), Some(&p), HOJE);
    assert!(avisado.erros.is_empty());
    assert!(avisado.avisos.iter().any(|a| a.contains("infinitivo")) && avisado.avisos.iter().any(|a| a.contains("como validar")) && avisado.avisos.iter().any(|a| a.contains("2 plano(s)")));
}

#[test]
fn checkpoints_dentro_do_periodo_com_data_e_nome() {
    let p = blueprint();
    falha(json!({"acoes": [acao(json!({"checkpoints": [{"nome": "Fora", "data": "2027-01-10"}]}))]}), "fora do período da ação", Some(&p));
    falha(json!({"acoes": [acao(json!({"checkpoints": [{"nome": "", "data": "2026-12-10"}]}))]}), "falta o nome", Some(&p));
    falha(json!({"acoes": [acao(json!({"checkpoints": [{"nome": "Sem data"}]}))]}), "`data` deve ser aaaa-mm-dd", Some(&p));
}

#[test]
fn acao_no_passado_so_avisa_e_sem_planos_e_esqueleto() {
    let r = validar_proposta(&json!({"acoes": [acao(json!({"inicio": "2026-09-01", "fim": "2026-09-30", "planos": [], "checkpoints": []}))]}), Some(&blueprint()), HOJE);
    assert!(r.erros.is_empty());
    assert!(r.avisos.iter().any(|a| a.contains("antes de hoje")) && r.avisos.iter().any(|a| a.contains("esqueleto")));
}

#[test]
fn blueprint_novo_exige_nome_e_o_existente_ignora_com_aviso() {
    falha(json!({"acoes": [acao(json!({}))]}), "Blueprint novo: informe", None);
    let novo = ok(json!({"blueprint": {"nome": "  Meu app  ", "descricao": "Para tudo"}, "acoes": [acao(json!({}))]}), None);
    assert_eq!(novo.proposta.unwrap()["blueprint"], json!({"nome": "Meu app", "descricao": "Para tudo"}));
    assert!(validar_proposta(&json!({"blueprint": {"nome": "Outro"}, "acoes": [acao(json!({}))]}), Some(&blueprint()), HOJE).avisos.iter().any(|a| a.contains("ignorado")));
}

fn blueprint_pela_metade() -> Value {
    json!({"id": "p", "acoes": [{"id": "f1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "concluido", "obs": "instruções longas",
        "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "desc 1", "feito": true}, {"id": "t2", "texto": "Criar tela", "descricao": "desc 2"}, {"id": "t3", "texto": "Testar", "descricao": ""}],
        "checkpoints": [{"id": "m1", "nome": "Rota no ar", "data": "2026-11-20"}]}]})
}

#[test]
fn acao_existente_reenviada_pela_metade_mantem_o_resto() {
    let p2 = blueprint_pela_metade();
    let r = ok(json!({"acoes": [{"id": "f1", "fim": "2026-12-05", "planos": [{"id": "t2", "texto": "Criar tela nova"}, plano(9)]}]}), Some(&p2));
    let parcial = &r.proposta.unwrap()["acoes"][0];
    assert_eq!(
        [&parcial["nome"], &parcial["inicio"], &parcial["fim"], &parcial["status"], &parcial["observacao"]],
        [&json!("Base"), &json!("2026-11-01"), &json!("2026-12-05"), &json!("andamento"), &json!("instruções longas")]
    );
    let planos: Vec<(String, String, String)> = parcial["planos"].as_array().unwrap().iter().map(|t| (t["id"].as_str().unwrap().into(), t["texto"].as_str().unwrap().into(), t["descricao"].as_str().unwrap().into())).collect();
    assert_eq!(
        planos,
        vec![("t2".into(), "Criar tela nova".into(), "desc 2".into()), ("".into(), "Criar item 9".into(), plano(9)["descricao"].as_str().unwrap().into()), ("t1".into(), "Criar rota".into(), "desc 1".into()), ("t3".into(), "Testar".into(), "".into())]
    );
    assert_eq!(parcial["checkpoints"], json!([{"id": "m1", "nome": "Rota no ar", "data": "2026-11-20"}]));
    assert!(validar_proposta(&json!({"acoes": [{"id": "f1", "planos": [{"id": "t2"}]}]}), Some(&p2), HOJE).avisos.iter().any(|a| a.contains("2 plano(s) existente(s) não vieram")));
    assert_eq!(ok(json!({"acoes": [{"id": "f1"}]}), Some(&p2)).proposta.unwrap()["acoes"][0]["planos"].as_array().unwrap().len(), 3, "só o id da ação: tudo fica igual");
    falha(json!({"acoes": [{"id": "f1", "planos": [{"texto": "Criar sem descrição"}]}]}), "falta a descrição", Some(&p2));
    falha(json!({"acoes": [{"id": "f1", "checkpoints": [{"id": "m1", "data": "2027-01-01"}]}]}), "fora do período", Some(&p2));
}

fn blueprint_de_duas_acoes() -> Value {
    json!({"id": "p", "acoes": [
        {"id": "fa", "nome": "A", "inicio": "2026-11-01", "fim": "2026-11-10", "status": "planejado", "planos": [{"id": "a1", "texto": "Criar A1", "descricao": "da", "feito": true}, {"id": "a2", "texto": "Criar A2", "descricao": "d2"}], "checkpoints": [{"id": "ma", "nome": "Checkpoint A", "data": "2026-11-05"}]},
        {"id": "fb", "nome": "B", "inicio": "2026-11-11", "fim": "2026-11-20", "status": "planejado", "planos": [{"id": "b1", "texto": "Criar B1", "descricao": "db"}], "checkpoints": []}]})
}

#[test]
fn mover_leva_o_item_e_a_origem_nao_o_mantem_de_volta() {
    let p3 = blueprint_de_duas_acoes();
    let mov = ok(json!({"acoes": [
        {"nome": "Fim", "inicio": "2026-12-01", "fim": "2026-12-10", "planos": [{"id": "a1"}, {"id": "b1", "texto": "Criar B1 final"}], "checkpoints": [{"id": "ma", "data": "2026-12-05"}]},
        {"id": "fa"}]}), Some(&p3)).proposta.unwrap();
    assert_eq!(mov["acoes"][0]["planos"], json!([{"id": "a1", "texto": "Criar A1", "descricao": "da", "de": "fa"}, {"id": "b1", "texto": "Criar B1 final", "descricao": "db", "de": "fb"}]));
    assert_eq!(mov["acoes"][0]["checkpoints"], json!([{"id": "ma", "nome": "Checkpoint A", "data": "2026-12-05", "de": "fa"}]));
    let origem: Vec<&str> = mov["acoes"][1]["planos"].as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert_eq!(origem, vec!["a2"]);
    assert_eq!(mov["acoes"][1]["checkpoints"], json!([]));
    falha(json!({"acoes": [{"nome": "Fim", "inicio": "2026-12-01", "fim": "2026-12-10", "planos": [], "checkpoints": [{"id": "ma"}]}]}), "fora do período", Some(&p3));
    falha(json!({"acoes": [{"nome": "Fim", "inicio": "2026-12-01", "fim": "2026-12-10", "planos": [{"id": "zz"}], "checkpoints": []}]}), "nem em outra do blueprint", Some(&p3));
}

#[test]
fn remover_so_monta_o_pedido_com_nomes_motivo_e_aviso() {
    let p3 = blueprint_de_duas_acoes();
    let r = ok(json!({"remover": [{"acao_id": "fa", "plano_id": "a1", "motivo": "vai para o fim"}, {"acao_id": "fa", "checkpoint_id": "ma"}, {"acao_id": "fb"}]}), Some(&p3));
    let proposta = r.proposta.unwrap();
    assert_eq!(proposta["acoes"], json!([]));
    assert_eq!(proposta["remover"][0], json!({"acaoId": "fa", "acao": "A", "planoId": "a1", "plano": "Criar A1", "checkpointId": "", "checkpoint": "", "motivo": "vai para o fim"}));
    assert_eq!(proposta["remover"][1]["checkpoint"], "Checkpoint A");
    assert_eq!(proposta["remover"][2]["planoId"], "");
    assert!(r.avisos.iter().any(|a| a.contains("já marcado como feito")));
    falha(json!({"remover": [{"acao_id": "nao"}]}), "não existe neste blueprint", Some(&p3));
    falha(json!({"remover": [{"acao_id": "fa", "plano_id": "b1"}]}), "não existe na ação", Some(&p3));
    falha(json!({"remover": [{"acao_id": "fa", "plano_id": "a2", "checkpoint_id": "ma"}]}), "não os dois", Some(&p3));
    falha(json!({"remover": [{"acao_id": "fa"}], "acoes": [{"id": "fa", "nome": "A"}]}), "escolha uma", Some(&p3));
    falha(json!({"remover": [{"acao_id": "fa", "plano_id": "a2"}], "acoes": [{"nome": "Fim", "inicio": "2026-12-01", "fim": "2026-12-10", "planos": [{"id": "a2"}], "checkpoints": []}]}), "já é movido", Some(&p3));
}
