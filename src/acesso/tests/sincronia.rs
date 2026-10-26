//! As rotas da sincronia no modo hospedado (src/sincronia/rotas.rs com o Contexto do despacho): quem grava, quem lê, o autor com o nome da
//! sessão e o fluxo de quem foi removido fechando com `sessao-encerrada`.
use super::*;
use crate::http::pedido::Corpo;
use std::io::Read;

fn alterar(ws_plano: &str, texto: &str) -> Value {
    json!({"cliente": "c1", "ops": [{"op": "alterar", "tipo": "acao", "id": ws_plano, "campos": {"nome": texto}}]})
}

fn ler_ate(r: Resposta, ate: &str) -> String {
    let Corpo::Fluxo(mut leitor) = r.corpo else { panic!("não é fluxo") };
    let (mut texto, mut trecho) = (String::new(), [0u8; 4096]);
    while !texto.contains(ate) {
        let n = leitor.read(&mut trecho).unwrap();
        assert!(n > 0, "o fluxo fechou antes de {ate}: {texto}");
        texto.push_str(&String::from_utf8_lossy(&trecho[..n]));
    }
    texto
}

#[test]
fn colaborador_grava_com_o_proprio_nome_e_os_outros_papeis_nao() {
    let e = estado();
    let colab = pessoa(&e, Papel::Colaborador, "w2", "", "Colab");
    let sup = pessoa(&e, Papel::Supervisor, "w2", "", "Sup");
    let visit = pessoa(&e, Papel::Visitante, "w2", "p10", "");
    // a ação f1 é do workspace raiz (sm): o colaborador de w2 não alcança
    let r = despachar(&e, &pedido("POST", "/api/mudancas", Some(&colab), Some(alterar("f1", "x"))));
    assert_eq!((r.status, json_de(&r)["params"]["indice"].as_u64()), (403, Some(0)));
    let criar = json!({"ops": [{"op": "criar", "tipo": "acao", "id": "n1", "pai": "p9", "valor": {"nome": "Nova"}}]});
    let r = despachar(&e, &pedido("POST", "/api/mudancas", Some(&colab), Some(criar.clone())));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(despachar(&e, &pedido("POST", "/api/mudancas", Some(&sup), Some(alterar("n1", "y")))).status, 403, "supervisor só lê");
    assert_eq!(despachar(&e, &pedido("POST", "/api/mudancas", Some(&visit), Some(alterar("n1", "y")))).status, 403);
    let h = json_de(&despachar(&e, &pedido("GET", "/api/historico?workspace=w2&item=n1", Some(&sup), None)));
    assert_eq!(h["itens"][0]["membro"]["nome"], "Colab", "o autor é o nome da sessão");
    assert_eq!(despachar(&e, &pedido("GET", "/api/historico?workspace=w2&blueprint=p10", Some(&visit), None)).status, 403, "visitante nunca lê o histórico");
    assert_eq!(despachar(&e, &pedido("GET", "/api/eventos?workspace=w2", Some(&visit), None)).status, 403, "o visitante não lê o workspace inteiro");
    assert_eq!(despachar(&e, &pedido("GET", "/api/eventos?workspace=w2&blueprint=p10", Some(&visit), None)).status, 200, "só o blueprint do link");
    assert_eq!(despachar(&e, &pedido("GET", "/api/eventos?workspace=sm", Some(&colab), None)).status, 403);
}

#[test]
fn quem_foi_removido_recebe_sessao_encerrada_antes_do_proximo_lote() {
    let e = estado();
    let dono = pessoa(&e, Papel::Dono, "w2", "", "Dona");
    let colab = pessoa(&e, Papel::Colaborador, "w2", "", "Colab");
    let id = json_de(&despachar(&e, &pedido("GET", "/api/sessao", Some(&colab), None)))["membro"]["id"].as_str().unwrap().to_string();
    let fluxo = despachar(&e, &pedido("GET", "/api/eventos?workspace=w2", Some(&colab), None));
    assert_eq!(fluxo.status, 200);
    e.acesso.encerrar(&id);
    assert_eq!(despachar(&e, &pedido("POST", "/api/mudancas", Some(&dono), Some(alterar("x", "y")))).status, 200);
    let criar = json!({"ops": [{"op": "criar", "tipo": "acao", "id": "n2", "pai": "p9", "valor": {"nome": "Segredo"}}]});
    assert_eq!(despachar(&e, &pedido("POST", "/api/mudancas", Some(&dono), Some(criar))).status, 200);
    let texto = ler_ate(fluxo, "sessao-encerrada");
    assert!(!texto.contains("Segredo"), "nenhum lote chega a quem foi removido: {texto}");
}
