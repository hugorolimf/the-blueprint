//! O formato `blueprint.plano/1`: o que o agente escreve e o que as regras puras (`validar`, `comparar`, `arvore`) leem. Sem serde derive (a raiz não tem).
//! Campo desconhecido é ignorado; `estado`, `aprovado` e `feito` no JSON não têm efeito nenhum. `figura_info` nunca vem de `de_json`: só `arvore` e o fechamento o preenchem.
//! Aqui só entra o que a struct não saberia guardar (tipo errado, `acoes` ausente, formato de outra versão); as regras de conteúdo são de `validar`. Mensagens em inglês: vão ao agente.
use super::formato_ler::{exige_objeto, itens, ler_acao, ler_decisao, ler_detalhe, ler_pergunta, ler_tabela, opcional, resultado, texto};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const FORMATO: &str = "blueprint.plano/1";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanoDoc { pub formato: String, pub blueprint: String, pub resumo: Option<String>, pub estilo: Option<String>, pub tema: BTreeMap<String, String>, pub projetos: Vec<ProjetoDoc>, pub decisoes: Vec<Decisao>, pub perguntas: Vec<Pergunta> }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjetoDoc { pub nome: Option<String>, pub acoes: Vec<AcaoDoc> }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AcaoDoc { pub id: String, pub nome: String, pub de: Option<String>, pub ate: Option<String>, pub texto: Option<String>, pub planos: Vec<PlanoItem>, pub checkpoints: Vec<CheckpointDoc>, pub detalhes: Vec<Detalhe>, pub tabelas: Vec<Tabela> }
/// `id_gerado`: o plano veio sem id e ganhou `<acao>.<i+1>`; não vai para o JSON (inserir no meio desloca os ids, e `validar::avisos` avisa).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanoItem { pub id: String, pub texto: String, pub area: Option<String>, pub depende_de: Vec<String>, pub instrucoes: Option<String>, pub id_gerado: bool }
/// O id é opcional no JSON; o padrão é `<acao>.c<i+1>`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CheckpointDoc { pub id: String, pub texto: String, pub data: Option<String> }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Detalhe { pub id: String, pub titulo: Option<String>, pub legenda: Option<String>, pub grupo: Option<String>, pub figura: Option<String>, pub figura_info: Option<FiguraInfo>, pub pede_aprovacao: bool }
/// Só em `extra.detalhes`: as medidas para a prancha.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FiguraInfo { pub hash: String, pub tipo: String, pub largura: Option<u32>, pub altura: Option<u32> }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tabela { pub titulo: Option<String>, pub colunas: Vec<String>, pub larguras: Vec<f64>, pub linhas: Vec<Vec<String>> }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Decisao { pub id: String, pub rotulo: Option<String>, pub texto: String, pub data: Option<String>, pub acoes: Vec<String>, pub pede_aprovacao: bool }
/// `recomendada` é o texto da opção; no JSON também vale o índice dela em `opcoes`, como escrevem os exemplos do protótipo.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pergunta { pub id: String, pub texto: String, pub opcoes: Vec<String>, pub recomendada: Option<String>, pub acoes: Vec<String> }

fn por(m: &mut Map<String, Value>, campo: &str, valor: &Option<String>) {
    if let Some(s) = valor {
        m.insert(campo.into(), json!(s));
    }
}

fn por_lista<T>(m: &mut Map<String, Value>, campo: &str, lista: &[T], f: impl Fn(&T) -> Value) {
    if !lista.is_empty() {
        m.insert(campo.into(), Value::Array(lista.iter().map(f).collect()));
    }
}

fn por_verdade(m: &mut Map<String, Value>, campo: &str, valor: bool) {
    if valor {
        m.insert(campo.into(), json!(true));
    }
}

impl PlanoItem {
    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id));
        m.insert("texto".into(), json!(self.texto));
        por(&mut m, "area", &self.area);
        por_lista(&mut m, "depende_de", &self.depende_de, |d| json!(d));
        por(&mut m, "instrucoes", &self.instrucoes);
        Value::Object(m)
    }
}

