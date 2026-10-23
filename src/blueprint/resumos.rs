//! O blueprint como o agente o enxerga, com os ids que ele precisa reaproveitar: porte de contextoBlueprint, estruturaBlueprint, resumoBlueprint e
//! contextoAcao do antigo plano.mjs, mais a estimativa (`horas`) de cada plano e a média do tempo real medido.
use super::datas::ms_do_iso;
use super::projetos::{nome_do_projeto, projetos_do};
use super::texto::{lista, str_de, verdade};
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
    let mut o = pegar(f, &["id", "nome", "inicio", "fim", "status"]);
    if let Some(projeto) = nome_do_projeto(p, f) {
        o.insert("projeto".into(), Value::String(projeto));
    }
    o.insert("observacao".into(), Value::String(str_de(f, "obs")));
    o.insert("planos".into(), Value::Array(itens(f, "planos", &["id", "texto", "horas"], true)));
    let checkpoints = lista(f.get("checkpoints")).iter().map(|m| {
        let mut c = pegar(m, &["id", "nome", "data"]);
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
        let mut o = pegar(f, &["id", "nome", "inicio", "fim", "status"]);
        if let Some(projeto) = nome_do_projeto(p, f) {
            o.insert("projeto".into(), Value::String(projeto));
        }
        o.insert("planos".into(), Value::Array(lista(f.get("planos")).iter().map(|t| marca(t, pegar(t, &["id", "texto", "horas"]))).collect()));
        o.insert("checkpoints".into(), Value::Array(lista(f.get("checkpoints")).iter().map(|m| marca(m, pegar(m, &["id", "nome", "data"]))).collect()));
        Value::Object(o)
    });
    let mut o = pegar(p, &["id", "nome"]);
    o.insert("descricao".into(), Value::String(str_de(p, "descricao")));
    o.insert("acoes".into(), Value::Array(acoes.collect()));
    let projetos = projetos_do(p);
    if !projetos.is_empty() {
        o.insert("projetos".into(), Value::Array(projetos.into_iter().map(|(id, nome)| json!({"id": id, "nome": nome})).collect()));
    }
    if let Some(real) = tempo_real(p) {
        o.insert("tempoReal".into(), real);
    }
    Value::Object(o)
}

/// Poucos planos medidos não dizem nada: só devolve a média com ao menos esta quantidade.
const MEDIDOS_MINIMO: usize = 3;

/// Média real de horas por plano, medida entre o primeiro aviso de progresso (`agente.inicioEm`) e a conclusão (`agente.feitoEm`) dos planos feitos.
fn tempo_real(p: &Value) -> Option<Value> {
    let mut horas: Vec<f64> = Vec::new();
    for acao in lista(p.get("acoes")) {
        for plano in lista(acao.get("planos")).iter().filter(|x| marcado(x)) {
            let (Some(ini), Some(fim)) = (plano.pointer("/agente/inicioEm").and_then(Value::as_str).and_then(ms_do_iso), plano.pointer("/agente/feitoEm").and_then(Value::as_str).and_then(ms_do_iso)) else { continue };
            if fim >= ini {
                horas.push((fim - ini) as f64 / 3_600_000.0);
            }
        }
    }
    if horas.len() < MEDIDOS_MINIMO {
        return None;
    }
    let media = horas.iter().sum::<f64>() / horas.len() as f64;
    Some(json!({"planosMedidos": horas.len(), "horasPorPlano": (media * 10.0).round() / 10.0}))
}

pub fn contexto_acao(p: &Value, acao_id: &str) -> Option<Value> {
    lista(p.get("acoes")).iter().find(|f| str_de(f, "id") == acao_id).map(|f| acao_completa(p, f))
}
