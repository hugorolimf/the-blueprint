//! Testes de `hook/estado` (permissão, erro da API e compactação viram o estado do agente), do contexto que o SubagentStart devolve e do papel dos agentes do plugin.
use super::hooks::{identidade, subagente};
use super::hooks_estado::estado;
use super::hooks_tests::{dados, ligar, mcp};
use super::Mcp;
use serde_json::{json, Value};

fn claude(m: &Mcp) -> Value {
    m.agentes()["agentes"].as_array().and_then(|l| l.iter().find(|a| a["agente"] == "Claude").cloned()).expect("o Claude aparece")
}

fn evento(nome: &str, extra: Value) -> Value {
    let mut c = json!({"session_id": "s1", "cwd": "C:\\Proj", "hook_event_name": nome});
    c.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    c
}

#[test]
fn cada_evento_vira_o_estado_certo_e_a_resposta_nunca_decide_nada() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let casos = [
        (evento("PermissionRequest", json!({"tool_name": "Bash", "tool_input": {"command": "rm -rf x"}})), "aguardando", "aguardando permissão: Bash"),
        (evento("PermissionDenied", json!({"tool_name": "Bash", "reason": "[Irreversible Local Destruction]"})), "bloqueado", "permissão negada: Bash ([Irreversible Local Destruction])"),
        (evento("StopFailure", json!({"error": "rate_limit", "error_details": "429 Too Many Requests"})), "bloqueado", "erro da API: rate_limit (429 Too Many Requests)"),
        (evento("PreCompact", json!({"trigger": "auto", "custom_instructions": null})), "aguardando", "compactando a conversa"),
        (evento("PostCompact", json!({"trigger": "auto", "compact_summary": "um resumo enorme que não deve ir para lugar nenhum"})), "escrevendo", "conversa compactada, de volta ao trabalho"),
    ];
    for (corpo, esperado, frase) in casos {
        let resposta = estado(&m, &d, &corpo).unwrap();
        assert_eq!(resposta, json!({}), "{corpo}: sem decision, sem hookSpecificOutput");
        let a = claude(&m);
        assert_eq!((a["estado"].as_str(), a["mensagem"].as_str()), (Some(esperado), Some(frase)), "{corpo}");
        assert_eq!(a["blueprintId"], "p1");
    }
    assert!(m.itens().is_empty(), "presença só: nada vai para a fila nem para a linha do tempo");
}

#[test]
fn o_estado_mantem_a_acao_e_o_plano_do_ultimo_aviso() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let mut aviso = serde_json::Map::new();
    for (k, v) in [("blueprintId", "p1"), ("acaoId", "f2"), ("planoId", "t2"), ("estado", "escrevendo")] {
        aviso.insert(k.into(), json!(v));
    }
    m.registrar_atividade("Claude", aviso);
    estado(&m, &d, &evento("PermissionRequest", json!({"tool_name": "Edit"}))).unwrap();
    let a = claude(&m);
    assert_eq!((a["estado"].as_str(), a["acaoId"].as_str(), a["planoId"].as_str()), (Some("aguardando"), Some("f2"), Some("t2")), "a figura espera ao lado do plano");
    estado(&m, &d, &evento("PostCompact", json!({}))).unwrap();
    assert_eq!(claude(&m)["estado"], "escrevendo", "o PostCompact volta a escrevendo");
}

#[test]
fn ignora_o_que_nao_e_dele_ou_a_pasta_sem_blueprint() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    for corpo in [evento("PostToolUseFailure", json!({"tool_name": "Bash", "error": "x"})), evento("Stop", json!({})), evento("Notification", json!({})), json!({}), Value::Null] {
        assert_eq!(estado(&m, &d, &corpo).unwrap(), json!({}), "{corpo}");
    }
    let solta = json!({"hook_event_name": "PermissionRequest", "cwd": "C:\\Solta", "tool_name": "Bash"});
    assert_eq!(estado(&m, &d, &solta).unwrap(), json!({}));
    assert!(m.agentes()["agentes"].as_array().unwrap().is_empty(), "nada foi registrado");
}

#[test]
fn o_texto_que_vem_de_fora_sai_curto_e_sem_controle() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let longo = "x".repeat(900);
    estado(&m, &d, &evento("PermissionDenied", json!({"tool_name": format!("Ba\u{7}sh\n{longo}"), "reason": format!("\u{1b}]52;c;segredo\u{7}{longo}")}))).unwrap();
    let frase = claude(&m)["mensagem"].as_str().unwrap().to_string();
    assert!(frase.chars().count() <= 200 && !frase.chars().any(char::is_control), "{frase}");
    estado(&m, &d, &evento("StopFailure", json!({}))).unwrap(); // sem campos: ainda uma frase
    assert_eq!(claude(&m)["mensagem"], "erro da API: unknown");
    estado(&m, &d, &evento("PermissionRequest", json!({}))).unwrap();
    assert_eq!(claude(&m)["mensagem"], "aguardando permissão: tool");
}

