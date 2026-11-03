//! O blueprint como o agente o enxerga, com os ids que ele precisa reaproveitar: porte de contextoBlueprint, estruturaBlueprint, resumoBlueprint e
//! contextoAcao do antigo plano.mjs.
use super::decisoes::{decisoes_do, resumo_das_decisoes};
use super::impressao::impressao_do_projeto;
use super::projetos::{nome_do_projeto, projetos_do};
use super::texto::{lista, str_de, truncar, verdade};
use serde_json::{json, Map, Value};

/// copia as chaves que existirem em `de` (o JS deixava de fora as `undefined`)
fn pegar(de: &Value, chaves: &[&str]) -> Map<String, Value> {
    chaves.iter().filter_map(|c| de.get(*c).map(|v| ((*c).to_string(), v.clone()))).collect()
}

fn marcado(x: &Value) -> bool {
    verdade(x.get("feito"))
}

fn itens(acao: &Value, chave: &str, campos: &[&str], descricao: bool) -> Vec<Value> {
    lista(acao.get(chave))
        .iter()
        .map(|x| {
            let mut o = pegar(x, campos);
            if descricao {
                o.insert("descricao".into(), Value::String(str_de(x, "descricao")));
                o.insert("feito".into(), Value::Bool(marcado(x)));
            }
            Value::Object(o)
        })
        .collect()
}

fn acao_completa(p: &Value, f: &Value) -> Value {
    let mut o = pegar(f, &["id", "nome", "status"]);
    if let Some(projeto) = nome_do_projeto(p, f) {
        o.insert("projeto".into(), Value::String(projeto));
    }
    o.insert("observacao".into(), Value::String(str_de(f, "obs")));
    o.insert("planos".into(), Value::Array(itens(f, "planos", &["id", "texto", "responsavel"], true)));
    let checkpoints = lista(f.get("checkpoints")).iter().map(|m| {
        let mut c = pegar(m, &["id", "nome"]);
        c.insert("feito".into(), Value::Bool(marcado(m)));
        Value::Object(c)
    });
    o.insert("checkpoints".into(), Value::Array(checkpoints.collect()));
    let prototipos = lista(f.get("prototipos")).iter().map(|x| Value::Object(pegar(x, &["id", "titulo"])));
    o.insert("prototipos".into(), Value::Array(prototipos.collect()));
    Value::Object(o)
}

pub fn contexto_blueprint(p: &Value) -> Value {
    let mut o = pegar(p, &["id", "nome"]);
    o.insert("descricao".into(), Value::String(str_de(p, "descricao")));
    o.insert("observacoes".into(), Value::String(str_de(p, "obs")));
    o.insert("acoes".into(), Value::Array(lista(p.get("acoes")).iter().map(|f| acao_completa(p, f)).collect()));
    Value::Object(o)
}

pub fn resumo_blueprint(p: &Value) -> Value {
    let mut o = pegar(p, &["id", "nome"]);
    o.insert("descricao".into(), Value::String(str_de(p, "descricao")));
    o.insert("acoes".into(), json!(lista(p.get("acoes")).len()));
    Value::Object(o)
}

/// Só o esqueleto, para o agente se localizar sem gastar contexto; instruções e descrições vêm por ação (`contexto_acao`).
pub fn estrutura_blueprint(p: &Value) -> Value {
    let marca = |x: &Value, mut o: Map<String, Value>| {
        if marcado(x) {
            o.insert("feito".into(), Value::Bool(true));
        }
        Value::Object(o)
    };
    let acoes = lista(p.get("acoes")).iter().map(|f| {
        let mut o = pegar(f, &["id", "nome", "status"]);
        if let Some(projeto) = nome_do_projeto(p, f) {
            o.insert("projeto".into(), Value::String(projeto));
        }
        o.insert("planos".into(), Value::Array(lista(f.get("planos")).iter().map(|t| marca(t, pegar(t, &["id", "texto", "responsavel"]))).collect()));
        o.insert("checkpoints".into(), Value::Array(lista(f.get("checkpoints")).iter().map(|m| marca(m, pegar(m, &["id", "nome"]))).collect()));
        Value::Object(o)
    });
    let mut o = pegar(p, &["id", "nome"]);
    o.insert("descricao".into(), Value::String(str_de(p, "descricao")));
    o.insert("acoes".into(), Value::Array(acoes.collect()));
    if !projetos_do(p).is_empty() {
        // `impressao`: o que a figura de apresentação daquele projeto guarda, para refazer só as que mudaram
        let projetos = lista(p.get("projetos")).iter().filter(|x| !str_de(x, "id").is_empty()).map(|x| json!({"id": str_de(x, "id"), "nome": str_de(x, "nome"), "impressao": impressao_do_projeto(p, x)}));
        o.insert("projetos".into(), Value::Array(projetos.collect()));
    }
    if !decisoes_do(p).is_empty() {
        o.insert("decisoes".into(), Value::Array(resumo_das_decisoes(p)));
    }
    if let Some(url) = p.pointer("/repositorio/url").and_then(Value::as_str).filter(|u| !u.is_empty()) {
        o.insert("repositorio".into(), json!({"url": truncar(url, 200)})); // como foi guardada: quem roda algo com ela valida antes (src/repositorio)
    }
    Value::Object(o)
}

pub fn contexto_acao(p: &Value, acao_id: &str) -> Option<Value> {
    lista(p.get("acoes")).iter().find(|f| str_de(f, "id") == acao_id).map(|f| acao_completa(p, f))
}
