//! Do convite à sessão: conferir, entrar com nome, cookie, segredo nunca na resposta, convite inválido, limite de tentativas, link do dono.
use super::*;
use crate::acesso::convites::{link_dono, LinkDono};

fn texto(r: &Resposta) -> String {
    format!("{:?} {}", r.cabecalhos, String::from_utf8_lossy(r.bytes()))
}

#[test]
fn colaborador_entra_pelo_link_com_nome_e_cookie() {
    let e = estado();
    let segredo = convite(&e, Papel::Colaborador, "w2", "", Some(1));
    let conf = despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": segredo}))));
    assert_eq!(conf.status, 200);
    assert_eq!(json_de(&conf), json!({"papel": "colaborador", "workspace": {"id": "w2", "nome": "Outro"}, "blueprint": null, "abre": "folha", "pedeNome": true}));
    let sem_nome = despachar(&e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo}))));
    assert_eq!((sem_nome.status, json_de(&sem_nome)["codigo"].as_str()), (400, Some("falta-nome")));
    let comprido = despachar(&e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo, "nome": "a".repeat(61)}))));
    assert_eq!(json_de(&comprido)["params"]["max"], 60);
    let r = despachar(&e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo, "nome": " Bruno "}))));
    assert_eq!(r.status, 200);
    assert_eq!(json_de(&r), json!({"ok": true, "destino": "/#/c/w2"}));
    let set = r.cabecalho("set-cookie").unwrap();
    for parte in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=2592000", "Secure"] {
        assert!(set.contains(parte), "{set}");
    }
    assert!(!texto(&r).contains(&segredo) && !texto(&conf).contains(&segredo), "o segredo do convite nunca volta");
    let cookie = cookie_de(&r);
    let sessao = json_de(&despachar(&e, &pedido("GET", "/api/sessao", Some(&cookie), None)));
    assert_eq!(sessao["membro"]["nome"], "Bruno");
    assert_eq!(sessao["acessos"], json!([{"workspaceId": "w2", "blueprintId": "", "papel": "colaborador", "abre": "folha"}]));
    // banco: só o hash do convite e do cookie
    let d = e.dados.lock().unwrap();
    let guardado: i64 = d.conexao().query_row("SELECT count(*) FROM convites WHERE hash = ?1", [crate::acesso::segredo::hash(&segredo)], |l| l.get(0)).unwrap();
    let em_claro: i64 = d.conexao().query_row("SELECT count(*) FROM sessoes WHERE hash = ?1", [&cookie], |l| l.get(0)).unwrap();
    assert_eq!((guardado, em_claro), (1, 0));
    let entrou: i64 = d.conexao().query_row("SELECT count(*) FROM atividade WHERE tipo = 'entrou' AND nome = 'Bruno'", [], |l| l.get(0)).unwrap();
    assert_eq!(entrou, 1);
}

#[test]
fn convite_usado_vencido_revogado_ou_adulterado_responde_igual() {
    let e = estado();
    let usado = convite(&e, Papel::Colaborador, "w2", "", Some(1));
    entrar(&e, &usado, "Ana");
    let revogado = convite(&e, Papel::Supervisor, "w2", "", Some(5));
    let id: String = e.dados.lock().unwrap().conexao().query_row("SELECT id FROM convites WHERE papel = 'supervisor'", [], |l| l.get(0)).unwrap();
    e.dados.lock().unwrap().transacao(|t| crate::dados::convites::revogar(t, &id, "2026-01-01T00:00:00.000Z")).unwrap();
    let mut adulterado = convite(&e, Papel::Colaborador, "w2", "", None);
    adulterado.replace_range(0..1, if adulterado.starts_with('A') { "B" } else { "A" });
    let mut respostas = Vec::new();
    for s in [usado.as_str(), revogado.as_str(), adulterado.as_str(), "curto"] {
        for rota in ["/api/convite/conferir", "/api/convite/entrar"] {
            let r = despachar(&e, &pedido("POST", rota, None, Some(json!({"segredo": s, "nome": "X"}))));
            assert_eq!(r.status, 404, "{rota}");
            assert!(!texto(&r).contains(s) || s == "curto");
            respostas.push(json_de(&r));
        }
    }
    assert!(respostas.windows(2).all(|w| w[0] == w[1]), "a mesma resposta em todos os casos: {respostas:?}");
    assert_eq!(respostas[0]["codigo"], "convite-invalido");
}

#[test]
fn passou_do_limite_de_tentativas_leva_429() {
    let e = estado();
    let bom = convite(&e, Papel::Colaborador, "w2", "", None);
    for _ in 0..crate::acesso::limite::POR_IP {
        assert_eq!(despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": "x".repeat(43)})))).status, 404);
    }
    let r = despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": bom}))));
    assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (429, Some("muitas-tentativas")));
    assert!(json_de(&r)["params"]["segundos"].as_i64().unwrap() > 0);
    // o limite é por IP do cliente (X-Forwarded-For), não pelo do proxy
    let mut outro = pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": bom})));
    outro.cabecalhos.retain(|(k, _)| k != "X-Forwarded-For");
    outro.cabecalhos.push(("X-Forwarded-For".into(), "198.51.100.1, 203.0.113.99".into()));
    assert_eq!(despachar(&e, &outro).status, 200);
}

