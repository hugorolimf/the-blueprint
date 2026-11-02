//! Passa de 300 linhas (teto 500): são os casos do contrato sobre a mesma cena; `gestos_tests/revisar.rs` já leva a parte de revisão.
//! Os gestos pela rota inteira; `gesto/revisar` e `gesto/colocar` em `gestos_tests/revisar.rs` (`despachar`), com exemplo genérico. A versão pendente nasce direto pelo `Dados`, sem depender da rota de escrita. `gesto/integrar` usa git real em pasta temporária.
use super::git;
use super::revisao::hash_diff;
use crate::contexto::Contexto;
use crate::dados::planejar::{Lancamento, Mudanca, NovaExecucao};
use crate::http::pedido::{Pedido, Resposta};
use crate::http::rotas::despachar;
use crate::http::tests::{estado, json_de, pedido};
use crate::http::Estado;
use crate::plano::arvore::do_blueprint;
use crate::plano::formato::{AcaoDoc, PlanoDoc};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

mod revisar;

static PASTAS: AtomicU32 = AtomicU32::new(0);

fn pasta_temp() -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-gestos-{}-{}", std::process::id(), PASTAS.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn base() -> Value {
    json!({"blueprints": [{"id": "b1", "nome": "Exemplo", "acoes": [
        {"id": "a1", "ref": "1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "planejado", "planos": [
            {"id": "q1", "ref": "1.1", "texto": "Fazer A", "area": "app"},
            {"id": "q2", "ref": "1.2", "texto": "Fazer B", "feito": true, "agente": {"estado": "concluido", "feitoEm": "2026-11-02T00:00:00Z"}}]},
        {"id": "a2", "ref": "2", "nome": "Depois", "inicio": "2026-12-01", "fim": "2026-12-30", "status": "planejado", "planos": [
            {"id": "q3", "ref": "2.1", "texto": "Fazer C", "dependeDe": ["1.1"]}]}],
        "planoDoc": {"decisoes": [{"id": "d1", "texto": "Usar um banco local", "pede_aprovacao": true}], "perguntas": [{"id": "p1", "texto": "Qual cor?", "opcoes": ["azul", "verde"]}]}}]})
}

struct Cena { e: Estado, token: String }

fn cena() -> Cena {
    let e = estado(false);
    e.dados.lock().unwrap().gravar_tudo(&Contexto::local(), &base()).unwrap();
    let token = e.mcp.ligar_janela();
    Cena { e, token }
}

impl Cena {
    fn gesto(&self, rota: &str, corpo: Value) -> Resposta {
        despachar(&self.e, &pedido("POST", &format!("/api/mcp/gesto/{rota}"), &[("X-Blueprint-Gesto", &self.token)], Some(corpo)))
    }
    fn arvore(&self) -> Value {
        let mut d = self.e.dados.lock().unwrap();
        d.ler_tudo(&Contexto::local()).unwrap()["blueprints"][0].clone()
    }
    fn doc(&self) -> PlanoDoc {
        do_blueprint(&self.arvore())
    }
    fn plano(&self, a: usize, q: usize) -> Value {
        self.arvore()["acoes"][a]["planos"][q].clone()
    }
    /// Fecha uma versão com o doc da árvore mudado por `f`; devolve o número.
    fn pendente(&self, origem: &str, f: impl FnOnce(&mut PlanoDoc)) -> i64 {
        let mut doc = self.doc();
        f(&mut doc);
        let mut d = self.e.dados.lock().unwrap();
        let (numero, escrita) = d.plano_abrir("sm", "b1", origem, "agente", &doc.para_json(), &[], 1).unwrap();
        d.plano_fechar(&escrita, 2).unwrap();
        numero
    }
    fn acao_nova(&self, r: &str) -> Option<AcaoDoc> {
        let d = self.e.dados.lock().unwrap();
        let v = d.plano_pendente("b1").unwrap()?;
        let doc = PlanoDoc::de_json(&v.plano).unwrap();
        doc.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.id == r).cloned()
    }
    /// O hash do diff como está valendo agora.
    fn hash(&self, r: &str) -> String {
        let doc = self.doc();
        let antiga = doc.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.id == r);
        hash_diff(antiga, self.acao_nova(r).as_ref())
    }
    fn revisar(&self, r: &str, numero: i64, estado: &str, extra: Value) -> Resposta {
        let mut corpo = json!({"workspaceId": "sm", "blueprintId": "b1", "peca": format!("diff:{r}"), "numero": numero, "hash": self.hash(r), "estado": estado});
        for (k, v) in extra.as_object().into_iter().flatten() {
            corpo[k] = v.clone();
        }
        self.gesto("revisar", corpo)
    }
    fn ligar_pasta(&self) -> PathBuf {
        let p = pasta_temp();
        self.e.dados.lock().unwrap().vincular_pasta(&Contexto::local(), p.to_str().unwrap(), "sm", "b1").unwrap();
        p
    }
    fn execucao(&self, plano: &str, r: &str) -> String {
        let n = NovaExecucao { workspace_id: "sm", blueprint_id: "b1", acao_id: "a1", plano_id: plano, plano_ref: r, marcada_em: 1, ..NovaExecucao::default() };
        self.e.dados.lock().unwrap().execucao_criar(&n).unwrap()
    }
    fn versao(&self) -> u64 {
        self.e.versao_dados.load(Ordering::SeqCst)
    }
    fn pendente_existe(&self) -> bool {
        self.e.dados.lock().unwrap().plano_pendente("b1").unwrap().is_some()
    }
}

