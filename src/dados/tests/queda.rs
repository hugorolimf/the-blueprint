//! Gravação atômica: uma gravação que falha no meio não pode deixar o banco pela metade.
use super::exemplo;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::erro::Erro;
use serde_json::json;

#[test]
fn uma_gravacao_invalida_no_meio_nao_muda_nada() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    // o primeiro blueprint é válido e já seria inserido; o segundo plano não tem id: tudo deve ser desfeito
    let mut ruim = exemplo();
    ruim["blueprints"][0]["acoes"][0]["planos"][1] = json!({"texto": "sem id"});
    assert!(matches!(d.gravar_tudo(&ctx, &ruim), Err(Erro::Formato(_))));
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), exemplo());
    assert_eq!(d.contagens(&ctx).unwrap().planos, 4);
}

#[test]
fn id_reservado_ou_nao_objeto_sao_recusados_sem_perder_nada() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    let mut reservado = exemplo();
    reservado["workspaces"][0]["id"] = json!("sm");
    assert!(d.gravar_tudo(&ctx, &reservado).is_err());
    assert!(d.gravar_tudo(&ctx, &json!([1, 2])).is_err());
    let mut item_solto = exemplo();
    item_solto["blueprints"] = json!(["não sou um objeto"]);
    assert!(d.gravar_tudo(&ctx, &item_solto).is_err());
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), exemplo());
}

#[test]
fn id_repetido_derruba_a_gravacao_inteira() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    let mut repetido = exemplo();
    repetido["blueprints"][1]["id"] = json!("p1");
    assert!(matches!(d.gravar_tudo(&ctx, &repetido), Err(Erro::Banco(_))));
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), exemplo());
}

#[test]
fn o_que_foi_gravado_sobrevive_a_fechar_e_abrir_o_banco() {
    let caminho = std::env::temp_dir().join(format!("blueprint-queda-{}.db", std::process::id()));
    let ctx = Contexto::local();
    {
        let mut d = Dados::abrir(&caminho).unwrap();
        d.gravar_tudo(&ctx, &exemplo()).unwrap();
        let mut ruim = exemplo();
        ruim["blueprints"][0]["acoes"][1]["id"] = json!(null);
        assert!(d.gravar_tudo(&ctx, &ruim).is_err());
    } // fechado sem fechar "direito": o WAL cuida
    let mut d = Dados::abrir(&caminho).unwrap();
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), exemplo());
    drop(d);
    for sufixo in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{sufixo}", caminho.display()));
    }
}
