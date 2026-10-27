use super::perfil::Perfil;
use super::stdio::tratar_com;
use crate::idioma::{com, Idioma};
use serde_json::{json, Value};

fn rpc(perfil: Perfil, metodo: &str, params: Value) -> Value {
    tratar_com(&json!({"jsonrpc": "2.0", "id": 1, "method": metodo, "params": params}), None, perfil).unwrap()
}

fn nomes(perfil: Perfil) -> Vec<String> {
    rpc(perfil, "tools/list", json!({}))["result"]["tools"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap().to_string()).collect()
}

#[test]
fn le_o_valor_da_variavel_e_recusa_o_desconhecido() {
    assert_eq!(Perfil::ler(None), Ok(Perfil::Completo));
    assert_eq!(Perfil::ler(Some(" ")), Ok(Perfil::Completo));
    assert_eq!(Perfil::ler(Some("executor")), Ok(Perfil::Executor));
    assert_eq!(Perfil::ler(Some("planejador")), Ok(Perfil::Planejador));
    let erro = com(Idioma::En, || Perfil::ler(Some("worker"))).unwrap_err();
    assert!(erro.contains("\"worker\"") && erro.contains("executor, planejador or completo"), "{erro}");
}

#[test]
fn o_completo_lista_tudo_e_os_parciais_so_o_que_precisam() {
    let todas = nomes(Perfil::Completo);
    let (executor, planejador) = (nomes(Perfil::Executor), nomes(Perfil::Planejador));
    for n in executor.iter().chain(&planejador) {
        assert!(todas.contains(n), "{n} existe nas ferramentas");
    }
    // nenhuma ferramenta fica sem perfil parcial, e cada perfil mantém o fluxo dele inteiro
    assert!(todas.iter().all(|n| executor.contains(n) || planejador.contains(n)), "toda ferramenta está em algum perfil parcial");
    for n in ["ler_blueprint", "registrar_progresso", "marcar_concluido", "marcar_checkpoint_feito", "registrar_trabalho", "reportar_bloqueio"] {
        assert!(executor.contains(&n.to_string()), "executor tem {n}");
    }
    for n in ["ler_blueprint", "guia_de_planejamento", "propor_blueprint", "responder_conversa"] {
        assert!(planejador.contains(&n.to_string()), "planejador tem {n}");
    }
    assert!(!executor.contains(&"propor_blueprint".to_string()) && !planejador.contains(&"marcar_concluido".to_string()));
    assert!(executor.len() < todas.len() && planejador.len() < todas.len());
}

#[test]
fn chamar_uma_ferramenta_fora_do_perfil_e_recusado() {
    let r = rpc(Perfil::Executor, "tools/call", json!({"name": "propor_blueprint", "arguments": {}}));
    assert_eq!(r["error"]["code"], -32602);
    assert!(r["error"]["message"].as_str().unwrap().contains("not in the executor profile"), "{r}");
    let r = rpc(Perfil::Planejador, "tools/call", json!({"name": "marcar_concluido", "arguments": {}}));
    assert!(r["error"]["message"].as_str().unwrap().contains("planejador"), "{r}");
}

#[test]
fn as_instrucoes_nao_mudam_com_o_perfil() {
    let instr = |p| rpc(p, "initialize", json!({}))["result"]["instructions"].clone();
    assert_eq!(instr(Perfil::Executor), instr(Perfil::Completo));
    assert_eq!(instr(Perfil::Planejador), instr(Perfil::Completo));
}
