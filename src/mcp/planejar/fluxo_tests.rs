//! O Planejar de ponta a ponta, pelas rotas de verdade (`despachar`), com o motor e o git reais e só o lançador falso. O plano é o exemplo genérico `src/plano/fixtures/monitor`.
//! Passa de 300 linhas (teto 500): o fluxo do contrato é uma história só, e a fatia é dona deste único arquivo. `ui/src/plano/fixtures/*.json` saem daqui, com o que varia a cada execução trocado por valor fixo.
use super::motor::{passo, Ambiente, Relogio};
use crate::contexto::Contexto;
use crate::dados::planejar::ExecucaoLinha;
use crate::http::pedido::Resposta;
use crate::http::rotas::despachar;
use crate::http::tests::{estado, json_de, pedido};
use crate::http::Estado;
use crate::plano::pacote::{ler_pasta, Pacote};
use crate::tempo::agora_ms;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

static SEQUENCIA: AtomicUsize = AtomicUsize::new(0);

type Lancados = Arc<Mutex<Vec<Vec<OsString>>>>;

struct Cena {
    e: Estado,
    token: String,
    base: PathBuf,
    amb: Ambiente,
    lancados: Lancados,
}

impl Drop for Cena {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
        let _ = std::fs::remove_dir_all(self.e.mcp.pasta_figuras());
    }
}

fn git(pasta: &Path, args: &[&str]) -> String {
    let saida = Command::new("git").args(args).current_dir(pasta).env("GIT_TERMINAL_PROMPT", "0").output().unwrap();
    assert!(saida.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&saida.stderr));
    String::from_utf8_lossy(&saida.stdout).trim().to_string()
}

fn commit(pasta: &Path, nome: &str) {
    std::fs::write(pasta.join(nome), "feito\n").unwrap();
    git(pasta, &["add", "-A"]);
    git(pasta, &["commit", "-q", "-m", nome]);
}

fn ok(r: Resposta) -> Value {
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)
}

fn recusa(r: Resposta) -> (u16, String) {
    (r.status, json_de(&r)["codigo"].as_str().unwrap_or("").to_string())
}

/// A pasta do exemplo; `Cena::anterior` monta outra com o `anterior.json` no lugar do `plano.json`.
fn monitor() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plano/fixtures/monitor")
}

fn acao<'a>(folha: &'a Value, r: &str) -> &'a Value {
    folha["projetos"].as_array().unwrap().iter().flat_map(|p| p["acoes"].as_array().unwrap()).find(|a| a["ref"] == r).unwrap_or_else(|| panic!("sem a ação {r}"))
}

fn plano<'a>(a: &'a Value, r: &str) -> &'a Value {
    a["planos"].as_array().unwrap().iter().find(|q| q["ref"] == r).unwrap_or_else(|| panic!("sem o plano {r}"))
}

fn refs(v: &Value) -> Vec<&str> {
    v.as_array().unwrap().iter().map(|x| x.as_str().unwrap_or_else(|| x["ref"].as_str().unwrap())).collect()
}

