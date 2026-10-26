//! O repositório GitHub do blueprint (`blueprints[].repositorio`, docs/repositorio-github.md) vai em `extra`: sem migração, sincroniza pelas
//! operações por item como qualquer campo do blueprint, avisa quando duas pessoas trocam ao mesmo tempo e desligar volta ao dado de antes.
use crate::contexto::Contexto;
use crate::dados::{Autor, Dados, Op};
use crate::erro::Resultado;
use serde_json::{json, Map, Value};

fn dados() -> Dados {
    let mut d = Dados::em_memoria().unwrap();
    d.gravar_tudo(&Contexto::local(), &json!({"empresa": "E", "blueprints": [{"id": "b1", "nome": "B", "acoes": []}]})).unwrap();
    d
}

fn pessoa(id: &str) -> Autor {
    Autor { id: id.into(), nome: id.to_uppercase() }
}

fn alterar(campos: Value, antes: Value) -> Op {
    let obj = |v: Value| -> Map<String, Value> { v.as_object().unwrap().clone() };
    Op::Alterar { tipo: "blueprint".into(), id: "b1".into(), campos: obj(campos), antes: obj(antes) }
}

fn livre(_: bool, _: &str, _: Option<&str>) -> Resultado<()> {
    Ok(())
}

fn blueprint(d: &mut Dados) -> Value {
    d.ler_tudo(&Contexto::local()).unwrap()["blueprints"][0].clone()
}

#[test]
fn ligar_trocar_e_desligar_o_repositorio_pelas_operacoes() {
    let mut d = dados();
    let original = blueprint(&mut d);
    let url = |u: &str| json!({"repositorio": {"url": u}});
    let s = d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar(url("https://github.com/dono/repo"), json!({"repositorio": null}))], &livre).unwrap();
    assert!(s.conflitos.is_empty() && s.ignoradas.is_empty());
    assert_eq!(blueprint(&mut d)["repositorio"], json!({"url": "https://github.com/dono/repo"}));
    // a leitura do agente (ler_blueprint) devolve o que foi guardado
    let tudo = d.ler_tudo(&Contexto::local()).unwrap().clone();
    assert_eq!(crate::mcp::acoes::blueprint(&tudo, "sm", "b1", None).unwrap()["blueprint"]["repositorio"], json!({"url": "https://github.com/dono/repo"}));
    d.aplicar_operacoes(&pessoa("ana"), 102, 0, &[alterar(url("https://github.com/dono/outro"), url("https://github.com/dono/repo"))], &livre).unwrap();
    assert_eq!(blueprint(&mut d)["repositorio"]["url"], "https://github.com/dono/outro");
    d.aplicar_operacoes(&pessoa("ana"), 103, 0, &[alterar(json!({"repositorio": null}), url("https://github.com/dono/outro"))], &livre).unwrap();
    assert_eq!(blueprint(&mut d), original, "desligar volta ao dado de antes, sem sobra do campo");
}

#[test]
fn duas_pessoas_trocando_o_repositorio_ao_mesmo_tempo_levam_o_aviso() {
    let mut d = dados();
    let url = |u: &str| json!({"repositorio": {"url": u}});
    d.aplicar_operacoes(&pessoa("ana"), 101, 0, &[alterar(url("https://github.com/ana/repo"), json!({"repositorio": null}))], &livre).unwrap();
    let s = d.aplicar_operacoes(&pessoa("bruno"), 102, 0, &[alterar(url("https://github.com/bruno/repo"), json!({"repositorio": null}))], &livre).unwrap();
    assert_eq!(s.conflitos.len(), 1);
    assert_eq!((&s.conflitos[0]["campo"], &s.conflitos[0]["quem"]["id"]), (&json!("repositorio"), &json!("ana")));
    assert_eq!(blueprint(&mut d)["repositorio"]["url"], "https://github.com/bruno/repo", "a última gravação vence");
}

#[test]
fn o_repositorio_volta_inteiro_na_ida_e_volta_do_arquivo() {
    let entrada = json!({"empresa": "E", "blueprints": [{"id": "b1", "nome": "B", "repositorio": {"url": "https://github.com/dono/repo"}, "acoes": []}]});
    let mut d = Dados::em_memoria().unwrap();
    d.gravar_tudo(&Contexto::local(), &entrada).unwrap();
    assert_eq!(d.ler_tudo(&Contexto::local()).unwrap().clone(), entrada);
}
