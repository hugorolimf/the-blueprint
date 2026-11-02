//! As oito ferramentas pela rota de verdade (`rota_estado`, o caminho de `POST /mcp`), e o que trava a lista: nenhuma ferramenta de gesto, nada disso por HTTP.
//! Passa de 300 linhas: a fatia dona (F41) não edita `mcp/mod.rs`, e todos dividem os mesmos auxiliares de rota real.
use super::*;
use crate::contexto::{Acesso, Contexto, Papel};
use crate::dados::planejar::{Lancamento, NovaExecucao};
use crate::http::tests::{estado, json_de, pedido};
use crate::http::Estado;
use crate::mcp::argumentos::com_execucao;
use crate::mcp::rotas::rota_estado;
use crate::mcp::stdio::{aviso_sem_consumidor, tratar_com, tratar_via};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};

static SEQ: AtomicU32 = AtomicU32::new(0);
type Chamadas = RefCell<Vec<(String, Value)>>;

struct Temp(PathBuf);
impl Temp {
    fn nova() -> Temp {
        let d = std::env::temp_dir().join(format!("bp-ferr-plano-{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&d).unwrap();
        Temp(d)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A via com o app de verdade por trás, guardando na ordem o que o `--mcp` pediu.
fn via_real<'a>(e: &'a Estado, chamadas: &'a Chamadas) -> impl Fn(&str, &str, Option<&str>) -> Result<(u16, String), String> + 'a {
    move |metodo, rota, corpo| {
        let valor: Option<Value> = corpo.map(|c| serde_json::from_str(c).unwrap());
        chamadas.borrow_mut().push((rota.to_string(), valor.clone().unwrap_or(Value::Null)));
        let r = rota_estado(e, &pedido(metodo, &format!("/api/mcp/{rota}"), &[], valor), &Contexto::local());
        Ok((r.status, String::from_utf8_lossy(r.bytes()).into_owned()))
    }
}

fn exemplo(nome: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plano/fixtures").join(nome).display().to_string()
}

fn nomes(lista: &Value) -> Vec<String> {
    lista["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect()
}

fn membro() -> Contexto {
    Contexto::membro("m1", "Ana", vec![Acesso::novo("w2", "*", Papel::Dono)])
}

fn por_post_mcp(e: &Estado, corpo: Value) -> Value {
    let r = crate::mcp::http::rota(e, &pedido("POST", "/mcp", &[], Some(corpo)), &membro());
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)
}

fn argumentos_das_oito() -> [(&'static str, Value); 8] {
    let (ws, bp) = ("w2", "p9");
    [
        ("abrir_plano", json!({"workspace_id": ws, "blueprint_id": bp})),
        ("escrever_acao", json!({"escrita_id": "e", "acao": {"id": "1"}})),
        ("fechar_plano", json!({"escrita_id": "e"})),
        ("salvar_plano", json!({"pasta": exemplo("biblioteca"), "workspace_id": ws, "blueprint_id": bp})),
        ("ler_plano", json!({"workspace_id": ws, "blueprint_id": bp})),
        ("ler_respostas", json!({"workspace_id": ws, "blueprint_id": bp})),
        ("ler_workspace", json!({"workspace_id": ws})),
        ("falar_com_voce", json!({"workspace_id": ws, "texto": "oi"})),
    ]
}

#[test]
fn nenhuma_ferramenta_de_gesto() {
    const PROIBIDOS: [&str; 13] = ["aprovar", "reprovar", "revisar", "colocar", "integrar", "iniciar", "marcar_plano", "ligar", "cancelar", "retomar", "limite", "comando", "descartar"];
    let lista = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}});
    for perfil in [Perfil::Completo, Perfil::Executor, Perfil::Planejador, Perfil::Orquestrador] {
        for n in nomes(&tratar_com(&lista, None, perfil).unwrap()) {
            assert!(!PROIBIDOS.iter().any(|p| n.contains(p)), "{n} em {perfil:?}");
        }
    }
    // E o processo do agente nem sabe o caminho: nem rota de gesto, nem o cabeçalho do token.
    for (arquivo, fonte) in [("ferr_plano", include_str!("ferr_plano.rs")), ("stdio", include_str!("stdio.rs")), ("argumentos", include_str!("argumentos.rs")), ("cliente", include_str!("cliente.rs")), ("cliente_curl", include_str!("cliente_curl.rs"))] {
        assert!(!fonte.contains("gesto/") && !fonte.to_lowercase().contains("blueprint-gesto"), "{arquivo}");
    }
}

#[test]
fn salvar_recusa_por_http() {
    let e = estado(false);
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let args = json!({"pasta": exemplo("biblioteca"), "workspace_id": "w2", "blueprint_id": "p9"});
    assert_eq!(chamar(Some(&via), "salvar_plano", &args).unwrap_err(), SO_LOCAL);
    let r = tratar_via(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "salvar_plano", "arguments": args}}), Some(&via)).unwrap();
    assert_eq!(r["error"]["code"], -32602);
    assert!(r["error"]["message"].as_str().unwrap().contains("local app"), "{r}");
    assert!(chamadas.borrow().is_empty(), "nenhuma rota foi chamada");
}