impl Cena {
    fn nova() -> Cena {
        let base = std::env::temp_dir().join(format!("blueprint-fluxo-{}-{}", std::process::id(), SEQUENCIA.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let e = estado(false);
        e.dados.lock().unwrap().gravar_tudo(&Contexto::local(), &json!({"empresa": "Exemplo", "blueprints": []})).unwrap();
        let token = e.mcp.ligar_janela();
        let lancados: Lancados = Default::default();
        let anotados = Arc::clone(&lancados);
        let lancar = Box::new(move |_: &Path, _: &Path, args: &[OsString]| {
            anotados.lock().unwrap().push(args.to_vec());
            Ok("falso")
        });
        let amb = Ambiente { lancar, git: "git".into(), pasta_copias: base.join("copias") };
        Cena { e, token, base, amb, lancados }
    }

    fn repo(&self) -> PathBuf {
        let repo = self.base.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        for (chave, valor) in [("user.name", "Teste"), ("user.email", "teste@exemplo.invalid"), ("commit.gpgsign", "false"), ("core.autocrlf", "false")] {
            git(&repo, &["config", chave, valor]);
        }
        commit(&repo, "base.txt");
        repo
    }

    fn anterior(&self) -> Pacote {
        let pasta = self.base.join("anterior");
        std::fs::create_dir_all(pasta.join("figuras")).unwrap();
        std::fs::copy(monitor().join("anterior.json"), pasta.join("plano.json")).unwrap();
        std::fs::copy(monitor().join("figuras/painel.svg"), pasta.join("figuras/painel.svg")).unwrap();
        ler_pasta(&pasta).unwrap()
    }

    fn post(&self, rota: &str, corpo: Value) -> Resposta {
        despachar(&self.e, &pedido("POST", &format!("/api/mcp/{rota}"), &[], Some(corpo)))
    }

    fn gesto(&self, rota: &str, corpo: Value) -> Resposta {
        despachar(&self.e, &pedido("POST", &format!("/api/mcp/gesto/{rota}"), &[("X-Blueprint-Gesto", &self.token)], Some(corpo)))
    }

    fn get(&self, rota: &str) -> Value {
        ok(despachar(&self.e, &pedido("GET", &format!("/api/mcp/{rota}"), &[], None)))
    }

    fn folha(&self, bp: &str) -> Value {
        self.get(&format!("planejar/folha?workspace=sm&blueprint={bp}"))
    }

    fn agentes(&self) -> Value {
        self.get("planejar/agentes?workspace=sm")
    }

    /// O que `salvar_plano` compõe: abrir, figuras, escrever por ação, fechar.
    fn salvar(&self, p: &Pacote, bp: Option<&str>, pasta: Option<&Path>) -> (String, Value) {
        let acoes = |pr: &Value| pr["acoes"].as_array().unwrap().iter().map(|a| json!({"id": a["id"], "nome": a["nome"]})).collect::<Vec<_>>();
        let projetos: Vec<Value> = p.plano["projetos"].as_array().unwrap().iter().map(|pr| json!({"acoes": acoes(pr)})).collect();
        let mut corpo = json!({"workspaceId": "sm", "blueprint": p.plano["blueprint"], "resumo": p.plano["resumo"], "origem": "salvar", "agente": "exemplo", "projetos": projetos});
        if let Some(b) = bp {
            corpo["blueprintId"] = json!(b);
        }
        if let Some(c) = pasta {
            corpo["pasta"] = json!(c.to_string_lossy());
        }
        let aberta = ok(self.post("plano/abrir", corpo));
        let (escrita, bp_id) = (aberta["escritaId"].clone(), aberta["blueprintId"].as_str().unwrap().to_string());
        for (caminho, bytes) in &p.figuras {
            ok(self.post("plano/figura", json!({"escritaId": escrita, "caminho": caminho, "tipo": "svg", "svg": String::from_utf8_lossy(bytes)})));
        }
        for (i, a) in p.plano["projetos"][0]["acoes"].as_array().unwrap().iter().enumerate() {
            let mut pedaco = json!({"escritaId": escrita, "acao": a});
            if i == 0 {
                pedaco["decisoes"] = p.plano.get("decisoes").cloned().unwrap_or_else(|| json!([]));
                pedaco["perguntas"] = p.plano.get("perguntas").cloned().unwrap_or_else(|| json!([]));
            }
            ok(self.post("plano/escrever", pedaco));
        }
        let fechado = ok(self.post("plano/fechar", json!({"escritaId": escrita})));
        (bp_id, fechado)
    }

    /// Um tique do motor, sem carência.
    fn tique(&self) {
        let agora = agora_ms();
        let mut r = Relogio { partida_ms: agora - 10_000_000, ultimo_ms: agora - 2_000 };
        passo(&self.amb, &self.e.dados, &self.e.mcp, agora, &mut r);
    }

    fn execucao(&self, r: &str) -> ExecucaoLinha {
        self.e.dados.lock().unwrap().execucoes_do_ws("sm").unwrap().into_iter().find(|x| x.plano_ref == r).unwrap_or_else(|| panic!("sem execução de {r}"))
    }

    /// O agente faz um commit na cópia e conclui com evidência.
    fn agente_conclui(&self, acao: &str, r: &str, segredo: &str) {
        commit(Path::new(&self.execucao(r).pasta_trabalho), &format!("do-{r}.txt"));
        ok(self.post("evento", json!({"tipo": "concluir", "texto": "teste passou", "workspaceId": "sm", "acaoId": acao, "planoId": r, "blueprintId": self.execucao(r).blueprint_id, "execucaoSegredo": segredo})));
    }

    fn partida(&self, r: &str) -> Value {
        ok(self.post("execucao/partida", json!({"id": self.execucao(r).id})))
    }

    fn arvore(&self, bp: &str) -> Value {
        let tudo = self.e.dados.lock().unwrap().ler_tudo(&Contexto::local()).unwrap().clone();
        tudo["blueprints"].as_array().unwrap().iter().find(|b| b["id"] == bp).unwrap().clone()
    }
}

fn eh_id(s: &str) -> bool {
    let hex = s.strip_prefix('p').unwrap_or(s);
    hex.len() == 16 && hex.bytes().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f'))
}

fn juntar_ids(v: &Value, ids: &mut BTreeMap<String, String>) {
    match v {
        Value::String(s) if eh_id(s) && !ids.contains_key(s) => {
            let novo = format!("id-{}", ids.len() + 1);
            ids.insert(s.clone(), novo);
        }
        Value::Array(l) => l.iter().for_each(|x| juntar_ids(x, ids)),
        Value::Object(o) => o.values().for_each(|x| juntar_ids(x, ids)),
        _ => {}
    }
}

fn trocar(v: &mut Value, ids: &BTreeMap<String, String>) {
    match v {
        Value::String(s) => ids.iter().for_each(|(de, para)| *s = s.replace(de, para)),
        Value::Array(l) => l.iter_mut().for_each(|x| trocar(x, ids)),
        Value::Object(o) => {
            for (k, x) in o.iter_mut() {
                match k.as_str() {
                    "versaoDados" => *x = json!(1),
                    "revisao" if x.is_number() => *x = json!(1),
                    "em" => *x = json!(1_790_000_000_000_i64),
                    "paradaMs" => *x = json!(0),
                    _ => trocar(x, ids),
                }
            }
        }
        _ => {}
    }
}

/// Grava o JSON para os testes da interface (`plano::modelo`): sem nada desta execução, e só se mudou.
fn gravar_exemplo(nome: &str, v: &Value, proibidos: &[&str]) {
    let (mut limpo, mut ids) = (v.clone(), BTreeMap::new());
    juntar_ids(&limpo, &mut ids);
    trocar(&mut limpo, &ids);
    let texto = serde_json::to_string_pretty(&limpo).unwrap() + "\n";
    for p in proibidos {
        assert!(!texto.contains(p), "{nome} não pode trazer {p}");
    }
    let pasta = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/src/plano/fixtures");
    std::fs::create_dir_all(&pasta).unwrap();
    let destino = pasta.join(format!("{nome}.json"));
    if std::fs::read_to_string(&destino).ok().as_deref() != Some(texto.as_str()) {
        let tmp = pasta.join(format!(".{nome}.{}.tmp", std::process::id()));
        std::fs::write(&tmp, &texto).unwrap();
        std::fs::rename(&tmp, &destino).unwrap();
    }
}

#[test]
fn exemplo_monitor_ponta_a_ponta() {
    let c = Cena::nova();
    let repo = c.repo();

    // salvar: a primeira versão entra por inteiro em "novas" e espera o gesto
    let (bp, fechado) = c.salvar(&c.anterior(), None, Some(&repo));
    assert_eq!((refs(&fechado["novas"]), refs(&fechado["colocadas"])), (vec!["1.1", "1.2"], vec![]));
    assert!(c.arvore(&bp)["acoes"].as_array().unwrap().is_empty(), "salvar não coloca sozinho");
    assert_eq!(recusa(c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": bp, "planos": ["1.1.1"]}))).1, "plano-nao-encontrado", "fora da árvore não inicia");

    // colocar: as duas entram na árvore, e nada mais falta na versão
    let chegou = c.get(&format!("planejar/chegou?workspace=sm&blueprint={bp}"));
    assert_eq!((chegou["numero"].as_i64(), chegou["novas"][0]["podeColocar"].as_bool()), (Some(1), Some(true)));
    assert_eq!(ok(c.gesto("colocar", json!({"workspaceId": "sm", "blueprintId": bp, "numero": 1, "acoes": ["1.1", "1.2"]})))["resolvida"], true);
    let folha = c.folha(&bp);
    assert!(folha["pendente"].is_null());
    assert_eq!((folha["semPasta"].as_bool(), plano(acao(&folha, "1.1"), "1.1.1")["situacao"].as_str()), (Some(false), Some("nao")));
    let figura = acao(&folha, "1.2")["detalhes"][0]["figura"].clone();
    assert_eq!(figura["tipo"], "svg");
    let servida = despachar(&c.e, &pedido("GET", &format!("/api/mcp/planejar/figura?hash={}", figura["hash"].as_str().unwrap()), &[], None));
    assert_eq!((servida.status, servida.cabecalho("x-content-type-options")), (200, Some("nosniff")));

    // marcar: só o gesto cria execução; sem ele o motor não lança nada
    c.tique();
    assert!(c.lancados.lock().unwrap().is_empty());
    let marcar = |planos: Value| c.gesto("marcar", json!({"workspaceId": "sm", "blueprintId": bp, "planos": planos}));
    ok(marcar(json!(["1.1.1", "1.1.2", "1.2.1"])));

    // lançar: um por tique, e o dependente espera
    c.tique();
    c.tique();
    let primeiro = c.execucao("1.1.1");
    assert_eq!((primeiro.situacao.as_str(), primeiro.ramo.as_str(), primeiro.base.as_str()), ("rodando", "agente/1.1.1", "HEAD"));
    assert_eq!(*c.lancados.lock().unwrap(), vec![vec![OsString::from("agente-plano"), OsString::from(&primeiro.id)]], "um só: os outros dependem dele, e o texto do plano nunca vai na linha");
    assert_eq!(c.execucao("1.1.2").situacao, "marcada");
    let folha = c.folha(&bp);
    assert_eq!((plano(acao(&folha, "1.1"), "1.1.1")["situacao"].as_str(), plano(acao(&folha, "1.1"), "1.1.2")["situacao"].as_str()), (Some("trabalhando"), Some("dependencia")));

    let partida = c.partida("1.1.1");
    assert!(partida["prompt"].as_str().unwrap().contains("Ler os medidores pela rede local"));
    assert_eq!(partida["pasta"].as_str(), Some(primeiro.pasta_trabalho.as_str()));
    assert_eq!(recusa(c.post("execucao/partida", json!({"id": primeiro.id}))), (409, "ja-partiu".to_string()));
    let segredo = partida["segredo"].as_str().unwrap().to_string();
    assert_eq!(ok(c.post("execucao/sinal", json!({"id": primeiro.id, "segredo": segredo})))["parar"], false);

    // concluir: sem o segredo da tentativa é recusado; com ele grava no servidor, sem passar pela fila
    let concluir_sem = json!({"tipo": "concluir", "texto": "teste passou", "workspaceId": "sm", "blueprintId": bp, "acaoId": "1.1", "planoId": "1.1.1"});
    assert_eq!(recusa(c.post("evento", concluir_sem)), (403, "nao-e-seu-plano".to_string()));
    c.agente_conclui("1.1", "1.1.1", &segredo);
    assert_eq!(c.execucao("1.1.1").situacao, "pronta");
    assert_eq!(c.arvore(&bp)["acoes"][0]["planos"][0]["feito"], true);
    ok(c.post("execucao/fim", json!({"id": primeiro.id, "segredo": segredo, "codigo": 0})));
    assert_eq!(c.execucao("1.1.1").situacao, "pronta", "o fim do invólucro não desfaz");

    // liberar dependente: parte do ramo pronto, que não está no principal
    c.tique();
    let segundo = c.execucao("1.1.2");
    assert_eq!((segundo.situacao.as_str(), segundo.base.as_str()), ("rodando", "agente/1.1.1"));
    assert!(Path::new(&segundo.pasta_trabalho).join("do-1.1.1.txt").exists());
    assert!(!repo.join("do-1.1.1.txt").exists());
    let segredo = c.partida("1.1.2")["segredo"].as_str().unwrap().to_string();
    c.agente_conclui("1.1", "1.1.2", &segredo);

    c.tique();
    let terceiro = c.execucao("1.2.1");
    assert_eq!((terceiro.situacao.as_str(), terceiro.base.as_str()), ("rodando", "agente/1.1.2"));
    let segredo_vivo = c.partida("1.2.1")["segredo"].as_str().unwrap().to_string();
    assert_eq!(ok(marcar(json!(["1.2.2"])))["avisos"], json!([]), "a dependência já anda");

    // salvar por cima: o que já está na árvore não muda, e o diff espera
    let (bp2, fechado) = c.salvar(&ler_pasta(&monitor()).unwrap(), Some(&bp), None);
    assert_eq!(bp2, bp);
    assert_eq!((fechado["numero"].as_i64(), refs(&fechado["novas"]), refs(&fechado["alteradas"]), fechado["iguais"].as_u64()), (Some(2), vec!["1.3"], vec!["1.2"], Some(1)));
    assert_eq!(fechado["alteradas"][0]["resumo"], json!({"mais": 1, "menos": 1, "alterados": 1, "detalhes": 1, "outros": 0}));
    assert_eq!(c.arvore(&bp)["acoes"][1]["planos"][0]["texto"], "Somar as leituras", "a árvore só muda ao aprovar");

    // diff: dentro da ação, com as palavras e a ordem do contrato
    let folha = c.folha(&bp);
    let diff = acao(&folha, "1.2")["diff"].clone();
    let tipos: Vec<&str> = diff["linhas"].as_array().unwrap().iter().map(|l| l["tipo"].as_str().unwrap()).collect();
    assert_eq!(tipos, ["alterado", "igual", "novo", "removido"]);
    assert_eq!(diff["numero"].as_i64(), Some(2));
    assert_eq!(diff["linhas"][0]["palavras"], json!([{"t": "igual", "s": "Somar as leituras"}, {"t": "mais", "s": " por sala"}]));
    assert_eq!(diff["detalhesNovos"][0]["id"], "02");
    assert_eq!(acao(&folha, "1.2")["numeros"]["detalhes"].as_u64(), Some(2), "o detalhe que chega conta na ação");
    // o detalhe que chega se revisa na ação, antes de aprovar o diff
    let revisar_det = |estado: &str| c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": bp, "peca": "det:1.2/02", "numero": 2, "hash": diff["detalhesNovos"][0]["hash"], "estado": estado}));
    ok(revisar_det("aprovada"));
    assert_eq!(acao(&c.folha(&bp), "1.2")["diff"]["detalhesNovos"][0]["revisao"]["estado"], "aprovada");
    ok(revisar_det("pendente"));
    assert!(acao(&folha, "1.1")["diff"].is_null());
    assert_eq!((acao(&folha, "1.3")["nova"].as_bool(), plano(acao(&folha, "1.3"), "1.3.1")["situacao"].as_str()), (Some(true), Some("revisao")));
    assert_eq!((folha["pendente"]["numero"].as_i64(), folha["numeros"]["diffs"].as_i64(), folha["numeros"]["novas"].as_i64()), (Some(2), Some(1), Some(1)));

    // o resto do blueprint entrou ao fechar: decisão e pergunta já se decidem
    let decisao = folha["decisoes"][0].clone();
    ok(c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": bp, "peca": "d:banco", "numero": 2, "hash": decisao["hash"], "estado": "aprovada"})));
    ok(c.gesto("responder", json!({"workspaceId": "sm", "blueprintId": bp, "pergunta": "P1", "resposta": "15 min"})));
    let folha = c.folha(&bp);
    assert_eq!((folha["decisoes"][0]["revisao"]["estado"].as_str(), folha["perguntas"][0]["resposta"].as_str(), folha["perguntas"][0]["recomendada"].as_str()), (Some("aprovada"), Some("15 min"), Some("15 min")));

    let agentes = c.agentes();
    let situacoes: Vec<(&str, &str)> = agentes["planos"].as_array().unwrap().iter().map(|q| (q["ref"].as_str().unwrap(), q["situacao"].as_str().unwrap())).collect();
    assert_eq!(situacoes, [("1.1.1", "pronto"), ("1.1.2", "pronto"), ("1.2.1", "trabalhando"), ("1.2.2", "dependencia")]);
    assert_eq!(agentes["numeros"], json!({"trabalhando": 1, "fila": 0, "esperando": 1, "parados": 0, "prontos": 2}));
    assert_eq!((agentes["planos"][0]["podeIntegrar"].as_bool(), agentes["planos"][0]["destino"].as_str()), (Some(true), Some("main")));

    let chegou = c.get(&format!("planejar/chegou?workspace=sm&blueprint={bp}"));
    assert_eq!((refs(&chegou["novas"]), refs(&chegou["alteradas"]), chegou["iguais"].as_u64()), (vec!["1.3"], vec!["1.2"], Some(1)));

    let (base, token) = (c.base.to_string_lossy().into_owned(), c.token.clone());
    let proibidos = [base.as_str(), token.as_str(), segredo_vivo.as_str(), segredo.as_str()];
    gravar_exemplo("folha", &folha, &proibidos);
    gravar_exemplo("agentes", &agentes, &proibidos);
    gravar_exemplo("chegou", &chegou, &proibidos);

    // aprovar: com agente no plano alterado não entra; hash velho não entra; depois de concluir, entra e reabre o feito
    let revisar = |hash: &Value| c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": bp, "peca": "diff:1.2", "numero": 2, "hash": hash, "estado": "aprovada"}));
    assert_eq!(recusa(revisar(&diff["hash"])), (409, "rodando".to_string()));
    assert_eq!(recusa(revisar(&json!("0"))), (409, "plano-mudou".to_string()));
    c.agente_conclui("1.2", "1.2.1", &segredo_vivo);
    let diff = acao(&c.folha(&bp), "1.2")["diff"].clone();
    assert_eq!(diff["linhas"][0]["situacao"], "reabre");
    ok(c.gesto("cancelar", json!({"execucaoId": c.execucao("1.2.2").id})));
    let aprovado = ok(revisar(&diff["hash"]));
    assert_eq!((refs(&aprovado["reabertos"]), aprovado["resolvida"].as_bool()), (vec!["1.2.1"], Some(false)));

    let arvore = c.arvore(&bp);
    let a12 = &arvore["acoes"][1];
    let planos: Vec<&str> = a12["planos"].as_array().unwrap().iter().map(|q| q["ref"].as_str().unwrap()).collect();
    assert_eq!((planos, a12["planos"][0]["texto"].as_str(), a12["planos"][0]["feito"].as_bool()), (vec!["1.2.1", "1.2.2", "1.2.3"], Some("Somar as leituras por sala"), Some(false)));
    assert_eq!(a12["detalhes"].as_array().unwrap().len(), 2);
    assert_eq!((arvore["acoes"][0]["planos"][0]["feito"].as_bool(), arvore["acoes"][0]["planos"][1]["feito"].as_bool()), (Some(true), Some(true)), "a ação igual não perde o feito");

    assert_eq!(ok(c.gesto("colocar", json!({"workspaceId": "sm", "blueprintId": bp, "numero": 2, "acoes": ["1.3"]})))["resolvida"], true);
    let folha = c.folha(&bp);
    assert!(folha["pendente"].is_null() && acao(&folha, "1.2")["diff"].is_null());
    assert_eq!(plano(acao(&folha, "1.3"), "1.3.1")["situacao"], "nao");

    // integrar: confere o ramo, leva ao principal, some com a cópia e o ramo
    let agentes = c.agentes();
    let integraveis: Vec<&str> = agentes["planos"].as_array().unwrap().iter().filter(|q| q["podeIntegrar"] == true).map(|q| q["ref"].as_str().unwrap()).collect();
    assert_eq!(integraveis, ["1.1.1", "1.1.2", "1.2.1"]);
    let id = c.execucao("1.1.1").id;
    assert_eq!(recusa(c.gesto("integrar", json!({"execucaoId": id, "ramoEsperado": "outro"}))), (409, "ramo-mudou".to_string()));
    for r in integraveis {
        let x = c.execucao(r);
        let feito = ok(c.gesto("integrar", json!({"execucaoId": x.id, "ramoEsperado": "main"})));
        assert_eq!((feito["ramo"].as_str(), feito["avisos"].as_array().map(Vec::len)), (Some(x.ramo.as_str()), Some(0)));
        assert!(repo.join(format!("do-{r}.txt")).exists(), "{r} no principal");
        assert!(!Path::new(&x.pasta_trabalho).exists(), "a cópia de {r} saiu");
    }
    assert_eq!(recusa(c.gesto("integrar", json!({"execucaoId": id, "ramoEsperado": "main"}))).1, "nao-integravel");
    assert_eq!(git(&repo, &["branch", "--list", "agente/*"]), "");
    assert_eq!(git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    assert!(c.agentes()["planos"].as_array().unwrap().is_empty());
}

#[test]
fn segunda_versao_por_escrita() {
    let c = Cena::nova();

    // a primeira escrita: as ações aparecem aguardando e entram na árvore ao fechar
    let abrir = json!({"workspaceId": "sm", "blueprint": "Reforma da sala", "origem": "escrita", "agente": "orquestrador", "projetos": [{"acoes": [{"id": "1", "nome": "Medir"}, {"id": "2", "nome": "Pintar"}]}]});
    let aberta = ok(c.post("plano/abrir", abrir));
    let (bp, escrita) = (aberta["blueprintId"].as_str().unwrap().to_string(), aberta["escritaId"].clone());
    assert_eq!((refs(&aberta["aguardando"]), aberta["numero"].as_i64()), (vec!["1", "2"], Some(1)));
    let um = json!({"id": "1", "planos": [{"id": "1.1", "texto": "Medir as paredes", "area": "obra"}]});
    let dois = json!({"id": "2", "texto": "Duas demãos.", "planos": [{"id": "2.1", "texto": "Pintar as paredes", "area": "obra"}]});
    assert_eq!(refs(&ok(c.post("plano/escrever", json!({"escritaId": escrita, "acao": um})))["faltam"]), vec!["2"]);
    let folha = c.folha(&bp);
    assert_eq!((acao(&folha, "2")["escrita"].as_str(), folha["escrita"]["agente"].as_str(), folha["semPasta"].as_bool()), (Some("aguardando"), Some("orquestrador"), Some(true)));
    ok(c.post("plano/escrever", json!({"escritaId": escrita, "acao": dois})));
    let fechado = ok(c.post("plano/fechar", json!({"escritaId": escrita})));
    assert_eq!(refs(&fechado["colocadas"]), vec!["1", "2"]);
    let folha = c.folha(&bp);
    assert!(folha["pendente"].is_null() && folha["escrita"].is_null());

    // um plano da ação que a segunda versão não toca fica feito
    let q11 = plano(acao(&folha, "1"), "1.1")["id"].as_str().unwrap().to_string();
    {
        let mut d = c.e.dados.lock().unwrap();
        d.concluir_por_execucao(&q11, "medido", "2026-11-02T00:00:00.000Z").unwrap();
    }

    // a segunda: abrir, escrever só uma ação, fechar. A árvore espera
    let aberta = ok(c.post("plano/abrir", json!({"workspaceId": "sm", "blueprintId": bp, "origem": "escrita", "agente": "orquestrador"})));
    let escrita = aberta["escritaId"].clone();
    assert_eq!((aberta["numero"].as_i64(), aberta["continuacao"].as_bool(), aberta["aguardando"].as_array().map(Vec::len)), (Some(2), Some(false), Some(0)));
    let mudada = json!({"id": "2", "planos": [{"id": "2.1", "texto": "Pintar as paredes de branco", "area": "obra"}, {"id": "2.2", "texto": "Trocar o rodapé", "area": "obra"}]});
    ok(c.post("plano/escrever", json!({"escritaId": escrita, "acao": mudada})));
    let fechado = ok(c.post("plano/fechar", json!({"escritaId": escrita})));
    assert_eq!((fechado["numero"].as_i64(), refs(&fechado["alteradas"]), fechado["iguais"].as_u64(), fechado["removidas"].as_array().map(Vec::len)), (Some(2), vec!["2"], Some(1), Some(0)), "a ação não mandada segue igual, sem virar remoção");
    assert_eq!(c.arvore(&bp)["acoes"][1]["planos"][0]["texto"], "Pintar as paredes");

    // aprovar, com o hash e o número que a tela mostrou
    let folha = c.folha(&bp);
    let diff = acao(&folha, "2")["diff"].clone();
    assert_eq!((folha["pendente"]["numero"].as_i64(), diff["numero"].as_i64()), (Some(2), Some(2)));
    let antes = folha["versaoDados"].as_u64().unwrap();
    let aprovado = ok(c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": bp, "peca": "diff:2", "numero": 2, "hash": diff["hash"], "estado": "aprovada"})));
    assert_eq!(aprovado["resolvida"], true);

    let arvore = c.arvore(&bp);
    assert_eq!((arvore["acoes"][1]["planos"][0]["texto"].as_str(), arvore["acoes"][1]["planos"][1]["texto"].as_str(), arvore["acoes"][1]["obs"].as_str()), (Some("Pintar as paredes de branco"), Some("Trocar o rodapé"), Some("Duas demãos.")));
    assert_eq!(arvore["acoes"][0]["planos"][0]["feito"].as_bool(), Some(true), "a ação não tocada fica como estava");
    let folha = c.folha(&bp);
    assert!(folha["pendente"].is_null() && acao(&folha, "2")["diff"].is_null());
    assert!(folha["versaoDados"].as_u64().unwrap() > antes, "aprovar muda a árvore: a versão sobe");
    let respostas = c.get(&format!("plano/respostas?workspace=sm&blueprint={bp}"));
    assert_eq!(respostas["revisoes"], json!([{"peca": "diff:2", "estado": "aprovada", "observacao": "", "numero": 2}]));
}
