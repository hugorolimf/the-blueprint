use super::*;
use serde_json::json;

fn exemplo() -> Value {
    json!({"blueprints": [{"id": "b1", "nome": "App de Entregas", "descricao": "Piloto", "acoes": [
        {"id": "a1", "nome": "Protótipo clicável", "inicio": "2026-10-05", "fim": "2026-10-12", "status": "concluido", "obs": "Objetivo: validar.",
         "planos": [
            {"id": "p1", "texto": "Desenhar telas", "feito": true, "agente": {"comentarios": [{"tipo": "concluir", "texto": "Revisado com a operação.\nOk."}]}},
            {"id": "p2", "texto": "Montar fluxo", "feito": false}],
         "checkpoints": [{"id": "c1", "nome": "Teste com um entregador", "data": "2026-10-12", "feito": false}]}]}]})
}

#[test]
fn gera_indice_e_um_arquivo_por_acao() {
    let arquivos = todos(&exemplo());
    let nomes: Vec<&str> = arquivos.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(nomes, ["app-de-entregas/README.md", "app-de-entregas/01-protótipo-clicável.md"]);
    assert!(arquivos[0].1.contains("[Protótipo clicável](01-protótipo-clicável.md) (done, 1/2 plans)"));
}

#[test]
fn acao_mostra_planos_checkpoints_e_so_a_evidencia_do_que_esta_feito() {
    let texto = acao(&exemplo()["blueprints"][0]["acoes"][0]);
    assert!(texto.contains("- [x] Desenhar telas\n  - evidence: Revisado com a operação. Ok."));
    assert!(texto.contains("- [ ] Montar fluxo\n"));
    assert!(texto.contains("- [ ] Teste com um entregador (2026-10-12)"));
    assert!(texto.contains("- dates: 2026-10-05 to 2026-10-12"));
}

#[test]
fn nome_vazio_ou_com_barras_nao_escapa_da_pasta() {
    assert_eq!(slug("../../x"), "x");
    assert_eq!(slug("///"), "sem-nome");
    assert!(slug(&"a".repeat(200)).len() <= 60);
}