#[test]
fn mcp_http_nao_alcanca_plano() {
    let e = estado(false);
    e.mcp.ligar_janela(); // sem isso `sem-janela` esconderia o que está sendo testado
    for (nome, args) in argumentos_das_oito() {
        let r = por_post_mcp(&e, json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": nome, "arguments": args}}));
        assert_eq!(r["error"]["code"], -32602, "{nome}: {r}");
        assert!(r["error"]["message"].as_str().unwrap().contains("local app"), "{nome}: {r}");
        assert!(r.get("result").is_none(), "{nome}");
    }
    let d = e.dados.lock().unwrap();
    assert!(d.plano_aberta("p9").unwrap().is_none() && d.plano_pendente("p9").unwrap().is_none());
    assert!(d.recados_do("w2", 10).unwrap().is_empty());
    drop(d);
    // Mesmo se a ferramenta deixasse passar, a rota com contexto de membro não existe.
    let p = pedido("POST", "/api/mcp/plano/abrir", &[], Some(json!({"workspaceId": "w2", "blueprintId": "p9", "origem": "escrita"})));
    let r = rota_estado(&e, &p, &membro());
    assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (404, Some("so-local")));
}

#[test]
fn tools_list_http_nao_lista() {
    let e = estado(false);
    e.mcp.ligar_janela();
    let lista = nomes(&por_post_mcp(&e, json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})));
    for n in FERRAMENTAS {
        assert!(!lista.iter().any(|x| x == n), "{n} não aparece por HTTP");
    }
    assert!(lista.iter().any(|x| x == "propor_blueprint"), "lá o caminho continua sendo ela");
    let instrucoes = por_post_mcp(&e, json!({"jsonrpc": "2.0", "id": 2, "method": "initialize", "params": {}}));
    let instrucoes = instrucoes["result"]["instructions"].as_str().unwrap();
    assert!(instrucoes.contains("propor_blueprint") && !instrucoes.contains("abrir_plano"), "{instrucoes}");
}

#[test]
fn salvar_compoe_as_rotas() {
    let e = estado(false);
    e.mcp.ligar_janela();
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let texto = salvar(Some(&via), &json!({"pasta": exemplo("reforma"), "workspace_id": "w2", "blueprint_id": "p9"}), Perfil::Completo, None).unwrap();
    let feitas = chamadas.borrow();
    let rotas: Vec<&str> = feitas.iter().map(|(r, _)| r.as_str()).collect();
    let doc = PlanoDoc::de_json(&serde_json::from_str(&std::fs::read_to_string(Path::new(&exemplo("reforma")).join("plano.json")).unwrap()).unwrap()).unwrap();
    let n_acoes: usize = doc.projetos.iter().map(|p| p.acoes.len()).sum();
    assert_eq!(rotas[0], "plano/abrir");
    assert_eq!(rotas[1], "plano/figura", "a figura vai antes da ação que a cita");
    assert_eq!(rotas.iter().filter(|r| **r == "plano/escrever").count(), n_acoes);
    assert_eq!(rotas.last(), Some(&"plano/fechar"));
    assert!(rotas.iter().all(|r| r.starts_with("plano/")), "nenhum gesto: {rotas:?}");
    let abrir = &feitas[0].1;
    assert_eq!((abrir["origem"].as_str(), abrir["blueprintId"].as_str()), (Some("salvar"), Some("p9")));
    assert!(abrir.get("pasta").is_none(), "blueprint que existe não muda de pasta");
    assert_eq!(feitas[1].1["caminho"], "figuras/planta.svg");
    assert!(feitas[1].1["svg"].as_str().unwrap().contains("<svg"));
    assert!(texto.contains("waiting for the user to place them"), "{texto}");
    assert!(std::fs::read_dir(e.mcp.pasta_figuras()).unwrap().count() >= 1, "a figura foi para o disco");
    let mut d = e.dados.lock().unwrap();
    assert!(d.plano_pendente("p9").unwrap().is_some(), "as novas esperam o Colocar");
    let tudo = d.ler_tudo(&Contexto::local()).unwrap();
    assert_eq!(tudo["workspaces"][0]["blueprints"][0]["acoes"], json!([]), "a árvore não mudou");
}

