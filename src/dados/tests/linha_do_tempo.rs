use crate::contexto::Contexto;
use crate::dados::{Dados, EventoLinha, LIMITE_POR_ACAO};
use serde_json::{json, Value};

fn ev(acao: &str, texto: &str) -> EventoLinha {
    EventoLinha { workspace_id: "w".into(), blueprint_id: "b".into(), acao_id: acao.into(), agente: Some("Claude".into()), tipo: "progresso".into(), texto: texto.into(), ..Default::default() }
}

fn textos(d: &Dados, acao: &str) -> Vec<String> {
    let l = d.linha_do_tempo(&Contexto::local(), "w", "b", acao).unwrap();
    l.as_array().unwrap().iter().map(|e| e["texto"].as_str().unwrap().to_string()).collect()
}

#[test]
fn os_eventos_da_acao_saem_do_mais_recente_ao_mais_antigo_sem_misturar_acoes() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_linha_do_tempo(&ctx, 1_000, &ev("a1", "lendo")).unwrap();
    d.gravar_linha_do_tempo(&ctx, 2_000, &EventoLinha { alvo: Some("p1".into()), estado: Some("escrevendo".into()), onde: Some("src/x.rs".into()), ..ev("a1", "Editando src/x.rs") }).unwrap();
    d.gravar_linha_do_tempo(&ctx, 3_000, &ev("a2", "outra ação")).unwrap();
    assert_eq!(textos(&d, "a1"), ["Editando src/x.rs", "lendo"]);
    assert_eq!(textos(&d, "a2"), ["outra ação"]);
    let primeiro: Value = d.linha_do_tempo(&ctx, "w", "b", "a1").unwrap()[0].clone();
    assert_eq!(primeiro, json!({"em": "1970-01-01T00:00:02.000Z", "agente": "Claude", "tipo": "progresso", "estado": "escrevendo", "texto": "Editando src/x.rs", "onde": "src/x.rs", "alvo": "p1"}));
}

#[test]
fn o_mesmo_evento_em_seguida_vira_um_so_com_a_hora_nova() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    for t in [1_000, 2_000, 3_000] {
        d.gravar_linha_do_tempo(&ctx, t, &ev("a1", "Editando a.rs")).unwrap();
    }
    let l = d.linha_do_tempo(&ctx, "w", "b", "a1").unwrap();
    assert_eq!((l.as_array().unwrap().len(), l[0]["em"].as_str()), (1, Some("1970-01-01T00:00:03.000Z")));
    d.gravar_linha_do_tempo(&ctx, 4_000, &ev("a1", "Editando b.rs")).unwrap();
    d.gravar_linha_do_tempo(&ctx, 5_000, &ev("a1", "Editando a.rs")).unwrap();
    assert_eq!(textos(&d, "a1"), ["Editando a.rs", "Editando b.rs", "Editando a.rs"], "só junta com o último");
}

#[test]
fn guarda_so_os_mais_recentes_por_acao_e_nada_com_mais_de_30_dias() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_linha_do_tempo(&ctx, 1_000, &ev("velha", "há muito tempo")).unwrap();
    let dia = 86_400_000;
    for i in 0..LIMITE_POR_ACAO + 5 {
        d.gravar_linha_do_tempo(&ctx, 31 * dia + i, &ev("a1", &format!("e{i}"))).unwrap();
    }
    let t = textos(&d, "a1");
    assert_eq!(t.len() as i64, LIMITE_POR_ACAO);
    assert_eq!((t[0].as_str(), t.last().unwrap().as_str()), ("e54", "e5"));
    assert!(textos(&d, "velha").is_empty(), "passou de 30 dias: sai na próxima gravação");
}

#[test]
fn gravar_o_data_json_nao_apaga_a_linha_do_tempo() {
    let mut d = Dados::em_memoria().unwrap();
    let ctx = Contexto::local();
    d.gravar_linha_do_tempo(&ctx, 1_000, &ev("a1", "lendo")).unwrap();
    d.gravar_tudo(&ctx, &json!({"blueprints": []})).unwrap();
    assert_eq!(textos(&d, "a1"), ["lendo"]);
}
