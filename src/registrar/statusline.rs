//! Instalação opt-in: preserva a configuração anterior e só desfaz a nossa alteração.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn ler(path: &Path) -> Result<Value, String> {
    if !path.exists() { return Ok(json!({})); }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    if !v.is_object() { return Err(format!("{} precisa ser um objeto JSON", path.display())); }
    Ok(v)
}

fn gravar(path: &Path, v: &Value) -> Result<(), String> {
    let temporario = path.with_extension("json.tmp");
    std::fs::write(&temporario, serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    std::fs::rename(&temporario, path).map_err(|e| e.to_string())
}

fn configurar(pasta: &Path, exe: &Path, remover: bool) -> Result<(), String> {
    let settings = pasta.join("settings.json");
    let estado = pasta.join("blueprint-statusline.json");
    if remover && !estado.exists() { return Ok(()); }
    let mut dados = ler(&settings)?;
    let salvo = if estado.exists() { Some(ler(&estado)?) } else { None };
    if remover {
        let salvo = salvo.unwrap();
        if dados.get("statusLine") == salvo.get("instalada") {
            if salvo["existia"] == true { dados["statusLine"] = salvo["anterior"].clone(); }
            else { dados.as_object_mut().unwrap().remove("statusLine"); }
            gravar(&settings, &dados)?;
        }
        std::fs::remove_file(estado).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if let Some(s) = &salvo {
        if dados.get("statusLine") != s.get("instalada") {
            return Err("statusLine foi alterada desde a instalação; remova o registro anterior antes de instalar novamente".into());
        }
    }
    let anterior = salvo.as_ref().map(|s| s["anterior"].clone()).unwrap_or_else(|| dados.get("statusLine").cloned().unwrap_or(Value::Null));
    let mut instalada = crate::hook::trecho_da_status_line(&exe.display().to_string())["statusLine"].clone();
    if !anterior.is_null() {
        let comando = anterior["command"].as_str().filter(|c| !c.trim().is_empty()).ok_or("statusLine existente não é um comando compatível")?;
        if anterior["type"] != "command" { return Err("statusLine existente não é do tipo command".into()); }
        // O wrapper entrega o mesmo JSON aos dois comandos; um pipe destruiria a entrada do segundo.
        instalada = anterior.clone();
        instalada["command"] = json!(format!("\"{}\" hook statusline --anterior {}", exe.display().to_string().replace('\\', "/"), citar_shell(comando)));
    }
    if dados.get("statusLine") == Some(&instalada) { return Ok(()); }
    std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
    let backup = pasta.join("settings.json.bak");
    if settings.exists() && !backup.exists() { std::fs::copy(&settings, backup).map_err(|e| e.to_string())?; }
    let existia = salvo.as_ref().map(|s| s["existia"].clone()).unwrap_or_else(|| json!(dados.get("statusLine").is_some()));
    gravar(&estado, &json!({"existia": existia, "anterior": anterior, "instalada": instalada}))?;
    dados["statusLine"] = instalada;
    gravar(&settings, &dados)
}

fn citar_shell(comando: &str) -> String {
    format!("'{}'", comando.replace('\'', "'\"'\"'"))
}

pub fn executar(remover: bool) -> (String, i32) {
    let mut pastas = Vec::new();
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) { pastas.push(PathBuf::from(home).join(".claude")); }
    if let Some(dir) = std::env::var_os("CLAUDE_CONFIG_DIR").filter(|d| !d.is_empty()) {
        let pasta = PathBuf::from(dir);
        if !pastas.contains(&pasta) { pastas.push(pasta); }
    }
    let exe = match std::env::current_exe() { Ok(e) => e, Err(e) => return (e.to_string(), 1) };
    let mut linhas = Vec::new(); let mut codigo = 0;
    if pastas.is_empty() { return ("Não encontrei a pasta do usuário".into(), 1); }
    for pasta in pastas {
        match configurar(&pasta, &exe, remover) {
            Ok(()) => linhas.push(format!("{}: status line {}", pasta.display(), if remover { "removida" } else { "instalada" })),
            Err(e) => { codigo = 1; linhas.push(format!("{}: {e}", pasta.display())); }
        }
    }
    (linhas.join("\n"), codigo)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instala_repete_compoe_e_remove_preservando_outros_campos() {
        let pasta = std::env::temp_dir().join(format!("blueprint-statusline-{}-{}", std::process::id(), crate::tempo::agora_ms()));
        std::fs::create_dir_all(&pasta).unwrap();
        let settings = pasta.join("settings.json");
        for anterior in [json!({"tema":"dark"}), json!({"tema":"dark", "statusLine":{"type":"command", "command":"echo 'oi'", "padding":2}})] {
            gravar(&settings, &anterior).unwrap();
            configurar(&pasta, Path::new("C:/Meus Apps/blueprint.exe"), false).unwrap();
            let instalado = ler(&settings).unwrap();
            assert_eq!(instalado["tema"], "dark");
            configurar(&pasta, Path::new("C:/Meus Apps/blueprint.exe"), false).unwrap();
            assert_eq!(ler(&settings).unwrap(), instalado);
            let mut alterado = instalado; alterado["novo"] = json!(true); gravar(&settings, &alterado).unwrap();
            configurar(&pasta, Path::new("unused"), true).unwrap();
            let mut esperado = anterior; esperado["novo"] = json!(true);
            assert_eq!(ler(&settings).unwrap(), esperado);
            assert!(pasta.join("settings.json.bak").exists());
            assert!(!pasta.join("blueprint-statusline.json").exists());
        }
        std::fs::remove_dir_all(pasta).unwrap();
    }
}