impl CheckpointDoc {
    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id));
        m.insert("texto".into(), json!(self.texto));
        por(&mut m, "data", &self.data);
        Value::Object(m)
    }
}

impl FiguraInfo {
    /// Para quem lê o que o próprio servidor gravou (`extra.detalhes`, a versão fechada). Nunca sobre o que o agente mandou.
    pub fn de_json(v: &Value) -> Option<FiguraInfo> {
        let medida = |campo: &str| v.get(campo).and_then(Value::as_u64).and_then(|n| u32::try_from(n).ok());
        Some(FiguraInfo { hash: v.get("hash")?.as_str()?.to_string(), tipo: v.get("tipo")?.as_str()?.to_string(), largura: medida("largura"), altura: medida("altura") })
    }

    pub fn para_json(&self) -> Value {
        json!({"hash": self.hash, "tipo": self.tipo, "largura": self.largura, "altura": self.altura})
    }
}

impl Detalhe {
    pub fn de_json(v: &Value) -> Result<Detalhe, Vec<String>> {
        let mut erros = Vec::new();
        let d = ler_detalhe(v, "detail", &mut erros);
        resultado(d, erros)
    }

    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id));
        por(&mut m, "titulo", &self.titulo);
        por(&mut m, "legenda", &self.legenda);
        por(&mut m, "grupo", &self.grupo);
        por(&mut m, "figura", &self.figura);
        if let Some(info) = &self.figura_info {
            m.insert("figura_info".into(), info.para_json());
        }
        por_verdade(&mut m, "pede_aprovacao", self.pede_aprovacao);
        Value::Object(m)
    }
}

impl Tabela {
    pub fn de_json(v: &Value) -> Result<Tabela, Vec<String>> {
        let mut erros = Vec::new();
        let t = ler_tabela(v, "table", &mut erros);
        resultado(t, erros)
    }

    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        por(&mut m, "titulo", &self.titulo);
        m.insert("colunas".into(), json!(self.colunas));
        por_lista(&mut m, "larguras", &self.larguras, |l| json!(l));
        m.insert("linhas".into(), json!(self.linhas));
        Value::Object(m)
    }
}

impl AcaoDoc {
    /// Uma ação sozinha (`plano/escrever`). Em `escrever_acao` o id do plano é obrigatório: quem chama recusa `id_gerado`.
    pub fn de_json(v: &Value) -> Result<AcaoDoc, Vec<String>> {
        let mut erros = Vec::new();
        let a = ler_acao(v, "action", &mut erros);
        resultado(a, erros)
    }

    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id));
        m.insert("nome".into(), json!(self.nome));
        por(&mut m, "de", &self.de);
        por(&mut m, "ate", &self.ate);
        por(&mut m, "texto", &self.texto);
        por_lista(&mut m, "planos", &self.planos, PlanoItem::para_json);
        por_lista(&mut m, "checkpoints", &self.checkpoints, CheckpointDoc::para_json);
        por_lista(&mut m, "detalhes", &self.detalhes, Detalhe::para_json);
        por_lista(&mut m, "tabelas", &self.tabelas, Tabela::para_json);
        Value::Object(m)
    }
}

impl Decisao {
    pub fn de_json(v: &Value) -> Result<Decisao, Vec<String>> {
        let mut erros = Vec::new();
        let d = ler_decisao(v, &mut erros);
        resultado(d, erros)
    }

    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id));
        por(&mut m, "rotulo", &self.rotulo);
        m.insert("texto".into(), json!(self.texto));
        por(&mut m, "data", &self.data);
        por_lista(&mut m, "acoes", &self.acoes, |a| json!(a));
        por_verdade(&mut m, "pede_aprovacao", self.pede_aprovacao);
        Value::Object(m)
    }
}

