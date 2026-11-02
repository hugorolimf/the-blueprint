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

/// Uma pasta temporária só deste teste, vazia.
fn temporaria(nome: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-pastas-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// O vínculo como ficou gravado antes da migração 009: só o caminho em minúsculas.
fn vinculo_antigo(d: &mut Dados, caminho: &std::path::Path) -> String {
    let guardado = crate::dados::pastas::normalizar(caminho.to_str().unwrap());
    d.conexao().execute("INSERT INTO pastas (caminho, workspace_id, blueprint_id) VALUES (?1, 'ws', 'bp')", [&guardado]).unwrap();
    guardado
}

#[test]
fn caminho_real_mantem_maiuscula() {
    let base = temporaria("real");
    let pasta = base.join("Meu Projeto");
    std::fs::create_dir_all(&pasta).unwrap();
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.vincular_pasta(&ctx, &format!("  {}/ ", pasta.display()), "ws", "bp").unwrap();
    assert_eq!(d.pasta_real_do_blueprint("ws", "bp").unwrap().as_deref(), pasta.to_str());
    let guardado = d.pasta_do_blueprint(&ctx, "ws", "bp").unwrap().unwrap();
    assert_eq!(guardado, pasta.to_str().unwrap().to_lowercase(), "pasta_do_blueprint continua no formato guardado");
    assert_eq!(d.pasta_real_do_blueprint("ws", "outro").unwrap(), None);
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn legado_resolve_caixa_no_disco() {
    let base = temporaria("legado");
    let pasta = base.join("Trabalho").join("Hugo");
    std::fs::create_dir_all(pasta.join("Projeto")).unwrap();
    let pasta = pasta.join("Projeto");
    let mut d = Dados::em_memoria().unwrap();
    let guardado = vinculo_antigo(&mut d, &pasta);
    assert_eq!(d.pasta_real_do_blueprint("ws", "bp").unwrap().as_deref(), pasta.to_str());
    let gravado: Option<String> = d.conexao().query_row("SELECT caminho_real FROM pastas WHERE caminho = ?1", [&guardado], |l| l.get(0)).unwrap();
    assert_eq!(gravado.as_deref(), pasta.to_str(), "fica gravado, sem andar no disco de novo");
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn dois_candidatos_nao_resolve() {
    let base = temporaria("dois");
    std::fs::create_dir_all(base.join("Proj")).unwrap();
    std::fs::create_dir_all(base.join("PROJ")).unwrap();
    let mut d = Dados::em_memoria().unwrap();
    let guardado = vinculo_antigo(&mut d, &base.join("Proj"));
    assert_eq!(d.pasta_real_do_blueprint("ws", "bp").unwrap(), None);
    let gravado: Option<String> = d.conexao().query_row("SELECT caminho_real FROM pastas WHERE caminho = ?1", [&guardado], |l| l.get(0)).unwrap();
    assert_eq!(gravado, None, "sem decidir, não grava nada");
    // igual exato vence: `proj` existindo, os outros dois não atrapalham
    std::fs::create_dir_all(base.join("proj")).unwrap();
    let resolvido = d.pasta_real_do_blueprint("ws", "bp").unwrap().unwrap();
    assert!(resolvido.ends_with("/proj"), "{resolvido}");
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn pasta_que_nao_existe_e_none() {
    let base = temporaria("some");
    let pasta = base.join("Sumiu");
    std::fs::create_dir_all(&pasta).unwrap();
    let mut d = Dados::em_memoria().unwrap();
    d.vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "ws", "bp").unwrap();
    assert!(d.pasta_real_do_blueprint("ws", "bp").unwrap().is_some());
    std::fs::remove_dir_all(&base).unwrap();
    assert_eq!(d.pasta_real_do_blueprint("ws", "bp").unwrap(), None, "com caminho_real gravado");
    let mut antigo = Dados::em_memoria().unwrap();
    vinculo_antigo(&mut antigo, &pasta);
    assert_eq!(antigo.pasta_real_do_blueprint("ws", "bp").unwrap(), None, "sem caminho_real");
    let arquivo = temporaria("arquivo").join("nao-e-pasta");
    std::fs::write(&arquivo, "x").unwrap();
    let mut d = Dados::em_memoria().unwrap();
    d.vincular_pasta(&Contexto::local(), arquivo.to_str().unwrap(), "ws", "bp").unwrap();
    assert_eq!(d.pasta_real_do_blueprint("ws", "bp").unwrap(), None, "arquivo não é pasta");
    std::fs::remove_dir_all(arquivo.parent().unwrap()).unwrap();
}
