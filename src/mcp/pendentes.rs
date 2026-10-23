//! Pendentes: o que o agente registrou enquanto o app estava fechado. Sem o servidor de pé, o MCP (`--mcp`) guarda cada registro (progresso, plano ou checkpoint
//! concluído com evidência, proposta, protótipo, vínculo de pasta) num arquivo pequeno em `pendentes/`, ao lado do banco; ao abrir, o servidor reenvia cada um, na
//! ordem, às mesmas rotas que o agente usaria ao vivo (`rotas::reenviar`), com a mesma validação. Nada altera o banco por aqui: o que entra vira item da fila
//! do MCP e a página o aplica como sempre. Um arquivo por registro (escrito num `.tmp` e renomeado): vários agentes escrevem ao mesmo tempo sem se atropelar.
//! shortcut: o horário do registro é o do reenvio (a fila carimba `em` ao receber); a ordem, não.
use super::rotas;
use super::Mcp;
use crate::dados::Dados;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// As rotas POST que valem guardar com o app fechado (o resto precisa dos dados ao vivo).
pub const ROTAS: [&str; 5] = ["evento", "proposta", "registro", "prototipo", "pasta/vincular"];
const RECUSADOS: &str = "recusados";

/// A pasta dos pendentes do banco em uso (a mesma para quem guarda e para o servidor que reenvia).
pub fn pasta_do_banco(banco: &Path) -> PathBuf {
    banco.parent().filter(|p| !p.as_os_str().is_empty()).map_or_else(|| PathBuf::from("pendentes"), |p| p.join("pendentes"))
}

/// Guarda um registro para quando o app abrir. Devolve quantos pendentes há agora.
pub fn guardar(pasta: &Path, rota: &str, corpo: &Value) -> Result<usize, String> {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    if !ROTAS.contains(&rota) {
        return Err(format!("a rota {rota} não pode ser guardada com o app fechado"));
    }
    std::fs::create_dir_all(pasta).map_err(|e| format!("não consegui criar a pasta de pendentes: {e}"))?;
    let nome = format!("{:020}-{}-{}", crate::tempo::agora_ms(), std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed));
    let (tmp, final_) = (pasta.join(format!("{nome}.tmp")), pasta.join(format!("{nome}.json")));
    let registro = json!({"rota": rota, "corpo": corpo, "guardadoEm": crate::tempo::iso_utc_agora()});
    std::fs::write(&tmp, registro.to_string()).and_then(|_| std::fs::rename(&tmp, &final_)).map_err(|e| format!("não consegui guardar o registro: {e}"))?;
    Ok(listar(pasta).len())
}

/// Os registros esperando, do mais antigo ao mais novo.
fn listar(pasta: &Path) -> Vec<PathBuf> {
    let mut arquivos: Vec<PathBuf> = std::fs::read_dir(pasta).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    arquivos.sort();
    arquivos
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Resumo {
    pub reenviados: usize,
    /// o servidor recusou na validação (ficam em `pendentes/recusados/` com o motivo, para o usuário ver)
    pub recusados: usize,
}

/// Reenvia tudo o que está em `pasta` às rotas do servidor e apaga o que foi aceito. Chamado ao abrir o app, antes de atender.
pub fn reenviar_todos(mcp: &Mcp, dados: &Mutex<Dados>, pasta: &Path) -> Resumo {
    let mut resumo = Resumo::default();
    for arquivo in listar(pasta) {
        let Some(registro) = std::fs::read_to_string(&arquivo).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else {
            recusar(pasta, &arquivo, "arquivo ilegível");
            resumo.recusados += 1;
            continue;
        };
        let rota = registro["rota"].as_str().unwrap_or("");
        let (status, resposta) = if ROTAS.contains(&rota) { rotas::reenviar(mcp, dados, rota, &registro["corpo"]) } else { (400, json!({"erro": "rota desconhecida"})) };
        if (200..300).contains(&status) {
            let _ = std::fs::remove_file(&arquivo);
            resumo.reenviados += 1;
        } else {
            let motivo = resposta["erro"].as_str().map(String::from).unwrap_or_else(|| resposta["erros"].to_string());
            recusar(pasta, &arquivo, &motivo);
            resumo.recusados += 1;
        }
    }
    resumo
}

/// Tira o registro da fila de espera sem perdê-lo: vai para `recusados/` e o motivo fica num `.txt` ao lado.
fn recusar(pasta: &Path, arquivo: &Path, motivo: &str) {
    let destino = pasta.join(RECUSADOS);
    let _ = std::fs::create_dir_all(&destino);
    let Some(nome) = arquivo.file_name() else { return };
    let _ = std::fs::write(destino.join(Path::new(nome).with_extension("txt")), motivo);
    let _ = std::fs::rename(arquivo, destino.join(nome));
}
