//! `ler_blueprint` enxuto: blueprint grande resume concluídas e planejadas distantes, pequeno não muda, `tudo` é o de antes, `projeto` filtra.
use super::{estrutura_blueprint, estrutura_lida};
use serde_json::{json, Value};

fn tamanho(v: &Value) -> usize {
    v.to_string().chars().count()
}

/// ~100 ações como as do blueprint «App»: 62 concluídas, 3 em andamento, 35 planejadas; 3 a 6 planos (texto e descrição realistas) e 1 a 2 checkpoints.
fn grande() -> Value {
    let acoes: Vec<Value> = (0..100)
        .map(|i| {
            let (status, mes) = if i < 62 { ("concluido", 1 + i / 12) } else if i < 65 { ("andamento", 8) } else { ("planejado", 9 + (i - 65) / 12) };
            let planos: Vec<Value> = (0..3 + i % 4)
                .map(|k| json!({"id": format!("t{i}-{k}"), "texto": format!("Implementar a parte {k} da ação {i}: rota, validação e teste de ponta a ponta no módulo"), "horas": 2,
                    "descricao": "Escopo: ajustar o módulo e a interface. Resultado: o fluxo funciona sem regressão. Validação: cargo test passa e a tela abre sem erro no console. ".repeat(3),
                    "feito": status == "concluido" || (status == "andamento" && k == 0)}))
                .collect();
            let checkpoints: Vec<Value> = (0..1 + i % 2).map(|k| json!({"id": format!("c{i}-{k}"), "nome": format!("Usuário confere a entrega {k} da ação {i}"), "data": format!("2026-{:02}-20", mes), "feito": status == "concluido"})).collect();
            let mut a = json!({"id": format!("f{i}"), "nome": format!("Ação {i}: reorganizar o módulo e cobrir com testes"), "inicio": format!("2026-{mes:02}-01"), "fim": format!("2026-{mes:02}-28"),
                "status": status, "obs": "Instruções longas da ação, que a estrutura nunca devolve.", "planos": planos, "checkpoints": checkpoints});
            if i % 10 != 9 {
                a["projetoId"] = json!(format!("pr{}", i % 4));
            }
            a
        })
        .collect();
    let projetos: Vec<Value> = ["Interface", "Servidor", "Instalador", "Documentação"].iter().enumerate().map(|(i, n)| json!({"id": format!("pr{i}"), "nome": n})).collect();
    json!({"id": "app", "nome": "App", "descricao": "d", "obs": "", "projetos": projetos, "acoes": acoes})
}

fn pequeno() -> Value {
    json!({"id": "p", "nome": "Pequeno", "descricao": "d", "obs": "", "acoes": [
        {"id": "a1", "nome": "Feita", "inicio": "2026-10-01", "fim": "2026-10-10", "status": "concluido", "obs": "", "planos": [{"id": "t1", "texto": "x", "feito": true}], "checkpoints": []},
        {"id": "a2", "nome": "Depois", "inicio": "2026-12-01", "fim": "2026-12-10", "status": "planejado", "obs": "", "planos": [{"id": "t2", "texto": "y"}], "checkpoints": [{"id": "c1", "nome": "ok", "data": "2026-12-09"}]}]})
}

fn lida(p: &Value, incluir: &str, projeto: &str) -> Value {
    estrutura_lida(p, incluir, projeto).unwrap()
}

#[test]
fn cem_acoes_ficam_bem_abaixo_de_25_mil_caracteres() {
    let p = grande();
    let antes = tamanho(&estrutura_blueprint(&p));
    let depois = tamanho(&lida(&p, "ativas", ""));
    eprintln!("100 ações: antes {antes} caracteres, depois {depois}");
    assert!(antes > 50_000, "o caso reproduz o problema: {antes}");
    assert!(depois < 25_000, "{depois}");
}

#[test]
fn o_que_importa_continua_completo_e_o_resto_vira_linha() {
    let e = lida(&grande(), "ativas", "");
    let acoes = e["acoes"].as_array().unwrap();
    assert_eq!(acoes.len(), 100, "nenhuma ação some, a ordem é a mesma");
    let por_id = |id: &str| acoes.iter().find(|a| a["id"] == id).unwrap();
    for id in ["f62", "f63", "f64"] {
        let a = por_id(id);
        assert!(a["planos"].as_array().unwrap().iter().all(|t| t["id"].is_string() && t["texto"].is_string() && t["horas"] == 2) && a["checkpoints"].is_array(), "em andamento: completa");
    }
    assert!(por_id("f65")["planos"].is_array() && por_id("f68")["planos"].is_array() && por_id("f69").get("planos").is_none(), "as próximas planejadas ficam completas");
    let feita = por_id("f3");
    assert_eq!(feita, &json!({"id": "f3", "nome": "Ação 3: reorganizar o módulo e cobrir com testes", "fim": "2026-01-28", "status": "concluido", "projeto": "Documentação"}), "concluída: uma linha com o id");
    let distante = por_id("f99");
    assert!(distante.get("planos").is_none() && distante["id"] == "f99" && distante["inicio"].is_string() && distante["nPlanos"] == 6 && distante["nCheckpoints"] == 2, "{distante}");
    assert_eq!(e["resumidas"]["concluidas"], 62);
    assert_eq!(e["resumidas"]["planejadas"], 31);
    let como = e["resumidas"]["como"].as_str().unwrap();
    assert!(como.contains("acao_id") && como.contains("incluir") && como.contains("projeto"), "diz como pedir o resto");
}

#[test]
fn blueprint_pequeno_sai_exatamente_como_antes() {
    let p = pequeno();
    assert_eq!(lida(&p, "ativas", ""), estrutura_blueprint(&p));
    assert_eq!(lida(&p, "", ""), estrutura_blueprint(&p));
    assert!(lida(&p, "ativas", "").get("resumidas").is_none());
}

#[test]
fn incluir_tudo_e_igual_ao_antigo() {
    let p = grande();
    assert_eq!(lida(&p, "tudo", ""), estrutura_blueprint(&p));
}

#[test]
fn projeto_filtra_por_id_nome_ou_geral() {
    let p = grande();
    let por_id = lida(&p, "tudo", "pr1");
    assert_eq!(por_id, lida(&p, "tudo", " servidor "), "o nome casa sem ligar para caixa");
    let acoes = por_id["acoes"].as_array().unwrap();
    assert!(!acoes.is_empty() && acoes.iter().all(|a| a["projeto"] == "Servidor"));
    assert_eq!(por_id["filtro"], json!({"projeto": "Servidor", "acoes": acoes.len(), "deTotal": 100}));
    assert!(lida(&p, "tudo", "Geral")["acoes"].as_array().unwrap().iter().all(|a| a.get("projeto").is_none()));
    let enxuto = lida(&p, "ativas", "pr3");
    assert!(enxuto["acoes"].as_array().unwrap().iter().all(|a| a["projeto"] == "Documentação") && enxuto["filtro"]["acoes"] == 20, "o filtro vale também no padrão");
    assert!(tamanho(&enxuto) < tamanho(&lida(&p, "tudo", "pr3")) && tamanho(&enxuto) < LIMITE_ESPERADO, "e o projeto grande também é resumido");
    let erro = estrutura_lida(&p, "tudo", "nao-existe").unwrap_err();
    assert!(erro.contains("Unknown project") && erro.contains("Interface"), "{erro}");
}

#[test]
fn incluir_invalido_e_recusado() {
    assert!(estrutura_lida(&pequeno(), "quase", "").unwrap_err().contains("incluir must be"));
}

const LIMITE_ESPERADO: usize = super::enxuta::LIMITE;
