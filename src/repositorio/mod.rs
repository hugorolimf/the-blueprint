//! Repositório GitHub do blueprint (contrato em docs/repositorio-github.md): `blueprints[].repositorio = {url}` (vai em `extra`, sem tabela).
//! Aqui: a regra da URL (o texto vem de outra pessoa e nada roda com ele sem passar por `repositorio_valido`), a conferência do acesso com o
//! git da própria pessoa, o prompt de "Configurar projeto" e as rotas `/api/repositorio/*` (só no app local).
pub mod acesso;
pub mod prompt;
pub mod rotas;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_rotas;

#[derive(Debug, PartialEq, Eq)]
pub struct Repo {
    /// `https://github.com/<dono>/<nome>`, sem `.git` e sem barra no fim
    pub url: String,
    pub dono: String,
    pub nome: String,
}

fn dono_valido(d: &str) -> bool {
    (1..=39).contains(&d.len()) && !d.starts_with('-') && d.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn nome_valido(n: &str) -> bool {
    (1..=100).contains(&n.len()) && !matches!(n, "." | "..") && !n.starts_with('-') && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// A única URL aceita: `https://github.com/<dono>/<repo>` (com `.git` e/ou `/` no fim). Sem usuário/senha, consulta, fragmento, espaço, outro host
/// ou esquema (`ext::`, `file:`, `ssh`). shortcut: só github.com e só o ramo padrão; GitHub Enterprise e ramo escolhido ficam para quando alguém pedir.
pub fn repositorio_valido(texto: &str) -> Option<Repo> {
    let resto = texto.trim().strip_prefix("https://github.com/")?;
    let resto = resto.strip_suffix('/').unwrap_or(resto);
    let resto = resto.strip_suffix(".git").unwrap_or(resto);
    let (dono, nome) = resto.split_once('/')?;
    (dono_valido(dono) && nome_valido(nome)).then(|| Repo { url: format!("https://github.com/{dono}/{nome}"), dono: dono.into(), nome: nome.into() })
}
