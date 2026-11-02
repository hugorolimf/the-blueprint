//! Passa de 300 linhas: ida, volta, mescla e `recompor` usam os mesmos auxiliares, e a fatia dona (F22) não pode criar módulo novo nem editar `plano/mod.rs`. Candidato a dividir (`arvore/`) depois das ondas paralelas.
//! A ida e a volta entre o formato e a árvore do blueprint (funções puras sobre `Value`). A árvore é o plano vigente: `extra` de cada linha volta como campo do próprio objeto.
//! `projetar` parte do objeto existente e sobrescreve só os campos mapeados: `feito`, `agente`, `evidencia`, `prototipos` e qualquer outro seguem. Id real só é reusado dentro da mesma ação; sem correspondência, o item nasce com `feito=false`.
use super::formato::{AcaoDoc, CheckpointDoc, Decisao, Detalhe, FiguraInfo, Pergunta, PlanoDoc, PlanoItem, ProjetoDoc, Tabela, FORMATO};
use super::ids::{id_real, sha256_hex};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

/// O nome reservado ao grupo das ações sem projeto (`blueprint/projetos.rs`).
const GERAL: &str = "Geral";

/// Quais ações (ref) entram ou saem; as outras ficam como estão; removidas = as de `So` ausentes em `novo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escopo<'a> { Tudo, So(&'a [String]), SoFora(&'a [String]) }

impl Escopo<'_> {
    /// `nomes`: tudo por que o item responde (ref e id).
    fn entra(&self, nomes: &[&str]) -> bool {
        match self {
            Escopo::Tudo => true,
            Escopo::So(l) => l.iter().any(|r| nomes.contains(&r.as_str())),
            Escopo::SoFora(l) => !l.iter().any(|r| nomes.contains(&r.as_str())),
        }
    }
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn texto_de<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

fn nao_vazio(v: &Value, campo: &str) -> Option<String> {
    Some(texto_de(v, campo)).filter(|s| !s.is_empty()).map(String::from)
}

/// O `ref` do item; blueprint antigo não tem, e o ref é o próprio id.
fn ref_de(v: &Value) -> &str {
    Some(texto_de(v, "ref")).filter(|s| !s.is_empty()).unwrap_or_else(|| texto_de(v, "id"))
}

