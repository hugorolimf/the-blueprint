use super::exemplo;
use crate::contexto::{Contexto, Papel};
use crate::dados::Dados;
use crate::erro::Erro;
use serde_json::json;

fn com_exemplo() -> (Dados, Contexto) {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    (d, ctx)
}

fn plano<'a>(v: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    v["blueprints"][0]["acoes"][0]["planos"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap()
}

#[test]
fn marcar_plano_muda_so_o_feito_e_sobe_a_revisao() {
    let (mut d, ctx) = com_exemplo();
    d.marcar_plano(&ctx, "t2", true).unwrap();
    assert_eq!(plano(d.ler_tudo(&ctx).unwrap(), "t2")["feito"], true);
    assert_eq!(plano(d.ler_tudo(&ctx).unwrap(), "t1")["feito"], true, "os outros ficam como estavam");
    let revisao: i64 = d.conexao().query_row("SELECT revisao FROM planos WHERE id = 't2'", [], |l| l.get(0)).unwrap();
    assert_eq!(revisao, 2);
    assert!(matches!(d.marcar_plano(&ctx, "nao", true), Err(Erro::NaoEncontrado(_))));
}

#[test]
fn comentario_vai_para_o_fim_e_cria_o_agente_se_faltar() {
    let (mut d, ctx) = com_exemplo();
    d.adicionar_comentario(&ctx, "t1", &json!({"texto": "novo", "em": "2026-10-22T09:00:00Z", "tipo": "comentario"})).unwrap();
    let t1 = plano(d.ler_tudo(&ctx).unwrap(), "t1").clone();
    assert_eq!(t1["agente"]["comentarios"].as_array().unwrap().len(), 3);
    assert_eq!(t1["agente"]["comentarios"][2]["texto"], "novo");
    d.adicionar_comentario(&ctx, "t3", &json!({"texto": "primeiro"})).unwrap();
    let t3 = plano(d.ler_tudo(&ctx).unwrap(), "t3").clone();
    assert_eq!(t3["agente"], json!({"comentarios": [{"texto": "primeiro"}]}));
    assert!(matches!(d.adicionar_comentario(&ctx, "nao", &json!({})), Err(Erro::NaoEncontrado(_))));
    assert!(matches!(d.adicionar_comentario(&ctx, "t1", &json!("texto solto")), Err(Erro::Formato(_))));
}

#[test]
fn prototipo_novo_vai_para_o_fim_e_o_mesmo_id_substitui_no_lugar() {
    let (mut d, ctx) = com_exemplo();
    d.gravar_prototipo(&ctx, "f1", &json!({"id": "pr3", "titulo": "Novo", "em": "2026-10-23T10:00:00Z"})).unwrap();
    d.gravar_prototipo(&ctx, "f1", &json!({"id": "pr1", "titulo": "Tela v2", "em": "2026-10-24T10:00:00Z"})).unwrap();
    let ids_e_titulos: Vec<(String, String)> = d.ler_tudo(&ctx).unwrap()["blueprints"][0]["acoes"][0]["prototipos"]
        .as_array().unwrap().iter().map(|p| (p["id"].as_str().unwrap().into(), p["titulo"].as_str().unwrap().into())).collect();
    assert_eq!(ids_e_titulos, vec![("pr1".into(), "Tela v2".into()), ("pr2".into(), "Figura".into()), ("pr3".into(), "Novo".into())]);
    // numa ação que ainda não tinha a lista, ela passa a existir
    d.gravar_prototipo(&ctx, "f2", &json!({"id": "pr9", "titulo": "Primeiro"})).unwrap();
    assert_eq!(d.ler_tudo(&ctx).unwrap()["blueprints"][0]["acoes"][1]["prototipos"], json!([{"id": "pr9", "titulo": "Primeiro"}]));
    assert!(matches!(d.gravar_prototipo(&ctx, "f2", &json!({"id": "pr1", "titulo": "Roubado"})), Err(Erro::Formato(_))));
    assert!(matches!(d.gravar_prototipo(&ctx, "nao", &json!({"id": "x"})), Err(Erro::NaoEncontrado(_))));
    assert!(matches!(d.gravar_prototipo(&ctx, "f1", &json!({"titulo": "sem id"})), Err(Erro::Formato(_))));
}

#[test]
fn remover_prototipo_diz_se_existia() {
    let (mut d, ctx) = com_exemplo();
    assert!(d.remover_prototipo(&ctx, "f1", "pr1").unwrap());
    assert!(!d.remover_prototipo(&ctx, "f1", "pr1").unwrap());
    assert!(!d.remover_prototipo(&ctx, "f2", "pr2").unwrap(), "só remove da ação certa");
    assert_eq!(d.ler_tudo(&ctx).unwrap()["blueprints"][0]["acoes"][0]["prototipos"].as_array().unwrap().len(), 1);
}

#[test]
fn quem_nao_edita_nao_grava() {
    let (mut d, _) = com_exemplo();
    let visitante = Contexto { usuario: "ana".into(), papel: Papel::Visitante };
    assert!(matches!(d.marcar_plano(&visitante, "t2", true), Err(Erro::Proibido(_))));
    assert!(matches!(d.gravar_tudo(&visitante, &exemplo()), Err(Erro::Proibido(_))));
    assert!(matches!(d.gravar_prototipo(&visitante, "f1", &json!({"id": "x"})), Err(Erro::Proibido(_))));
    assert!(d.ler_tudo(&visitante).is_ok(), "ler qualquer um pode");
}

#[test]
fn a_leitura_vem_da_memoria_e_se_refaz_depois_de_gravar() {
    let (mut d, ctx) = com_exemplo();
    let antes = d.ler_tudo(&ctx).unwrap().clone();
    d.conexao().execute("UPDATE planos SET feito = 1 WHERE id = 't2'", []).unwrap(); // por fora da camada: a cópia em memória não vê
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), antes);
    d.marcar_plano(&ctx, "t4", true).unwrap(); // pela camada: a cópia é refeita e agora vê as duas mudanças
    assert_eq!(plano(d.ler_tudo(&ctx).unwrap(), "t2")["feito"], true);
}
