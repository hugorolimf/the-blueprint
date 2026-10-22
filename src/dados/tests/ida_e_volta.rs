use super::exemplo;
use crate::contexto::Contexto;
use crate::dados::arquivo::{exportar_arquivo, importar_arquivo};
use crate::dados::{Contagens, Dados};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-teste-{nome}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn o_exemplo_volta_igual() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), exemplo());
}

#[test]
fn as_contagens_do_exemplo() {
    let mut d = Dados::em_memoria().unwrap();
    let c = d.gravar_tudo(&Contexto::local(), &exemplo()).unwrap();
    assert_eq!(
        c,
        Contagens { workspaces: 3, blueprints: 3, acoes: 2, planos: 4, planos_feitos: 1, checkpoints: 2, checkpoints_feitos: 1, comentarios: 2, prototipos: 2 }
    );
}

#[test]
fn gravar_de_novo_substitui_tudo() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    let menor = serde_json::json!({"empresa": "Só isso", "blueprints": []});
    d.gravar_tudo(&ctx, &menor).unwrap();
    assert_eq!(*d.ler_tudo(&ctx).unwrap(), menor);
    assert_eq!(d.contagens(&ctx).unwrap().planos, 0);
}

/// O data.exemplo.json (fictício, do repositório) e o data.json de verdade (se existir): importa, exporta e compara com o original, valor a valor.
#[test]
fn os_dados_de_verdade_voltam_sem_perda() {
    for nome in ["data.exemplo.json", "data.json"] {
        let original = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(nome);
        let Ok(texto) = fs::read_to_string(&original) else { continue };
        let esperado: Value = serde_json::from_str(&texto).unwrap();
        let mut d = Dados::em_memoria().unwrap();
        let ctx = Contexto::local();
        d.gravar_tudo(&ctx, &esperado).unwrap();
        assert_eq!(*d.ler_tudo(&ctx).unwrap(), esperado, "{nome}");
        assert!(d.contagens(&ctx).unwrap().planos > 0, "{nome}");
    }
}

#[test]
fn importar_e_exportar_arquivos_sem_mexer_no_original() {
    let dir = pasta("arquivos");
    let (json, banco, saida) = (dir.join("data.json"), dir.join("blueprint.db"), dir.join("saida.json"));
    let texto = serde_json::to_string_pretty(&exemplo()).unwrap();
    fs::write(&json, &texto).unwrap();
    let c = importar_arquivo(&json, &banco).unwrap();
    assert_eq!(c.planos, 4);
    assert_eq!(fs::read_to_string(&json).unwrap(), texto, "o original não pode mudar");
    exportar_arquivo(&banco, &saida).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&fs::read_to_string(&saida).unwrap()).unwrap(), exemplo());
    assert!(!dir.join("saida.json.tmp").exists(), "o temporário some depois da troca");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn exportar_um_banco_vazio_diz_o_que_fazer() {
    let mut d = Dados::em_memoria().unwrap();
    let erro = d.ler_tudo(&Contexto::local()).unwrap_err().to_string();
    assert!(erro.contains("importe um data.json"), "{erro}");
}
