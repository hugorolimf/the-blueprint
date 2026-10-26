//! O que cada papel recebe pelo SSE: quem só lê um blueprint não vê os outros, o visitante não vê comentários do agente nem presença e um
//! `mover` entre blueprints vira `recarregar` para ele; quem perdeu o acesso recebe `sessao-encerrada` e a conexão fecha.
use super::recebido;
use crate::contexto::{Acesso, Contexto, Papel};
use crate::sincronia::{exige_gravar, Evento, Sincronia};
use serde_json::json;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

fn evento(bp: &str, cruza: bool) -> Evento {
    let dados = json!({"workspaceId": "w1", "blueprintId": bp, "ops": [{"op": "alterar", "tipo": "plano", "id": "p1", "campos": {"agente": {"estado": "x", "comentarios": [{"texto": "segredo"}]}}}]});
    Evento { workspace_id: "w1".into(), blueprint_id: bp.into(), dados, cruza }
}

#[test]
fn cada_um_recebe_so_o_que_le_e_o_visitante_sem_comentarios() {
    let s = Sincronia::nova(1, true);
    let todo = Contexto::membro("m1", "Ana", vec![Acesso::novo("w1", "", Papel::Supervisor)]);
    let so_b2 = Contexto::membro("m2", "Bruno", vec![Acesso::novo("w1", "b2", Papel::Colaborador)]);
    let visitante = Contexto::visitante("c9", "w1", "b1", "folha");
    let (rx_todo, _a) = s.assinar("w1", None, &todo, None);
    let (rx_b2, _b) = s.assinar("w1", None, &so_b2, None);
    let (rx_vis, _c) = s.assinar("w1", None, &visitante, Some("c-v"));
    recebido(&rx_todo);
    s.publicar(2, vec![evento("b1", false)]);
    assert!(recebido(&rx_todo).contains("segredo"), "quem lê o workspace recebe tudo");
    assert_eq!(recebido(&rx_b2), "", "quem só lê o b2 não recebe o b1");
    let vis = recebido(&rx_vis);
    assert!(vis.contains("event: mudancas") && !vis.contains("segredo") && !vis.contains("presenca"), "{vis}");
    s.publicar(3, vec![evento("b1", true)]);
    assert!(recebido(&rx_vis).starts_with("event: recarregar"), "mover entre blueprints: o visitante recarrega");
    s.publicar(4, vec![Evento { blueprint_id: String::new(), ..evento("", false) }]);
    assert_eq!(recebido(&rx_vis), "", "o próprio workspace não é do visitante");
}

#[test]
fn quem_perdeu_o_acesso_recebe_sessao_encerrada_e_a_conexao_fecha() {
    let s = Sincronia::nova(1, false);
    let ana = Contexto::membro("m1", "Ana", vec![Acesso::novo("w1", "", Papel::Colaborador)]);
    let bruno = Contexto::membro("m2", "Bruno", vec![Acesso::novo("w1", "", Papel::Colaborador)]);
    let (rx_ana, _a) = s.assinar("w1", None, &ana, None);
    let (rx_bruno, _b) = s.assinar("w1", None, &bruno, None);
    s.podar(|c| c.usuario != "m2");
    assert_eq!(recebido(&rx_bruno), "event: sessao-encerrada\ndata: {}\n\n");
    assert_eq!(rx_bruno.recv_timeout(Duration::from_millis(10)), Err(RecvTimeoutError::Disconnected), "o canal fechou: o fluxo termina");
    s.publicar(2, vec![evento("b1", false)]);
    assert!(recebido(&rx_ana).contains("mudancas"));
    assert_eq!(s.assinantes(), 1);
}

#[test]
fn gravar_pede_o_papel_no_alvo_e_administrar_pede_o_dono_de_todos() {
    let colab = Contexto::membro("m1", "Ana", vec![Acesso::novo("w1", "", Papel::Colaborador)]);
    let sup = Contexto::membro("m2", "Bruno", vec![Acesso::novo("w1", "", Papel::Supervisor)]);
    let dono_ws = Contexto::membro("m3", "Caio", vec![Acesso::novo("w1", "", Papel::Dono)]);
    assert!(exige_gravar(&colab, false, "w1", Some("b1")).is_ok());
    assert!(exige_gravar(&colab, false, "w2", Some("b9")).is_err(), "outro workspace");
    assert!(exige_gravar(&sup, false, "w1", Some("b1")).is_err(), "supervisor só lê");
    assert!(exige_gravar(&dono_ws, true, "w1", None).is_err(), "criar ou apagar workspace é do dono de todos");
    assert!(exige_gravar(&Contexto::local(), true, "w1", None).is_ok());
}