fn codigo(r: &Resposta) -> String {
    json_de(r)["codigo"].as_str().unwrap_or("").to_string()
}

fn par(r: &Resposta) -> (u16, String) {
    (r.status, codigo(r))
}

fn sem_token(rota: &str, extra: &[(&str, &str)]) -> Pedido {
    pedido("POST", &format!("/api/mcp/gesto/{rota}"), extra, Some(json!({})))
}

const TODAS: [&str; 12] = ["marcar", "cancelar", "retomar", "integrar", "revisar", "colocar", "responder", "limite", "comando", "pasta", "orquestrador", "descartar-escrita"];

#[test]
fn sem_gesto_403() {
    let c = cena();
    for rota in TODAS {
        assert_eq!(par(&despachar(&c.e, &sem_token(rota, &[]))), (403, "gesto-negado".to_string()), "{rota}");
        assert_eq!(par(&despachar(&c.e, &sem_token(rota, &[("X-Blueprint-Gesto", "errado")]))), (403, "gesto-negado".to_string()), "{rota} com token errado");
        assert_ne!(codigo(&c.gesto(rota, json!({}))), "gesto-negado", "{rota} com o token");
        assert_ne!(codigo(&c.gesto(rota, json!({}))), "nao-implementado", "{rota} é de verdade");
    }
}

#[test]
fn gesto_com_origin_de_fora_403() {
    let c = cena();
    for rota in TODAS {
        let p = sem_token(rota, &[("X-Blueprint-Gesto", &c.token), ("Origin", "http://exemplo.invalid")]);
        assert_eq!(par(&despachar(&c.e, &p)), (403, "gesto-negado".to_string()), "{rota}");
    }
    assert_eq!(c.e.dados.lock().unwrap().execucoes_do_ws("sm").unwrap().len(), 0);
}

#[test]
fn marcar_iniciar_os_n() {
    let c = cena();
    let pasta = c.ligar_pasta();
    let r = c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": "b1", "planos": ["q1", "2.1"]}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(json_de(&r)["avisos"], json!([]), "marcados juntos, a dependência já anda");
    let execs = c.e.dados.lock().unwrap().execucoes_do_ws("sm").unwrap();
    assert_eq!(execs.len(), 2);
    assert!(execs.iter().all(|x| x.situacao == "marcada" && x.pasta_principal == pasta.to_str().unwrap()));
    let q1 = execs.iter().find(|x| x.plano_id == "q1").unwrap();
    assert_eq!((q1.area.as_str(), q1.plano_ref.as_str(), q1.acao_id.as_str()), ("app", "1.1", "a1"));
    assert_eq!(execs.iter().find(|x| x.plano_id == "q3").unwrap().plano_ref, "2.1", "aceita o ref");
}

#[test]
fn marcar_dependencia_sem_iniciar_so_avisa() {
    let c = cena();
    c.ligar_pasta();
    let r = c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": "b1", "planos": ["q3"]}));
    assert_eq!(r.status, 200);
    let avisos = json_de(&r)["avisos"].to_string();
    assert!(avisos.contains("1.1") && !avisos.contains("b1|"), "{avisos}");
}

