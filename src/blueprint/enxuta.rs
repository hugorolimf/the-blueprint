//! `ler_blueprint` sem `acao_id` em blueprint grande (2026-10-26): a estrutura inteira de ~100 ações passava de 60 mil caracteres. Aqui as ações em andamento e as
//! próximas planejadas ficam completas; as concluídas e as planejadas distantes viram uma linha. Blueprint pequeno devolve exatamente `estrutura_blueprint`.
//! Texto que o agente lê: sempre em inglês.
use super::projetos::{projetos_do, GERAL};
use super::resumos::estrutura_blueprint;
use super::texto::{lista, str_de};
use serde_json::{json, Map, Value};
use std::collections::HashSet;

/// Acima disto (caracteres do JSON compacto) o padrão `ativas` passa a resumir.
pub const LIMITE: usize = 20_000;
/// Quantas planejadas, pela data de início, ficam completas junto com as em andamento.
const PROXIMAS: usize = 4;
const COMO: &str = "Summarized actions carry no plans or checkpoints. Read one in full with acao_id, call again with incluir: \"tudo\" for everything (large), or with projeto: <id or name> for one project.";

fn status(a: &Value) -> String {
    str_de(a, "status")
}

/// Só as chaves pedidas que a ação tem.
fn linha(a: &Value, chaves: &[&str]) -> Map<String, Value> {
    chaves.iter().filter_map(|c| a.get(*c).map(|v| ((*c).to_string(), v.clone()))).collect()
}

/// O nome do projeto pedido (None = Geral). Aceita id, nome (sem ligar para caixa) ou "Geral".
fn projeto_pedido(p: &Value, pedido: &str) -> Result<Option<String>, String> {
    if pedido.to_lowercase() == GERAL.to_lowercase() {
        return Ok(None);
    }
    let projetos = projetos_do(p);
    match projetos.iter().find(|(id, nome)| id == pedido || nome.to_lowercase() == pedido.to_lowercase()) {
        Some((_, nome)) => Ok(Some(nome.clone())),
        None => Err(format!("Unknown project \"{pedido}\". Use an id or name from `projetos` ({}) or \"{GERAL}\".", projetos.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join(", "))),
    }
}

/// Concluídas e planejadas distantes viram uma linha; devolve quantas de cada.
fn resumir(acoes: Vec<Value>) -> (Vec<Value>, usize, usize) {
    let mut planejadas: Vec<usize> = (0..acoes.len()).filter(|&i| status(&acoes[i]) == "planejado").collect();
    planejadas.sort_by_key(|&i| str_de(&acoes[i], "inicio")); // yyyy-mm-dd ordena como texto
    let proximas: HashSet<usize> = planejadas.into_iter().take(PROXIMAS).collect();
    let (mut concluidas, mut distantes) = (0, 0);
    let novas = acoes.into_iter().enumerate().map(|(i, a)| match status(&a).as_str() {
        "concluido" => {
            concluidas += 1;
            Value::Object(linha(&a, &["id", "nome", "fim", "status", "projeto"]))
        }
        "planejado" if !proximas.contains(&i) => {
            distantes += 1;
            let mut o = linha(&a, &["id", "nome", "inicio", "fim", "status", "projeto"]);
            o.insert("nPlanos".into(), json!(lista(a.get("planos")).len()));
            o.insert("nCheckpoints".into(), json!(lista(a.get("checkpoints")).len()));
            Value::Object(o)
        }
        _ => a,
    });
    (novas.collect(), concluidas, distantes)
}

/// `incluir`: "ativas" (padrão, vazio vale o mesmo) ou "tudo". `projeto`: id ou nome, vazio = todos. O erro é a mensagem para o agente.
pub fn estrutura_lida(p: &Value, incluir: &str, projeto: &str) -> Result<Value, String> {
    let tudo = match incluir.trim() {
        "" | "ativas" => false,
        "tudo" => true,
        outro => return Err(format!("incluir must be \"ativas\" or \"tudo\" (got \"{outro}\").")),
    };
    let mut e = estrutura_blueprint(p);
    let pedido = projeto.trim();
    if !pedido.is_empty() {
        let nome = projeto_pedido(p, pedido)?;
        let todas = lista(e.get("acoes")).len();
        let doprojeto: Vec<Value> = lista(e.get("acoes")).iter().filter(|a| a.get("projeto").and_then(Value::as_str).map(str::to_string) == nome).cloned().collect();
        e["filtro"] = json!({"projeto": nome.unwrap_or_else(|| GERAL.to_string()), "acoes": doprojeto.len(), "deTotal": todas});
        e["acoes"] = Value::Array(doprojeto);
    }
    if tudo || e.to_string().chars().count() <= LIMITE {
        return Ok(e);
    }
    let (acoes, concluidas, planejadas) = resumir(lista(e.get("acoes")).to_vec());
    e["acoes"] = Value::Array(acoes);
    if concluidas + planejadas > 0 {
        e["resumidas"] = json!({"concluidas": concluidas, "planejadas": planejadas, "como": COMO});
    }
    Ok(e)
}
