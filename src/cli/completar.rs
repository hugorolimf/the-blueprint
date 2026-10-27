//! `blueprint completions <powershell|bash>`: imprime o script que completa os subcomandos no terminal.

/// Os subcomandos e opções de nível 1; o teste confere que todos estão em `USO`.
pub const SUBCOMANDOS: &[&str] = &[
    "dev", "serve", "--mcp", "watch", "claude", "importar", "exportar", "hook", "atualizar", "restaurar", "registrar", "conectar", "doctor", "completions", "--version",
];

/// Script do shell pedido, ou `None` se o nome não for conhecido.
pub fn script(shell: &str) -> Option<String> {
    let lista = SUBCOMANDOS.join(" ");
    match shell {
        "bash" => Some(format!(
            "_blueprint() {{ COMPREPLY=($(compgen -W \"{lista}\" -- \"${{COMP_WORDS[COMP_CWORD]}}\")); }}\ncomplete -F _blueprint blueprint\n"
        )),
        "powershell" => {
            let itens = SUBCOMANDOS.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(", ");
            Some(format!(
                "Register-ArgumentCompleter -Native -CommandName blueprint -ScriptBlock {{\n  param($palavra, $linha, $cursor)\n  {itens} | Where-Object {{ $_ -like \"$palavra*\" }} | ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}\n}}\n"
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "completar_tests.rs"]
mod tests;