#[test]
fn marcar_sem_pasta_409() {
    let c = cena();
    let corpo = json!({"workspaceId": "sm", "blueprintId": "b1", "planos": ["q1"]});
    assert_eq!(par(&c.gesto("marcar", corpo.clone())), (409, "sem-pasta".to_string()));
    let p = c.ligar_pasta();
    std::fs::remove_dir_all(&p).unwrap();
    assert_eq!(par(&c.gesto("marcar", corpo)), (409, "sem-pasta".to_string()), "a pasta ligada não existe mais");
    assert!(c.e.dados.lock().unwrap().execucoes_do_ws("sm").unwrap().is_empty());
}

#[test]
fn marcar_com_falhou_409() {
    let c = cena();
    c.ligar_pasta();
    let id = c.execucao("q1", "1.1");
    let m = Mudanca { situacao: Some("falhou"), ..Mudanca::default() };
    c.e.dados.lock().unwrap().execucao_mudar(&id, &m).unwrap();
    let r = c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": "b1", "planos": ["q1"]}));
    assert_eq!(par(&r), (409, "ja-tem-execucao".to_string()));
    let r = c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": "b1", "planos": ["q2"]}));
    assert_eq!(par(&r), (409, "ja-tem-execucao".to_string()), "plano feito");
    // nenhum do lote entra se um é recusado
    let r = c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": "b1", "planos": ["q3", "q2"]}));
    assert_eq!(r.status, 409);
    assert_eq!(c.e.dados.lock().unwrap().execucoes_do_ws("sm").unwrap().len(), 1);
}

#[test]
fn responder_grava_e_acorda() {
    let c = cena();
    let corpo = |p: &str| json!({"workspaceId": "sm", "blueprintId": "b1", "pergunta": p, "resposta": "azul"});
    assert_eq!(par(&c.gesto("responder", corpo("nao-existe"))), (404, "pergunta-nao-encontrada".to_string()));
    assert_eq!(c.gesto("responder", corpo("p1")).status, 200);
    let d = c.e.dados.lock().unwrap();
    let rev = d.revisoes_do("b1").unwrap();
    assert_eq!((rev[0].peca.as_str(), rev[0].estado.as_str(), rev[0].observacao.as_str()), ("q:p1", "respondida", "azul"));
    assert_eq!(d.orq_ler("sm").unwrap().motivos.len(), 1);
}

#[test]
fn limite_1_a_16() {
    let c = cena();
    let corpo = |n: Value| json!({"workspaceId": "sm", "limite": n});
    for ruim in [json!(0), json!(17), json!(-1), json!("4"), json!(2.5)] {
        assert_eq!(par(&c.gesto("limite", corpo(ruim.clone()))), (422, "limite-invalido".to_string()), "{ruim}");
    }
    for bom in [1, 16] {
        assert_eq!(c.gesto("limite", corpo(json!(bom))).status, 200);
        assert_eq!(c.e.dados.lock().unwrap().config_ler("limite:sm").unwrap(), Some(bom.to_string()));
    }
    assert_eq!(c.gesto("limite", json!({"workspaceId": "nao-existe", "limite": 2})).status, 404);
}

#[test]
fn comando_nao_vai_em_dados() {
    let c = cena();
    assert_eq!(c.gesto("comando", json!({"agente": ["agente-falso", "--opcao"], "orquestrador": ["orq-falso"]})).status, 200);
    let lido = despachar(&c.e, &pedido("GET", "/api/dados", &[], None));
    let texto = String::from_utf8_lossy(lido.bytes()).into_owned();
    assert!(!texto.contains("agente-falso") && !texto.contains("orq-falso"), "{texto}");
    // e o PUT não muda: campo com o mesmo nome só vira dado do arquivo
    let mut tudo = base();
    tudo["comando_agente"] = json!(["hostil"]);
    tudo["planejar_config"] = json!({"comando_agente": "[\"hostil\"]"});
    let versao = c.versao().to_string();
    let r = despachar(&c.e, &pedido("PUT", "/api/dados", &[("X-Blueprint-Gesto", &c.token), ("If-Match", &versao)], Some(tudo)));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    let d = c.e.dados.lock().unwrap();
    assert_eq!(d.config_ler("comando_agente").unwrap().as_deref(), Some("[\"agente-falso\",\"--opcao\"]"));
    assert_eq!(d.config_ler("comando_orquestrador").unwrap().as_deref(), Some("[\"orq-falso\"]"));
}

