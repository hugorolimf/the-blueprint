//! Testes de plano::entrada (seção 3.4 do contrato), junto com o escalonador: é a resposta que a tela e o motor recebem.
use super::entrada::{entrada_esc, ExecEsc};
use super::escalonar::{escalonar, Causa, EstadoEsc, Situacao};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const AGORA: i64 = 1_000_000;

fn com_git(_: &str) -> bool {
    true
}

fn sem_pasta(_: &str) -> Option<String> {
    None
}

fn exec(bp: &str, id: &str, situacao: &str) -> ExecEsc {
    ExecEsc { blueprint: bp.into(), plano_id: id.into(), plano_ref: id.into(), situacao: situacao.into(), ..ExecEsc::default() }
}

fn calcula(ws: &Value, execs: &[ExecEsc], limite: usize) -> BTreeMap<String, EstadoEsc> {
    let (planos, e) = entrada_esc(ws, execs, &[], limite, &com_git, &sem_pasta, AGORA);
    escalonar(&planos, &e)
}

fn ws(bps: Value) -> Value {
    json!({"id": "w", "blueprints": bps})
}

fn um_bp(id: &str, planos: Value) -> Value {
    json!({"id": id, "acoes": [{"id": format!("a-{id}"), "planos": planos}]})
}

#[test]
fn entre_blueprints() {
    let w = ws(json!([um_bp("a", json!([{"id": "pa", "ref": "1", "area": "app"}, {"id": "feito", "ref": "2", "feito": true}])), um_bp("b", json!([{"id": "pb", "ref": "1", "area": "app/barra", "dependeDe": []}, {"id": "pc", "ref": "2", "dependeDe": ["1", "sumiu"]}]))]));
    // A área é copiada da árvore para a execução ao marcar.
    let (mut pa, mut pb) = (exec("a", "pa", "rodando"), exec("b", "pb", "marcada"));
    pa.area = "app".into();
    pb.area = "app/barra".into();
    let execs = [pa, pb, exec("b", "pc", "marcada")];
    let (planos, e) = entrada_esc(&w, &execs, &["b|2".to_string()], 7, &com_git, &sem_pasta, AGORA);
    let chaves: Vec<&str> = planos.iter().map(|q| q.chave.as_str()).collect();
    assert_eq!(chaves, ["a|1", "a|2", "b|1", "b|2"], "todos, na ordem do documento");
    assert_eq!(planos[3].depende_de, ["b|1", "b|sumiu"], "a dependência é do mesmo blueprint, e a que sumiu continua");
    assert_eq!(e.prontos, ["a|2"]);
    assert_eq!(e.iniciados, ["b|1", "b|2"]);
    assert_eq!(e.rodando, ["a|1"]);
    assert_eq!(e.travados, ["b|2"]);
    assert_eq!(e.limite, Some(7));

    let r = escalonar(&planos, &e);
    assert_eq!(r["a|1"].situacao, Situacao::Trabalhando);
    assert_eq!(r["b|1"].situacao, Situacao::Area, "a área vale entre blueprints");
    assert_eq!(r["b|1"].por.as_deref(), Some("a|1"));
    assert_eq!(r["b|2"].situacao, Situacao::Revisao);

    // Sem travar: a dependência que não existe mais segura o plano.
    let r = calcula(&w, &execs, 7);
    assert_eq!(r["b|2"].situacao, Situacao::Dependencia);
}

#[test]
fn area_da_execucao_vence_a_da_arvore() {
    let w = ws(json!([um_bp("a", json!([{"id": "p1", "area": "nova-na-arvore"}, {"id": "p2", "area": "copiada"}, {"id": "p3", "area": "nova-na-arvore"}]))]));
    let mut rodando = exec("a", "p1", "rodando");
    rodando.area = "copiada".into();
    let execs = [rodando, exec("a", "p2", "marcada"), exec("a", "p3", "marcada")];
    let (planos, _) = entrada_esc(&w, &execs, &[], 4, &com_git, &sem_pasta, AGORA);
    assert_eq!(planos[0].area.as_deref(), Some("copiada"));
    let r = calcula(&w, &execs, 4);
    // p2 e p3 têm execução sem área gravada: a linha da execução vale, e não disputam.
    assert_eq!(r["a|p2"].situacao, Situacao::Trabalhando);

    let mut p2 = exec("a", "p2", "marcada");
    p2.area = "copiada".into();
    let mut p3 = exec("a", "p3", "marcada");
    p3.area = "nova-na-arvore".into();
    let mut rodando = exec("a", "p1", "rodando");
    rodando.area = "copiada".into();
    let r = calcula(&w, &[rodando, p2, p3], 4);
    assert_eq!(r["a|p2"].situacao, Situacao::Area, "disputa com a área que o agente recebeu");
    assert_eq!(r["a|p3"].situacao, Situacao::Trabalhando, "a nova área da árvore não segura ninguém");

    // Sem execução viva, vale a da árvore; a execução pronta não conta mais.
    let (planos, _) = entrada_esc(&w, &[exec("a", "p1", "pronta")], &[], 4, &com_git, &sem_pasta, AGORA);
    assert_eq!(planos[0].area.as_deref(), Some("nova-na-arvore"));
}

