//! Testes das rotas dos hooks: ligar pasta, contexto da sessão e edição como progresso.
use super::hooks::{edicao, sessao, subagente, vincular};
use super::Mcp;
use crate::contexto::Contexto;
use crate::dados::Dados;
use serde_json::{json, Value};
use std::sync::Mutex;

pub(super) fn mcp() -> Mcp {
    Mcp::novo("127.0.0.1", false, "blueprint.exe", "http://127.0.0.1:3000", std::env::temp_dir())
}

pub(super) fn dados() -> Mutex<Dados> {
    let mut d = Dados::em_memoria().unwrap();
    let json = json!({"empresa": "Eu", "blueprints": [{"id": "p1", "nome": "Portal", "acoes": [
        {"id": "f1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "concluido", "planos": [{"id": "t1", "texto": "Feito", "feito": true}]},
        {"id": "f2", "nome": "Tela", "inicio": "2026-12-01", "fim": "2026-12-05", "status": "andamento", "planos": [{"id": "t2", "texto": "Criar tela", "feito": false}], "checkpoints": []}
    ]}]});
    d.gravar_tudo(&Contexto::local(), &json).unwrap();
    Mutex::new(d)
}

pub(super) fn ligar(d: &Mutex<Dados>, cwd: &str) {
    vincular(d, &json!({"cwd": cwd, "workspaceId": "sm", "blueprintId": "p1"})).unwrap();
}

#[test]
fn so_liga_a_blueprint_que_existe_e_a_pasta_valida() {
    let d = dados();
    assert_eq!(vincular(&d, &json!({"cwd": "C:\\p", "workspaceId": "sm", "blueprintId": "nao"})).unwrap_err().status, 404);
    assert_eq!(vincular(&d, &json!({"cwd": "", "workspaceId": "sm", "blueprintId": "p1"})).unwrap_err().status, 400);
    assert_eq!(vincular(&d, &json!({"cwd": "a\nb", "workspaceId": "sm", "blueprintId": "p1"})).unwrap_err().status, 400);
}

#[test]
fn a_sessao_numa_pasta_ligada_recebe_a_acao_atual_e_so_os_planos_pendentes() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let r = sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:\\Proj\\sub"})).unwrap();
    assert_eq!(r["vinculado"], true);
    let c = r["contexto"].as_str().unwrap();
    assert!(c.contains("«Portal»") && c.contains("blueprint_id p1") && c.contains("1.2 «Tela»") && c.contains("plano_id t2"), "{c}");
    assert!(!c.contains("«Feito»") && !c.contains("«Base»"), "{c}");
    assert_eq!(m.sessao("s1"), Some(("sm".into(), "p1".into())));
}

#[test]
fn a_sessao_numa_pasta_solta_manda_ligar_e_nao_inventa_blueprint() {
    let (m, d) = (mcp(), dados());
    let r = sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:\\Outra"})).unwrap();
    assert_eq!(r["vinculado"], false);
    assert!(r["contexto"].as_str().unwrap().contains("vincular_pasta"));
    assert_eq!(m.sessao("s1"), None);
}

#[test]
fn a_edicao_vira_progresso_com_o_arquivo_relativo_e_sem_pasta_ligada_nao_faz_nada() {
    let (m, d) = (mcp(), dados());
    let corpo = |cwd: &str| json!({"session_id": "s9", "cwd": cwd, "tool_name": "Edit", "tool_input": {"file_path": "C:\\Proj\\src\\main.rs"}});
    assert_eq!(edicao(&m, &d, &corpo("C:\\Solta")).unwrap(), json!({}));
    assert!(m.itens().is_empty());
    ligar(&d, "C:\\Proj");
    edicao(&m, &d, &corpo("C:\\Proj")).unwrap();
    let itens = m.itens();
    assert_eq!(itens.len(), 1);
    assert_eq!(itens[0]["tipo"], "progresso");
    assert_eq!(itens[0]["blueprintId"], "p1");
    assert_eq!(itens[0]["onde"], json!(["src/main.rs"]));
    assert_eq!(m.agentes()["agentes"][0]["agente"], "Claude");
}