impl Pergunta {
    pub fn de_json(v: &Value) -> Result<Pergunta, Vec<String>> {
        let mut erros = Vec::new();
        let p = ler_pergunta(v, &mut erros);
        resultado(p, erros)
    }

    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id));
        m.insert("texto".into(), json!(self.texto));
        por_lista(&mut m, "opcoes", &self.opcoes, |o| json!(o));
        por(&mut m, "recomendada", &self.recomendada);
        por_lista(&mut m, "acoes", &self.acoes, |a| json!(a));
        Value::Object(m)
    }
}

impl PlanoDoc {
    pub fn de_json(v: &Value) -> Result<PlanoDoc, Vec<String>> {
        if !v.is_object() {
            return Err(vec!["plan: expected a JSON object".into()]);
        }
        match v.get("formato").and_then(Value::as_str).unwrap_or("") {
            FORMATO => {}
            "" => return Err(vec![format!("formato: use \"{FORMATO}\"")]),
            outro => return Err(vec![format!("formato: \"{outro}\" is not supported by this version of the app; use \"{FORMATO}\"")]),
        }
        let mut erros = Vec::new();
        let mut tema = BTreeMap::new();
        match v.get("tema") {
            None | Some(Value::Null) => {}
            Some(Value::Object(m)) => {
                for (k, valor) in m {
                    match valor.as_str() {
                        Some(s) => drop(tema.insert(k.clone(), s.to_string())),
                        None => erros.push(format!("tema: {k} must be a string")),
                    }
                }
            }
            Some(_) => erros.push("tema: expected an object".into()),
        }
        if !v.get("projetos").is_some_and(Value::is_array) {
            erros.push("projetos: at least one is required, with its acoes (the name is optional)".into());
        }
        let mut projetos = Vec::new();
        for (i, p) in itens(v, "projetos", "plan", &mut erros).iter().enumerate() {
            let onde = format!("project {}", i + 1);
            if !exige_objeto(p, &onde, &mut erros) {
                continue;
            }
            if !p.get("acoes").is_some_and(Value::is_array) {
                erros.push(format!("{onde}: acoes is required (use [] for a project with no action yet)"));
            }
            let acoes = itens(p, "acoes", &onde, &mut erros).iter().enumerate().map(|(j, a)| ler_acao(a, &format!("{onde}, action {}", j + 1), &mut erros)).collect();
            projetos.push(ProjetoDoc { nome: opcional(p, "nome", &onde, &mut erros), acoes });
        }
        let decisoes = itens(v, "decisoes", "plan", &mut erros).iter().map(|d| ler_decisao(d, &mut erros)).collect();
        let perguntas = itens(v, "perguntas", "plan", &mut erros).iter().map(|q| ler_pergunta(q, &mut erros)).collect();
        let doc = PlanoDoc { formato: FORMATO.into(), blueprint: texto(v, "blueprint", "plan", &mut erros), resumo: opcional(v, "resumo", "plan", &mut erros), estilo: opcional(v, "estilo", "plan", &mut erros), tema, projetos, decisoes, perguntas };
        resultado(doc, erros)
    }

    pub fn para_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("formato".into(), json!(self.formato));
        m.insert("blueprint".into(), json!(self.blueprint));
        por(&mut m, "resumo", &self.resumo);
        por(&mut m, "estilo", &self.estilo);
        if !self.tema.is_empty() {
            m.insert("tema".into(), json!(self.tema));
        }
        let projeto = |p: &ProjetoDoc| {
            let mut o = Map::new();
            por(&mut o, "nome", &p.nome);
            o.insert("acoes".into(), Value::Array(p.acoes.iter().map(AcaoDoc::para_json).collect()));
            Value::Object(o)
        };
        m.insert("projetos".into(), Value::Array(self.projetos.iter().map(projeto).collect()));
        por_lista(&mut m, "decisoes", &self.decisoes, Decisao::para_json);
        por_lista(&mut m, "perguntas", &self.perguntas, Pergunta::para_json);
        Value::Object(m)
    }
}