#[test]
fn plano_apagado_segura_vaga() {
    let w = ws(json!([um_bp("a", json!([{"id": "p1", "area": "x"}, {"id": "p2"}]))]));
    let mut apagado = exec("a", "pz", "rodando");
    apagado.plano_ref = "9.9".into();
    apagado.area = "x/y".into();
    let execs = [apagado, exec("a", "p1", "marcada"), exec("a", "p2", "marcada")];
    let (planos, e) = entrada_esc(&w, &execs, &[], 1, &com_git, &sem_pasta, AGORA);
    let sintetico = planos.iter().find(|q| q.chave == "a|9.9").expect("vira plano sintético");
    assert_eq!((sintetico.area.as_deref(), sintetico.depende_de.len()), (Some("x/y"), 0));
    assert_eq!(e.rodando, ["a|9.9"]);
    let r = escalonar(&planos, &e);
    assert_eq!(r["a|9.9"].situacao, Situacao::Trabalhando);
    assert_eq!(r["a|p2"].situacao, Situacao::Fila, "continua ocupando a vaga");

    // Blueprint inteiro apagado segura do mesmo jeito; e não duplica o sintético.
    let mut sem_bp = exec("sumido", "pz", "rodando");
    sem_bp.area = "q".into();
    let (planos, e) = entrada_esc(&ws(json!([])), &[sem_bp], &[], 4, &com_git, &sem_pasta, AGORA);
    assert_eq!(planos.len(), 1);
    assert_eq!(e.rodando, ["sumido|pz"]);

    // Execução terminada de plano apagado não pesa.
    let (planos, e) = entrada_esc(&ws(json!([])), &[exec("a", "pz", "pronta"), exec("a", "py", "cancelada")], &[], 4, &com_git, &sem_pasta, AGORA);
    assert!(planos.is_empty() && e.rodando.is_empty());
}

#[test]
fn pasta_sem_git_e_exclusiva() {
    let w = ws(json!([um_bp("sem", json!([{"id": "p1"}, {"id": "p2"}])), um_bp("com", json!([{"id": "p3"}, {"id": "p4"}])), um_bp("solto", json!([{"id": "p5"}]))]));
    let pasta = |bp: &str| match bp {
        "sem" => Some("/pasta/sem".to_string()),
        "com" => Some("/pasta/com".to_string()),
        _ => None,
    };
    let tem_git = |p: &str| p == "/pasta/com";
    let execs = [exec("sem", "p1", "marcada"), exec("sem", "p2", "marcada"), exec("com", "p3", "marcada"), exec("com", "p4", "marcada")];
    let (planos, e) = entrada_esc(&w, &execs, &[], 4, &tem_git, &pasta, AGORA);
    assert_eq!(planos[0].exclusivo.as_deref(), Some("/pasta/sem"));
    assert_eq!(planos[2].exclusivo, None, "com git cada agente tem a sua cópia");
    assert_eq!(planos[4].exclusivo, None, "sem pasta ligada não há o que disputar");
    let r = escalonar(&planos, &e);
    assert_eq!(r["sem|p1"].situacao, Situacao::Trabalhando);
    assert_eq!(r["sem|p2"].situacao, Situacao::Area);
    assert_eq!(r["sem|p2"].area.as_deref(), Some("/pasta/sem"));
    assert_eq!(r["com|p3"].situacao, Situacao::Trabalhando);
    assert_eq!(r["com|p4"].situacao, Situacao::Trabalhando);
}