/// Trim + minúsculas, a regra de `blueprint/projetos.rs`.
fn nome_igual(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// `Geral` e nome em branco são o grupo sem projeto.
fn nome_de_projeto(nome: &Option<String>) -> Option<&str> {
    nome.as_deref().map(str::trim).filter(|n| !n.is_empty() && !nome_igual(n, GERAL))
}

fn plano_de(v: &Value) -> PlanoItem {
    let depende_de = lista(v, "dependeDe").iter().filter_map(Value::as_str).map(String::from).collect();
    PlanoItem { id: ref_de(v).into(), texto: texto_de(v, "texto").into(), area: nao_vazio(v, "area"), depende_de, instrucoes: nao_vazio(v, "descricao"), id_gerado: false }
}

fn checkpoint_de(v: &Value) -> CheckpointDoc {
    CheckpointDoc { id: ref_de(v).into(), texto: texto_de(v, "nome").into(), data: nao_vazio(v, "data") }
}

fn detalhe_de(v: &Value) -> Option<Detalhe> {
    let mut d = Detalhe::de_json(v).ok()?;
    d.figura_info = v.get("figura_info").and_then(FiguraInfo::de_json);
    Some(d)
}

fn acao_de(a: &Value) -> AcaoDoc {
    let sem_data = a.get("semData").and_then(Value::as_bool) == Some(true);
    let data = |campo: &str| if sem_data { None } else { nao_vazio(a, campo) };
    AcaoDoc {
        id: ref_de(a).into(),
        nome: texto_de(a, "nome").into(),
        de: data("inicio"),
        ate: data("fim"),
        texto: nao_vazio(a, "obs"),
        planos: lista(a, "planos").iter().map(plano_de).collect(),
        checkpoints: lista(a, "checkpoints").iter().map(checkpoint_de).collect(),
        detalhes: lista(a, "detalhes").iter().filter_map(detalhe_de).collect(),
        tabelas: lista(a, "tabelas").iter().filter_map(|t| Tabela::de_json(t).ok()).collect(),
    }
}

/// Árvore para doc; item sem ref usa o id como ref. Projeto na ordem em que aparece nas ações; o que não tem ação vem no fim.
pub fn do_blueprint(bp: &Value) -> PlanoDoc {
    let cadastrados: Vec<(&str, &str)> = lista(bp, "projetos").iter().map(|p| (texto_de(p, "id"), texto_de(p, "nome"))).filter(|(id, nome)| !id.is_empty() && !nome.is_empty()).collect();
    let mut projetos: Vec<ProjetoDoc> = Vec::new();
    for a in lista(bp, "acoes") {
        let id = texto_de(a, "projetoId");
        let nome = cadastrados.iter().find(|(i, _)| *i == id).map(|(_, n)| n.to_string());
        let acao = acao_de(a);
        match projetos.iter_mut().find(|p| p.nome == nome) {
            Some(p) => p.acoes.push(acao),
            None => projetos.push(ProjetoDoc { nome, acoes: vec![acao] }),
        }
    }
    for (_, nome) in &cadastrados {
        if !projetos.iter().any(|p| p.nome.as_deref() == Some(*nome)) {
            projetos.push(ProjetoDoc { nome: Some(nome.to_string()), acoes: Vec::new() });
        }
    }
    let extra = bp.get("planoDoc").unwrap_or(&Value::Null);
    let tema: BTreeMap<String, String> = extra.get("tema").and_then(Value::as_object).into_iter().flatten().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect();
    PlanoDoc {
        formato: FORMATO.into(),
        blueprint: texto_de(bp, "nome").into(),
        resumo: nao_vazio(bp, "descricao"),
        estilo: nao_vazio(extra, "estilo"),
        tema,
        projetos,
        decisoes: lista(extra, "decisoes").iter().filter_map(|d| Decisao::de_json(d).ok()).collect(),
        perguntas: lista(extra, "perguntas").iter().filter_map(|p| Pergunta::de_json(p).ok()).collect(),
    }
}

/// Para cada ref, o índice do item que ele reusa: primeiro pelo ref, depois pelo id, cada item uma vez só (senão dois refs ganhariam o mesmo id real).
fn casar(itens: &[Value], refs: &[&str]) -> Vec<Option<usize>> {
    let mut usados = vec![false; itens.len()];
    let mut saida = vec![None; refs.len()];
    let mut passada = |igual: &dyn Fn(&Value, &str) -> bool| {
        for (i, r) in refs.iter().enumerate() {
            if saida[i].is_none() {
                saida[i] = itens.iter().enumerate().position(|(j, item)| !usados[j] && igual(item, r));
                if let Some(j) = saida[i] {
                    usados[j] = true;
                }
            }
        }
    };
    passada(&|item, r| ref_de(item) == r);
    passada(&|item, r| texto_de(item, "id") == r);
    saida
}

/// Opcional ausente não apaga; `""` apaga.
fn por_opcional(m: &mut Map<String, Value>, campo: &str, valor: &Option<String>) {
    match valor.as_deref() {
        None => {}
        Some("") => drop(m.remove(campo)),
        Some(s) => drop(m.insert(campo.into(), json!(s))),
    }
}

/// Lista que o doc sempre traz inteira: vazia, a chave sai.
fn por_ou_tira(m: &mut Map<String, Value>, campo: &str, valores: Vec<Value>) {
    if valores.is_empty() {
        m.remove(campo);
    } else {
        m.insert(campo.into(), Value::Array(valores));
    }
}

fn base_do_item(existente: Option<&Value>, novo: impl FnOnce() -> Value) -> Map<String, Value> {
    match existente.map(Value::as_object) {
        Some(Some(o)) => o.clone(),
        _ => novo().as_object().cloned().unwrap_or_default(),
    }
}

fn plano_para(bp_id: &str, acao_ref: &str, existente: Option<&Value>, q: &PlanoItem) -> Value {
    let mut m = base_do_item(existente, || json!({"id": id_real(bp_id, "plano", &format!("{acao_ref}/{}", q.id)), "feito": false}));
    m.insert("ref".into(), json!(q.id));
    m.insert("texto".into(), json!(q.texto));
    if let Some(s) = &q.instrucoes {
        m.insert("descricao".into(), json!(s));
    }
    por_opcional(&mut m, "area", &q.area);
    por_ou_tira(&mut m, "dependeDe", q.depende_de.iter().map(|d| json!(d)).collect());
    Value::Object(m)
}

/// A data vai sempre como o doc diz: `comparar` conta a diferença, e o que foi aprovado tem de ser o que fica.
fn checkpoint_para(bp_id: &str, acao_ref: &str, existente: Option<&Value>, c: &CheckpointDoc) -> Value {
    let mut m = base_do_item(existente, || json!({"id": id_real(bp_id, "checkpoint", &format!("{acao_ref}/{}", c.id)), "feito": false}));
    m.insert("ref".into(), json!(c.id));
    m.insert("nome".into(), json!(c.texto));
    match &c.data {
        Some(d) if !d.is_empty() => drop(m.insert("data".into(), json!(d))),
        _ => drop(m.remove("data")),
    }
    Value::Object(m)
}

/// Sem data: marca `semData` e deixa em `inicio`/`fim` o que já havia (ou hoje), para a web legada não quebrar; a volta ignora os dois.
fn por_datas(m: &mut Map<String, Value>, a: &AcaoDoc, hoje: &str) {
    let de = a.de.as_deref().filter(|s| !s.is_empty());
    let ate = a.ate.as_deref().filter(|s| !s.is_empty());
    let (Some(inicio), Some(fim)) = (de.or(ate), ate.or(de)) else {
        m.insert("semData".into(), json!(true));
        for campo in ["inicio", "fim"] {
            if m.get(campo).and_then(Value::as_str).unwrap_or("").is_empty() {
                m.insert(campo.into(), json!(hoje));
            }
        }
        return;
    };
    m.remove("semData");
    m.insert("inicio".into(), json!(inicio));
    m.insert("fim".into(), json!(fim));
}

fn filhos_para(existente: Option<&Value>, campo: &str, ids: &[&str], f: impl Fn(Option<&Value>, usize) -> Value) -> Vec<Value> {
    let velhos = existente.map_or(&[][..], |e| lista(e, campo));
    casar(velhos, ids).into_iter().enumerate().map(|(i, j)| f(j.map(|j| &velhos[j]), i)).collect()
}

fn acao_para(bp_id: &str, existente: Option<&Value>, a: &AcaoDoc, projeto: Option<String>, hoje: &str) -> Value {
    let mut m = base_do_item(existente, || json!({"id": id_real(bp_id, "acao", &a.id), "status": "planejado", "progresso": 0, "planos": [], "checkpoints": []}));
    m.insert("ref".into(), json!(a.id));
    if !a.nome.is_empty() || !m.contains_key("nome") {
        m.insert("nome".into(), json!(a.nome));
    }
    por_datas(&mut m, a, hoje);
    if let Some(t) = &a.texto {
        m.insert("obs".into(), json!(t));
    }
    match projeto {
        Some(id) => drop(m.insert("projetoId".into(), json!(id))),
        None => drop(m.remove("projetoId")),
    }
    let refs: Vec<&str> = a.planos.iter().map(|q| q.id.as_str()).collect();
    let planos = filhos_para(existente, "planos", &refs, |v, i| plano_para(bp_id, &a.id, v, &a.planos[i]));
    if !planos.is_empty() || m.contains_key("planos") {
        m.insert("planos".into(), Value::Array(planos));
    }
    let refs: Vec<&str> = a.checkpoints.iter().map(|c| c.id.as_str()).collect();
    let checkpoints = filhos_para(existente, "checkpoints", &refs, |v, i| checkpoint_para(bp_id, &a.id, v, &a.checkpoints[i]));
    if !checkpoints.is_empty() || m.contains_key("checkpoints") {
        m.insert("checkpoints".into(), Value::Array(checkpoints));
    }
    por_ou_tira(&mut m, "detalhes", a.detalhes.iter().map(Detalhe::para_json).collect());
    por_ou_tira(&mut m, "tabelas", a.tabelas.iter().map(Tabela::para_json).collect());
    Value::Object(m)
}

/// O id do projeto de mesmo nome; novo, se não há. Nunca remove nem mexe no que existe.
fn garantir_projeto(projetos: &mut Vec<Value>, bp_id: &str, nome: &Option<String>) -> Option<String> {
    let nome = nome_de_projeto(nome)?;
    if let Some(p) = projetos.iter().find(|p| !texto_de(p, "id").is_empty() && nome_igual(texto_de(p, "nome"), nome)) {
        return Some(texto_de(p, "id").to_string());
    }
    let id = id_real(bp_id, "projeto", nome);
    projetos.push(json!({"id": id, "nome": nome}));
    Some(id)
}

/// `tema`, `estilo`, `decisoes` e `perguntas`, em `extra.planoDoc`. O que o doc não traz fica como está.
fn por_plano_doc(obj: &mut Map<String, Value>, novo: &PlanoDoc) {
    let mut extra = obj.get("planoDoc").and_then(Value::as_object).cloned().unwrap_or_default();
    por_opcional(&mut extra, "estilo", &novo.estilo);
    if !novo.tema.is_empty() {
        extra.insert("tema".into(), json!(novo.tema));
    }
    if !novo.decisoes.is_empty() {
        extra.insert("decisoes".into(), Value::Array(novo.decisoes.iter().map(Decisao::para_json).collect()));
    }
    if !novo.perguntas.is_empty() {
        extra.insert("perguntas".into(), Value::Array(novo.perguntas.iter().map(Pergunta::para_json).collect()));
    }
    if !extra.is_empty() {
        obj.insert("planoDoc".into(), Value::Object(extra));
    }
}

/// Doc para blueprint novo. O que é do blueprint (nome, resumo, tema, decisões, perguntas) entra em qualquer escopo; `escopo` só decide as ações.
/// Sem nome de projeto, a ação fica sem `projetoId`. `feito` não é mexido aqui: reabrir plano alterado é do gesto de aprovar.
pub fn projetar(bp: &Value, novo: &PlanoDoc, escopo: Escopo<'_>, hoje: &str) -> Value {
    let mut saida = bp.clone();
    let Some(obj) = saida.as_object_mut() else { return saida };
    let bp_id = texto_de(bp, "id");
    if !novo.blueprint.is_empty() {
        obj.insert("nome".into(), json!(novo.blueprint));
    }
    if let Some(r) = &novo.resumo {
        obj.insert("descricao".into(), json!(r));
    }
    por_plano_doc(obj, novo);

    let doc: Vec<(&Option<String>, &AcaoDoc)> = novo.projetos.iter().flat_map(|p| p.acoes.iter().map(move |a| (&p.nome, a))).collect();
    let refs: Vec<&str> = doc.iter().map(|(_, a)| a.id.as_str()).collect();
    let atuais = lista(bp, "acoes");
    let casados = casar(atuais, &refs);
    let ids: Vec<String> = doc.iter().zip(&casados).map(|((_, a), j)| j.map_or_else(|| id_real(bp_id, "acao", &a.id), |j| texto_de(&atuais[j], "id").to_string())).collect();

    let mut projetos = lista(bp, "projetos").to_vec();
    let antes = projetos.len();
    for p in novo.projetos.iter().filter(|p| p.acoes.is_empty()) {
        garantir_projeto(&mut projetos, bp_id, &p.nome);
    }
    let mut feitas: Vec<Option<Value>> = doc.iter().zip(&casados).map(|((nome, a), j)| {
        escopo.entra(&[a.id.as_str()]).then(|| acao_para(bp_id, j.map(|j| &atuais[j]), a, garantir_projeto(&mut projetos, bp_id, nome), hoje))
    }).collect();
    if projetos.len() != antes {
        obj.insert("projetos".into(), Value::Array(projetos));
    }

    let no_doc: HashMap<usize, usize> = casados.iter().enumerate().filter_map(|(i, j)| j.map(|j| (j, i))).collect();
    let mut acoes: Vec<Value> = Vec::with_capacity(atuais.len() + doc.len());
    for (j, item) in atuais.iter().enumerate() {
        match no_doc.get(&j) {
            Some(&i) => acoes.push(feitas[i].take().unwrap_or_else(|| item.clone())),
            None if escopo.entra(&[ref_de(item), texto_de(item, "id")]) => {}
            None => acoes.push(item.clone()),
        }
    }
    let posicao = |acoes: &[Value], id: &str| acoes.iter().position(|a| texto_de(a, "id") == id);
    for i in 0..doc.len() {
        let Some(nova) = feitas[i].take() else { continue };
        // Logo depois da anterior no doc; a primeira entra antes de quem vem depois dela no doc, ou no fim.
        let depois = (0..i).rev().find_map(|k| posicao(&acoes, &ids[k])).map(|p| p + 1);
        let lugar = depois.or_else(|| ids.iter().find_map(|id| posicao(&acoes, id))).unwrap_or(acoes.len());
        acoes.insert(lugar, nova);
    }
    if escopo == Escopo::Tudo {
        let ordem: HashMap<&str, usize> = ids.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
        acoes.sort_by_key(|a| ordem.get(texto_de(a, "id")).copied().unwrap_or(usize::MAX));
    }
    if !acoes.is_empty() || obj.contains_key("acoes") {
        obj.insert("acoes".into(), Value::Array(acoes));
    }
    saida
}

fn chave_projeto(nome: &Option<String>) -> Option<String> {
    nome_de_projeto(nome).map(|n| n.trim().to_lowercase())
}

/// O fechamento de uma escrita: `tocadas` vêm de `doc`, `removidas` saem, e o resto é refeito de `base` (a de agora), para o que o usuário fez durante a escrita não virar remoção nem reversão.
/// Ação só de `doc`, fora de `tocadas`, some: é a que o usuário apagou durante a escrita. Por isso quem abre a escrita põe em `tocadas` a ação nova declarada. O que é do blueprint vem de `doc`.
pub fn recompor(doc: &PlanoDoc, base: &PlanoDoc, tocadas: &[String], removidas: &[String]) -> PlanoDoc {
    let toca: HashSet<&str> = tocadas.iter().map(String::as_str).collect();
    let remove: HashSet<&str> = removidas.iter().map(String::as_str).collect();
    let no_doc: HashMap<&str, (&Option<String>, &AcaoDoc)> = doc.projetos.iter().flat_map(|p| p.acoes.iter().map(move |a| (a.id.as_str(), (&p.nome, a)))).collect();

    let mut projetos = base.projetos.clone();
    for p in &mut projetos {
        let nome = chave_projeto(&p.nome);
        p.acoes.retain_mut(|a| {
            if remove.contains(a.id.as_str()) {
                return false;
            }
            match no_doc.get(a.id.as_str()).filter(|_| toca.contains(a.id.as_str())) {
                Some((pn, nova)) if chave_projeto(pn) == nome => {
                    *a = (*nova).clone();
                    true
                }
                Some(_) => false, // mudou de projeto: entra lá embaixo
                None => true,
            }
        });
    }
    for p in &doc.projetos {
        let nome = chave_projeto(&p.nome);
        for (i, a) in p.acoes.iter().enumerate() {
            let existe = projetos.iter().any(|q| q.acoes.iter().any(|x| x.id == a.id));
            if existe || !toca.contains(a.id.as_str()) || remove.contains(a.id.as_str()) {
                continue;
            }
            let j = projetos.iter().position(|q| chave_projeto(&q.nome) == nome).unwrap_or_else(|| {
                projetos.push(ProjetoDoc { nome: p.nome.clone(), acoes: Vec::new() });
                projetos.len() - 1
            });
            let destino = &mut projetos[j].acoes;
            let lugar = p.acoes[..i].iter().rev().find_map(|ant| destino.iter().position(|x| x.id == ant.id)).map_or(0, |k| k + 1);
            destino.insert(lugar, a.clone());
        }
        if p.acoes.is_empty() && nome.is_some() && !projetos.iter().any(|q| chave_projeto(&q.nome) == nome) {
            projetos.push(p.clone());
        }
    }
    projetos.retain(|p| !p.acoes.is_empty() || chave_projeto(&p.nome).is_some());
    PlanoDoc { projetos, ..doc.clone() }
}

fn canonico(v: &Value, saida: &mut String) {
    match v {
        Value::Object(m) => {
            let mut chaves: Vec<&String> = m.keys().collect();
            chaves.sort();
            saida.push('{');
            for (i, k) in chaves.iter().enumerate() {
                if i > 0 {
                    saida.push(',');
                }
                saida.push_str(&Value::String((*k).clone()).to_string());
                saida.push(':');
                canonico(&m[*k], saida);
            }
            saida.push('}');
        }
        Value::Array(l) => {
            saida.push('[');
            for (i, x) in l.iter().enumerate() {
                if i > 0 {
                    saida.push(',');
                }
                canonico(x, saida);
            }
            saida.push(']');
        }
        _ => saida.push_str(&v.to_string()),
    }
}

/// O sha256 do JSON canônico (chaves em ordem); usado por vista e gestos.
pub fn hash_peca(peca: &Value) -> String {
    let mut s = String::new();
    canonico(peca, &mut s);
    sha256_hex(s.as_bytes())
}
