//! Projetos pelo MCP: a ferramenta propor_blueprint conhece `blueprint.projetos` e `projeto`, e a proposta chega à fila com o projeto resolvido.
use super::{acoes, stdio::tratar, Mcp};
use serde_json::{json, Value};

fn dados() -> Value {
    json!({"blueprints": [{"id": "p1", "nome": "Portal", "descricao": "", "projetos": [{"id": "pr1", "nome": "Interface"}], "acoes": [
        {"id": "a1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-05", "status": "planejado", "planos": [], "checkpoints": []}]}]})
}

#[test]
fn a_ferramenta_conhece_projetos_e_o_guia_explica() {
    let r = tratar(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})).unwrap();
    let propor = r["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "propor_blueprint").unwrap();
    let props = &propor["inputSchema"]["properties"];
    assert!(props["blueprint"]["properties"].get("projetos").is_some(), "blueprint.projetos");
    assert!(props["acoes"]["items"]["properties"].get("projeto").is_some(), "acoes[].projeto");
    for topico in ["planejar", "editar"] {
        let g = tratar(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "guia_de_planejamento", "arguments": {"topico": topico}}})).unwrap();
        assert!(g["result"]["content"][0]["text"].as_str().unwrap().contains("blueprint.projetos"), "{topico}");
    }
}

#[test]
fn a_proposta_com_projeto_chega_resolvida_e_o_projeto_desconhecido_volta_422() {
    let m = Mcp::novo("127.0.0.1", false, "blueprint.exe", "http://127.0.0.1:3000", std::env::temp_dir());
    let corpo = json!({"workspaceId": "sm", "blueprintId": "p1", "blueprint": {"projetos": ["Dados"]}, "acoes": [
        {"id": "a1", "projeto": "interface"}, {"nome": "Nova", "inicio": "2026-12-01", "fim": "2026-12-02", "projeto": "Dados", "planos": [{"texto": "Criar tabela", "descricao": "Escopo: x. Resultado: y. Validação: cargo test passa."}], "checkpoints": []}]});
    let (status, _) = acoes::proposta(&m, &dados(), &corpo).unwrap();
    assert_eq!(status, 200);
    let proposta = &m.itens()[0]["proposta"];
    assert_eq!(proposta["projetos"], json!([{"id": "", "nome": "Dados"}]));
    assert_eq!(proposta["acoes"][0]["projeto"], json!({"id": "pr1", "nome": "Interface"}));
    assert_eq!(proposta["acoes"][1]["projeto"], json!({"id": "", "nome": "Dados"}));

    let ruim = json!({"workspaceId": "sm", "blueprintId": "p1", "acoes": [{"id": "a1", "projeto": "Mobile"}]});
    let (status, resposta) = acoes::proposta(&m, &dados(), &ruim).unwrap();
    assert_eq!(status, 422);
    assert!(resposta["erros"][0].as_str().unwrap().contains("o projeto \"Mobile\" não existe"));
}
