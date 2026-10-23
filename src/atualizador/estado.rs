//! O que o updater sabe em cada momento: a fase e, quando há versão nova, qual é e onde está o instalador.
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Fase {
    /// ainda não verificou nesta execução
    #[default]
    Parada,
    /// o usuário (ou `BLUEPRINT_SEM_ATUALIZACAO=1`) desligou a verificação automática
    Desligada,
    Verificando,
    EmDia,
    Baixando(u8),
    /// instalador baixado e conferido: só falta o clique do usuário
    Pronta,
    Instalando,
    Erro(String),
}

#[derive(Clone, Debug, Default)]
pub struct Info {
    pub fase: Fase,
    /// a versão nova (sem o `v`), quando há
    pub versao: Option<String>,
    pub notas: String,
    pub arquivo: Option<PathBuf>,
    /// o SHA-256 que o arquivo precisa ter
    pub sha256: Option<String>,
    /// por que o último clique em Atualizar foi recusado (fila do MCP cheia, agente trabalhando…); some na próxima mudança de fase
    pub motivo: Option<String>,
}

impl Info {
    pub fn json(&self, atual: &str, automatica: bool) -> Value {
        let (fase, percentual, erro) = match &self.fase {
            Fase::Parada => ("parada", None, None),
            Fase::Desligada => ("desligada", None, None),
            Fase::Verificando => ("verificando", None, None),
            Fase::EmDia => ("em-dia", None, None),
            Fase::Baixando(p) => ("baixando", Some(*p), None),
            Fase::Pronta => ("pronta", None, None),
            Fase::Instalando => ("instalando", None, None),
            Fase::Erro(e) => ("erro", None, Some(e.as_str())),
        };
        json!({"fase": fase, "atual": atual, "versao": self.versao, "percentual": percentual, "notas": self.notas, "erro": erro, "motivo": self.motivo, "automatica": automatica})
    }
}
