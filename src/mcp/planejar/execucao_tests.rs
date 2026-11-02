//! Testes de mcp::planejar::execucao pela rota de verdade: `execucao/*` e os eventos do agente que gravam no servidor (D8).
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, Lancamento, Mudanca, NovaExecucao};
use crate::dados::Dados;
use crate::mcp::rotas::reenviar;
use crate::mcp::Mcp;
use crate::plano::ids::nome_agente;
use serde_json::{json, Value};
use std::sync::Mutex;

const REF_LONGO: &str = "plano-com-um-nome-bem-comprido-de-quase-sessenta-e-quatro-letra";

fn mcp() -> Mcp {
    Mcp::novo("127.0.0.1", false, "blueprint", "http://127.0.0.1:3000", std::env::temp_dir())
}

fn dados() -> Mutex<Dados> {
    let mut d = Dados::em_memoria().unwrap();
    let arvore = json!({"empresa": "Eu", "blueprints": [{"id": "bp1", "nome": "Monitor", "acoes": [
        {"id": "ra1", "ref": "1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "planos": [{"id": "rq1", "ref": "1.1", "texto": "Fazer"}, {"id": "rq2", "ref": "1.2", "texto": "Outro"}], "checkpoints": [{"id": "rc1", "ref": "1.c1", "nome": "Marco", "data": "2026-11-10"}]},
        {"id": "ra2", "ref": "2", "nome": "Fora", "inicio": "2026-12-01", "fim": "2026-12-05", "planos": [], "checkpoints": [{"id": "rc2", "ref": "2.c1", "nome": "Outro marco", "data": "2026-12-02"}]}
    ]}]});
    d.gravar_tudo(&Contexto::local(), &arvore).unwrap();
    Mutex::new(d)
}

fn marcada(d: &Mutex<Dados>, plano: &str, r: &str) -> String {
    d.lock().unwrap().execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: "bp1", acao_id: "ra1", plano_id: plano, plano_ref: r, pasta_principal: "/p", marcada_em: 1, ..NovaExecucao::default() }).unwrap()
}

fn lancar(d: &Mutex<Dados>, id: &str) -> String {
    let mut g = d.lock().unwrap();
    assert!(g.execucao_reivindicar(id).unwrap());
    assert!(g.execucao_lancar(id, &Lancamento { pasta_trabalho: "/c", ramo: "agente/1.1", agora_ms: 1, ..Lancamento::default() }).unwrap());
    g.execucao_de(id).unwrap().unwrap().segredo
}

fn rodando(d: &Mutex<Dados>) -> (String, String) {
    let id = marcada(d, "rq1", "1.1");
    let segredo = lancar(d, &id);
    (id, segredo)
}

fn linha(d: &Mutex<Dados>, id: &str) -> ExecucaoLinha {
    d.lock().unwrap().execucao_de(id).unwrap().unwrap()
}

fn evento(tipo: &str, texto: &str, segredo: Option<&str>) -> Value {
    let mut c = json!({"tipo": tipo, "texto": texto, "workspaceId": "sm", "blueprintId": "bp1", "acaoId": "1", "planoId": "1.1"});
    if let Some(s) = segredo {
        c["execucaoSegredo"] = json!(s);
    }
    c
}

fn plano(d: &Mutex<Dados>, i: usize) -> Value {
    d.lock().unwrap().ler_tudo(&Contexto::local()).unwrap()["blueprints"][0]["acoes"][0]["planos"][i].clone()
}

fn checkpoint(d: &Mutex<Dados>, acao: usize) -> Value {
    d.lock().unwrap().ler_tudo(&Contexto::local()).unwrap()["blueprints"][0]["acoes"][acao]["checkpoints"][0].clone()
}

#[test]
fn concluir_grava_no_servidor() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    let revisao = d.lock().unwrap().revisao_tempos();
    let (status, r) = reenviar(&m, &d, "evento", &evento("concluir", "cargo test: 12 passaram", Some(&segredo)));
    assert_eq!(status, 200, "{r}");
    let q = plano(&d, 0);
    assert_eq!(q["feito"], true);
    assert_eq!(q["agente"]["comentarios"][0]["tipo"], "relatorio");
    assert_eq!(q["agente"]["comentarios"][0]["texto"], "cargo test: 12 passaram");
    assert!(q["agente"]["feitoEm"].is_string());
    assert_eq!(linha(&d, &id).situacao, "pronta");
    assert!(m.itens().is_empty(), "nada vai à fila");
    assert!(d.lock().unwrap().revisao_tempos() > revisao, "a versão dos dados sobe");
    assert!(d.lock().unwrap().orq_ler("sm").unwrap().motivos.iter().any(|x| x == "pronta:1.1"), "acorda o orquestrador");
    assert_eq!(reenviar(&m, &d, "evento", &evento("concluir", "de novo", Some(&segredo))).0, 409, "plano já feito, sem execução viva: a validação antiga");
}