#[test]
fn visitante_entra_sem_nome_e_cai_quando_o_convite_e_revogado() {
    let e = estado();
    let dono = pessoa(&e, Papel::Dono, "w2", "", "Dona");
    let criado = despachar(&e, &pedido("POST", "/api/convites", Some(&dono), Some(json!({"workspaceId": "w2", "blueprintId": "p10", "papel": "visitante", "abre": "apresentacao"}))));
    assert_eq!(criado.status, 201, "{}", texto(&criado));
    let c = json_de(&criado);
    assert_eq!((c["convite"]["usosMax"].clone(), c["convite"]["abre"].as_str()), (Value::Null, Some("apresentacao")));
    let link = c["link"].as_str().unwrap();
    let segredo = link.strip_prefix("https://equipe.teste/convite#").expect("o segredo vai no fragmento");
    assert_eq!(json_de(&despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": segredo})))))["pedeNome"], false);
    let r = despachar(&e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo}))));
    assert_eq!(json_de(&r)["destino"], "/#/p/p10");
    let visitante = cookie_de(&r);
    assert!(r.cabecalho("set-cookie").unwrap().contains("Max-Age=604800"));
    let s = json_de(&despachar(&e, &pedido("GET", "/api/sessao", Some(&visitante), None)));
    assert_eq!((s["visitante"].as_bool(), s["membro"].clone()), (Some(true), Value::Null));
    let listados = json_de(&despachar(&e, &pedido("GET", "/api/convites?workspace=w2", Some(&dono), None)));
    assert!(!listados.to_string().contains(segredo), "a lista nunca traz o segredo");
    let id = c["convite"]["id"].as_str().unwrap();
    assert_eq!(despachar(&e, &pedido("POST", "/api/convites/revogar", Some(&dono), Some(json!({"id": id})))).status, 200);
    assert_eq!(despachar(&e, &pedido("GET", "/api/dados", Some(&visitante), None)).status, 401, "revogar derruba o visitante");
    assert!(!e.acesso.sessao_ativa(&Contexto::visitante(id, "w2", "p10", "folha")), "e o SSE dele fecha");
}

#[test]
fn trocar_o_link_revoga_o_antigo_e_entrega_outro() {
    let e = estado();
    let dono = pessoa(&e, Papel::Dono, "w2", "", "Dona");
    let c = json_de(&despachar(&e, &pedido("POST", "/api/convites", Some(&dono), Some(json!({"workspaceId": "w2", "papel": "colaborador", "usosMax": 3})))));
    let antigo = c["link"].as_str().unwrap().rsplit('#').next().unwrap().to_string();
    let t = despachar(&e, &pedido("POST", "/api/convites/trocar", Some(&dono), Some(json!({"id": c["convite"]["id"]}))));
    assert_eq!(t.status, 201);
    let novo = json_de(&t)["link"].as_str().unwrap().rsplit('#').next().unwrap().to_string();
    assert_ne!(antigo, novo);
    assert_eq!(json_de(&t)["convite"]["usosMax"], 3);
    assert_eq!(despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": antigo})))).status, 404);
    assert_eq!(despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": novo})))).status, 200);
}

#[test]
fn quem_ja_tem_sessao_soma_o_acesso_sem_novo_membro() {
    let e = estado();
    let ana = pessoa(&e, Papel::Supervisor, "w2", "", "Ana");
    let s = convite(&e, Papel::Colaborador, "sm", "p1", Some(1));
    let r = despachar(&e, &pedido("POST", "/api/convite/entrar", Some(&ana), Some(json!({"segredo": s}))));
    assert_eq!(r.status, 200);
    let sessao = json_de(&despachar(&e, &pedido("GET", "/api/sessao", Some(&cookie_de(&r)), None)));
    assert_eq!((sessao["membro"]["nome"].as_str(), sessao["acessos"].as_array().unwrap().len()), (Some("Ana"), 2));
    let membros: i64 = e.dados.lock().unwrap().conexao().query_row("SELECT count(*) FROM membros", [], |l| l.get(0)).unwrap();
    assert_eq!(membros, 1);
}

#[test]
fn link_do_dono_impresso_uma_vez_e_trocado_com_novo_link_dono() {
    let e = estado();
    let mut d = e.dados.lock().unwrap();
    let LinkDono::Novo(primeiro) = link_dono(&mut d, URL, false, crate::tempo::agora_ms()).unwrap() else { panic!("sem link") };
    assert!(primeiro.starts_with("https://equipe.teste/convite#"));
    assert_eq!(link_dono(&mut d, URL, false, crate::tempo::agora_ms()).unwrap(), LinkDono::JaImpresso, "a segunda execução não imprime");
    let LinkDono::Novo(segundo) = link_dono(&mut d, URL, true, crate::tempo::agora_ms()).unwrap() else { panic!("sem link novo") };
    let segredo = |l: &str| l.rsplit('#').next().unwrap().to_string();
    let em_claro: i64 = d.conexao().query_row("SELECT count(*) FROM convites WHERE hash = ?1 OR hash = ?2", [segredo(&primeiro), segredo(&segundo)], |l| l.get(0)).unwrap();
    assert_eq!(em_claro, 0, "o banco só guarda o hash");
    drop(d);
    assert_eq!(despachar(&e, &pedido("POST", "/api/convite/conferir", None, Some(json!({"segredo": segredo(&primeiro)})))).status, 404, "--novo-link-dono invalida o anterior");
    let r = despachar(&e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo(&segundo), "nome": "Dona"}))));
    assert_eq!(json_de(&r)["destino"], "/");
    let mut d = e.dados.lock().unwrap();
    assert_eq!(link_dono(&mut d, URL, false, crate::tempo::agora_ms()).unwrap(), LinkDono::TemDono);
    let impressos: i64 = d.conexao().query_row("SELECT count(*) FROM atividade WHERE tipo = 'link-dono-impresso'", [], |l| l.get(0)).unwrap();
    assert_eq!(impressos, 2);
}