#[test]
fn o_pedido_de_permissao_de_um_subagente_e_dele() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    estado(&m, &d, &evento("PermissionRequest", json!({"tool_name": "Bash", "agent_id": "ab12cd34", "agent_type": "blueprint:blueprint-builder"}))).unwrap();
    let lista = m.agentes();
    let a = lista["agentes"].as_array().unwrap().iter().find(|a| a["agente"] == "blueprint:blueprint-builder ab12").expect("o subagente aparece");
    assert_eq!((a["estado"].as_str(), a["papel"].as_str()), (Some("aguardando"), Some("construtor")));
    assert_eq!(lista["agentes"].as_array().unwrap().len(), 1, "o Claude principal não mudou");
}

#[test]
fn a_rota_responde_json_vazio_so_em_loopback_e_com_json() {
    use crate::http::pedido::Pedido;
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    let corpo = evento("PermissionRequest", json!({"cwd": "C:/Proj", "tool_name": "Bash"}));
    let pedido = |host: &str, tipo: &str| Pedido::novo("POST", "/api/mcp/hook/estado", vec![("host".into(), host.into()), ("content-type".into(), tipo.into())], corpo.to_string().into_bytes());
    let r = super::rotas::rota(&m, &d, &pedido("127.0.0.1:3000", "application/json"));
    assert_eq!(r.status, 200);
    assert_eq!(serde_json::from_slice::<Value>(r.bytes()).unwrap(), json!({}));
    assert_eq!(claude(&m)["estado"], "aguardando");
    assert_eq!(super::rotas::rota(&m, &d, &pedido("evil.example", "application/json")).status, 403);
    assert_eq!(super::rotas::rota(&m, &d, &pedido("127.0.0.1:3000", "text/plain")).status, 415);
}

fn inicio(extra: Value) -> Value {
    let mut c = evento("SubagentStart", json!({"agent_id": "d4e5f6", "agent_type": "Explore"}));
    c.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    c
}

#[test]
fn o_subagente_recebe_o_blueprint_em_ingles_e_a_regra_de_nao_marcar_plano() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let r = subagente(&m, &d, &inicio(json!({}))).unwrap();
    assert_eq!(r["hookSpecificOutput"]["hookEventName"], "SubagentStart");
    assert!(r.get("decision").is_none());
    let c = r["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
    assert!(c.contains("«Portal»") && c.contains("blueprint_id p1") && c.contains("«Tela»") && c.contains("plano_id t2"), "{c}");
    assert!(c.contains("this folder belongs to blueprint"), "em inglês mesmo com o usuário em português: {c}");
    assert!(c.ends_with("You are a subagent: do not mark plans done; report with registrar_progresso only if asked."), "{c}");
    assert!(!c.contains("«Feito»") && !c.contains("marcar_concluido only with the evidence"), "só os planos pendentes e sem a regra da sessão: {c}");
    assert!(c.chars().count() <= 3000);
}

#[test]
fn o_contexto_do_subagente_acompanha_a_worktree_e_a_sessao_sem_blueprint_nao_tem() {
    let (m, d) = (mcp(), dados());
    let solta = subagente(&m, &d, &inicio(json!({"cwd": "C:\\Solta"}))).unwrap();
    assert_eq!(solta, json!({}), "pasta sem blueprint: sem contexto");
    ligar(&d, "C:\\Proj");
    let sub = subagente(&m, &d, &inicio(json!({"cwd": "C:\\Proj\\sub"}))).unwrap();
    assert!(sub["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("blueprint_id p1"));
    let sem_id = evento("SubagentStart", json!({}));
    assert_eq!(subagente(&m, &d, &sem_id).unwrap(), json!({}), "a sessão principal não é subagente");
}

#[test]
fn os_agentes_do_plugin_viram_explorador_revisor_e_construtor() {
    let papel = |tipo: &str| identidade(&json!({"agent_id": "a1b2c3", "agent_type": tipo})).1;
    assert_eq!(papel("blueprint:blueprint-explorer"), "explorador");
    assert_eq!(papel("blueprint:blueprint-reviewer"), "revisor");
    assert_eq!(papel("blueprint:blueprint-builder"), "construtor");
    assert_eq!(papel("blueprint:blueprint"), "construtor");
    assert_eq!((papel("Explore"), papel("code-reviewer"), papel("general-purpose"), papel("Plan")), ("explorador", "revisor", "construtor", "construtor"));
    let (nome, _, sub) = identidade(&json!({"agent_id": "a1b2c3", "agent_type": "blueprint:blueprint-explorer"}));
    assert!(sub && nome == "blueprint:blueprint-explorer a1b2", "{nome}");
    assert_eq!(identidade(&json!({})).2, false);
}

#[test]
fn o_id_com_prefixo_agent_nao_junta_todos_os_subagentes_num_so() {
    let nome = |id: &str| identidade(&json!({"agent_id": id, "agent_type": "Explore"})).0;
    assert_eq!(nome("agent-abc123"), "Explore abc1", "o prefixo agent- sai antes dos 4 caracteres");
    assert_eq!(nome("agent-def456"), "Explore def4");
    assert_eq!(nome("def456"), "Explore def4", "sem prefixo, como sempre");
    assert_eq!(identidade(&json!({"agent_id": "agent-"})).2, false, "só o prefixo não é um id");
    // o painel de subagentes usa a mesma regra
    assert_eq!(super::subagentes_status::nome_do("agent-abc123", "Explore").as_deref(), Some("Explore abc1"));
}