#[test]
fn comando_so_por_gesto() {
    let c = cena();
    let corpo = json!({"agente": ["hostil"]});
    let sem = despachar(&c.e, &pedido("POST", "/api/mcp/gesto/comando", &[], Some(corpo.clone())));
    assert_eq!(par(&sem), (403, "gesto-negado".to_string()));
    assert_eq!(despachar(&c.e, &pedido("POST", "/api/mcp/plano/comando", &[], Some(corpo))).status, 404, "o agente não tem rota");
    for ruim in [json!({"agente": []}), json!({"agente": [""]}), json!({"agente": ["a\nb"]}), json!({"orquestrador": "claude"}), json!({"agente": [1]}), json!({})] {
        assert_eq!(par(&c.gesto("comando", ruim.clone())), (422, "comando-invalido".to_string()), "{ruim}");
    }
    let valido_e_ruim = json!({"agente": ["bom"], "orquestrador": []});
    assert_eq!(c.gesto("comando", valido_e_ruim).status, 422);
    assert_eq!(c.e.dados.lock().unwrap().config_ler("comando_agente").unwrap(), None, "tudo ou nada");
}

fn git_em(pasta: &Path, args: &[&str]) {
    let s = Command::new("git").arg("-C").arg(pasta).args(args).output().unwrap();
    assert!(s.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&s.stderr));
}

fn repositorio() -> PathBuf {
    let raiz = pasta_temp();
    git_em(&raiz, &["init", "-q", "-b", "principal"]);
    for (k, v) in [("user.name", "Teste"), ("user.email", "teste@exemplo.invalid"), ("commit.gpgsign", "false")] {
        git_em(&raiz, &["config", k, v]);
    }
    std::fs::write(raiz.join("a.txt"), "um\n").unwrap();
    git_em(&raiz, &["add", "."]);
    git_em(&raiz, &["commit", "-qm", "inicio"]);
    raiz
}

/// Execução `pronta` com um commit no ramo dela, como o motor e o agente deixam.
fn pronta_com_ramo(c: &Cena, raiz: &Path) -> (String, PathBuf) {
    let destino = pasta_temp().join("copia");
    let copia = git::preparar_copia(&git::programa(), raiz, "agente/1.1", true, &destino, &[]).unwrap();
    std::fs::write(destino.join("b.txt"), "dois\n").unwrap();
    git_em(&destino, &["add", "."]);
    git_em(&destino, &["commit", "-qm", "plano"]);
    let n = NovaExecucao { workspace_id: "sm", blueprint_id: "b1", acao_id: "a1", plano_id: "q1", plano_ref: "1.1", pasta_principal: raiz.to_str().unwrap(), marcada_em: 1, ..NovaExecucao::default() };
    let mut d = c.e.dados.lock().unwrap();
    let id = d.execucao_criar(&n).unwrap();
    assert!(d.execucao_reivindicar(&id).unwrap());
    let l = Lancamento { pasta_trabalho: destino.to_str().unwrap(), ramo: &copia.ramo, base: "HEAD", agora_ms: 2, ..Lancamento::default() };
    assert!(d.execucao_lancar(&id, &l).unwrap());
    assert!(d.execucao_mudar(&id, &Mudanca { situacao: Some("pronta"), ..Mudanca::default() }).unwrap());
    (id, destino)
}

