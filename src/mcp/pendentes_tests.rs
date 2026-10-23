//! Testes dos pendentes: o que o agente registra com o app fechado entra na fila, na ordem, quando o servidor abre.
use super::hooks_tests::{dados, mcp};
use super::pendentes::{guardar, pasta_do_banco, reenviar_todos, Resumo};
use serde_json::json;
use std::path::PathBuf;

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-pendentes-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn evento(tipo: &str, texto: &str, plano: &str) -> serde_json::Value {
    json!({"tipo": tipo, "texto": texto, "workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "planoId": plano})
}

#[test]
fn a_pasta_fica_ao_lado_do_banco() {
    assert_eq!(pasta_do_banco(std::path::Path::new("C:/dados/blueprint.db")), PathBuf::from("C:/dados/pendentes"));
    assert_eq!(pasta_do_banco(std::path::Path::new("blueprint.db")), PathBuf::from("pendentes"));
}

#[test]
fn guarda_em_ordem_e_reenvia_a_fila_de_quando_o_app_estava_fechado() {
    let p = pasta("ordem");
    assert_eq!(guardar(&p, "evento", &evento("progresso", "começando a tela", "t2")).unwrap(), 1);
    assert_eq!(guardar(&p, "evento", &evento("concluir", "cargo test: 3 passaram", "t2")).unwrap(), 2);
    let (m, d) = (mcp(), dados());
    assert_eq!(reenviar_todos(&m, &d, &p), Resumo { reenviados: 2, recusados: 0 });
    let itens = m.itens();
    assert_eq!(itens.iter().map(|i| i["tipo"].as_str().unwrap()).collect::<Vec<_>>(), ["progresso", "concluir"]);
    assert_eq!(itens[1]["texto"], "cargo test: 3 passaram");
    assert_eq!(itens[1]["plano"], "Criar tela"); // validado contra os dados só no reenvio
    assert!(std::fs::read_dir(&p).unwrap().next().is_none(), "o que foi aceito sai da pasta");
    assert_eq!(reenviar_todos(&m, &d, &p), Resumo::default(), "uma segunda abertura não reenvia nada");
}

#[test]
fn o_que_o_servidor_recusa_vai_para_recusados_com_o_motivo() {
    let p = pasta("recusa");
    guardar(&p, "evento", &evento("concluir", "feito", "plano-que-nao-existe")).unwrap();
    guardar(&p, "evento", &evento("progresso", "continua", "t2")).unwrap();
    let (m, d) = (mcp(), dados());
    assert_eq!(reenviar_todos(&m, &d, &p), Resumo { reenviados: 1, recusados: 1 });
    assert_eq!(m.itens().len(), 1, "um registro ruim não impede os seguintes");
    let recusados: Vec<_> = std::fs::read_dir(p.join("recusados")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(recusados.len(), 2); // o registro e o .txt com o motivo
    let motivo = recusados.iter().find(|n| n.ends_with(".txt")).unwrap();
    assert!(std::fs::read_to_string(p.join("recusados").join(motivo)).unwrap().contains("não encontrada"));
}

#[test]
fn so_guarda_as_rotas_que_nao_dependem_dos_dados_ao_vivo() {
    let p = pasta("rotas");
    assert!(guardar(&p, "iniciar", &json!({})).is_err());
    assert!(guardar(&p, "../fora", &json!({})).is_err());
    assert!(!p.exists(), "recusar não cria nem a pasta");
}

#[test]
fn arquivo_ilegivel_nao_trava_a_abertura() {
    let p = pasta("lixo");
    std::fs::create_dir_all(&p).unwrap();
    std::fs::write(p.join("00000000000000000001-1-0.json"), "isto não é json").unwrap();
    let (m, d) = (mcp(), dados());
    assert_eq!(reenviar_todos(&m, &d, &p), Resumo { reenviados: 0, recusados: 1 });
}