#[test]
fn salvar_plano_torto_nao_abre_escrita() {
    let e = estado(false);
    e.mcp.ligar_janela();
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let t = Temp::nova();
    std::fs::write(t.0.join("plano.json"), json!({"formato": "blueprint.plano/1", "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "A", "planos": [{"id": "1.1", "texto": "x", "depende_de": ["z"]}]}]}]}).to_string()).unwrap();
    let erro = salvar(Some(&via), &json!({"pasta": t.0.display().to_string(), "workspace_id": "w2", "blueprint_id": "p9"}), Perfil::Completo, None).unwrap_err();
    assert!(erro.contains("does not exist") && erro.contains('z'), "{erro}");
    std::fs::write(t.0.join("plano.json"), json!({"formato": "blueprint.plano/2"}).to_string()).unwrap();
    assert!(salvar(Some(&via), &json!({"pasta": t.0.display().to_string(), "workspace_id": "w2"}), Perfil::Completo, None).unwrap_err().contains("not supported"));
    assert!(chamadas.borrow().is_empty());
}

#[test]
fn abrir_escrever_fechar_com_figuras() {
    let e = estado(false);
    e.mcp.ligar_janela();
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let t = Temp::nova();
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x0a\0\0\0\x14".to_vec();
    png.extend([0xfb, 0xff, 0xfb, 0xff]); // no base64url viram `-` e `_`: a rota só aceita `+` e `/`
    std::fs::create_dir(t.0.join("figs")).unwrap();
    std::fs::write(t.0.join("figs/a.png"), &png).unwrap();
    let f = |nome: &str, a: Value| ferramenta(Some(&via), nome, &a, Perfil::Completo, Some(&t.0));

    let aberto = f("abrir_plano", json!({"workspace_id": "w2", "blueprint": "Novo", "projetos": [{"acoes": [{"id": "1", "nome": "Primeira"}]}]})).unwrap();
    assert!(aberto.contains("Waiting for content: 1"), "{aberto}");
    let corpo = chamadas.borrow()[0].1.clone();
    assert_eq!((corpo["origem"].as_str(), corpo["pasta"].as_str()), (Some("escrita"), Some(t.0.to_str().unwrap())), "blueprint novo leva a pasta da sessão");
    let trecho = |antes: &str| aberto.split(antes).nth(1).unwrap().split([',', ')']).next().unwrap().to_string();
    let (escrita, bp) = (trecho("escrita_id: "), trecho("blueprint_id: "));
    assert!(f("abrir_plano", json!({"workspace_id": "w2", "blueprint_id": bp})).unwrap().contains("already open"));

    let escrito = f("escrever_acao", json!({"escrita_id": escrita, "acao": {"id": "1", "planos": [{"id": "1.1", "texto": "Fazer", "area": "app"}], "remover_planos": ["nao-ha"], "detalhes": [
        {"id": "01", "figura_caminho": "figs/a.png"}, {"id": "02", "figura_svg": "<svg viewBox=\"0 0 30 40\"/>"}]}, "decisoes": [{"id": "d1", "texto": "Decidido"}]})).unwrap();
    assert!(escrito.contains("Action 1 is on the user's screen") && escrito.contains("fechar_plano"), "{escrito}");
    let ultima = chamadas.borrow().last().unwrap().1.clone();
    assert_eq!(ultima["acao"]["removerPlanos"], json!(["nao-ha"]));
    assert!(ultima["acao"].get("remover_planos").is_none());
    assert_eq!(ultima["acao"]["detalhes"], json!([{"id": "01", "figura": "figs/a.png"}, {"id": "02", "figura": "figuras/1-02.svg"}]), "o conteúdo não vai junto da ação");

    let hostil = f("escrever_acao", json!({"escrita_id": escrita, "acao": {"id": "1", "detalhes": [{"id": "03", "figura_svg": "<svg><script>x</script></svg>"}]}})).unwrap_err();
    assert!(hostil.contains("The plan has errors") && hostil.contains("script"), "{hostil}");

    let fechado = f("fechar_plano", json!({"escrita_id": escrita})).unwrap();
    assert!(fechado.contains("New, already on the sheet: 1") && fechado.contains("Only the user approves"), "{fechado}");
    let lido = f("ler_plano", json!({"workspace_id": "w2", "blueprint_id": bp})).unwrap();
    assert!(lido.contains("Fazer") && lido.contains("\"largura\":10") && lido.contains("\"altura\":40"), "as medidas vieram das figuras: {lido}");
    assert!(chamadas.borrow().iter().all(|(r, _)| !r.starts_with("gesto") && !r.starts_with("execucao")));
}

#[test]
fn orquestrador_nao_manda_pasta() {
    let e = estado(false);
    e.mcp.ligar_janela();
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let t = Temp::nova();
    ferramenta(Some(&via), "abrir_plano", &json!({"workspace_id": "w2", "blueprint": "Dele"}), Perfil::Orquestrador, Some(&t.0)).unwrap();
    let corpo = chamadas.borrow()[0].1.clone();
    assert!(corpo.get("pasta").is_none(), "{corpo}");
    assert_eq!(corpo["agente"], "orquestrador");
}

#[test]
fn figura_fora_da_sessao_recusada() {
    let t = Temp::nova();
    let fora = Temp::nova();
    std::fs::write(fora.0.join("a.svg"), "<svg/>").unwrap();
    for c in ["../a.svg", "x/../../a.svg", "/etc/a.svg", "\\a.svg", "c:/a.svg", ""] {
        assert!(ler_figura_da_sessao(&t.0, c).unwrap_err().contains("inside this session's folder"), "{c}");
    }
    assert!(ler_figura_da_sessao(&t.0, "a.exe").unwrap_err().contains("use SVG"));
    assert!(ler_figura_da_sessao(&t.0, "falta.svg").unwrap_err().contains("not found"));
    std::fs::write(t.0.join("grande.png"), vec![0u8; LIMITE_IMAGEM + 1]).unwrap();
    assert!(ler_figura_da_sessao(&t.0, "grande.png").unwrap_err().contains("larger than"));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(fora.0.join("a.svg"), t.0.join("link.svg")).unwrap();
        std::os::unix::fs::symlink(&fora.0, t.0.join("pasta")).unwrap();
        for c in ["link.svg", "pasta/a.svg"] {
            assert!(ler_figura_da_sessao(&t.0, c).unwrap_err().contains("symbolic"), "{c}");
        }
    }
}

#[test]
fn falar_e_ler_respostas_e_workspace() {
    let e = estado(false);
    e.mcp.ligar_janela();
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let f = |nome: &str, a: Value| ferramenta(Some(&via), nome, &a, Perfil::Orquestrador, None);
    f("falar_com_voce", json!({"workspace_id": "w2", "blueprint_id": "p9", "texto": "Recado", "para_execucao": "abc"})).unwrap();
    assert_eq!(chamadas.borrow()[0].1["de"], "orquestrador");
    assert_eq!(chamadas.borrow()[0].1["para"], "abc");
    let lidas: Value = serde_json::from_str(&f("ler_respostas", json!({"workspace_id": "w2", "blueprint_id": "p9", "execucao_id": "abc"})).unwrap()).unwrap();
    assert_eq!(lidas["recados"][0]["texto"], "Recado");
    assert!(chamadas.borrow()[1].0.contains("execucao=abc"), "{:?}", chamadas.borrow()[1].0);
    let ws: Value = serde_json::from_str(&f("ler_workspace", json!({"workspace_id": "w2"})).unwrap()).unwrap();
    assert_eq!(ws["recados"][0]["texto"], "Recado");
    assert!(f("ler_plano", json!({"workspace_id": "w2"})).unwrap_err().contains("blueprint_id is required"));
}

#[test]
fn segredo_vai_no_evento() {
    // Do ambiente, o invólucro; o argumento vale por cima, para quem não repassa o ambiente ao MCP.
    let env = |k: &str| match k {
        "BLUEPRINT_EXECUCAO" => Some("abc123".to_string()),
        "BLUEPRINT_EXECUCAO_SEGREDO" => Some("do-ambiente".to_string()),
        "BLUEPRINT_AGENTE_NOME" => Some("plano 1.1".to_string()),
        _ => None,
    };
    let m = com_execucao(Map::new(), &json!({}), &env);
    assert_eq!(Value::Object(m), json!({"execucaoId": "abc123", "execucaoSegredo": "do-ambiente", "agente": "plano 1.1"}));
    let com_agente = Map::from_iter([("agente".to_string(), json!("Outro"))]);
    let m = com_execucao(com_agente, &json!({"execucao_segredo": "do-argumento"}), &env);
    assert_eq!((m["execucaoSegredo"].as_str(), m["agente"].as_str()), (Some("do-argumento"), Some("Outro")));
    assert!(com_execucao(Map::new(), &json!({}), &|_| None).is_empty(), "sem execução, o evento de sempre");

    // O que cada ferramenta manda.
    let lista: Value = serde_json::from_str(include_str!("ferramentas.json")).unwrap();
    let chamadas = RefCell::new(Vec::new());
    let e = estado(false);
    let via = via_real(&e, &chamadas);
    let base = json!({"workspace_id": "sm", "blueprint_id": "p1", "acao_id": "f1", "plano_id": "t1", "checkpoint_id": "m1", "evidencia": "cargo test ok", "motivo": "falhou", "texto": "preso", "execucao_segredo": "s3gredo"});
    for nome in ["marcar_concluido", "marcar_nao_concluido", "marcar_checkpoint_feito", "marcar_checkpoint_nao_feito", "reportar_bloqueio"] {
        let props = &lista.as_array().unwrap().iter().find(|t| t["name"] == nome).unwrap()["inputSchema"]["properties"];
        assert!(props.get("execucao_segredo").is_some(), "{nome} declara o campo");
        tratar_via(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": nome, "arguments": base}}), Some(&via)).unwrap();
        assert_eq!(chamadas.borrow().last().unwrap().1["execucaoSegredo"], "s3gredo", "{nome}");
    }
}

#[test]
fn segredo_certo_conclui_no_servidor() {
    let e = estado(false);
    let segredo = {
        let mut d = e.dados.lock().unwrap();
        let id = d.execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: "p1", acao_id: "f1", plano_id: "t1", plano_ref: "t1", pasta_principal: "/p", marcada_em: 1, ..NovaExecucao::default() }).unwrap();
        assert!(d.execucao_reivindicar(&id).unwrap());
        assert!(d.execucao_lancar(&id, &Lancamento { pasta_trabalho: "/c", ramo: "agente/t1", agora_ms: 1, ..Lancamento::default() }).unwrap());
        d.execucao_de(&id).unwrap().unwrap().segredo
    };
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    let concluir = |extra: Value| {
        let mut a = json!({"workspace_id": "sm", "blueprint_id": "p1", "acao_id": "f1", "plano_id": "t1", "evidencia": "cargo test ok"});
        a.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        tratar_via(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "marcar_concluido", "arguments": a}}), Some(&via)).unwrap()
    };
    let feito = || e.dados.lock().unwrap().ler_tudo(&Contexto::local()).unwrap()["blueprints"][0]["acoes"][0]["planos"][0]["feito"] == true;
    let sem = concluir(json!({}));
    assert_eq!(sem["result"]["isError"], true, "{sem}");
    assert!(sem["result"]["content"][0]["text"].as_str().unwrap().contains("403"), "{sem}");
    assert!(!feito());
    let com = concluir(json!({"execucao_segredo": segredo}));
    assert!(com["result"].get("isError").is_none(), "{com}");
    assert!(feito(), "gravou sem passar pela fila");
    assert!(e.mcp.itens().is_empty());
}

#[test]
fn propor_avisa_sem_consumidor() {
    let e = estado(false);
    let chamadas = RefCell::new(Vec::new());
    let via = via_real(&e, &chamadas);
    assert_eq!(aviso_sem_consumidor(Some(&via)), "", "por HTTP a página consome a fila");
    let esperado = if cfg!(windows) { "" } else { "\nNo app window consumes this queue: use abrir_plano." };
    assert_eq!(aviso_sem_consumidor(None), esperado);
    let r = tratar_via(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "propor_blueprint", "arguments": {"workspace_id": "w2", "blueprint_id": "p9", "acoes": [{"nome": "Nova", "inicio": "2026-12-01", "fim": "2026-12-02"}]}}}), Some(&via)).unwrap();
    let texto = r["result"]["content"][0]["text"].as_str().unwrap();
    assert!(texto.contains("Proposal no. 1") && !texto.contains("No app window"), "{texto}");
}
