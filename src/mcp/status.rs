//! O estado do blueprint da pasta para quem olha (o comando /blueprint:status e a status line do Claude Code): ação atual, planos e próximo checkpoint.
//! Só leitura, da cópia em memória dos dados (não toca o banco), então a status line pode perguntar com frequência.
use super::hooks::{alvo, texto, travado};
use super::{acoes, Mcp};
use crate::blueprint::numeros_das_acoes;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::ErroHttp;
use serde_json::{json, Value};
use std::sync::Mutex;

type R<T> = Result<T, ErroHttp>;

pub(super) fn lista(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}

pub(super) fn pendente(a: &Value) -> bool {
    lista(&a["planos"]).iter().any(|x| x["feito"] != true)
}

/// A ação em que o trabalho está: a primeira em andamento que ainda tem plano pendente, senão a primeira com plano pendente.
/// Uma ação "em andamento" com todos os planos feitos é só uma ação que ninguém marcou como concluída: não é a atual.
pub(super) fn acao_atual(acoes: &[Value]) -> Option<usize> {
    acoes.iter().position(|a| a["status"] == "andamento" && pendente(a)).or_else(|| acoes.iter().position(pendente))
}

/// `2026-10-27` → `27/10`
fn dia(data: &str) -> String {
    match data.split('-').collect::<Vec<_>>()[..] {
        [_, m, d] => format!("{d}/{m}"),
        _ => data.to_string(),
    }
}

fn curto(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// O texto do comando e a linha curta da status line (`workspace › blueprint · ação …`).
fn montar(workspace: &str, estrutura: &Value, n: usize) -> (String, String) {
    let p = &estrutura["blueprint"];
    let onde = format!("{} › {}", curto(workspace, 16), curto(texto(p, "nome"), 22));
    let acoes = lista(&p["acoes"]);
    let Some(i) = acao_atual(acoes) else {
        let t = crate::tr!("The Blueprint · {workspace} › {}\nAll actions are done.", "The Blueprint · {workspace} › {}\nTodas as ações estão concluídas.", texto(p, "nome"));
        return (t, crate::tr!("Blueprint · {onde} · done", "Blueprint · {onde} · concluído"));
    };
    let a = &acoes[i];
    let numero = &numeros_das_acoes(p, n)[i];
    let planos = lista(&a["planos"]);
    let feitos = planos.iter().filter(|x| x["feito"] == true).count();
    let mut t = crate::tr!("The Blueprint · {workspace} › {}\nAction {numero}: {} ({} to {}) · {feitos}/{} plans\n", "The Blueprint · {workspace} › {}\nAção {numero}: {} ({} a {}) · {feitos}/{} planos\n", texto(p, "nome"), texto(a, "nome"), dia(texto(a, "inicio")), dia(texto(a, "fim")), planos.len());
    for x in planos {
        t += &format!("  [{}] {}\n", if x["feito"] == true { "x" } else { " " }, texto(x, "texto"));
    }
    // o checkpoint pendente mais próximo de todo o blueprint (datas aaaa-mm-dd ordenam como texto)
    let proximo = acoes.iter().flat_map(|a| lista(&a["checkpoints"])).filter(|c| c["feito"] != true).min_by_key(|c| texto(c, "data"));
    match proximo {
        Some(c) => t += &crate::tr!("Next checkpoint: {} · {}", "Próximo checkpoint: {} · {}", dia(texto(c, "data")), texto(c, "nome")),
        None => t += crate::idioma::ou("No pending checkpoints.", "Sem checkpoints pendentes."),
    }
    (t, crate::tr!("Blueprint · {onde} · action {numero} · {} · {feitos}/{}", "Blueprint · {onde} · ação {numero} · {} · {feitos}/{}", curto(texto(a, "nome"), 30), planos.len()))
}

/// `hook/status`: o estado do blueprint da pasta (`cwd`) ou da sessão. Sem blueprint ligado: `vinculado: false` e sem texto.
pub fn status(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let Some((ws, bp)) = alvo(mcp, dados, corpo)? else { return Ok(json!({"vinculado": false})) };
    let mut d = travado(dados);
    let tudo = d.ler_tudo(&Contexto::local())?;
    let n = acoes::numero_do_blueprint(tudo, &ws, &bp); // a folha numera as ações como <nº do projeto>.<nº da ação> (sem projetos, <nº do blueprint>.<nº da ação>)
    let Ok(estrutura) = acoes::blueprint(tudo, &ws, &bp, None) else { return Ok(json!({"vinculado": false})) };
    let nome_ws = acoes::localizar(tudo, &ws, &bp).map(|(w, _)| w.nome).unwrap_or_default();
    let (texto, linha) = montar(&nome_ws, &estrutura, n);
    Ok(json!({"vinculado": true, "texto": texto, "linha": linha}))
}
