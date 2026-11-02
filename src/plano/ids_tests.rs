use super::ids::*;
use serde_json::json;

#[test]
fn id_real_estavel() {
    let id = id_real("bp1", "plano", "1.2/1.2.1");
    assert_eq!(id, id_real("bp1", "plano", "1.2/1.2.1"));
    assert_eq!(id.len(), 17);
    assert!(id.starts_with('p') && id[1..].bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)), "{id}");
    assert_eq!(id, format!("p{}", &sha256_hex(b"bp1\0plano\x001.2/1.2.1")[..16]), "o valor não muda entre versões do app");
    assert_ne!(id, id_real("bp2", "plano", "1.2/1.2.1"));
    assert_ne!(id, id_real("bp1", "checkpoint", "1.2/1.2.1"));
    assert_ne!(id, id_real("bp1", "plano", "1.2/1.2.2"));
}

#[test]
fn ref_valido_casos() {
    for bom in ["1", "1.2.3", "a", "A-b_c.d", "P1", "x.lockk", &"a".repeat(64)] {
        assert!(ref_valido(bom), "{bom}");
    }
    for ruim in ["", "-a", ".a", "_a", "a/b", "a\\b", "a b", "a..b", "a.", "a.lock", "a.LOCK", "ação", "a\n", "a;b", &"a".repeat(65)] {
        assert!(!ref_valido(ruim), "{ruim:?}");
    }
}

#[test]
fn ramo_de_casos() {
    assert_eq!(ramo_de("1.2"), "agente/1.2");
    let hash = ramo_de("--upload-pack=x");
    assert_eq!(hash.len(), "agente/".len() + 12);
    assert_eq!(hash, format!("agente/{}", &sha256_hex(b"--upload-pack=x")[..12]));
    assert_eq!(ramo_de("../fora"), format!("agente/{}", &sha256_hex(b"../fora")[..12]));
    assert_ne!(ramo_de("a.lock"), "agente/a.lock");
}

#[test]
fn area_normal_casos() {
    for (a, b) in [("./App//x/", "app/x"), ("app/barra/", "app/barra"), ("App\\Barra", "app/barra"), (" app / barra ", "app/barra"), ("a/./b", "a/b"), ("ÁREA", "área"), ("", "")] {
        assert_eq!(area_normal(a), b, "{a}");
    }
    for boa in ["app", "app/barra", "./App//x/", "src/fusos", "área nova", "a.b_c-d"] {
        assert!(area_valida(boa), "{boa}");
    }
    for ruim in ["", "-a", "/a", "a/../b", "..", "a/ .. /b", "a\\b", "a\tb", "a;b", ".", "./", "   ", &"a".repeat(121)] {
        assert!(!area_valida(ruim), "{ruim:?}");
    }
    assert!(area_valida(&"a".repeat(120)));
}

#[test]
fn nome_agente_corta_em_40() {
    assert_eq!(nome_agente("1.2"), "plano 1.2");
    assert_eq!(nome_agente(&"a".repeat(64)).chars().count(), 40);
    assert_eq!(nome_agente(&"é".repeat(64)).chars().count(), 40, "conta caracteres, sem cortar no meio de um");
}

#[test]
fn traduzir_por_id_ou_ref() {
    let bp = json!({"acoes": [
        {"id": "pa", "ref": "1", "planos": [{"id": "pb", "ref": "1.1"}, {"id": "1.2"}], "checkpoints": [{"id": "pc", "ref": "1.c1"}]},
        {"id": "2", "planos": [{"id": "pd", "ref": "pb"}]}
    ]});
    assert_eq!(traduzir(&bp, "1", "acao").as_deref(), Some("pa"));
    assert_eq!(traduzir(&bp, "pa", "acao").as_deref(), Some("pa"));
    assert_eq!(traduzir(&bp, "1.1", "plano").as_deref(), Some("pb"));
    assert_eq!(traduzir(&bp, "1.2", "plano").as_deref(), Some("1.2"), "blueprint antigo: o ref é o id");
    assert_eq!(traduzir(&bp, "pb", "plano").as_deref(), Some("pb"), "o id vence um ref que o imita");
    assert_eq!(traduzir(&bp, "1.c1", "checkpoint").as_deref(), Some("pc"));
    assert_eq!(traduzir(&bp, "1.1", "acao"), None);
    assert_eq!(traduzir(&bp, "1.1", "outro"), None);
    assert_eq!(traduzir(&bp, "", "plano"), None);
    assert_eq!(traduzir(&json!({}), "1", "acao"), None);
}
