//! Os testes de `plano::rede`: o laço sem rede nem relógio.

use super::*;

fn pulso(estado: &mut Estado, versao: u64, revisao: u64) {
    let seq = estado.canal.proximo();
    estado.canal.pulso_em_voo = Some(seq);
    tratar(estado, Msg::Pulso(seq, Some(Pulso { versao_dados: versao, revisao })));
}

#[test]
fn pulso_so_pede_folha_quando_muda() {
    let mut e = Estado::default();
    pulso(&mut e, 1, 1);
    assert!(e.canal.pedido, "o primeiro confere a folha");
    assert!(!e.recarregar_doc, "o documento acabou de ser lido");
    e.canal.pedido = false;
    pulso(&mut e, 1, 1);
    assert!(!e.canal.pedido && !e.recarregar_doc);
    pulso(&mut e, 1, 2);
    assert!(e.canal.pedido && !e.recarregar_doc, "mudança fora da árvore não relê o documento");
    e.canal.pedido = false;
    pulso(&mut e, 2, 2);
    assert!(e.canal.pedido && e.recarregar_doc);
}

#[test]
fn pulso_que_falhou_nao_muda_nada_e_libera_o_proximo() {
    let mut e = Estado::default();
    e.canal.pulso_em_voo = Some(3);
    tratar(&mut e, Msg::Pulso(3, None));
    assert_eq!((e.canal.pulso_em_voo, e.canal.visto, e.canal.pedido), (None, None, false));
}

#[test]
fn intervalos() {
    let t = Instant::now();
    assert!(devido(None, t, PULSO));
    assert!(!devido(Some(t), t, PULSO));
    assert!(devido(Some(t), t + PULSO, PULSO));
    assert!(!devido(Some(t), t + Duration::from_secs(29), FOLHA));
}

#[test]
fn resposta_de_rodada_velha_e_descartada() {
    let mut e = Estado::default();
    e.canal.rodada = 5;
    tratar(&mut e, Msg::Folha(4, Ok(Folha { hoje: "velha".into(), ..Folha::default() })));
    assert!(e.folha.is_none());
    tratar(&mut e, Msg::Folha(5, Ok(Folha { hoje: "nova".into(), ..Folha::default() })));
    assert_eq!(e.folha.unwrap().hoje, "nova");
}

#[test]
fn falha_so_cai_na_folha_antiga_sem_folha() {
    let mut e = Estado::default();
    tratar(&mut e, Msg::Folha(0, Err("404".into())));
    assert!(e.folha_falhou);
    tratar(&mut e, Msg::Folha(0, Ok(Folha::default())));
    assert!(!e.folha_falhou);
    tratar(&mut e, Msg::Folha(0, Err("rede".into())));
    assert!(!e.folha_falhou && e.folha.is_some(), "uma falha passageira mantém a folha à vista");
}

#[test]
fn trocar_de_blueprint_limpa_e_descarta_o_anterior() {
    let mut e = Estado { folha: Some(Folha::default()), aviso: "x".into(), tela: crate::plano::Sub::Agentes, recarregar_doc: true, ..Estado::default() };
    e.canal.visto = Some((1, 1));
    e.canal.pulso_em_voo = Some(9);
    let rodada_velha = e.canal.rodada;
    trocar_alvo(&mut e, "w", "b");
    assert!(e.folha.is_none() && e.aviso.is_empty() && e.tela == crate::plano::Sub::Folha);
    assert!(e.recarregar_doc, "o documento vale para qualquer blueprint: o pedido não se perde");
    assert!(e.canal.pedido && e.canal.visto.is_none() && e.canal.pulso_em_voo.is_none());
    tratar(&mut e, Msg::Folha(rodada_velha, Ok(Folha::default())));
    assert!(e.folha.is_none());
}

#[test]
fn gesto_leva_token_e_json() {
    let p = pedido_gesto("http://x", "marcar", &serde_json::json!({"a": 1}), Some("tok"));
    assert_eq!(p.url, "http://x/api/mcp/gesto/marcar");
    assert_eq!(p.method, ehttp::Method::POST);
    assert_eq!(p.headers.get(CABECALHO_GESTO), Some("tok"));
    assert_eq!(p.headers.get("Content-Type"), Some("application/json"));
    assert_eq!(p.body, br#"{"a":1}"#);
}

#[test]
fn leitura_nunca_leva_token() {
    rede_app::definir_gesto("tok".into());
    let p = pedido_ler("http://x", "folha", "ws um", Some("b/2"));
    assert_eq!(p.url, "http://x/api/mcp/planejar/folha?workspace=ws%20um&blueprint=b%2F2");
    assert_eq!(p.headers.get(CABECALHO_GESTO), None);
    assert!(p.headers.get("Accept-Language").is_some());
    assert_eq!(pedido_ler("", "config", "", None).url, "/api/mcp/planejar/config");
}

#[test]
fn respostas_de_gesto() {
    assert_eq!(classificar(200, br#"{"ok":true,"avisos":["a","b"]}"#), RespostaGesto::Feito(vec!["a".into(), "b".into()]));
    assert_eq!(classificar(403, br#"{"erro":"x","codigo":"gesto-negado"}"#), RespostaGesto::SemToken);
    assert_eq!(classificar(409, br#"{"erro":"x","codigo":"plano-mudou"}"#), RespostaGesto::PlanoMudou);
    assert_eq!(classificar(409, br#"{"erro":"em uso","codigo":"rodando"}"#), RespostaGesto::Recusado("em uso".into()));
    assert_eq!(classificar(422, br#"{"erros":["um","dois"],"avisos":[]}"#), RespostaGesto::Recusado("um dois".into()));
    assert!(matches!(classificar(500, b"nao e json"), RespostaGesto::Recusado(m) if m.contains("500")));
}

#[test]
fn gestos_atualizam_a_folha_e_avisam() {
    let mut e = Estado::default();
    tratar(&mut e, Msg::Gesto(RespostaGesto::SemToken));
    assert!(!e.aviso.is_empty() && !e.canal.pedido, "sem token: só avisa");
    tratar(&mut e, Msg::Gesto(RespostaGesto::PlanoMudou));
    assert!(e.canal.pedido, "plano-mudou recarrega");
    e.canal.pedido = false;
    tratar(&mut e, Msg::Gesto(RespostaGesto::Feito(vec![])));
    assert!(e.aviso.is_empty() && e.canal.pedido);
}
