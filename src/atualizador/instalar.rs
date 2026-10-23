//! A instalação: só acontece quando o usuário clica em Atualizar (interface ou bandeja) ou digita `blueprint atualizar --instalar`. Antes de rodar o instalador,
//! confere o SHA-256 de novo (o arquivo ficou em disco desde o download), copia o banco e recusa se houver trabalho do agente que se perderia ao fechar o app.
use super::estado::Fase;
use super::{hash, Atualizador};
use std::path::Path;

/// Por que o clique não instalou. `codigo` é estável (a interface decide o texto por ele); `mensagem` vai direto ao usuário.
#[derive(Debug)]
pub struct Recusa {
    pub status: u16,
    pub codigo: &'static str,
    pub mensagem: String,
}

fn recusa(status: u16, codigo: &'static str, mensagem: impl Into<String>) -> Recusa {
    Recusa { status, codigo, mensagem: mensagem.into() }
}

/// Instalação em "Program Files" (para todos os usuários) pede administrador: o instalador aparece com o UAC em vez de rodar 100% em silêncio.
fn para_todos(exe: &Path) -> bool {
    ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"].iter().filter_map(|v| std::env::var_os(v)).any(|pf| exe.starts_with(pf))
}

/// Os argumentos do Inno Setup: sem janelas de pergunta, fechando o app se ainda estiver aberto e sem reiniciar o Windows. `/ATUALIZACAO=1` faz o
/// instalador abrir o app de novo no fim (o `.iss` só o faz nesse caso).
pub fn argumentos(exe_instalado: &Path) -> Vec<String> {
    let modo = if para_todos(exe_instalado) { "/SILENT" } else { "/VERYSILENT" };
    [modo, "/SUPPRESSMSGBOXES", "/CLOSEAPPLICATIONS", "/NORESTART", "/ATUALIZACAO=1"].iter().map(|s| (*s).to_string()).collect()
}

impl Atualizador {
    /// Instala a versão que está pronta. `bloqueio` é o motivo para não fechar o app agora (quem chama o calcula: fila do MCP, agente trabalhando); `backup(versao)`
    /// copia o banco; `lancar(instalador, argumentos)` roda o instalador e diz se o app deve sair em seguida. Devolve esse "deve sair".
    pub fn instalar(&self, bloqueio: Option<String>, backup: &dyn Fn(&str) -> Result<(), String>, lancar: &dyn Fn(&Path, &[String]) -> Result<bool, String>) -> Result<bool, Recusa> {
        let (versao, arquivo, esperado) = {
            let e = self.i.estado.lock().unwrap_or_else(|e| e.into_inner());
            match (&e.fase, &e.versao, &e.arquivo, &e.sha256) {
                (Fase::Pronta, Some(v), Some(a), Some(h)) => (v.clone(), a.clone(), h.clone()),
                _ => return Err(recusa(409, "sem-atualizacao", "Não há atualização pronta para instalar.")),
            }
        };
        if let Some(motivo) = bloqueio {
            self.mudar(|e| e.motivo = Some(motivo.clone()));
            return Err(recusa(409, "agora-nao", motivo));
        }
        if !arquivo.starts_with(&self.i.pasta) || !hash::confere(&arquivo, &esperado) {
            let _ = std::fs::remove_file(&arquivo);
            self.apagar_pronta();
            let texto = "O instalador baixado mudou depois do download e foi descartado; o app vai baixar de novo na próxima verificação.";
            self.mudar(|e| *e = super::estado::Info { fase: Fase::Erro(texto.into()), ..Default::default() });
            return Err(recusa(409, "arquivo-alterado", texto));
        }
        backup(&versao).map_err(|e| recusa(500, "backup", format!("Não instalei: {e}")))?;
        self.mudar(|e| {
            e.fase = Fase::Instalando;
            e.motivo = None;
        });
        let exe = std::env::current_exe().unwrap_or_default();
        match lancar(&arquivo, &argumentos(&exe)) {
            Ok(sair) => Ok(sair),
            Err(e) => {
                self.mudar(|i| {
                    i.fase = Fase::Pronta;
                    i.motivo = Some(e.clone());
                });
                Err(recusa(500, "instalador", e))
            }
        }
    }
}

/// O lançador de verdade: roda o instalador desacoplado (ele sobrevive ao app, que sai logo depois). `BLUEPRINT_ATUALIZACAO_SEM_EXECUTAR=<arquivo>` (testes):
/// só anota o que rodaria nesse arquivo e não manda o app sair.
pub fn lancar_instalador(arquivo: &Path, args: &[String]) -> Result<bool, String> {
    if let Some(anotar) = std::env::var_os("BLUEPRINT_ATUALIZACAO_SEM_EXECUTAR").filter(|v| !v.is_empty()) {
        std::fs::write(anotar, format!("{} {}", arquivo.display(), args.join(" "))).map_err(|e| format!("não consegui anotar: {e}"))?;
        return Ok(false);
    }
    let mut c = std::process::Command::new(arquivo);
    c.args(args);
    desacoplar(&mut c);
    c.spawn().map(|_| true).map_err(|e| format!("não consegui rodar o instalador: {e}"))
}

#[cfg(windows)]
fn desacoplar(c: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    // sem console (janela, bandeja) não há stdio para herdar: o spawn falharia com o erro 50 do Windows
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    c.creation_flags(0x0000_0008 | 0x0000_0200); // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
}
#[cfg(not(windows))]
fn desacoplar(_c: &mut std::process::Command) {}