#[test]
fn concluir_de_outro_403() {
    let (m, d) = (mcp(), dados());
    let (id, _) = rodando(&d);
    let outra = marcada(&d, "rq2", "1.2");
    let segredo_alheio = lancar(&d, &outra);
    for segredo in [None, Some("errado"), Some(segredo_alheio.as_str())] {
        let (status, r) = reenviar(&m, &d, "evento", &evento("concluir", "feito", segredo));
        assert_eq!((status, r["codigo"].as_str()), (403, Some("nao-e-seu-plano")), "{segredo:?}");
    }
    assert_ne!(plano(&d, 0)["feito"], true);
    assert_eq!(linha(&d, &id).situacao, "rodando");
    assert!(m.agentes()["agentes"].as_array().unwrap().is_empty(), "quem foi recusado não aparece na folha");
}

#[test]
fn bloqueio_de_outro_403() {
    let (m, d) = (mcp(), dados());
    let (id, _) = rodando(&d);
    let (status, r) = reenviar(&m, &d, "evento", &evento("bloqueio", "travei", Some("errado")));
    assert_eq!((status, r["codigo"].as_str()), (403, Some("nao-e-seu-plano")));
    assert_eq!(linha(&d, &id).situacao, "rodando");
}

#[test]
fn concluir_sem_evidencia_400() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    assert_eq!(reenviar(&m, &d, "evento", &evento("concluir", "   ", Some(&segredo))).0, 400);
    assert_ne!(plano(&d, 0)["feito"], true);
    assert_eq!(linha(&d, &id).situacao, "rodando");
}

#[test]
fn bloqueio_vira_ajuda() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    let antes = m.revisao_planejar();
    let (status, r) = reenviar(&m, &d, "evento", &evento("bloqueio", "preciso da senha do banco", Some(&segredo)));
    assert_eq!(status, 200, "{r}");
    let x = linha(&d, &id);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str()), ("ajuda", "preciso da senha do banco"));
    assert!(m.itens().is_empty());
    assert!(m.revisao_planejar() > antes, "o pulso muda");
    assert!(d.lock().unwrap().orq_ler("sm").unwrap().motivos.iter().any(|x| x == "ajuda:1.1"));
}

#[test]
fn progresso_volta_de_ajuda() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    reenviar(&m, &d, "evento", &evento("bloqueio", "travei", Some(&segredo)));
    assert_eq!(reenviar(&m, &d, "evento", &evento("progresso", "de outra sessão", None)).0, 200);
    assert_eq!(linha(&d, &id).situacao, "ajuda", "sem segredo é só o registro antigo");
    assert_eq!(m.itens().len(), 1);
    assert_eq!(reenviar(&m, &d, "evento", &evento("progresso", "voltei", Some(&segredo))).0, 200);
    let x = linha(&d, &id);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str()), ("rodando", ""));
    assert!(x.ultimo_sinal.is_some(), "renova o sinal");
}

