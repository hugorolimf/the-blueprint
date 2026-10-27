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
fn canal_declara_apenas_eventos_informativos() {
    let r = tratar(&json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{}})).unwrap();
    assert_eq!(r["result"]["capabilities"]["experimental"]["claude/channel"], json!({}));
    assert!(r["result"]["capabilities"]["experimental"].get("claude/channel/permission").is_none());
    let n = super::stdio::notificacao_canal("aviso");
    assert_eq!(n, json!({"jsonrpc":"2.0", "method":"notifications/claude/channel", "params":{"content":"aviso"}}));
}

#[test]
fn aviso_so_aceita_alvo_existente_e_checkpoint_persistido() {
    let m = mcp();
    let rx = m.assinar();
    let dados = json!({"blueprints": [{"id":"p1", "nome":"Teste", "acoes":[{"id":"a1", "checkpoints":[{"id":"c1", "feito":true}]}]}]});
    let mut corpo = json!({"tipo":"checkpoint-marcado", "workspaceId":"sm", "blueprintId":"p1", "checkpointId":"c1", "texto":"lixo"});
    assert!(super::eventos::aviso(&m, &dados, &corpo).is_ok());
    let evento = rx.recv().unwrap();
    assert_eq!(evento.nome, "checkpoint-marcado");
    assert_eq!(evento.dados.unwrap(), json!({"workspaceId":"sm", "blueprintId":"p1"}));
    corpo["checkpointId"] = json!("inexistente");
    assert!(super::eventos::aviso(&m, &dados, &corpo).is_err());
    corpo["tipo"] = json!("proposta-aplicada");
    assert!(super::eventos::aviso(&m, &dados, &corpo).is_ok());
    corpo["blueprintId"] = json!("outro");
    assert!(super::eventos::aviso(&m, &dados, &corpo).is_err());
    corpo["tipo"] = json!("arbitrario");
    assert!(super::eventos::aviso(&m, &dados, &corpo).is_err());
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
fn a_proposta_nova_entra_no_fim_com_numero_por_alvo_e_nao_apaga_nenhuma() {
    let m = mcp();
    let de = |blueprint: &str| {
        let mut i = item("proposta");
        i.insert("blueprintId".into(), json!(blueprint));
        i
    };
    let numero = |v: Value| v["numero"].as_i64().unwrap();
    assert_eq!(numero(m.empurra_proposta(de("p1"), json!({"n": "a"}), None)), 1); // id 1
    assert_eq!(numero(m.empurra_proposta(de("p2"), json!({}), None)), 1); // id 2: outro alvo, outra fila
    m.empurra(item("progresso")); // id 3
    assert_eq!(numero(m.empurra_proposta(de("p1"), json!({"n": "b"}), None)), 2); // id 4: não tira o 1
    let ids: Vec<i64> = m.itens().iter().map(|i| i["id"].as_i64().unwrap()).collect();
    assert_eq!(ids, vec![1, 2, 3, 4]);
    // com número, a proposta toma o lugar daquela (mesmo número e mesma criadaEm), sem mexer nas outras
    let criada = m.itens()[0]["criadaEm"].clone();
    let troca = m.empurra_proposta(de("p1"), json!({"n": "c"}), Some(1));
    assert_eq!((troca["numero"].as_i64(), troca["criadaEm"].clone(), troca["id"].as_i64()), (Some(1), criada, Some(5)));
    let numeros: Vec<i64> = m.itens().iter().filter(|i| i["blueprintId"] == "p1" && i["tipo"] == "proposta").map(|i| i["numero"].as_i64().unwrap()).collect();
    assert_eq!(numeros, vec![2, 1]);
    assert_eq!(m.entrada_da_proposta("sm", "p1", 1).unwrap(), json!({"n": "c"}));
    // só o ack da página tira a proposta (e a entrada guardada junto); o número seguinte não volta atrás
    m.ack(&HashSet::from([5]));
    assert_eq!(m.entrada_da_proposta("sm", "p1", 1).unwrap_err(), vec![2]);
    assert_eq!(numero(m.empurra_proposta(de("p1"), json!({}), None)), 3);
    assert_eq!(m.entrada_da_proposta("sm", "p9", 1).unwrap_err(), Vec::<i64>::new(), "fila vazia");
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
    for t in ["ALWAYS up to date", "CLAUDE.md", "listar_blueprints", "propor_blueprint", "registrar_progresso", "marcar_concluido", "marcar_checkpoint_feito", "reportar_bloqueio", "registrar_trabalho", "auto-applied"] {
        assert!(inicio.contains(t), "instruções: {t} nos primeiros 512");
    }
    assert_eq!(rpc("ping", json!({}))["result"], json!({}));
}

#[test]
fn as_ferramentas_do_mcp() {
    let r = rpc("tools/list", json!({}));
    let nomes: Vec<&str> = r["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        nomes,
        ["listar_blueprints", "guia_de_planejamento", "ler_blueprint", "propor_blueprint", "registrar_trabalho", "anexar_prototipo", "anexar_apresentacao", "ler_apresentacao", "editar_apresentacao", "registrar_progresso", "marcar_concluido", "marcar_nao_concluido", "marcar_checkpoint_feito", "marcar_checkpoint_nao_feito", "reportar_bloqueio", "vincular_pasta", "responder_conversa"]
    );
    let propor = r["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "propor_blueprint").unwrap();
    assert!(propor["inputSchema"]["properties"].get("remover").is_some(), "a ferramenta já conhece mover e apagar");
}

#[test]
fn propor_blueprint_conhece_a_fila_e_o_agente_aprende_a_ajustar() {
    let r = rpc("tools/list", json!({}));
    let propor = r["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "propor_blueprint").unwrap();
    for campo in ["ajustar", "substituir"] {
        assert_eq!(propor["inputSchema"]["properties"][campo]["type"], "integer", "{campo}");
    }
    assert!(propor["description"].as_str().unwrap().contains("QUEUE"));
    let editar = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "editar"}}));
    let editar = editar["result"]["content"][0]["text"].as_str().unwrap();
    assert!(editar.contains("ajustar: <number>") && editar.contains("+a1.p2") && editar.contains("substituir"), "o guia explica o ajuste");
    let instrucoes = rpc("initialize", json!({}))["result"]["instructions"].as_str().unwrap().to_string();
    assert!(instrucoes.contains("ajustar: <number>"), "as instruções falam do ajuste");
    let inicio: String = instrucoes.chars().take(512).collect();
    assert!(inicio.contains("propor_blueprint") && inicio.contains("ALWAYS up to date"), "a regra e os nomes continuam nos primeiros 512");
    let ids = json!([{"id": "+a1", "nome": "Nova", "planos": [{"id": "+a1.p1", "texto": "Criar a tela"}], "checkpoints": []}, {"id": "f1", "nome": "Base", "planos": [], "checkpoints": [{"id": "+f1.c1", "nome": "Pronto"}]}]);
    let texto = super::stdio::ids_provisorios(&ids);
    assert!(texto.contains("- action +a1 \"Nova\"; plans: +a1.p1 \"Criar a tela\"") && texto.contains("- action f1 \"Base\"; checkpoints: +f1.c1 \"Pronto\""), "{texto}");
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
    assert!(texto.starts_with("# Plan a blueprint") && !texto.contains("description:"), "{}", &texto[..60.min(texto.len())]);
    let editar = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "editar"}}));
    assert!(editar["result"]["content"][0]["text"].as_str().unwrap().contains("Move"));
    for topico in ["planejar", "editar", "executar", "prototipos", "importar", "apresentar", "apresentacao_ia", "apresentacao_conteudo", "apresentacao_figuras", "apresentacao_ajustar"] {
        let r = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": topico}}));
        assert!(r["result"]["content"][0]["text"].as_str().unwrap().chars().count() < 4500, "o tópico {topico} cabe em menos de 4.500 caracteres"); // o guia em inglês é ~5% mais longo que o português (era 4.000)
    }
    let planejar = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "planejar"}}));
    let planejar = planejar["result"]["content"][0]["text"].as_str().unwrap();
    assert!(planejar.contains("## Deadlines at AI pace") && planejar.contains("`horas`") && planejar.contains("1 to 3 calendar days"), "a régua de prazos da IA está no tópico planejar");
    let ruim = rpc("tools/call", json!({"name": "guia_de_planejamento", "arguments": {"topico": "nada"}}));
    assert_eq!(ruim["result"]["isError"], true);
    assert!(ruim["result"]["content"][0]["text"].as_str().unwrap().contains("Unknown topic: nada"));
}

#[test]
fn argumentos_que_faltam_voltam_como_erro_da_ferramenta() {
    let r = rpc("tools/call", json!({"name": "marcar_concluido", "arguments": {"workspace_id": "sm", "blueprint_id": "p", "acao_id": "f", "plano_id": "t"}}));
    assert_eq!(r["result"]["isError"], true);
    assert_eq!(r["result"]["content"][0]["text"], "evidencia is required.");
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
