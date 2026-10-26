//! Testes de idioma do que o MCP e os hooks escrevem para a pessoa: o mesmo caso em inglês e em português, e o código estável igual nos dois.
use super::hooks::sessao;
use super::hooks_tests::{dados, ligar, mcp};
use super::pendentes::{guardar, reenviar_todos};
use super::status::status;
use crate::contexto::Contexto;
use crate::idioma::{com, Idioma};
use serde_json::json;
use std::path::Path;

#[test]
fn a_status_line_e_o_status_seguem_o_idioma() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    let en = com(Idioma::En, || status(&m, &d, &json!({"cwd": "C:/Proj"})).unwrap());
    assert_eq!(en["linha"], "Blueprint · Eu › Portal · action 1.2 · Tela · 0/1");
    let t = en["texto"].as_str().unwrap();
    assert!(t.contains("Action 1.2: Tela (01/12 to 05/12) · 0/1 plans") && t.contains("No pending checkpoints."), "{t}");
    let pt = com(Idioma::Pt, || status(&m, &d, &json!({"cwd": "C:/Proj"})).unwrap());
    assert_eq!(pt["linha"], "Blueprint · Eu › Portal · ação 1.2 · Tela · 0/1");
}

#[test]
fn o_contexto_do_session_start_segue_o_idioma() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    let en = com(Idioma::En, || sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:/Proj"})).unwrap());
    let t = en["contexto"].as_str().unwrap();
    assert!(t.contains("this folder belongs to blueprint «Portal»") && t.contains("Current action: 1.2 «Tela»") && t.contains("Pending plans:"), "{t}");
    let pt = com(Idioma::Pt, || sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:/Proj"})).unwrap());
    assert!(pt["contexto"].as_str().unwrap().contains("Ação atual: 1.2 «Tela»"));
    let solta = com(Idioma::En, || sessao(&m, &d, &json!({"session_id": "s2", "cwd": "C:/Solta"})).unwrap());
    assert!(solta["contexto"].as_str().unwrap().contains("is not linked to any blueprint"));
}

#[test]
fn o_lembrete_do_stop_segue_o_idioma() {
    let m = mcp();
    super::lembrete::editou_sem_plano(&m, "s1", "sm", "p1");
    let en = com(Idioma::En, || super::lembrete::parada(&m, &json!({"session_id": "s1"})));
    assert!(en["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("there were edits outside a plan"));
    super::lembrete::editou_sem_plano(&m, "s1", "sm", "p1");
    let pt = com(Idioma::Pt, || super::lembrete::parada(&m, &json!({"session_id": "s1"})));
    assert!(pt["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("houve edições fora de um plano"));
}

#[test]
fn o_motivo_dos_pendentes_recusados_segue_o_idioma() {
    for (idioma, trecho) in [(Idioma::En, "not found"), (Idioma::Pt, "não encontrad")] {
        let p = std::env::temp_dir().join(format!("blueprint-pendentes-idioma-{}-{}", idioma.codigo(), std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        guardar(&p, "evento", &json!({"tipo": "concluir", "texto": "ok", "workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "planoId": "nao-existe"})).unwrap();
        let (m, d) = (mcp(), dados());
        com(idioma, || reenviar_todos(&m, &d, &p));
        let txt = std::fs::read_dir(p.join("recusados")).unwrap().flatten().find(|e| e.file_name().to_string_lossy().ends_with(".txt")).unwrap();
        let motivo = std::fs::read_to_string(txt.path()).unwrap();
        assert!(motivo.contains(trecho), "{idioma:?}: {motivo}");
    }
}

#[test]
fn o_pedido_do_botao_iniciar_segue_o_idioma_de_quem_clicou() {
    let d = dados();
    let pasta = std::env::temp_dir().join(format!("blueprint-iniciar-idioma-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    super::hooks::travado(&d).vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "sm", "p1").unwrap();
    let corpo = json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2"});
    let pedido = |idioma| {
        let visto = std::cell::RefCell::new(String::new());
        let falso = |_: &Path, _: &Path, prompt: &str| { *visto.borrow_mut() = prompt.to_string(); Ok("wt") };
        com(idioma, || super::iniciar::iniciar(&d, &corpo, &falso).unwrap());
        visto.into_inner()
    };
    assert!(pedido(Idioma::En).starts_with("Execute action 1.2 «Tela» of blueprint «Portal»"));
    assert!(pedido(Idioma::Pt).starts_with("Execute a ação 1.2 «Tela» do blueprint «Portal»"));
    let sem_ids = com(Idioma::En, || super::iniciar::iniciar(&d, &json!({}), &|_: &Path, _: &Path, _: &str| Ok("wt")).unwrap_err());
    assert_eq!((sem_ids.codigo, sem_ids.mensagem.as_str()), ("faltam-ids", "Send workspaceId, blueprintId and acaoId."));
}
