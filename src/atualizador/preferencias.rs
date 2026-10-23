//! O que o updater lembra em arquivos pequenos na pasta `atualizacao/` (fora do banco: não precisa de migração e some junto com a pasta):
//! `verificacao.json` (quando verificou pela última vez), `pronta.json` (o instalador já baixado) e `desligada` (o usuário desligou a verificação).
use super::estado::{Fase, Info};
use super::Atualizador;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Passadas 24 h desde a última verificação, o app verifica de novo ao abrir (nunca em laço de repouso).
pub const INTERVALO_MS: i64 = 24 * 60 * 60 * 1000;

impl Atualizador {
    pub(super) fn arquivo(&self, nome: &str) -> PathBuf {
        self.i.pasta.join(nome)
    }

    /// `BLUEPRINT_SEM_ATUALIZACAO=1` ou o usuário desligou (item da bandeja): não verifica nem baixa sozinho. A verificação pedida pelo usuário ainda vale.
    pub fn automatica(&self) -> bool {
        std::env::var("BLUEPRINT_SEM_ATUALIZACAO").as_deref() != Ok("1") && !self.arquivo("desligada").exists()
    }

    pub fn definir_automatica(&self, ligada: bool) {
        let _ = std::fs::create_dir_all(&self.i.pasta);
        if ligada {
            let _ = std::fs::remove_file(self.arquivo("desligada"));
        } else {
            let _ = std::fs::write(self.arquivo("desligada"), "");
        }
        self.mudar(|e| {
            if !ligada && matches!(e.fase, Fase::Parada | Fase::EmDia) {
                e.fase = Fase::Desligada;
            } else if ligada && e.fase == Fase::Desligada {
                e.fase = Fase::Parada;
            }
        });
    }

    pub(super) fn gravar_verificacao(&self) {
        let _ = std::fs::create_dir_all(&self.i.pasta);
        let _ = std::fs::write(self.arquivo("verificacao.json"), json!({"emMs": crate::tempo::agora_ms()}).to_string());
    }

    /// Passou o intervalo desde a última verificação (ou nunca houve)?
    pub fn passou_o_intervalo(&self, agora_ms: i64) -> bool {
        let ultima = std::fs::read_to_string(self.arquivo("verificacao.json")).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v["emMs"].as_i64());
        ultima.is_none_or(|u| agora_ms - u >= INTERVALO_MS || agora_ms < u)
    }

    pub(super) fn gravar_pronta(&self, info: &Info) {
        let corpo = json!({"versao": info.versao, "notas": info.notas, "arquivo": info.arquivo.as_ref().map(|a| a.to_string_lossy()), "sha256": info.sha256});
        let _ = std::fs::write(self.arquivo("pronta.json"), corpo.to_string());
    }

    pub(super) fn ler_pronta(&self) -> Option<Info> {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(self.arquivo("pronta.json")).ok()?).ok()?;
        Some(Info { fase: Fase::Pronta, versao: Some(v["versao"].as_str()?.to_string()), notas: v["notas"].as_str().unwrap_or("").to_string(), arquivo: Some(PathBuf::from(v["arquivo"].as_str()?)), sha256: Some(v["sha256"].as_str()?.to_string()), motivo: None })
    }

    pub(super) fn apagar_pronta(&self) {
        let _ = std::fs::remove_file(self.arquivo("pronta.json"));
    }
}
