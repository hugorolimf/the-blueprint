//! Testes de `POST /api/mcp/apresentar` (o botão Gerar com IA): lançador falso (nenhum terminal abre), o prompt nos dois idiomas, as recusas e a entrega a uma sessão aberta.
use super::hooks::{sessao, travado};
use super::hooks_tests::{dados, mcp};
use super::iniciar::em_sessao_de;
use super::pedido_apresentacao::{apresentar, prompt};
use crate::contexto::Contexto;
use crate::idioma::{com, Idioma};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

fn pasta_real(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-apresentar-{nome}-{}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn corpo(publico: &str, nivel: &str) -> Value {
    json!({"workspaceId": "sm", "blueprintId": "p1", "publico": publico, "nivel": nivel})
}

#[test]
fn o_prompt_leva_publico_nivel_e_estimativa_nos_dois_idiomas() {
    let en = com(Idioma::En, || prompt("Portal", "sm", "p1", "cliente", "padrao"));
    for trecho in ["«Portal»", "workspace_id sm, blueprint_id p1", "the client (a project plan, without the technical development)", "conteudo.publico \"cliente\"", "conteudo.nivel \"padrao\"", "estimate about 1 to 3 h", "do not ask for them again", "guia_de_planejamento(\"apresentar\")", "anexar_apresentacao"] {
        assert!(en.contains(trecho), "falta em inglês: {trecho}\n{en}");
    }
    let pt = com(Idioma::Pt, || prompt("Portal", "sm", "p1", "interno", "completo"));
    for trecho in ["uso interno (a visão técnica completa)", "estimativa cerca de 2 a 4 h", "não pergunte de novo", "anexar_apresentacao"] {
        assert!(pt.contains(trecho), "falta em português: {trecho}\n{pt}");
    }
    for (nivel, tempo) in [("rapido", "up to about 1 h"), ("epico", "6 h or more")] {
        assert!(com(Idioma::En, || prompt("P", "a", "b", "cliente", nivel)).contains(&format!("estimate {tempo}")), "{nivel}");
    }
    assert!(!en.contains(';') && !pt.contains(';'), "o Windows Terminal leria o ; como outro comando");
}

#[test]
fn abre_o_terminal_na_pasta_do_blueprint_com_o_prompt_como_um_so_argumento() {
    let d = dados();
    let pasta = pasta_real("abre");
    travado(&d).vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "sm", "p1").unwrap();
    let chamadas: RefCell<Vec<(PathBuf, String)>> = RefCell::new(vec![]);
    let falso = |p: &Path, e: &Path, a: &str| {
        assert_eq!(e, std::env::current_exe().unwrap());
        chamadas.borrow_mut().push((p.into(), a.into()));
        Ok("wt")
    };
    let (status, r) = com(Idioma::En, || apresentar(&d, &corpo("cliente", "rapido"), &falso)).unwrap();
    assert_eq!((status, r["ok"].clone(), r["terminal"].clone()), (200, json!(true), json!("wt")), "{r}");
    let c = chamadas.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].1, com(Idioma::En, || prompt("Portal", "sm", "p1", "cliente", "rapido")), "o mesmo texto que o contrato descreve");
}

#[test]
fn as_recusas_nao_abrem_terminal() {
    let d = dados();
    let nunca = |_: &Path, _: &Path, _: &str| -> Result<&'static str, String> { panic!("não podia abrir terminal") };
    assert_eq!(apresentar(&d, &json!({}), &nunca).unwrap_err().status, 400);
    assert_eq!(apresentar(&d, &json!({"workspaceId": "sm", "blueprintId": "p\u{0}1", "publico": "cliente", "nivel": "padrao"}), &nunca).unwrap_err().status, 400, "id com controle");
    for (publico, nivel, codigo) in [("todos", "padrao", "publico-invalido"), ("cliente", "gigante", "nivel-invalido"), ("", "", "publico-invalido")] {
        let e = apresentar(&d, &corpo(publico, nivel), &nunca).unwrap_err();
        assert_eq!((e.status, e.codigo), (400, codigo), "{publico}/{nivel}");
    }
    assert_eq!(apresentar(&d, &json!({"workspaceId": "nao", "blueprintId": "p1", "publico": "cliente", "nivel": "padrao"}), &nunca).unwrap_err().status, 404);
    assert_eq!(apresentar(&d, &json!({"workspaceId": "sm", "blueprintId": "nao", "publico": "cliente", "nivel": "padrao"}), &nunca).unwrap_err().status, 404);
    let (status, r) = apresentar(&d, &corpo("cliente", "padrao"), &nunca).unwrap();
    assert_eq!((status, r["codigo"].clone()), (409, json!("sem-pasta")), "blueprint sem pasta ligada");
    let sem_terminal = |_: &Path, _: &Path, _: &str| -> Result<&'static str, String> { Err("não há terminal".into()) };
    let pasta = pasta_real("sem-terminal");
    travado(&d).vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "sm", "p1").unwrap();
    let (status, r) = apresentar(&d, &corpo("interno", "epico"), &sem_terminal).unwrap();
    assert_eq!((status, r["codigo"].clone()), (501, json!("sem-terminal")), "epico vale quando vem escrito");
}

#[test]
fn o_pedido_tambem_vai_a_uma_sessao_aberta_ou_so_devolve_o_texto_para_copiar() {
    let (m, d) = (mcp(), dados());
    let pasta = pasta_real("sessao");
    let cwd = pasta.to_str().unwrap().to_string();
    travado(&d).vincular_pasta(&Contexto::local(), &cwd, "sm", "p1").unwrap();
    sessao(&m, &d, &json!({"session_id": "s1", "cwd": cwd})).unwrap();
    let _guarda = m.abrir_canal("abc123", &cwd).unwrap();
    let eventos = m.assinar();
    let via = |c: &Value| com(Idioma::En, || em_sessao_de(&m, &d, c, &|d, c, l| apresentar(d, c, l))).unwrap();
    let mut pedido = corpo("cliente", "padrao");
    pedido["sessaoId"] = json!("abc123");
    let (status, r) = via(&pedido);
    assert_eq!((status, r["terminal"].clone()), (200, json!("canal")), "{r}");
    let texto = r["prompt"].as_str().unwrap().to_string();
    assert!(texto.starts_with("Build the presentation of blueprint «Portal»"), "{texto}");
    let e = eventos.try_recv().unwrap();
    assert_eq!(e.nome, "iniciar-acao", "o mesmo evento do pedido de iniciar: só o canal endereçado o repassa como ordem do usuário");
    assert_eq!(e.dados.unwrap(), json!({"workspaceId": "sm", "blueprintId": "p1", "canalId": "abc123", "texto": texto}));
    pedido["sessaoId"] = json!("s1");
    pedido["semCanal"] = json!(true);
    let (status, r) = via(&pedido);
    assert_eq!((status, r["terminal"].clone()), (200, json!("copiar")));
    assert!(r["prompt"].as_str().unwrap().contains("anexar_apresentacao") && eventos.try_recv().is_err(), "copiar não emite evento");
    pedido["sessaoId"] = json!("nao-existe");
    pedido["semCanal"] = json!(false);
    let (status, r) = via(&pedido);
    assert_eq!((status, r["codigo"].clone()), (410, json!("sessao-caiu")));
}
