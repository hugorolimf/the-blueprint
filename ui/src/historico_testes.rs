//! Os testes do histórico: a ordem dos lugares, o que some e o caminho de ida e volta pelo estado da tela.

use super::*;
use serde_json::json;

fn ws(id: &str) -> Lugar {
    Lugar { tela: Tela::Workspace(id.into()), sub: Sub::default(), aberta: None, detalhe: None, antiga: false }
}

fn com(lugares: &[&str]) -> Historico {
    let mut h = Historico::default();
    for l in lugares {
        h.visitou(ws(l));
    }
    h
}

fn tudo(_: &Lugar) -> bool {
    true
}

#[test]
fn volta_e_avanca_na_ordem() {
    let mut h = com(&["a", "b", "c"]);
    assert_eq!(h.voltar(tudo), Some(ws("b")));
    assert_eq!(h.voltar(tudo), Some(ws("a")));
    assert_eq!(h.voltar(tudo), None, "não passa do começo");
    assert_eq!(h.atual, Some(ws("a")));
    assert_eq!(h.avancar(tudo), Some(ws("b")));
    assert_eq!(h.avancar(tudo), Some(ws("c")));
    assert_eq!(h.avancar(tudo), None, "não passa do fim");
}

#[test]
fn lugar_novo_depois_de_voltar_apaga_a_frente() {
    let mut h = com(&["a", "b", "c"]);
    h.voltar(tudo);
    h.visitou(ws("d"));
    assert_eq!(h.avancar(tudo), None);
    assert_eq!(h.voltar(tudo), Some(ws("b")));
}

#[test]
fn o_mesmo_lugar_a_cada_quadro_nao_conta() {
    let mut h = com(&["a", "a", "a"]);
    assert!(h.atras.is_empty());
    assert_eq!(h.voltar(tudo), None);
}

#[test]
fn o_botao_da_tela_que_volta_ao_anterior_nao_faz_vaivem() {
    let mut h = com(&["a", "b", "a", "b", "a"]);
    assert!(h.atras.is_empty(), "ir e voltar pelo botão da tela não empilha");
    assert_eq!(h.avancar(tudo), Some(ws("b")), "e o lugar de onde se voltou segue à frente");
}

#[test]
fn lugar_que_sumiu_e_pulado_sem_prender() {
    let mut h = com(&["a", "apagado", "b"]);
    assert_eq!(h.voltar(|l| *l != ws("apagado")), Some(ws("a")));
    assert_eq!(h.avancar(tudo), Some(ws("b")));
}

#[test]
fn nao_volta_para_onde_ja_esta() {
    // a tela apagada caiu no workspace de onde veio: a, apagado, a
    let mut h = com(&["x", "a", "apagado", "c"]);
    h.visitou(ws("a"));
    assert_eq!(h.voltar(|l| *l != ws("apagado")), Some(ws("x")));
}

#[test]
fn o_historico_tem_limite() {
    let mut h = Historico::default();
    for i in 0..=LIMITE + 5 {
        h.visitou(ws(&i.to_string()));
    }
    assert_eq!(h.atras.len(), LIMITE);
}

#[test]
fn so_existe_o_que_esta_no_documento() {
    let doc = Documento { raw: json!({ "blueprints": [{ "id": "p1" }], "workspaces": [{ "id": "w1", "nome": "W", "blueprints": [{ "id": "p2" }] }] }), versao: 1 };
    let bp = |w: &str, b: &str| Lugar { tela: Tela::Blueprint(w.into(), b.into()), ..ws("") };
    assert!(Lugar { tela: Tela::Entrada, ..ws("") }.existe(&doc));
    assert!(ws("w1").existe(&doc));
    assert!(!ws("sumiu").existe(&doc));
    assert!(bp("w1", "p2").existe(&doc));
    assert!(!bp("w1", "p1").existe(&doc));
    assert!(!bp("sumiu", "p2").existe(&doc));
}

#[test]
fn ir_a_um_lugar_deixa_a_tela_exatamente_nele() {
    let ctx = Context::default();
    let mut vista = Vista::nova();
    let lugar = Lugar { tela: Tela::Blueprint("w".into(), "b".into()), sub: Sub::Agentes, aberta: Some("A1".into()), detalhe: Some("d1".into()), antiga: false };
    lugar.clone().ir(&ctx, &mut vista);
    assert_eq!(Lugar::de(&ctx, &vista), lugar, "senão o quadro seguinte anotaria um lugar a mais");
    plano::rede::passo(&ctx, &mut vista.plano, "w", "b", "http://127.0.0.1:1");
    assert_eq!(Lugar::de(&ctx, &vista), lugar, "o desenho não zera o Planejar que acabou de voltar");
}

#[test]
fn entrar_em_outro_blueprint_nao_anota_o_que_era_do_anterior() {
    let ctx = Context::default();
    let mut vista = Vista::nova();
    plano::rede::entrar(&mut vista.plano, "w", "a");
    vista.plano.tela = Sub::Agentes;
    vista.plano.aberta = Some("A1".into());
    vista.tela = Tela::Blueprint("w".into(), "b".into());
    let antes_de_desenhar = Lugar::de(&ctx, &vista);
    assert_eq!(antes_de_desenhar.sub, Sub::Folha);
    plano::rede::entrar(&mut vista.plano, "w", "b");
    assert_eq!(Lugar::de(&ctx, &vista), antes_de_desenhar);
}

#[test]
fn voltar_a_folha_antiga_e_sair_dela() {
    let ctx = Context::default();
    let mut vista = Vista::nova();
    let antiga = Lugar { tela: Tela::Blueprint("w".into(), "b".into()), antiga: true, ..ws("") };
    antiga.clone().ir(&ctx, &mut vista);
    assert_eq!(Lugar::de(&ctx, &vista), antiga);
    let nova = Lugar { antiga: false, ..antiga };
    nova.clone().ir(&ctx, &mut vista);
    assert_eq!(Lugar::de(&ctx, &vista), nova);
    assert!(vista.folha_antiga.is_empty());
}

#[test]
fn por_cima_fecha_e_nao_anda() {
    let mut vista = Vista::nova();
    assert!(!fechar_por_cima(&mut vista));
    vista.novo_bp = Some(crate::telas::NovoBp::em("w"));
    assert!(fechar_por_cima(&mut vista));
    assert!(vista.novo_bp.is_none() && !tem_algo_por_cima(&vista));
}
