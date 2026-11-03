use super::*;

fn vazio() -> Documento {
    Documento { raw: json!({ "empresa": "Minha empresa", "blueprints": [], "workspaces": [] }), versao: 1 }
}

#[test]
fn a_raiz_vem_primeiro_e_a_ordem_manda() {
    let mut d = vazio();
    let a = d.criar_workspace("Alfa", "", "blueprint").unwrap();
    let b = d.criar_workspace("Beta", "", "chip").unwrap();
    let ids: Vec<_> = d.workspaces().into_iter().map(|w| w.id).collect();
    assert_eq!(ids, [RAIZ, &a, &b]);
    assert!(d.mover_workspace(&b, -1));
    let ids: Vec<_> = d.workspaces().into_iter().map(|w| w.id).collect();
    assert_eq!(ids, [RAIZ, &b, &a]);
    assert!(!d.mover_workspace(RAIZ, -1), "não passa do começo");
}

#[test]
fn nome_vazio_ou_repetido_e_recusado() {
    let mut d = vazio();
    assert_eq!(d.criar_workspace("   ", "", "blueprint").unwrap_err(), "Digite o nome do workspace.");
    d.criar_workspace("Café", "", "blueprint").unwrap();
    assert_eq!(d.criar_workspace("  CAFE ", "", "blueprint").unwrap_err(), "Já existe um workspace com esse nome.");
    assert!(d.criar_workspace("minha EMPRESA", "", "blueprint").is_err(), "a raiz também conta");
}

#[test]
fn excluir_tira_da_ordem() {
    let mut d = vazio();
    let a = d.criar_workspace("Alfa", "", "blueprint").unwrap();
    d.mover_workspace(&a, -1);
    assert!(d.excluir_workspace(&a));
    assert!(d.workspace(&a).is_none());
    assert!(!d.raw["ordemWorkspaces"].as_array().unwrap().iter().any(|x| x == &json!(a)));
}

#[test]
fn campo_desconhecido_volta_intacto() {
    let mut d = Documento { raw: json!({ "blueprints": [], "chats": [1, 2], "novidade": { "x": true } }), versao: 3 };
    d.definir_usuario("  Hugo ");
    d.criar_workspace("Alfa", "desc", "blueprint").unwrap();
    assert_eq!(d.raw["chats"], json!([1, 2]));
    assert_eq!(d.raw["novidade"]["x"], json!(true));
    assert_eq!(d.nome_usuario(), "Hugo");
}

#[test]
fn exemplo_vira_workspace_novo_com_ids_novos() {
    let modelo: Value = serde_json::from_str(include_str!("../../../data.exemplo.json")).unwrap();
    let mut d = vazio();
    assert!(!d.tem_blueprint());
    let id = d.adicionar_exemplo(&modelo);
    assert!(d.tem_blueprint());
    assert_eq!(d.exemplo().unwrap().id, id);
    let ws = d.workspace(&id).unwrap();
    let antiga = &modelo["blueprints"][0]["acoes"][0];
    let nova = &ws["blueprints"][0]["acoes"][0];
    assert_ne!(antiga["id"], nova["id"]);
    assert_eq!(nova["prototipos"], json!([]));
    assert_eq!(ws["exemplo"], json!(true));
}

#[test]
fn ids_novos_nao_se_repetem() {
    let ids: std::collections::HashSet<_> = (0..500).map(|_| novo_id()).collect();
    assert_eq!(ids.len(), 500);
    assert!(ids.iter().all(|i| i.len() == 7));
}

#[test]
fn arrastar_poe_no_lugar_e_empurra_os_outros() {
    let mut d = vazio();
    let ids: Vec<String> = ["A", "B", "C"].iter().map(|n| d.criar_workspace(n, "", "blueprint").unwrap()).collect();
    assert!(d.mover_para(&ids[2], 1));
    let ordem: Vec<_> = d.workspaces().into_iter().map(|w| w.nome).collect();
    assert_eq!(ordem, ["Minha empresa", "C", "A", "B"]);
    assert!(!d.mover_para(&ids[2], 1), "já está lá");
    assert!(!d.mover_para(&ids[2], 9), "fora do fim");
}

