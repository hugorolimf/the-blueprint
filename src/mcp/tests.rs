//! Testes do MCP: a fila, o SSE e o servidor stdio (o que não precisa de um servidor HTTP de pé).
use super::cliente::codificar;
use super::stdio::tratar;
use super::{FluxoSse, Mcp};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::io::Read;

fn mcp() -> Mcp {
    Mcp::novo("127.0.0.1", false, "blueprint.exe", "http://127.0.0.1:3000", std::env::temp_dir())
}

fn item(tipo: &str) -> Map<String, Value> {
    json!({"tipo": tipo, "workspaceId": "sm", "blueprintId": "p1"}).as_object().unwrap().clone()
}

#[test]
fn a_fila_numera_carimba_e_so_guarda_200() {
    let m = mcp();
    let primeiro = m.empurra(item("progresso"));
    assert_eq!(primeiro["id"], 1);
    assert!(primeiro["em"].as_str().unwrap().ends_with('Z'));
    for _ in 0..250 {
        m.empurra(item("progresso"));
    }
    let itens = m.itens();
    assert_eq!(itens.len(), 200);
    assert_eq!(itens[0]["id"], 52);
    m.ack(&HashSet::from([52, 53, 9999]));
    assert_eq!(m.itens().len(), 198);
}

#[test]
fn a_proposta_nova_tira_a_pendente_do_mesmo_alvo_e_so_dele() {
    let m = mcp();
    let de = |tipo: &str, blueprint: &str| {
        let mut i = item(tipo);
        i.insert("blueprintId".into(), json!(blueprint));
        i
    };
    m.empurra_proposta(de("proposta", "p1"), "sm", "p1"); // id 1
    m.empurra_proposta(de("proposta", "p2"), "sm", "p2"); // id 2
    m.empurra(item("progresso")); // id 3
    m.empurra_proposta(de("proposta", "p1"), "sm", "p1"); // id 4: tira o 1, deixa o 2 e o 3
    let ids: Vec<i64> = m.itens().iter().map(|i| i["id"].as_i64().unwrap()).collect();
    assert_eq!(ids, vec![2, 3, 4]);
}

#[test]
fn o_sse_abre_avisa_junta_avisos_seguidos_e_fecha() {
    let m = mcp();
    let mut f = FluxoSse::novo(m.assinar());
    let mut buf = [0u8; 256];
    let n = f.read(&mut buf).unwrap();
    assert!(String::from_utf8_lossy(&buf[..n]).starts_with("retry: 3000"));
    for _ in 0..5 {
        m.empurra(item("progresso"));
    }
    let n = f.read(&mut buf).unwrap();
    assert_eq!(String::from_utf8_lossy(&buf[..n]), "event: fila\ndata: nova\n\n", "cinco itens, um aviso");
    drop(m);
    assert_eq!(f.read(&mut buf).unwrap(), 0, "sem o MCP, o fluxo termina");
}

#[test]
fn quem_fecha_a_conexao_sai_da_lista_de_assinantes() {
    let m = mcp();
    drop(FluxoSse::novo(m.assinar()));
    let viva = FluxoSse::novo(m.assinar());
    m.empurra(item("progresso"));
    assert_eq!(m.assinantes.lock().unwrap().len(), 1);
    drop(viva);
}

#[test]
fn encodeuricomponent() {
    assert_eq!(codificar("sm"), "sm");
    assert_eq!(codificar("a b&c=ç/"), "a%20b%26c%3D%C3%A7%2F");
    assert_eq!(codificar("it's (ok)!~*"), "it's%20(ok)!~*");
}

fn rpc(metodo: &str, params: Value) -> Value {
    tratar(&json!({"jsonrpc": "2.0", "id": 7, "method": metodo, "params": params})).unwrap()
}

#[test]
fn o_servidor_stdio_se_apresenta_como_o_de_node() {
    let r = rpc("initialize", json!({"protocolVersion": "2025-03-26"}));
    assert_eq!((r["id"].as_i64(), r["result"]["protocolVersion"].as_str()), (Some(7), Some("2025-03-26")));
    assert_eq!(r["result"]["serverInfo"], json!({"name": "blueprint", "version": "2.0.0"}));
    assert_eq!(rpc("initialize", json!({"protocolVersion": "1999"}))["result"]["protocolVersion"], "2025-06-18");
    // o que o Codex lê: regras e nomes das ferramentas nos primeiros 512 caracteres
    let inicio: String = r["result"]["instructions"].as_str().unwrap().chars().take(512).collect();
    for t in ["SEMPRE atualizado", "CLAUDE.md", "listar_blueprints", "propor_blueprint", "registrar_progresso", "marcar_concluido", "marcar_checkpoint_feito", "reportar_bloqueio"] {
        assert!(inicio.contains(t), "instruções: {t} nos primeiros 512");
    }
    assert_eq!(rpc("ping", json!({}))["result"], json!({}));
}