#[test]
fn cancelada_segura_60s() {
    let w = ws(json!([um_bp("a", json!([{"id": "p1", "area": "x"}, {"id": "p2", "area": "x"}]))]));
    let cancelada = |em: i64, terminada: Option<i64>| {
        let mut c = exec("a", "p1", "cancelada");
        c.area = "x".into();
        c.cancelada_em = Some(em);
        c.terminada_em = terminada;
        c
    };
    let mut p2 = exec("a", "p2", "marcada");
    p2.area = "x".into();

    let (_, e) = entrada_esc(&w, &[cancelada(AGORA - 59_000, None), p2.clone()], &[], 4, &com_git, &sem_pasta, AGORA);
    assert_eq!(e.seguram_area, ["a|p1"]);
    assert!(e.rodando.is_empty(), "não ocupa vaga");
    assert_eq!(calcula(&w, &[cancelada(AGORA - 59_000, None), p2.clone()], 4)["a|p2"].situacao, Situacao::Area);

    // Passou de 60 s, ou o agente já saiu: libera.
    for velha in [cancelada(AGORA - 60_000, None), cancelada(AGORA - 1_000, Some(AGORA - 500))] {
        let (_, e) = entrada_esc(&w, &[velha.clone(), p2.clone()], &[], 4, &com_git, &sem_pasta, AGORA);
        assert!(e.seguram_area.is_empty());
        assert_eq!(calcula(&w, &[velha, p2.clone()], 4)["a|p2"].situacao, Situacao::Trabalhando);
    }

    // Cancelado e marcado de novo: a área é a da execução viva, e a cancelada ainda segura até o agente sair.
    let mut de_novo = exec("a", "p1", "marcada");
    de_novo.area = "z".into();
    let (planos, e) = entrada_esc(&w, &[cancelada(AGORA - 1_000, None), de_novo], &[], 4, &com_git, &sem_pasta, AGORA);
    assert_eq!(planos[0].area.as_deref(), Some("z"));
    assert_eq!((&e.iniciados[..], &e.seguram_area[..]), (&["a|p1".to_string()][..], &["a|p1".to_string()][..]));
}

#[test]
fn causas() {
    let w = ws(json!([um_bp("a", json!([{"id": "aj", "area": "1"}, {"id": "su", "area": "2"}, {"id": "fc", "area": "3"}, {"id": "fs", "area": "4"}, {"id": "ok"}]))]));
    let mut com_copia = exec("a", "fc", "falhou");
    com_copia.motivo = "base-em-conflito".into();
    com_copia.ramo = "agente/fc".into();
    com_copia.area = "3".into();
    let mut sem_copia = exec("a", "fs", "falhou");
    sem_copia.motivo = "repositorio-sem-commit".into();
    let mut ajuda = exec("a", "aj", "ajuda");
    ajuda.area = "1".into();
    let mut sumiu = exec("a", "su", "sumiu");
    sumiu.area = "2".into();
    let execs = [ajuda, sumiu, com_copia, sem_copia, exec("a", "ok", "preparando")];
    let (planos, e) = entrada_esc(&w, &execs, &[], 4, &com_git, &sem_pasta, AGORA);
    assert_eq!(e.causas["a|aj"], Causa::Ajuda);
    assert_eq!(e.causas["a|su"], Causa::Sumiu);
    assert_eq!(e.causas["a|fc"], Causa::Falhou("base-em-conflito".into()));
    assert_eq!(e.causas["a|fs"], Causa::Falhou("repositorio-sem-commit".into()));
    assert_eq!(e.bloqueados, ["a|aj", "a|su", "a|fc", "a|fs"]);
    assert_eq!(e.rodando, ["a|aj", "a|ok"], "quem pediu ajuda segue com a vaga; preparando já ocupa");
    assert_eq!(e.seguram_area, ["a|su", "a|fc"], "falhou sem cópia não tem o que segurar");
    let r = escalonar(&planos, &e);
    assert_eq!(r["a|aj"].situacao, Situacao::Bloqueado);
    assert_eq!(r["a|aj"].causa, Some(Causa::Ajuda));
    assert_eq!(r["a|fc"].causa, Some(Causa::Falhou("base-em-conflito".into())));
    assert_eq!(r["a|ok"].situacao, Situacao::Trabalhando);
}
