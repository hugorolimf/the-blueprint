//! Projetos dentro do blueprint (decisão de 2026-10-22): `blueprint.projetos` (`[{id, nome}]`) e, na ação, `projetoId`.
//! Na proposta do agente: `blueprint.projetos` (nomes que devem existir) e `projeto` em cada ação (id ou nome). Nada é apagado nem renomeado por aqui.
use super::texto::{campo, lista, str_de, truncar};
use serde_json::{json, Value};

/// A ação sem projeto aparece no grupo "Geral"; o nome fica reservado.
pub const GERAL: &str = "Geral";
const MAX_NOVOS: usize = 12;

fn igual(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// `(id, nome)` dos projetos que o blueprint já tem.
pub fn projetos_do(p: &Value) -> Vec<(String, String)> {
    lista(p.get("projetos")).iter().map(|x| (str_de(x, "id"), str_de(x, "nome"))).filter(|(id, _)| !id.is_empty()).collect()
}

/// O nome do projeto da ação; None se ela está no Geral (sem `projetoId`, ou com um id que não existe mais).
pub fn nome_do_projeto(p: &Value, acao: &Value) -> Option<String> {
    let id = str_de(acao, "projetoId");
    projetos_do(p).into_iter().find(|(i, _)| !id.is_empty() && *i == id).map(|(_, nome)| nome)
}

/// Os projetos novos que a proposta pede em `blueprint.projetos` (texto ou `{nome}`), com `id` vazio. O que já existe (por id ou nome) não é novo.
pub fn projetos_novos(dados_blueprint: Option<&Value>, atual: Option<&Value>, erros: &mut Vec<String>) -> Vec<Value> {
    let existentes = atual.map(projetos_do).unwrap_or_default();
    let mut novos: Vec<String> = Vec::new();
    for (i, x) in lista(dados_blueprint.and_then(|b| b.get("projetos"))).iter().enumerate() {
        let (id, nome) = match x {
            Value::String(s) => (String::new(), s.trim().to_string()),
            _ => (campo(x, "id"), campo(x, "nome")),
        };
        let nome = truncar(&nome, 80);
        let rotulo = format!("`blueprint.projetos[{i}]`");
        if !id.is_empty() {
            if !existentes.iter().any(|(eid, _)| *eid == id) {
                erros.push(crate::tr!("{rotulo}: id \"{id}\" does not exist in this blueprint (use only ids from ler_blueprint).", "{rotulo}: o id \"{id}\" não existe neste blueprint (use só ids de ler_blueprint)."));
            }
        } else if nome.is_empty() {
            erros.push(crate::tr!("{rotulo}: the project name is missing.", "{rotulo}: falta o nome do projeto."));
        } else if igual(&nome, GERAL) {
            erros.push(crate::tr!("{rotulo}: \"{GERAL}\" is reserved for the group of actions without a project; choose another name.", "{rotulo}: \"{GERAL}\" é reservado ao grupo das ações sem projeto; escolha outro nome."));
        } else if !existentes.iter().any(|(_, en)| igual(en, &nome)) && !novos.iter().any(|n| igual(n, &nome)) {
            novos.push(nome);
        }
    }
    if novos.len() > MAX_NOVOS {
        erros.push(crate::tr!("Too many new projects (at most {MAX_NOVOS} per proposal).", "Projetos novos demais (máximo {MAX_NOVOS} por proposta)."));
    }
    novos.into_iter().map(|nome| json!({"id": "", "nome": nome})).collect()
}

/// O que a ação diz sobre o projeto dela: nada (fica onde está), sair do projeto, ou ir para este (`{id, nome}`; `id` vazio = projeto novo da proposta).
pub enum Escolha {
    Mantem,
    Nenhum,
    Projeto(Value),
}

pub fn escolher(acao: &Value, atual: Option<&Value>, novos: &[Value], rotulo: &str, erros: &mut Vec<String>) -> Escolha {
    let texto = match acao.get("projeto") {
        None | Some(Value::Null) => return Escolha::Mantem,
        Some(Value::String(s)) => s.trim().to_string(),
        Some(_) => {
            erros.push(crate::tr!("{rotulo}: `projeto` must be the id or the name of a project (text).", "{rotulo}: `projeto` deve ser o id ou o nome de um projeto (texto)."));
            return Escolha::Mantem;
        }
    };
    if texto.is_empty() || igual(&texto, GERAL) {
        return Escolha::Nenhum;
    }
    let existentes = atual.map(projetos_do).unwrap_or_default();
    if let Some((id, nome)) = existentes.iter().find(|(id, _)| *id == texto).or_else(|| existentes.iter().find(|(_, n)| igual(n, &texto))) {
        return Escolha::Projeto(json!({"id": id, "nome": nome}));
    }
    if let Some(n) = novos.iter().find(|n| igual(&str_de(n, "nome"), &texto)) {
        return Escolha::Projeto(json!({"id": "", "nome": str_de(n, "nome")}));
    }
    let nomes: Vec<String> = existentes.iter().map(|(_, n)| n.clone()).chain(novos.iter().map(|n| str_de(n, "nome"))).collect();
    let dica = if nomes.is_empty() { crate::tr!("this blueprint has no projects; list them in `blueprint.projetos`", "este blueprint não tem projetos; liste-os em `blueprint.projetos`") } else { crate::tr!("projects: {}", "projetos: {}", nomes.join(", ")) };
    erros.push(crate::tr!("{rotulo}: project \"{}\" does not exist in this blueprint ({dica}; to create it, list it in `blueprint.projetos`; use \"{GERAL}\" or empty to take the action out of the project).", "{rotulo}: o projeto \"{}\" não existe neste blueprint ({dica}; para criar, liste-o em `blueprint.projetos`; use \"{GERAL}\" ou vazio para tirar a ação do projeto).", truncar(&texto, 80)));
    Escolha::Mantem
}

/// O número de cada ação na folha, na ordem de `acoes`: `<projeto>.<posição no projeto>` quando o blueprint tem projetos (as seções seguem o início da primeira
/// ação de cada projeto e o Geral é o último) e `<blueprint>.<posição>` quando não tem. Serve ao blueprint dos dados (`projetoId`) e à estrutura de ler_blueprint
/// (`projeto` = nome). `n_blueprint`: a posição do blueprint no workspace (a partir de 1). Mesma regra de `numeroDaAcao` em web/js/projetos.js.
pub fn numeros_das_acoes(p: &Value, n_blueprint: usize) -> Vec<String> {
    let acoes = lista(p.get("acoes"));
    let projetos = projetos_do(p);
    if projetos.is_empty() {
        return (0..acoes.len()).map(|i| format!("{n_blueprint}.{}", i + 1)).collect();
    }
    let chaves: Vec<String> = acoes
        .iter()
        .map(|a| {
            let id = str_de(a, "projetoId");
            if !id.is_empty() && projetos.iter().any(|(i, _)| *i == id) {
                return id;
            }
            let nome = str_de(a, "projeto");
            projetos.iter().find(|(_, n)| !nome.is_empty() && igual(n, &nome)).map(|(i, _)| i.clone()).unwrap_or_default()
        })
        .collect();
    let inicio = |id: &str| acoes.iter().zip(&chaves).filter(|(_, c)| c.as_str() == id).map(|(a, _)| str_de(a, "inicio")).filter(|d| !d.is_empty()).min();
    let mut ordem: Vec<(usize, &str)> = projetos.iter().enumerate().map(|(i, (id, _))| (i, id.as_str())).collect();
    ordem.sort_by(|(ia, a), (ib, b)| match (inicio(a), inicio(b)) {
        (Some(x), Some(y)) => x.cmp(&y).then(ia.cmp(ib)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => ia.cmp(ib),
    });
    let mut ids: Vec<&str> = ordem.into_iter().map(|(_, id)| id).collect();
    ids.push(""); // o Geral
    let mut vistas = vec![0usize; ids.len()];
    chaves
        .iter()
        .map(|c| {
            let k = ids.iter().position(|i| *i == c.as_str()).unwrap_or(ids.len() - 1);
            vistas[k] += 1;
            format!("{}.{}", k + 1, vistas[k])
        })
        .collect()
}