#[test]
fn as_ferramentas_sao_as_mesmas_do_node() {
    let r = rpc("tools/list", json!({}));
    let nomes: Vec<&str> = r["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        nomes,
        ["listar_blueprints", "guia_de_planejamento", "ler_blueprint", "propor_blueprint", "anexar_prototipo", "registrar_progresso", "marcar_concluido", "marcar_nao_concluido", "marcar_checkpoint_feito", "marcar_checkpoint_nao_feito", "reportar_bloqueio"]
    );
    let propor = r["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "propor_blueprint").unwrap();
    assert!(propor["inputSchema"]["properties"].get("remover").is_some(), "a ferramenta já conhece mover e apagar");
}

#[test]
fn erros_de_protocolo() {
    assert!(tratar(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none(), "notificação não tem resposta");
    assert_eq!(rpc("nao/existe", json!({}))["error"]["code"], -32601);
    assert_eq!(rpc("tools/call", json!({"name": "inventada"}))["error"]["code"], -32602);
}

#[test]
fn o_guia_vem_por_partes_sem_o_cabecalho() {
    let indice = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {}}));
    let texto = indice["result"]["content"][0]["text"].as_str().unwrap();
    assert!(texto.starts_with("# Planejar um blueprint") && !texto.contains("description:"), "{}", &texto[..60.min(texto.len())]);
    let editar = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "editar"}}));
    assert!(editar["result"]["content"][0]["text"].as_str().unwrap().contains("Mover"));
    for topico in ["planejar", "editar", "executar", "prototipos"] {
        let r = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": topico}}));
        assert!(r["result"]["content"][0]["text"].as_str().unwrap().chars().count() < 4000, "o tópico {topico} cabe em menos de 4.000 caracteres");
    }
    let planejar = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "planejar"}}));
    let planejar = planejar["result"]["content"][0]["text"].as_str().unwrap();
    assert!(planejar.contains("## Prazos no ritmo da IA") && planejar.contains("`horas`") && planejar.contains("de 1 a 3 dias"), "a régua de prazos da IA está no tópico planejar");
    let ruim = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "nada"}}));
    assert_eq!(ruim["result"]["isError"], true);
    assert!(ruim["result"]["content"][0]["text"].as_str().unwrap().contains("Tópico desconhecido: nada"));
}

#[test]
fn argumentos_que_faltam_voltam_como_erro_da_ferramenta() {
    let r = rpc("tools/call", json!({"name": "marcar_concluido", "arguments": {"workspace_id": "sm", "blueprint_id": "p", "acao_id": "f", "plano_id": "t"}}));
    assert_eq!(r["result"]["isError"], true);
    assert_eq!(r["result"]["content"][0]["text"], "evidencia é obrigatório.");
}

#[test]
fn a_ferramenta_de_progresso_conhece_os_campos_de_atividade() {
    let r = rpc("tools/list", json!({}));
    let ferramentas = r["result"]["tools"].as_array().unwrap();
    for nome in ["registrar_progresso", "reportar_bloqueio"] {
        let props = &ferramentas.iter().find(|t| t["name"] == nome).unwrap()["inputSchema"]["properties"];
        for campo in ["agente", "papel", "estado", "onde", "percentual"] {
            assert!(props.get(campo).is_some(), "{nome} sem {campo}");
        }
        assert_eq!(props["estado"]["enum"], json!(["lendo", "escrevendo", "testando", "aguardando", "bloqueado", "concluido"]), "{nome}");
    }
    let instrucoes = rpc("initialize", json!({}))["result"]["instructions"].as_str().unwrap().chars().take(512).collect::<String>();
    assert!(instrucoes.contains("agente, estado, onde"), "a regra de reportar a atividade fica nos primeiros 512 caracteres");
}

#[test]
fn o_sse_separa_os_avisos_de_fila_e_de_agentes() {
    let m = mcp();
    let mut f = FluxoSse::novo(m.assinar());
    let mut buf = [0u8; 256];
    f.read(&mut buf).unwrap();
    m.registrar_atividade("ana", json!({"estado": "lendo"}).as_object().unwrap().clone());
    m.registrar_atividade("ana", json!({"estado": "testando"}).as_object().unwrap().clone());
    m.empurra(item("progresso"));
    let n = f.read(&mut buf).unwrap();
    assert_eq!(String::from_utf8_lossy(&buf[..n]), "event: agentes\ndata: nova\n\nevent: fila\ndata: nova\n\n");
    assert_eq!(m.agentes()["agentes"][0]["estado"], "testando");
}