#[test]
fn marcar_item_liga_e_desliga_so_o_item() {
    let mut d = Documento { raw: json!({ "blueprints": [{ "id": "b", "acoes": [{ "id": "a", "planos": [{ "id": "p1", "feito": false }, { "id": "p2", "feito": true }], "checkpoints": [{ "id": "c1" }] }] }] }), versao: 1 };
    assert!(d.marcar_item(RAIZ, "b", "a", "planos", "p1"));
    assert!(d.marcar_item(RAIZ, "b", "a", "checkpoints", "c1"));
    assert!(d.marcar_item(RAIZ, "b", "a", "planos", "p2"));
    let a = &d.raw["blueprints"][0]["acoes"][0];
    assert_eq!((a["planos"][0]["feito"].clone(), a["planos"][1]["feito"].clone(), a["checkpoints"][0]["feito"].clone()), (json!(true), json!(false), json!(true)));
    assert!(!d.marcar_item(RAIZ, "b", "a", "planos", "nao-existe"));
}

#[test]
fn criar_blueprint_vai_para_o_fim_do_workspace() {
    let mut d = vazio();
    let ws = d.criar_workspace("Alfa", "", "blueprint").unwrap();
    assert!(d.criar_blueprint(&ws, "   ", "").is_err(), "sem nome não cria");
    let a = d.criar_blueprint(&ws, " Portal ", " Para o fornecedor ").unwrap();
    let b = d.criar_blueprint(RAIZ, "Na raiz", "").unwrap();
    let bp = &d.workspace(&ws).unwrap()["blueprints"][0];
    assert_eq!((bp["id"].as_str(), bp["nome"].as_str(), bp["descricao"].as_str()), (Some(a.as_str()), Some("Portal"), Some("Para o fornecedor")));
    assert_eq!(bp["acoes"], json!([]));
    assert_eq!(d.raw["blueprints"][0]["id"].as_str(), Some(b.as_str()));
    assert!(d.criar_blueprint("nao-existe", "X", "").is_err());
}

fn com_acao() -> Documento {
    Documento { raw: json!({ "blueprints": [{ "id": "b", "acoes": [{ "id": "a", "status": "andamento", "progresso": 0.4, "inicio": "2026-10-30", "fim": "2026-10-31" }, { "id": "c" }] }, { "id": "z" }] }), versao: 1 }
}

#[test]
fn a_situacao_do_menu_ajusta_o_progresso() {
    let mut d = com_acao();
    assert!(d.definir_situacao(RAIZ, "b", "a", "concluido"));
    assert_eq!((d.raw["blueprints"][0]["acoes"][0]["status"].as_str(), d.raw["blueprints"][0]["acoes"][0]["progresso"].as_i64()), (Some("concluido"), Some(1)));
    assert!(d.definir_situacao(RAIZ, "b", "a", "planejado"));
    assert_eq!(d.raw["blueprints"][0]["acoes"][0]["progresso"].as_i64(), Some(0));
    assert!(!d.definir_situacao(RAIZ, "b", "nao-existe", "planejado"));
}

#[test]
fn excluir_leva_so_o_que_foi_pedido() {
    let mut d = com_acao();
    assert!(d.excluir_acao(RAIZ, "b", "a"));
    assert_eq!(d.raw["blueprints"][0]["acoes"].as_array().unwrap().len(), 1);
    assert!(!d.excluir_acao(RAIZ, "b", "a"));
    assert!(d.excluir_blueprint(RAIZ, "z"));
    assert_eq!(d.raw["blueprints"].as_array().unwrap().len(), 1);
    assert!(!d.excluir_blueprint("outro", "b"));
}

#[test]
fn excluir_o_principal_esvazia_e_tira_da_lista_e_um_blueprint_novo_o_traz_de_volta() {
    let mut d = Documento { raw: json!({ "empresa": "Minha empresa", "usuario": { "nome": "Ana" }, "blueprints": [{ "id": "b" }], "workspaces": [], "ordemWorkspaces": ["sm"] }), versao: 1 };
    assert!(d.excluir_workspace(RAIZ));
    assert!(d.workspaces().is_empty());
    assert_eq!(d.raw["blueprints"].as_array().unwrap().len(), 0);
    assert_eq!(d.nome_usuario(), "Ana", "o nome da pessoa não é do workspace");
    assert!(d.raw.get("empresa").is_none());
    assert!(!d.excluir_workspace(RAIZ), "já foi excluído");
    d.criar_blueprint(RAIZ, "Do agente", "").unwrap();
    assert_eq!(d.workspaces().len(), 1);
}
