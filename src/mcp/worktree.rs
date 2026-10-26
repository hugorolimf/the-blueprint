//! Worktrees do git. O `cwd` que os hooks mandam acompanha a worktree em que o Claude está (`.claude/worktrees/<nome>/` ou qualquer pasta criada com
//! `git worktree add`), mas o vínculo pasta -> blueprint é da pasta principal do repositório. Aqui fica a resolução, só com `std::fs`.
//! Como o git guarda: na worktree, `.git` é um ARQUIVO `gitdir: <repo>/.git/worktrees/<nome>`; essa pasta tem o arquivo `commondir` (em geral `../..`,
//! relativo a ela) que aponta para o `.git` principal, e a pasta principal é a mãe dele. Pasta comum (`.git` é diretório), submódulo
//! (`gitdir: …/.git/modules/…`, sem `commondir`) e repositório sem `.git` principal (bare) não são worktrees: nada muda para eles.
use super::hooks::travado;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::ErroHttp;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// Sobe no máximo isto à procura do `.git` (o `cwd` pode estar numa subpasta da worktree).
const NIVEIS: usize = 40;
/// O arquivo `.git` de uma worktree tem uma linha; maior que isto não é.
const GIT_MAX: u64 = 4096;

/// `\` vira `/` e `.`/`..` somem: `C:\a\b\..\c` e `C:/a/c` ficam iguais. Raiz (`/`, `//servidor` e `C:`) preservada.
pub(super) fn limpar(caminho: &str) -> String {
    let c = caminho.trim().replace('\\', "/");
    let raiz = if c.starts_with("//") { "//" } else if c.starts_with('/') { "/" } else { "" };
    let mut partes: Vec<&str> = Vec::new();
    for p in c.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                if partes.len() > 1 || partes.first().is_some_and(|u| !u.ends_with(':')) {
                    partes.pop();
                }
            }
            _ => partes.push(p),
        }
    }
    format!("{raiz}{}", partes.join("/"))
}

fn absoluto(base: &str, relativo: &str) -> String {
    let r = relativo.trim().replace('\\', "/");
    if r.starts_with('/') || r.as_bytes().get(1) == Some(&b':') { limpar(&r) } else { limpar(&format!("{base}/{r}")) }
}

/// O `gitdir:` do arquivo `.git` que está em `dir`, absoluto.
pub(super) fn gitdir_de(dir: &str, conteudo: &str) -> Option<String> {
    let g = conteudo.lines().find_map(|l| l.strip_prefix("gitdir:"))?.trim();
    (!g.is_empty()).then(|| absoluto(dir, g))
}

/// O `.git` principal, a partir do gitdir da worktree e do conteúdo do `commondir` dele; sem esse arquivo, `<repo>/.git/worktrees/<nome>` ainda diz.
pub(super) fn comum_de(gitdir: &str, commondir: Option<&str>) -> Option<String> {
    if let Some(c) = commondir.map(str::trim).filter(|c| !c.is_empty()) {
        return Some(absoluto(gitdir, c));
    }
    let (pai, _) = gitdir.rsplit_once('/')?;
    let (comum, nome) = pai.rsplit_once('/')?;
    (nome == "worktrees").then(|| comum.to_string())
}

/// A pasta de trabalho principal: a mãe de um `.git` (um `.git` com outro nome é repositório bare, sem pasta).
pub(super) fn pasta_de(comum: &str) -> Option<String> {
    let (pasta, nome) = comum.rsplit_once('/')?;
    (nome == ".git" && !pasta.is_empty()).then(|| pasta.to_string())
}

/// Se `cwd` está dentro de uma worktree do git, a pasta principal do repositório (com `/`); senão `None`.
pub(super) fn pasta_principal(cwd: &str) -> Option<String> {
    let mut dir = limpar(cwd);
    for _ in 0..NIVEIS {
        if dir.is_empty() || dir.ends_with(':') {
            return None;
        }
        let git = Path::new(&dir).join(".git");
        match fs::metadata(&git) {
            Ok(m) if m.is_dir() => return None,
            Ok(m) if m.is_file() && m.len() <= GIT_MAX => {
                let gitdir = gitdir_de(&dir, &fs::read_to_string(&git).ok()?)?;
                let commondir = fs::read_to_string(Path::new(&gitdir).join("commondir")).ok();
                let principal = pasta_de(&comum_de(&gitdir, commondir.as_deref())?)?;
                return (principal.to_lowercase() != dir.to_lowercase()).then_some(principal);
            }
            Ok(_) => return None,
            Err(_) => {}
        }
        dir = dir.rsplit_once('/')?.0.to_string();
    }
    None
}

/// A pasta principal da worktree, ou o próprio `cwd` quando não é uma.
pub(super) fn principal_ou(cwd: &str) -> String {
    pasta_principal(cwd).unwrap_or_else(|| cwd.to_string())
}

/// O blueprint da pasta: o do `cwd` e, se não houver vínculo, o da pasta principal da worktree. O disco só é consultado quando falta o vínculo.
pub(super) fn blueprint_da_pasta(dados: &Mutex<Dados>, cwd: &str) -> Result<Option<(String, String)>, ErroHttp> {
    let achado = travado(dados).blueprint_da_pasta(&Contexto::local(), cwd)?;
    if achado.is_some() {
        return Ok(achado);
    }
    let Some(principal) = pasta_principal(cwd) else { return Ok(None) };
    Ok(travado(dados).blueprint_da_pasta(&Contexto::local(), &principal)?)
}
