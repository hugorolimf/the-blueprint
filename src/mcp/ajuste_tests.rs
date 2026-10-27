//! Testes do ajuste de propostas na fila: título trocado, plano acrescentado, número desconhecido, fila vazia e a revalidação da proposta inteira.
use super::acoes::proposta;
use super::Mcp;
use serde_json::{json, Value};

fn mcp() -> Mcp {
    Mcp::novo("127.0.0.1", false, "blueprint.exe", "http://127.0.0.1:3000", std::env::temp_dir())
}

fn dados() -> Value {
    json!({"blueprints": [{"id": "p1", "nome": "Portal", "acoes": [{"id": "f1", "nome": "Base", "inicio": "2030-01-01", "fim": "2030-03-01", "status": "planejado", "obs": "", "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "Escopo x. Validação: roda.", "feito": false}], "checkpoints": []}]}]})
}

fn plano(texto: &str) -> Value {
    json!({"texto": texto, "descricao": "Escopo: x. Validação: o teste passa."})
}

fn nova(m: &Mcp) -> Value {
    let corpo = json!({"workspaceId": "sm", "blueprintId": "p1", "mensagem": "m", "acoes": [{"nome": "Nova ação", "inicio": "2030-04-01", "fim": "2030-04-30", "planos": [plano("Criar a tela"), plano("Criar a rota")], "checkpoints": [{"nome": "Pronto", "data": "2030-04-29"}]}]});
    proposta(m, &dados(), &corpo).unwrap().1
}

fn ajusta(m: &Mcp, extra: Value) -> Result<(u16, Value), crate::http::pedido::ErroHttp> {
    let mut corpo = json!({"workspaceId": "sm", "blueprintId": "p1"});
    for (k, v) in extra.as_object().unwrap() {
        corpo[k] = v.clone();
    }
    proposta(m, &dados(), &corpo)
}

fn na_fila(m: &Mcp, numero: i64) -> Value {
    m.itens().into_iter().find(|i| i["numero"] == numero).unwrap()
}

#[test]
fn a_resposta_traz_o_numero_e_os_ids_provisorios_compactos() {
    let m = mcp();
    let r = nova(&m);
    assert_eq!(r["numero"], 1);
    assert_eq!(r["ids"], json!([{"id": "+a1", "nome": "Nova ação", "planos": [{"id": "+a1.p1", "texto": "Criar a tela"}, {"id": "+a1.p2", "texto": "Criar a rota"}], "checkpoints": [{"id": "+a1.c1", "nome": "Pronto"}]}]));
    assert_eq!(nova(&m)["numero"], 2, "a segunda não apaga a primeira");
    assert_eq!(m.itens().len(), 2);
}

#[test]
fn ajustar_o_titulo_muda_so_ele_e_revalida_a_proposta_inteira() {
    let m = mcp();
    nova(&m);
    let (status, r) = ajusta(&m, json!({"ajustar": 1, "acoes": [{"id": "+a1", "planos": [{"id": "+a1.p2", "texto": "Montar a rota"}]}]})).unwrap();
    assert_eq!((status, r["numero"].as_i64()), (200, Some(1)));
    let a = &na_fila(&m, 1)["proposta"]["acoes"][0];
    assert_eq!((a["planos"][0]["texto"].as_str(), a["planos"][1]["texto"].as_str()), (Some("Criar a tela"), Some("Montar a rota")));
    assert!(a["planos"][1]["descricao"].as_str().unwrap().contains("Validação"), "a descrição não some por omissão");
    assert_eq!((a["nome"].as_str(), a["checkpoints"].as_array().unwrap().len()), (Some("Nova ação"), 1));
    assert_eq!(m.itens().len(), 1, "o ajuste atualiza a proposta, não cria outra");
    // Títulos longos são preservados na proposta ajustada.
    let longo = "Montar uma rota com um título bem mais comprido que o limite de 45";
    let (status, r) = ajusta(&m, json!({"ajustar": 1, "acoes": [{"id": "+a1", "planos": [{"id": "+a1.p2", "texto": longo}]}]})).unwrap();
    assert_eq!(status, 200);
    assert_eq!(r["numero"], 1);
    assert_eq!(na_fila(&m, 1)["proposta"]["acoes"][0]["planos"][1]["texto"], longo);
}

#[test]
fn ajustar_pode_acrescentar_plano_e_mexer_numa_acao_existente() {
    let m = mcp();
    nova(&m);
    let r = ajusta(&m, json!({"ajustar": 1, "mensagem": "Com mais um plano", "acoes": [{"id": "+a1", "planos": [plano("Cobrir erros")]}, {"id": "f1", "fim": "2030-03-10"}]})).unwrap().1;
    let p = &na_fila(&m, 1)["proposta"];
    assert_eq!(p["mensagem"], "Com mais um plano");
    assert_eq!(p["acoes"].as_array().unwrap().len(), 2);
    assert_eq!(p["acoes"][0]["planos"].as_array().unwrap().len(), 3, "o plano entra no fim, os outros ficam");
    assert_eq!((p["acoes"][1]["id"].as_str(), p["acoes"][1]["fim"].as_str(), p["acoes"][1]["nome"].as_str()), (Some("f1"), Some("2030-03-10"), Some("Base")));
    assert_eq!(r["ids"][0]["planos"][2], json!({"id": "+a1.p3", "texto": "Cobrir erros"}), "o id provisório novo segue o último");
    // os ids que já existiam não mudam depois de outro ajuste
    let r = ajusta(&m, json!({"ajustar": 1, "acoes": [{"id": "+a1", "nome": "Outra"}]})).unwrap().1;
    assert_eq!(r["ids"][0]["planos"][0]["id"], "+a1.p1");
    assert_eq!(r["ids"][0]["planos"][2]["id"], "+a1.p3");
}

#[test]
fn ajustar_com_numero_desconhecido_ou_fila_vazia_e_erro_claro() {
    let m = mcp();
    let vazia = ajusta(&m, json!({"ajustar": 1, "acoes": [{"nome": "x"}]})).unwrap_err();
    assert_eq!(vazia.status, 404);
    assert!(vazia.mensagem.contains("está vazia"), "{}", vazia.mensagem);
    nova(&m);
    nova(&m);
    let desconhecido = ajusta(&m, json!({"ajustar": 9, "acoes": [{"id": "+a1", "nome": "x"}]})).unwrap_err();
    assert_eq!(desconhecido.status, 404);
    assert!(desconhecido.mensagem.contains("nº 9") && desconhecido.mensagem.contains("nº 1, nº 2"), "{}", desconhecido.mensagem);
    assert_eq!(ajusta(&m, json!({"ajustar": 0, "acoes": []})).unwrap_err().status, 400, "número inválido");
    assert_eq!(ajusta(&m, json!({"ajustar": 1, "substituir": 2, "acoes": []})).unwrap_err().status, 400, "ajustar OU substituir");
    let id_ruim = ajusta(&m, json!({"ajustar": 1, "acoes": [{"id": "+a7", "nome": "x"}]})).unwrap_err();
    assert_eq!(id_ruim.status, 400);
    assert!(id_ruim.mensagem.contains("+a7"));
    assert_eq!(m.itens().len(), 2, "nada disso mexe na fila");
}

#[test]
fn substituir_troca_a_proposta_inteira_e_fica_com_o_numero() {
    let m = mcp();
    nova(&m);
    nova(&m);
    let corpo = json!({"workspaceId": "sm", "blueprintId": "p1", "substituir": 1, "acoes": [{"nome": "Só esta", "inicio": "2030-05-01", "fim": "2030-05-20", "planos": [plano("Criar o mínimo")]}]});
    let r = proposta(&m, &dados(), &corpo).unwrap().1;
    assert_eq!(r["numero"], 1);
    assert_eq!(na_fila(&m, 1)["proposta"]["acoes"].as_array().unwrap().len(), 1);
    assert_eq!(na_fila(&m, 1)["proposta"]["acoes"][0]["nome"], "Só esta");
    assert_eq!(na_fila(&m, 2)["proposta"]["acoes"][0]["nome"], "Nova ação", "a outra proposta não é tocada");
}
