//! `blueprint exportar <banco.db> <pasta> --markdown`: um índice e um arquivo por ação, para versionar o plano junto do código. Só leitura: o banco continua sendo a fonte.
use super::Dados;
use crate::contexto::Contexto;
use crate::erro::Resultado;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Nome de arquivo seguro: letras e números em minúsculas, o resto vira hífen.
fn slug(texto: &str) -> String {
    let s: String = texto.chars().map(|c| if c.is_alphanumeric() { c.to_lowercase().next().unwrap_or(c) } else { '-' }).collect();
    let s = s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if s.is_empty() { "sem-nome".into() } else { s.chars().take(60).collect() }
}

fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// Os estados guardados no banco, em inglês para o leitor do repositório.
fn estado(a: &Value) -> &str {
    match texto(a, "status") {
        "planejado" => "planned",
        "andamento" => "in progress",
        "concluido" => "done",
        outro => outro,
    }
}

fn marca(feito: bool) -> &'static str {
    if feito { "[x]" } else { "[ ]" }
}

/// A evidência de um plano é o último comentário de conclusão do agente.
fn evidencia(plano: &Value) -> Option<&str> {
    let comentarios = plano.pointer("/agente/comentarios")?.as_array()?;
    comentarios.iter().rev().find(|c| texto(c, "tipo") == "concluir").map(|c| texto(c, "texto"))
}

/// O arquivo de uma ação: cabeçalho com id e datas, instruções, planos e checkpoints.
pub fn acao(a: &Value) -> String {
    let mut s = format!("# {}\n\n", texto(a, "nome"));
    s += &format!("- id: `{}`\n- dates: {} to {}\n- status: {}\n\n", texto(a, "id"), texto(a, "inicio"), texto(a, "fim"), estado(a));
    if !texto(a, "obs").is_empty() {
        s += &format!("{}\n\n", texto(a, "obs"));
    }
    s += "## Plans\n\n";
    for p in lista(a, "planos") {
        let feito = p.get("feito").and_then(Value::as_bool).unwrap_or(false);
        s += &format!("- {} {}\n", marca(feito), texto(p, "texto"));
        if let Some(e) = evidencia(p).filter(|_| feito) {
            s += &format!("  - evidence: {}\n", e.replace('\n', " "));
        }
    }
    s += "\n## Checkpoints\n\n";
    for c in lista(a, "checkpoints") {
        let feito = c.get("feito").and_then(Value::as_bool).unwrap_or(false);
        s += &format!("- {} {} ({})\n", marca(feito), texto(c, "nome"), texto(c, "data"));
    }
    s
}

/// Os arquivos de um blueprint: (caminho relativo, conteúdo), com o índice primeiro.
pub fn blueprint(b: &Value) -> Vec<(String, String)> {
    let pasta = slug(texto(b, "nome"));
    let mut indice = format!("# {}\n\n", texto(b, "nome"));
    if !texto(b, "descricao").is_empty() {
        indice += &format!("{}\n\n", texto(b, "descricao"));
    }
    indice += "## Actions\n\n";
    let mut arquivos = Vec::new();
    for (i, a) in lista(b, "acoes").iter().enumerate() {
        let nome = format!("{:02}-{}.md", i + 1, slug(texto(a, "nome")));
        let feitos = lista(a, "planos").iter().filter(|p| p.get("feito").and_then(Value::as_bool) == Some(true)).count();
        indice += &format!("- [{}]({}) ({}, {}/{} plans)\n", texto(a, "nome"), nome, estado(a), feitos, lista(a, "planos").len());
        arquivos.push((format!("{pasta}/{nome}"), acao(a)));
    }
    arquivos.insert(0, (format!("{pasta}/README.md"), indice));
    arquivos
}

/// Todos os blueprints do JSON (os da raiz e os dos workspaces).
pub fn todos(tudo: &Value) -> Vec<(String, String)> {
    let da_raiz = lista(tudo, "blueprints").iter();
    let dos_workspaces = lista(tudo, "workspaces").iter().flat_map(|w| lista(w, "blueprints").iter());
    da_raiz.chain(dos_workspaces).flat_map(blueprint).collect()
}

pub fn exportar_markdown(banco: &Path, pasta: &Path) -> Resultado<usize> {
    let mut dados = Dados::abrir(banco)?;
    let arquivos = todos(dados.ler_tudo(&Contexto::local())?);
    for (caminho, conteudo) in &arquivos {
        let destino = pasta.join(caminho);
        if let Some(p) = destino.parent() {
            fs::create_dir_all(p)?;
        }
        fs::write(destino, conteudo)?;
    }
    Ok(arquivos.len())
}

#[cfg(test)]
#[path = "markdown_tests.rs"]
mod tests;