#[test]
fn integrar_devolve_ramo() {
    let c = cena();
    let raiz = repositorio();
    let (id, destino) = pronta_com_ramo(&c, &raiz);
    let r = c.gesto("integrar", json!({"execucaoId": id, "ramoEsperado": "outro"}));
    assert_eq!(par(&r), (409, "ramo-mudou".to_string()));
    assert!(!raiz.join("b.txt").exists());
    let r = c.gesto("integrar", json!({"execucaoId": id, "ramoEsperado": "principal"}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    let v = json_de(&r);
    assert_eq!((v["ramo"].as_str(), v["destino"].as_str(), v["avisos"].clone()), (Some("agente/1.1"), Some("principal"), json!([])));
    assert!(raiz.join("b.txt").exists(), "o merge aconteceu");
    assert!(!destino.exists(), "a cópia saiu");
    assert!(c.e.dados.lock().unwrap().execucao_de(&id).unwrap().unwrap().integrada_em.is_some());
    assert_eq!(par(&c.gesto("integrar", json!({"execucaoId": id, "ramoEsperado": "principal"}))), (409, "nao-integravel".to_string()), "só uma vez");
    assert_eq!(c.gesto("integrar", json!({"execucaoId": "nao-existe", "ramoEsperado": "principal"})).status, 404);
}

#[test]
fn retomar_ajuda_vivo_409() {
    let c = cena();
    let id = c.execucao("q1", "1.1");
    let agora = crate::tempo::agora_ms();
    let pare_em = |situacao: &'static str, sinal: i64| {
        let m = Mudanca { situacao: Some(situacao), ultimo_sinal: Some(sinal), parar: Some(true), ..Mudanca::default() };
        c.e.dados.lock().unwrap().execucao_mudar(&id, &m).unwrap();
    };
    let corpo = json!({"execucaoId": id});
    pare_em("ajuda", agora);
    assert_eq!(par(&c.gesto("retomar", corpo.clone())), (409, "agente-vivo".to_string()));
    pare_em("rodando", agora);
    assert_eq!(par(&c.gesto("retomar", corpo.clone())), (409, "nao-retoma".to_string()));
    for situacao in ["sumiu", "falhou"] {
        pare_em(situacao, agora);
        assert_eq!(c.gesto("retomar", corpo.clone()).status, 200, "{situacao}");
    }
    pare_em("ajuda", agora - 121_000);
    assert_eq!(c.gesto("retomar", corpo).status, 200, "sem sinal há 120 s");
    let x = c.e.dados.lock().unwrap().execucao_de(&id).unwrap().unwrap();
    assert_eq!((x.situacao.as_str(), x.parar), ("marcada", false));
}

#[test]
fn descartar_escrita() {
    let c = cena();
    let doc = c.doc().para_json();
    let (_, escrita) = c.e.dados.lock().unwrap().plano_abrir("sm", "b1", "escrita", "agente", &doc, &[], 1).unwrap();
    let (arvore, versao) = (c.arvore(), c.versao());
    assert_eq!(c.gesto("descartar-escrita", json!({"escritaId": escrita})).status, 200);
    assert!(c.e.dados.lock().unwrap().plano_aberta("b1").unwrap().is_none());
    assert_eq!(c.arvore(), arvore, "a árvore não muda");
    assert_eq!(c.versao(), versao);
    assert_eq!(c.gesto("descartar-escrita", json!({"escritaId": escrita})).status, 404);
}

#[test]
fn pasta_liga_e_recusa_o_que_nao_e_pasta() {
    let c = cena();
    let p = pasta_temp();
    let corpo = |caminho: &str| json!({"workspaceId": "sm", "blueprintId": "b1", "caminho": caminho});
    assert_eq!(par(&c.gesto("pasta", corpo("/nao/existe/mesmo"))), (422, "nao-e-pasta".to_string()));
    assert_eq!(par(&c.gesto("pasta", corpo("relativa"))), (422, "nao-e-pasta".to_string()));
    assert_eq!(c.gesto("pasta", corpo(p.to_str().unwrap())).status, 200);
    assert_eq!(c.e.dados.lock().unwrap().pasta_real_do_blueprint("sm", "b1").unwrap().as_deref(), p.to_str());
}

#[test]
fn orquestrador_mensagem_liga_e_acorda_urgente() {
    let c = cena();
    assert_eq!(c.gesto("orquestrador", json!({"workspaceId": "sm"})).status, 422);
    assert_eq!(json_de(&c.gesto("orquestrador", json!({"workspaceId": "sm", "ligado": false})))["ligado"], false);
    assert_eq!(json_de(&c.gesto("orquestrador", json!({"workspaceId": "sm", "mensagem": "Como vai?"})))["ligado"], true);
    let d = c.e.dados.lock().unwrap();
    let o = d.orq_ler("sm").unwrap();
    assert!(o.urgente && o.motivos.len() == 1 && !o.motivos[0].contains("Como vai"), "o texto vai como recado, marcado como dado");
    let recados = d.recados_do("sm", 5).unwrap();
    assert_eq!((recados[0].de.as_str(), recados[0].texto.as_str()), ("usuario", "Como vai?"));
}

#[test]
fn gesto_avisa_o_pulso() {
    let c = cena();
    let antes = c.e.mcp.revisao_planejar();
    assert_eq!(c.gesto("limite", json!({"workspaceId": "sm", "limite": 2})).status, 200);
    assert!(c.e.mcp.revisao_planejar() > antes);
    let antes = c.e.mcp.revisao_planejar();
    let n = c.pendente("escrita", |d| d.projetos[0].acoes[0].planos[0].texto = "Ruim".into());
    assert_eq!(c.revisar("1", n, "reprovada", json!({})).status, 200);
    assert!(c.e.mcp.revisao_planejar() > antes);
}