#[test]
fn a_edicao_nao_tira_o_agente_do_plano_em_que_ele_avisou() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let mut aviso = serde_json::Map::new();
    for (k, v) in [("blueprintId", "p1"), ("acaoId", "f2"), ("planoId", "t2"), ("estado", "escrevendo")] {
        aviso.insert(k.into(), json!(v));
    }
    m.registrar_atividade("Claude", aviso);
    edicao(&m, &d, &json!({"cwd": "C:\\Proj", "tool_input": {"file_path": "C:\\Proj\\src\\main.rs"}})).unwrap();
    let a = &m.agentes()["agentes"][0];
    assert_eq!((a["acaoId"].as_str(), a["planoId"].as_str()), (Some("f2"), Some("t2")), "o hook herda a ação e o plano do último aviso");
    assert_eq!(a["onde"], json!(["src/main.rs"]));
}

fn agente_da_lista(m: &Mcp, nome: &str) -> Option<Value> {
    m.agentes()["agentes"].as_array().and_then(|l| l.iter().find(|a| a["agente"] == nome).cloned())
}

#[test]
fn a_edicao_de_um_subagente_vira_um_agente_proprio_com_o_papel_do_tipo() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let mut aviso = serde_json::Map::new();
    for (k, v) in [("blueprintId", "p1"), ("acaoId", "f2"), ("planoId", "t2"), ("estado", "escrevendo")] {
        aviso.insert(k.into(), json!(v));
    }
    m.registrar_atividade("Claude", aviso); // o Claude principal está num plano
    let corpo = |tipo: &str, id: &str| json!({"cwd": "C:\\Proj", "agent_id": id, "agent_type": tipo, "tool_input": {"file_path": "C:\\Proj\\src\\a.rs"}});
    edicao(&m, &d, &corpo("Explore", "a9cff85187b11bee4")).unwrap();
    edicao(&m, &d, &corpo("code-reviewer", "b1234")).unwrap();
    edicao(&m, &d, &corpo("general-purpose", "c777")).unwrap();
    let ex = agente_da_lista(&m, "Explore a9cf").expect("o subagente aparece com tipo e 4 caracteres do id");
    assert_eq!(ex["papel"], "explorador");
    assert!(ex["planoId"].is_null(), "o subagente não herda o plano do Claude principal: fica no cabeçalho");
    assert_eq!(agente_da_lista(&m, "code-reviewer b123").unwrap()["papel"], "revisor");
    assert_eq!(agente_da_lista(&m, "general-purpose c777").unwrap()["papel"], "construtor");
    assert_eq!(agente_da_lista(&m, "Claude").unwrap()["planoId"], "t2", "a edição do subagente não mexe no Claude principal");
    edicao(&m, &d, &corpo("Explore", "a9cff85187b11bee4")).unwrap(); // o mesmo agente de novo: não duplica
    assert_eq!(m.agentes()["agentes"].as_array().unwrap().len(), 4);
}

#[test]
fn o_subagente_aparece_ao_comecar_e_conclui_com_a_ultima_mensagem() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let corpo = |evento: &str, extra: Value| {
        let mut c = json!({"hook_event_name": evento, "cwd": "C:\\Proj", "agent_id": "d4e5f6", "agent_type": "Explore"});
        c.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        c
    };
    assert_eq!(subagente(&m, &d, &corpo("SubagentStart", json!({}))).unwrap(), json!({}));
    let a = agente_da_lista(&m, "Explore d4e5").unwrap();
    assert_eq!((a["estado"].as_str(), a["papel"].as_str()), (Some("lendo"), Some("explorador")));
    subagente(&m, &d, &corpo("SubagentStop", json!({"last_assistant_message": "Achei 3 arquivos: a.rs, b.rs e c.rs."}))).unwrap();
    let a = agente_da_lista(&m, "Explore d4e5").unwrap();
    assert_eq!((a["estado"].as_str(), a["mensagem"].as_str()), (Some("concluido"), Some("Achei 3 arquivos: a.rs, b.rs e c.rs.")));
    subagente(&m, &d, &corpo("SubagentStop", json!({}))).unwrap(); // sem mensagem final: uma frase padrão
    assert_eq!(agente_da_lista(&m, "Explore d4e5").unwrap()["mensagem"], "Explore d4e5 terminou");
}

