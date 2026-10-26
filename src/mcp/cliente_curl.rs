//! O `--mcp` falando com um servidor hospedado (`https://`): o app não tem TLS, então usa o `curl` do sistema, como o atualizador. A chave
//! (BLUEPRINT_CHAVE) e o corpo vão pela entrada padrão do curl (`--config -`), nunca nos argumentos do processo, que outros usuários da
//! máquina conseguem listar.
use std::io::Write;
use std::process::{Command, Stdio};

/// Uma linha de `--config` entre aspas: o curl entende `\\` e `\"` (o JSON compacto não tem quebra de linha crua).
fn aspas(s: &str) -> String {
    let mut saida = String::with_capacity(s.len() + 2);
    saida.push('"');
    for c in s.chars() {
        match c {
            '\\' => saida.push_str("\\\\"),
            '"' => saida.push_str("\\\""),
            '\n' => saida.push_str("\\n"),
            '\r' => saida.push_str("\\r"),
            '\t' => saida.push_str("\\t"),
            c => saida.push(c),
        }
    }
    saida.push('"');
    saida
}

/// O texto que vai para a entrada padrão do curl. O status volta na última linha da saída (`write-out`).
pub fn config(url: &str, metodo: &str, corpo: Option<&str>, chave: Option<&str>, idioma: &str) -> String {
    let mut linhas = vec![
        format!("url = {}", aspas(url)),
        format!("request = {}", aspas(metodo)),
        format!("header = {}", aspas("Accept: application/json")),
        format!("header = {}", aspas(&format!("Accept-Language: {idioma}"))),
        "max-time = 15".to_string(),
        format!("write-out = {}", aspas("\n%{http_code}")),
    ];
    if let Some(chave) = chave {
        linhas.push(format!("header = {}", aspas(&format!("Authorization: Bearer {chave}"))));
    }
    if let Some(corpo) = corpo {
        linhas.push(format!("header = {}", aspas("Content-Type: application/json")));
        linhas.push(format!("data-binary = {}", aspas(corpo)));
    }
    linhas.join("\n") + "\n"
}

/// (status, corpo) da saída do curl com `write-out`.
pub fn separar(saida: &str) -> Option<(u16, String)> {
    let (corpo, status) = saida.rsplit_once('\n')?;
    Some((status.trim().parse().ok().filter(|s| *s > 0)?, corpo.to_string()))
}

pub fn requisitar(url: &str, metodo: &str, corpo: Option<&str>, chave: Option<&str>, sem_resposta: impl Fn() -> String) -> Result<(u16, String), String> {
    let mut c = Command::new(crate::atualizador::rede::curl());
    c.args(["--silent", "--show-error", "--config", "-"]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // sem janela de console
    }
    let mut filho = c.spawn().map_err(|e| crate::tr!("could not run curl (it comes with Windows 10 and 11): {e}", "não consegui rodar o curl (ele vem no Windows 10 e 11): {e}"))?;
    let texto = config(url, metodo, corpo, chave, crate::idioma::atual().codigo());
    if let Some(mut entrada) = filho.stdin.take() {
        entrada.write_all(texto.as_bytes()).map_err(|_| sem_resposta())?;
    }
    let saida = filho.wait_with_output().map_err(|_| sem_resposta())?;
    separar(&String::from_utf8_lossy(&saida.stdout)).ok_or_else(sem_resposta)
}