#[test]
fn partida_so_rodando() {
    let (m, d) = (mcp(), dados());
    let id = marcada(&d, "rq1", "1.1");
    let (status, r) = reenviar(&m, &d, "execucao/partida", &json!({"id": id}));
    assert_eq!((status, r["codigo"].as_str()), (409, Some("nao-esta-rodando")));
    assert_eq!(reenviar(&m, &d, "execucao/partida", &json!({"id": "0000000000000000"})).0, 404);
    let segredo = lancar(&d, &id);
    let (status, r) = reenviar(&m, &d, "execucao/partida", &json!({"id": id}));
    assert_eq!(status, 200, "{r}");
    assert_eq!((r["segredo"].as_str(), r["tentativa"].as_i64(), r["pasta"].as_str()), (Some(segredo.as_str()), Some(1), Some("/c")));
    assert_eq!((r["comando"].clone(), r["claude"].clone(), r["nomeAgente"].as_str()), (json!(["claude"]), json!(true), Some("plano 1.1")));
    assert!(r["prompt"].as_str().unwrap().contains("Fazer"));
    let (status, r) = reenviar(&m, &d, "execucao/partida", &json!({"id": id}));
    assert_eq!((status, r["codigo"].as_str()), (409, Some("ja-partiu")));
    // comando configurado: lista de argumentos, sem ser o caminho do claude
    d.lock().unwrap().config_gravar("comando_agente", r#"["codex","--sim"]"#).unwrap();
    d.lock().unwrap().execucao_mudar(&id, &Mudanca { situacao: Some("marcada"), ..Mudanca::default() }).unwrap();
    lancar(&d, &id);
    let (_, r) = reenviar(&m, &d, "execucao/partida", &json!({"id": id}));
    assert_eq!((r["comando"].clone(), r["claude"].clone(), r["tentativa"].as_i64()), (json!(["codex", "--sim"]), json!(false), Some(2)));
}

#[test]
fn checkpoint_grava_e_traduz_id() {
    let (m, d) = (mcp(), dados());
    let (_, segredo) = rodando(&d);
    let corpo = |tipo: &str, acao: &str, cp: &str, s: &str| json!({"tipo": tipo, "texto": "demonstrado", "workspaceId": "sm", "blueprintId": "bp1", "acaoId": acao, "checkpointId": cp, "execucaoSegredo": s});
    let (status, r) = reenviar(&m, &d, "evento", &corpo("checkpoint-feito", "1", "1.c1", &segredo));
    assert_eq!(status, 200, "{r}");
    assert_eq!(r["checkpointId"], "rc1", "ref traduzido");
    let c = checkpoint(&d, 0);
    assert_eq!((c["feito"].clone(), c["evidencia"].as_str()), (json!(true), Some("demonstrado")));
    assert!(c["feitoEm"].is_string());
    assert!(m.itens().is_empty());
    let (status, r) = reenviar(&m, &d, "evento", &corpo("checkpoint-nao-feito", "1", "rc1", &segredo));
    assert_eq!(status, 200, "{r}");
    assert_eq!(checkpoint(&d, 0)["feito"], false, "desfazer precisa passar pelo que já está feito");
    assert!(checkpoint(&d, 0).get("evidencia").is_none());
    // o segredo de execução de outra ação não grava: é o fluxo antigo, e o pedido fica na fila
    assert_eq!(reenviar(&m, &d, "evento", &corpo("checkpoint-feito", "2", "2.c1", &segredo)).0, 200);
    assert_ne!(checkpoint(&d, 1)["feito"], true);
    assert_eq!(m.itens().len(), 1);
}

#[test]
fn retomar_nao_duplica() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    assert_eq!(reenviar(&m, &d, "execucao/fim", &json!({"id": id, "segredo": segredo, "codigo": 1})).0, 200);
    assert!(d.lock().unwrap().execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: "bp1", acao_id: "ra1", plano_id: "rq1", plano_ref: "1.1", marcada_em: 2, ..NovaExecucao::default() }).is_err(), "falhou conta como viva");
    d.lock().unwrap().execucao_mudar(&id, &Mudanca { so_de: &["falhou"], situacao: Some("marcada"), ..Mudanca::default() }).unwrap();
    let novo = lancar(&d, &id);
    assert_ne!(novo, segredo);
    assert_eq!(linha(&d, &id).tentativa, 2);
    assert_eq!(d.lock().unwrap().execucoes_do_ws("sm").unwrap().len(), 1, "a mesma execução, não outra");
    assert_eq!(reenviar(&m, &d, "execucao/partida", &json!({"id": id})).0, 200, "a nova tentativa parte de novo");
}

#[test]
fn fim_nao_desfaz_cancelada() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    assert!(d.lock().unwrap().execucao_cancelar(&id, 5).unwrap());
    assert_eq!(reenviar(&m, &d, "execucao/sinal", &json!({"id": id, "segredo": segredo})).1["parar"], true);
    assert_eq!(reenviar(&m, &d, "execucao/fim", &json!({"id": id, "segredo": segredo, "codigo": 143})).0, 200);
    let x = linha(&d, &id);
    assert_eq!(x.situacao, "cancelada");
    assert!(x.terminada_em.is_some(), "com o fim gravado, ela para de segurar a área");
    // pronta também fica
    let outra = marcada(&d, "rq2", "1.2");
    let s = lancar(&d, &outra);
    let corpo = json!({"tipo": "concluir", "texto": "ok", "workspaceId": "sm", "blueprintId": "bp1", "acaoId": "1", "planoId": "1.2", "execucaoSegredo": s});
    assert_eq!(reenviar(&m, &d, "evento", &corpo).0, 200);
    reenviar(&m, &d, "execucao/fim", &json!({"id": outra, "segredo": s, "codigo": 0}));
    assert_eq!(linha(&d, &outra).situacao, "pronta");
}