#[test]
fn o_hook_de_subagente_ignora_o_que_nao_e_dele() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let sem_id = json!({"hook_event_name": "SubagentStart", "cwd": "C:\\Proj"}); // a sessão principal não manda agent_id
    let outro = json!({"hook_event_name": "Stop", "cwd": "C:\\Proj", "agent_id": "x1", "agent_type": "Explore"});
    let solta = json!({"hook_event_name": "SubagentStart", "cwd": "C:\\Solta", "agent_id": "x1", "agent_type": "Explore"}); // pasta sem blueprint
    for corpo in [sem_id, outro, solta, json!({}), Value::Null] {
        assert_eq!(subagente(&m, &d, &corpo).unwrap(), json!({}));
    }
    assert!(m.agentes()["agentes"].as_array().unwrap().is_empty(), "nada foi registrado");
    // nome e tipo vêm de fora: sem controle e sem ';' (o Windows Terminal o lê como separador), curtos
    subagente(&m, &d, &json!({"hook_event_name": "SubagentStart", "cwd": "C:\\Proj", "agent_id": "z9\n", "agent_type": "a;b\u{7}c"})).unwrap();
    let nomes: Vec<String> = m.agentes()["agentes"].as_array().unwrap().iter().map(|a| a["agente"].as_str().unwrap().to_string()).collect();
    assert_eq!(nomes, ["abc z9"]);
}

#[test]
fn corpo_estranho_nao_derruba_o_hook() {
    let (m, d) = (mcp(), dados());
    for corpo in [json!({}), json!({"cwd": 5}), json!({"session_id": "x", "tool_input": []}), Value::Null] {
        assert!(edicao(&m, &d, &corpo).is_ok());
        assert!(sessao(&m, &d, &corpo).is_ok());
    }
}

fn post(host: &str, tipo: &str, corpo: &Value) -> crate::http::pedido::Pedido {
    let cab = |k: &str, v: &str| (k.to_string(), v.to_string());
    crate::http::pedido::Pedido::novo("POST", "/api/mcp/hook/edicao", vec![cab("host", host), cab("content-type", tipo)], corpo.to_string().into_bytes())
}

#[test]
fn a_rota_aceita_o_post_do_hook_sem_origin_e_recusa_host_de_fora_e_corpo_sem_json() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    let corpo = json!({"session_id": "s1", "cwd": "C:/Proj", "tool_input": {"file_path": "C:/Proj/a.rs"}});
    // o hook `http` não manda Origin
    assert_eq!(super::rotas::rota(&m, &d, &post("127.0.0.1:3000", "application/json", &corpo)).status, 200);
    assert_eq!(m.itens().len(), 1);
    assert_eq!(super::rotas::rota(&m, &d, &post("evil.example", "application/json", &corpo)).status, 403);
    assert_eq!(super::rotas::rota(&m, &d, &post("127.0.0.1:3000", "text/plain", &corpo)).status, 415);
    assert_eq!(m.itens().len(), 1, "nada entrou pelos pedidos recusados");
}


