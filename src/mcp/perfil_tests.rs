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
    assert_eq!(Perfil::ler(Some("orquestrador")), Ok(Perfil::Orquestrador));
    let erro = com(Idioma::En, || Perfil::ler(Some("worker"))).unwrap_err();
    assert!(erro.contains("\"worker\"") && erro.contains("executor, planejador, orquestrador or completo"), "{erro}");
}

#[test]
fn o_completo_lista_tudo_e_os_parciais_so_o_que_precisam() {
    let todas = nomes(Perfil::Completo);
    let (executor, planejador, orquestrador) = (nomes(Perfil::Executor), nomes(Perfil::Planejador), nomes(Perfil::Orquestrador));
    for n in executor.iter().chain(&planejador).chain(&orquestrador) {
        assert!(todas.contains(n), "{n} existe nas ferramentas");
    }
    // nenhuma ferramenta fica sem perfil parcial, e cada perfil mantém o fluxo dele inteiro
    assert!(todas.iter().all(|n| executor.contains(n) || planejador.contains(n) || orquestrador.contains(n)), "toda ferramenta está em algum perfil parcial");
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

#[test]
fn perfil_orquestrador_lista_exata() {
    // A mesma lista nos dois sistemas: no Windows as oito do Planejar somem do `tools/list`, mas o perfil continua sendo este.
    const LISTA: [&str; 11] = ["listar_blueprints", "guia_de_planejamento", "ler_blueprint", "ler_plano", "ler_respostas", "ler_workspace", "abrir_plano", "escrever_acao", "fechar_plano", "salvar_plano", "falar_com_voce"];
    let ferramentas: Value = serde_json::from_str(include_str!("ferramentas.json")).unwrap();
    let dele: Vec<String> = Perfil::Orquestrador.filtrar(ferramentas).as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(dele, LISTA);
    for n in ["marcar_concluido", "marcar_nao_concluido", "marcar_checkpoint_feito", "reportar_bloqueio", "registrar_progresso", "propor_blueprint", "vincular_pasta"] {
        assert!(!Perfil::Orquestrador.expoe(n), "o orquestrador não tem {n}");
    }
    assert_eq!(Perfil::Orquestrador.nome(), "orquestrador");
}

#[test]
fn executor_e_planejador_ganham_as_do_plano() {
    for n in ["ler_plano", "ler_respostas"] {
        assert!(Perfil::Executor.expoe(n) && Perfil::Planejador.expoe(n), "{n}");
    }
    for n in ["abrir_plano", "escrever_acao", "fechar_plano", "salvar_plano"] {
        assert!(Perfil::Planejador.expoe(n) && !Perfil::Executor.expoe(n), "{n}");
    }
    assert!(!Perfil::Executor.expoe("falar_com_voce") && !Perfil::Planejador.expoe("falar_com_voce"));
}
