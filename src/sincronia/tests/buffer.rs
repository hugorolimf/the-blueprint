use super::recebido;
use crate::contexto::Contexto;
use crate::sincronia::{Evento, Sincronia, MAX_LOTES};
use serde_json::json;

fn lote(s: &Sincronia, versao: u64, ws: &str) {
    s.publicar(versao, vec![Evento { workspace_id: ws.to_string(), blueprint_id: String::new(), dados: json!({"versao": versao, "workspaceId": ws}), cruza: false }]);
}

#[test]
fn quem_assina_recebe_so_o_proprio_workspace_com_o_id_da_versao() {
    let s = Sincronia::nova(100, false);
    let (rx, _g) = s.assinar("w1", Some(100), &Contexto::local(), None);
    let (rx2, _g2) = s.assinar("w2", None, &Contexto::local(), None);
    lote(&s, 101, "w1");
    assert_eq!(recebido(&rx), "id: 101\nevent: mudancas\ndata: {\"versao\":101,\"workspaceId\":\"w1\"}\n\n");
    assert_eq!(recebido(&rx2), "");
}

#[test]
fn retomar_reenvia_o_que_perdeu_em_ordem() {
    let s = Sincronia::nova(100, false);
    lote(&s, 101, "w1");
    lote(&s, 102, "w2");
    lote(&s, 103, "w1");
    let (rx, _g) = s.assinar("w1", Some(101), &Contexto::local(), None);
    let texto = recebido(&rx);
    assert!(texto.starts_with("id: 103\n") && !texto.contains("id: 101") && !texto.contains("w2"), "{texto}");
    let (rx, _g) = s.assinar("w1", Some(100), &Contexto::local(), None);
    assert_eq!(recebido(&rx).matches("event: mudancas").count(), 2);
}

#[test]
fn buraco_no_buffer_versao_estranha_ou_put_no_meio_pedem_recarregar() {
    let s = Sincronia::nova(100, false);
    for v in 101..=(101 + MAX_LOTES as u64) {
        lote(&s, v, "w1");
    }
    let (rx, _g) = s.assinar("w1", Some(100), &Contexto::local(), None);
    assert!(recebido(&rx).starts_with("event: recarregar\n"), "o 101 saiu do buffer");
    let (rx, _g) = s.assinar("w1", Some(999_999), &Contexto::local(), None);
    assert!(recebido(&rx).contains("recarregar"), "versão do futuro (o app reiniciou)");
    let (vivo, _g) = s.assinar("w1", None, &Contexto::local(), None);
    s.substituido(5000);
    assert_eq!(recebido(&vivo), "id: 5000\nevent: substituido\ndata: {\"versao\":5000}\n\n");
    let (rx, _g) = s.assinar("w1", Some(4000), &Contexto::local(), None);
    assert!(recebido(&rx).contains("recarregar"), "retomar de antes do PUT");
    let (rx, _g) = s.assinar("w1", Some(5000), &Contexto::local(), None);
    assert_eq!(recebido(&rx), "", "em dia: nada a mandar");
}

#[test]
fn a_conexao_que_cai_sai_dos_assinantes() {
    let s = Sincronia::nova(1, false);
    let (_rx, g) = s.assinar("w1", None, &Contexto::local(), None);
    assert_eq!(s.assinantes(), 1);
    drop(g);
    assert_eq!(s.assinantes(), 0);
    let (rx, _g) = s.assinar("w1", None, &Contexto::local(), None);
    drop(rx);
    lote(&s, 2, "w1"); // o canal fechado sai na primeira tentativa de envio
    assert_eq!(s.assinantes(), 0);
}
