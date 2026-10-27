//! O tópico `apresentar` do guia: o app monta as páginas; o agente só prepara o texto, e nunca promete nem expõe dado pessoal.
use super::stdio::tratar;
use serde_json::json;

#[test]
fn o_topico_apresentar_so_prepara_o_texto_e_protege_a_privacidade() {
    let r = tratar(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "guia_de_planejamento", "arguments": {"topico": "apresentar"}}})).unwrap();
    let texto = r["result"]["content"][0]["text"].as_str().unwrap();
    for trecho in ["Progress by project", "Full plan", "Do not invent a third type", "Never promise what the blueprint does not say", "anexar_prototipo", "anatomy skill", "No emoji", "Never put personal data", "Do not export or print for them"] {
        assert!(texto.contains(trecho), "falta no tópico: {trecho}");
    }
    let indice = tratar(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "guia_de_planejamento", "arguments": {}}})).unwrap();
    assert!(indice["result"]["content"][0]["text"].as_str().unwrap().contains("`apresentar`"), "o índice do guia aponta para o tópico");
}
