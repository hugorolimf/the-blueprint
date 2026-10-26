//! A chave de MCP: gerada uma vez, guardada só pelo hash, o agente age com o papel do membro e a chave revogada leva 401.
use super::*;

/// Um pedido do agente: sem cookie e sem Origin, com a chave.
fn do_agente(metodo: &str, url: &str, chave: &str, corpo: Option<Value>) -> Pedido {
    let mut p = pedido(metodo, url, None, corpo);
    p.cabecalhos.retain(|(k, _)| k != "Origin");
    p.cabecalhos.push(("Authorization".into(), format!("Bearer {chave}")));
    p
}

fn gerar(e: &Estado, cookie: &str) -> Value {
    let r = despachar(e, &pedido("POST", "/api/chave", Some(cookie), Some(json!({}))));
    assert_eq!(r.status, 201);
    json_de(&r)
}

#[test]
fn o_agente_do_colaborador_grava_com_a_chave_dele() {
    let e = estado();
    let colab = pessoa(&e, Papel::Colaborador, "w2", "", "Colab");
    let g = gerar(&e, &colab);
    let chave = g["chave"].as_str().unwrap().to_string();
    assert!(chave.starts_with("bpk_") && chave.len() == 47);
    assert_eq!(g["prefixo"].as_str(), Some(&chave[..8]));
    assert!(g["comandos"]["claude"].as_str().unwrap().contains("--env BLUEPRINT_URL=https://equipe.teste --env BLUEPRINT_CHAVE=bpk_"));
    assert!(g["comandos"]["codex"].as_str().unwrap().starts_with("codex mcp add blueprint-equipe"));
    let r = despachar(&e, &do_agente("POST", "/api/mcp/evento", &chave, Some(json!({"tipo": "progresso", "workspaceId": "w2", "blueprintId": "p9", "texto": "pelo agente"}))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    let fila = json_de(&despachar(&e, &do_agente("GET", "/api/mcp/fila", &chave, None)));
    assert_eq!(fila["itens"][0]["autor"]["nome"], "Colab");
    let vista = json_de(&despachar(&e, &pedido("GET", "/api/chave", Some(&colab), None)));
    assert_eq!(vista["chave"]["prefixo"].as_str(), Some(&chave[..8]));
    assert!(!vista.to_string().contains(&chave), "a chave só aparece ao ser gerada");
    let em_claro: i64 = e.dados.lock().unwrap().conexao().query_row("SELECT count(*) FROM chaves_mcp WHERE hash = ?1", [&chave], |l| l.get(0)).unwrap();
    assert_eq!(em_claro, 0);
}

#[test]
fn a_chave_do_supervisor_so_le() {
    let e = estado();
    let sup = pessoa(&e, Papel::Supervisor, "w2", "", "Sup");
    let chave = gerar(&e, &sup)["chave"].as_str().unwrap().to_string();
    assert_eq!(despachar(&e, &do_agente("GET", "/api/mcp/blueprint?workspace=w2&blueprint=p9", &chave, None)).status, 200);
    assert_eq!(despachar(&e, &do_agente("POST", "/api/mcp/evento", &chave, Some(json!({"tipo": "progresso", "workspaceId": "w2", "blueprintId": "p9", "texto": "x"})))).status, 403);
}

#[test]
fn chave_trocada_revogada_ou_inventada_leva_401() {
    let e = estado();
    let colab = pessoa(&e, Papel::Colaborador, "w2", "", "Colab");
    let primeira = gerar(&e, &colab)["chave"].as_str().unwrap().to_string();
    let segunda = gerar(&e, &colab)["chave"].as_str().unwrap().to_string();
    let codigo = |chave: &str| {
        let r = despachar(&e, &do_agente("GET", "/api/mcp/blueprints", chave, None));
        (r.status, json_de(&r)["codigo"].as_str().unwrap_or("").to_string())
    };
    assert_eq!(codigo(&primeira), (401, "chave-invalida".into()), "gerar outra troca a anterior");
    assert_eq!(codigo(&segunda).0, 200);
    assert_eq!(codigo("bpk_inventada").0, 401);
    assert_eq!(despachar(&e, &pedido("POST", "/api/chave/revogar", Some(&colab), Some(json!({})))).status, 200);
    assert_eq!(codigo(&segunda), (401, "chave-invalida".into()));
    // a chave só vale no MCP: nas outras rotas o navegador usa o cookie
    let r = despachar(&e, &do_agente("GET", "/api/dados", &segunda, None));
    assert_eq!(r.status, 401);
    let tipos: i64 = e.dados.lock().unwrap().conexao().query_row("SELECT count(*) FROM atividade WHERE tipo IN ('chave-criada', 'chave-revogada')", [], |l| l.get(0)).unwrap();
    assert_eq!(tipos, 3);
}

#[test]
fn visitante_nao_tem_chave() {
    let e = estado();
    let v = pessoa(&e, Papel::Visitante, "w2", "p10", "");
    assert_eq!(despachar(&e, &pedido("POST", "/api/chave", Some(&v), Some(json!({})))).status, 403);
    assert_eq!(despachar(&e, &pedido("GET", "/api/chave", Some(&v), None)).status, 403);
}

#[test]
fn le_a_chave_so_do_cabecalho_bearer() {
    use crate::acesso::chaves::do_cabecalho;
    let ok = format!("bpk_{}", "a".repeat(43));
    assert_eq!(do_cabecalho(&format!("Bearer {ok}")), Some(ok.as_str()));
    assert_eq!(do_cabecalho(&ok), None);
    assert_eq!(do_cabecalho("Bearer bpk_curta"), None);
    assert_eq!(do_cabecalho(&format!("Basic {ok}")), None);
}
