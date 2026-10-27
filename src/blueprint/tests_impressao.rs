//! Impressão por projeto em `ler_blueprint`: o projeto que não mudou mantém a figura; uma data nova (ou texto novo) a refaz.
use super::estrutura_blueprint;
use serde_json::{json, Value};

fn blueprint() -> Value {
    json!({"id": "p", "nome": "Portal", "descricao": "d", "projetos": [{"id": "pr1", "nome": "Interface", "descricao": "Telas"}, {"id": "pr2", "nome": "Servidor"}], "acoes": [
        {"id": "a1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-03", "status": "andamento", "obs": "interno", "projetoId": "pr1",
         "planos": [{"id": "t1", "texto": "Criar rota", "feito": false}], "checkpoints": [{"id": "c1", "nome": "No ar", "data": "2026-11-03"}]},
        {"id": "a2", "nome": "API", "inicio": "2026-11-04", "fim": "2026-11-06", "status": "planejado", "projetoId": "pr2", "planos": [], "checkpoints": []},
        {"id": "a3", "nome": "Solta", "inicio": "2026-11-07", "fim": "2026-11-08", "status": "planejado", "planos": [], "checkpoints": []}]})
}

fn impressoes(p: &Value) -> (String, String) {
    let e = estrutura_blueprint(p);
    let de = |i: usize| e["projetos"][i]["impressao"].as_str().unwrap().to_string();
    (de(0), de(1))
}

#[test]
fn projeto_sem_mudanca_mantem_a_figura_e_data_nova_a_refaz() {
    let p = blueprint();
    let (i1, i2) = impressoes(&p);
    assert_eq!((i1.len(), i2.len()), (16, 16), "hash curto em hexadecimal");
    assert_ne!(i1, i2);
    assert_eq!(impressoes(&p), (i1.clone(), i2.clone()), "estável entre leituras");

    let mut data = p.clone();
    data["acoes"][0]["fim"] = json!("2026-11-04");
    assert_eq!(impressoes(&data).1, i2, "o outro projeto mantém a impressão");
    assert_ne!(impressoes(&data).0, i1, "uma data nova refaz a figura do projeto");

    let mut texto = p.clone();
    texto["projetos"][0]["descricao"] = json!("Telas novas");
    assert_ne!(impressoes(&texto).0, i1, "texto do projeto muda a impressão");
    let mut checkpoint = p.clone();
    checkpoint["acoes"][0]["checkpoints"][0]["data"] = json!("2026-11-02");
    assert_ne!(impressoes(&checkpoint).0, i1, "data de checkpoint também");
}

#[test]
fn marcar_plano_e_mexer_fora_do_projeto_nao_refazem() {
    let p = blueprint();
    let (i1, i2) = impressoes(&p);
    let mut feito = p.clone();
    feito["acoes"][0]["planos"][0]["feito"] = json!(true);
    feito["acoes"][0]["obs"] = json!("instrução interna nova");
    feito["acoes"][2]["fim"] = json!("2026-11-20");
    assert_eq!(impressoes(&feito), (i1, i2), "plano feito, instruções internas e ação do Geral não mudam a impressão dos projetos");
}