#[test]
fn fim_sem_concluir_vira_falhou() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    reenviar(&m, &d, "execucao/fim", &json!({"id": id, "segredo": segredo, "codigo": 1}));
    let x = linha(&d, &id);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str()), ("falhou", "terminou-sem-concluir"));
    assert!(d.lock().unwrap().orq_ler("sm").unwrap().motivos.iter().any(|x| x == "falhou:1.1"));
}

#[test]
fn nao_concluida_desfaz_a_propria_conclusao() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    assert_eq!(reenviar(&m, &d, "evento", &evento("concluir", "feito", Some(&segredo))).0, 200);
    assert_eq!(reenviar(&m, &d, "evento", &evento("nao-concluida", "o teste quebrou", None)).0, 409, "sem o segredo, a validação antiga");
    assert_eq!(plano(&d, 0)["feito"], true);
    assert_eq!(reenviar(&m, &d, "evento", &evento("nao-concluida", "o teste quebrou", Some(&segredo))).0, 200);
    assert_eq!(plano(&d, 0)["feito"], false);
    let x = linha(&d, &id);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str()), ("falhou", "o teste quebrou"));
}

#[test]
fn sinal_revive_sumiu() {
    let (m, d) = (mcp(), dados());
    let (id, segredo) = rodando(&d);
    d.lock().unwrap().execucao_mudar(&id, &Mudanca { situacao: Some("sumiu"), ..Mudanca::default() }).unwrap();
    let antes = m.revisao_planejar();
    assert_eq!(reenviar(&m, &d, "execucao/sinal", &json!({"id": id, "segredo": segredo})).1, json!({"parar": false}));
    let x = linha(&d, &id);
    assert_eq!(x.situacao, "rodando");
    assert!(x.ultimo_sinal.is_some());
    assert!(m.revisao_planejar() > antes, "o pulso muda");
}

#[test]
fn tentativa_velha_ignorada() {
    let (m, d) = (mcp(), dados());
    let (id, velho) = rodando(&d);
    d.lock().unwrap().execucao_mudar(&id, &Mudanca { situacao: Some("marcada"), ..Mudanca::default() }).unwrap();
    let novo = lancar(&d, &id);
    assert_eq!(reenviar(&m, &d, "execucao/sinal", &json!({"id": id, "segredo": velho})).1["parar"], true, "o agente velho para");
    assert!(linha(&d, &id).ultimo_sinal.is_none(), "sem mudar nada");
    assert_eq!(reenviar(&m, &d, "execucao/fim", &json!({"id": id, "segredo": velho, "codigo": 1})).1["ignorado"], true);
    let x = linha(&d, &id);
    assert_eq!((x.situacao.as_str(), x.terminada_em), ("rodando", None));
    assert_eq!(reenviar(&m, &d, "evento", &evento("concluir", "feito", Some(&velho))).0, 403);
    assert_eq!(reenviar(&m, &d, "execucao/sinal", &json!({"id": id, "segredo": novo})).1["parar"], false);
    assert_eq!(reenviar(&m, &d, "execucao/sinal", &json!({"id": "nao-existe", "segredo": novo})).1["parar"], true);
}

#[test]
fn nome_longo_nao_da_400() {
    let (m, d) = (mcp(), dados());
    let id = marcada(&d, "rq1", REF_LONGO);
    let segredo = lancar(&d, &id);
    let (_, r) = reenviar(&m, &d, "execucao/partida", &json!({"id": id}));
    let nome = r["nomeAgente"].as_str().unwrap();
    assert_eq!(nome, nome_agente(REF_LONGO));
    assert_eq!(nome.chars().count(), 40);
    let mut corpo = evento("progresso", "indo", Some(&segredo));
    corpo["agente"] = json!(nome);
    let (status, r) = reenviar(&m, &d, "evento", &corpo);
    assert_eq!(status, 200, "{r}");
}
