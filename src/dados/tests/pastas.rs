use crate::contexto::Contexto;
use crate::dados::Dados;

#[test]
fn a_pasta_acha_o_blueprint_na_propria_pasta_e_nas_subpastas_sem_confundir_irmas() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.vincular_pasta(&ctx, "C:\\Dev\\Proj\\", "ws1", "bp1").unwrap();
    d.vincular_pasta(&ctx, "C:/Dev/Proj/api", "sm", "bp2").unwrap();
    let acha = |p: &str| d.blueprint_da_pasta(&ctx, p).unwrap();
    assert_eq!(acha("c:/dev/proj"), Some(("ws1".into(), "bp1".into())));
    assert_eq!(acha("C:\\Dev\\Proj\\web\\css"), Some(("ws1".into(), "bp1".into())));
    assert_eq!(acha("C:\\Dev\\Proj\\api\\src"), Some(("sm".into(), "bp2".into())), "a pasta mais específica vence");
    assert_eq!(acha("C:\\Dev\\Proj2"), None, "Proj2 não é subpasta de Proj");
    assert_eq!(acha("C:\\outra"), None);
}

#[test]
fn ligar_de_novo_troca_o_blueprint_e_a_gravacao_do_data_json_nao_apaga_o_vinculo() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.vincular_pasta(&ctx, "/p", "sm", "a").unwrap();
    d.vincular_pasta(&ctx, "/p", "sm", "b").unwrap();
    d.gravar_tudo(&ctx, &super::exemplo()).unwrap();
    assert_eq!(d.blueprint_da_pasta(&ctx, "/p").unwrap(), Some(("sm".into(), "b".into())));
}

#[test]
fn a_pasta_do_blueprint_e_a_mais_rasa_e_none_sem_ligacao() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.vincular_pasta(&ctx, "C:/Proj/sub", "ws1", "bp").unwrap();
    d.vincular_pasta(&ctx, "C:/Proj", "ws1", "bp").unwrap();
    d.vincular_pasta(&ctx, "C:/Outra", "ws1", "outro").unwrap();
    assert_eq!(d.pasta_do_blueprint(&ctx, "ws1", "bp").unwrap().as_deref(), Some("c:/proj"));
    assert_eq!(d.pasta_do_blueprint(&ctx, "sm", "bp").unwrap(), None, "o workspace também conta");
    assert_eq!(d.pasta_do_blueprint(&ctx, "ws1", "nenhum").unwrap(), None);
}
