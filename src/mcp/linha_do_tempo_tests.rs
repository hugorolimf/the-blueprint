//! Testes da linha do tempo pelo caminho de verdade: o evento entra pela rota (ou pelo hook), vai ao banco e sai em GET /api/mcp/linha-do-tempo.
use super::hooks::edicao;
use super::hooks_tests::{dados, ligar, mcp};
use super::linha_do_tempo::da_fila;
use super::rotas::{reenviar, rota};
use crate::http::pedido::Pedido;
use serde_json::{json, Value};

fn linha(m: &super::Mcp, d: &std::sync::Mutex<crate::dados::Dados>, acao: &str) -> Value {
    let pedido = Pedido::novo("GET", &format!("/api/mcp/linha-do-tempo?workspace=sm&blueprint=p1&acao={acao}"), vec![("host".into(), "127.0.0.1:3000".into())], vec![]);
    let r = rota(m, d, &pedido);
    assert_eq!(r.status, 200);
    serde_json::from_slice(r.bytes()).unwrap()
}

#[test]
fn progresso_edicao_e_conclusao_aparecem_na_acao_sem_a_evidencia() {
    let (m, d) = (mcp(), dados());
    let rx = m.assinar();
    let ev = |extra: Value| { let mut c = json!({"workspaceId": "sm", "blueprintId": "p1"}); c.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone()); c };
    assert_eq!(reenviar(&m, &d, "evento", &ev(json!({"tipo": "progresso", "texto": "começando a tela", "agente": "Claude", "estado": "escrevendo", "acaoId": "f2", "planoId": "t2"}))).0, 200);
    ligar(&d, "C:\\Proj");
    edicao(&m, &d, &json!({"session_id": "s1", "cwd": "C:\\Proj", "tool_input": {"file_path": "C:\\Proj\\web\\tela.js", "content": "SEGREDO DO ARQUIVO"}})).unwrap();
    assert_eq!(reenviar(&m, &d, "evento", &ev(json!({"tipo": "concluir", "texto": "cargo test: 12 passaram", "acaoId": "f2", "planoId": "t2"}))).0, 200);
    let l = linha(&m, &d, "f2");
    let eventos = l["eventos"].as_array().unwrap();
    assert_eq!(l["limite"], 50);
    assert_eq!(eventos.iter().map(|e| e["tipo"].as_str().unwrap()).collect::<Vec<_>>(), ["concluir", "progresso", "progresso"], "do mais recente ao mais antigo");
    assert_eq!((eventos[0]["texto"].as_str(), eventos[0]["alvo"].as_str()), (Some(""), Some("t2")), "a evidência não entra");
    assert_eq!((eventos[1]["onde"].as_str(), eventos[1]["agente"].as_str()), (Some("web/tela.js"), Some("Claude")));
    assert_eq!(eventos[2]["texto"], "começando a tela");
    assert!(!l.to_string().contains("SEGREDO"), "conteúdo de arquivo nunca entra");
    let avisos: Vec<_> = rx.try_iter().filter(|e| e.nome == "linha-do-tempo").collect();
    assert!(!avisos.is_empty() && avisos.iter().all(|e| e.dados == Some(json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2"}))), "o SSE leva só os ids");
}

#[test]
fn evento_sem_acao_ou_recusado_nao_entra() {
    let (m, d) = (mcp(), dados());
    assert_eq!(reenviar(&m, &d, "evento", &json!({"tipo": "progresso", "texto": "sem plano", "workspaceId": "sm", "blueprintId": "p1"})).0, 200);
    assert_eq!(reenviar(&m, &d, "evento", &json!({"tipo": "concluir", "texto": "x", "workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "planoId": "nao"})).0, 404);
    assert_eq!(linha(&m, &d, "f2")["eventos"], json!([]));
}

#[test]
fn os_horarios_sobrevivem_ao_reinicio_sem_confirmar_o_plano() {
    use crate::{contexto::Contexto, dados::Dados};
    use std::sync::Mutex;
    let path = std::env::temp_dir().join(format!("blueprint-tempos-{}.db", std::process::id()));
    let mut db = Dados::abrir(&path).unwrap();
    let original = dados().lock().unwrap().ler_tudo(&Contexto::local()).unwrap().clone();
    db.gravar_tudo(&Contexto::local(), &original).unwrap();
    let db = Mutex::new(db);
    let m = mcp();
    let evento = |tipo| json!({"tipo": tipo, "texto": "verificado", "workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "planoId": "t2"});
    let inicio = reenviar(&m, &db, "evento", &evento("progresso")).1["em"].clone();
    let fim = reenviar(&m, &db, "evento", &evento("concluir")).1["em"].clone();
    reenviar(&m, &db, "evento", &evento("progresso"));
    drop(db);
    let mut reaberto = Dados::abrir(&path).unwrap();
    let tudo = reaberto.ler_tudo(&Contexto::local()).unwrap();
    let plano = &tudo["blueprints"][0]["acoes"][1]["planos"][0];
    assert_eq!(plano["agente"]["inicioEm"], inicio);
    assert_eq!(plano["agente"]["feitoEm"], fim);
    assert_eq!(plano["feito"], false);
    reaberto.gravar_tudo(&Contexto::local(), &original).unwrap();
    let restaurado = reaberto.ler_tudo(&Contexto::local()).unwrap();
    assert!(restaurado["blueprints"][0]["acoes"][1]["planos"][0].get("agente").is_none(), "restaurar backup anterior não herda os horários posteriores");
    drop(reaberto);
    for sufixo in ["", "-wal", "-shm"] { let _ = std::fs::remove_file(format!("{}{sufixo}", path.display())); }
}

#[test]
fn conclusao_sem_inicio_nao_inventa_inicio_nem_e_sobrescrita() {
    let d = dados();
    let mut db = d.lock().unwrap();
    db.registrar_tempo_plano("t2", "concluir", "2026-10-26T10:00:00.000Z").unwrap();
    db.registrar_tempo_plano("t2", "progresso", "2026-10-26T11:00:00.000Z").unwrap();
    db.registrar_tempo_plano("t2", "concluir", "2026-10-26T12:00:00.000Z").unwrap();
    let tudo = db.ler_tudo(&crate::contexto::Contexto::local()).unwrap();
    let agente = &tudo["blueprints"][0]["acoes"][1]["planos"][0]["agente"];
    assert_eq!(agente["feitoEm"], "2026-10-26T10:00:00.000Z");
    assert!(agente.get("inicioEm").is_none());
}

#[test]
fn o_filtro_corta_o_texto_e_usa_o_checkpoint_como_alvo() {
    let item = json!({"tipo": "bloqueio", "texto": "x".repeat(500), "acaoId": "a", "planoId": "p", "onde": ["a.rs", "b.rs"]});
    let e = da_fila(&item).unwrap();
    assert_eq!((e.texto.chars().count(), e.alvo.as_deref(), e.onde.as_deref()), (200, Some("p"), Some("a.rs")));
    let c = da_fila(&json!({"tipo": "checkpoint-feito", "texto": "demonstrado com o comando y", "acaoId": "a", "checkpointId": "c1"})).unwrap();
    assert_eq!((c.alvo.as_deref(), c.texto.as_str()), (Some("c1"), ""));
    assert!(da_fila(&json!({"tipo": "progresso", "texto": "x"})).is_none());
}