#[test]
fn pasta_solta_avisa_uma_vez_e_vinculo_pode_ser_listado_e_removido() {
    let (m, d) = (mcp(), dados());
    let corpo = json!({"cwd": "C:/Solta"});
    sessao(&m, &d, &corpo).unwrap();
    sessao(&m, &d, &json!({"cwd": "c:\\solta\\"})).unwrap();
    assert_eq!(m.itens().len(), 1);
    assert_eq!(m.itens()[0]["tipo"], "pasta-sem-blueprint");
    ligar(&d, "C:/Solta");
    assert_eq!(super::hooks::travado(&d).listar_pastas(&Contexto::local()).unwrap()[0]["blueprintId"], "p1");
    sessao(&m, &d, &json!({"cwd":"C:/Solta", "session_id":"existente"})).unwrap();
    super::hooks::desvincular(&d, &corpo).unwrap();
    assert_eq!(sessao(&m, &d, &json!({"cwd":"C:/Solta", "session_id":"existente"})).unwrap()["vinculado"], false);
    assert_eq!(sessao(&m, &d, &corpo).unwrap()["vinculado"], false);
}

#[test]
fn proposta_nova_so_guarda_a_pasta_ate_usuario_aplicar() {
    let (m, d) = (mcp(), dados());
    let proposta = json!({"workspaceId":"sm", "pasta":"C:/Nova", "blueprint":{"nome":"Novo", "descricao":"Teste"}, "acoes":[{"nome":"Criar", "inicio":"2026-12-01", "fim":"2026-12-02", "planos":[{"texto":"Entregar", "descricao":"Escopo: criar. Resultado: entrega. Validação: teste."}], "checkpoints":[]}]});
    let (codigo, resposta) = super::acoes::proposta(&m, super::hooks::travado(&d).ler_tudo(&Contexto::local()).unwrap(), &proposta).unwrap();
    assert_eq!(codigo, 200, "{resposta}");
    assert_eq!(m.itens()[0]["pasta"], "C:/Nova");
    assert_eq!(super::hooks::travado(&d).blueprint_da_pasta(&Contexto::local(), "C:/Nova").unwrap(), None);
    let mut tudo = super::hooks::travado(&d).ler_tudo(&Contexto::local()).unwrap().clone();
    tudo["blueprints"].as_array_mut().unwrap().push(json!({"id":"novo", "nome":"Novo", "acoes":[]}));
    super::hooks::travado(&d).gravar_tudo(&Contexto::local(), &tudo).unwrap();
    vincular(&d, &json!({"cwd":"C:/Nova", "workspaceId":"sm", "blueprintId":"novo"})).unwrap();
    assert_eq!(sessao(&m, &d, &json!({"cwd":"C:/Nova/sub"})).unwrap()["blueprintId"], "novo");
}


#[test]
fn rotas_pastas_preservam_permissoes_e_exigem_json_ao_desvincular() {
    use crate::http::pedido::Pedido;
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Teste");
    let pedido = |metodo: &str, rota: &str, host: &str, tipo: &str| Pedido::novo(metodo, rota, vec![("host".into(), host.into()), ("content-type".into(), tipo.into())], br#"{"cwd":"C:/Teste"}"#.to_vec());
    assert_eq!(super::rotas::rota(&m, &d, &pedido("GET", "/api/mcp/pastas", "evil.example", "application/json")).status, 403);
    assert_eq!(super::rotas::rota(&m, &d, &pedido("GET", "/api/mcp/pastas", "127.0.0.1:3000", "application/json")).status, 200);
    assert_eq!(super::rotas::rota(&m, &d, &pedido("POST", "/api/mcp/pasta/desvincular", "127.0.0.1:3000", "text/plain")).status, 415);
    assert_eq!(super::hooks::travado(&d).listar_pastas(&Contexto::local()).unwrap().as_array().unwrap().len(), 1);
    assert_eq!(super::rotas::rota(&m, &d, &pedido("POST", "/api/mcp/pasta/desvincular", "127.0.0.1:3000", "application/json")).status, 200);
    assert!(super::hooks::travado(&d).listar_pastas(&Contexto::local()).unwrap().as_array().unwrap().is_empty());
}
