//! Testes das apresentações geradas pela IA: validação (limites, ids, avisos), armazenamento e montagem da página.
mod armazenar;
mod editar;
mod montar;
mod validar;

use serde_json::{json, Value};

pub(super) fn conteudo() -> Value {
    json!({"versao": 1, "publico": "cliente", "nivel": "padrao", "idioma": "pt-BR", "titulo": "Atendimento de usuários",
           "resumo": ["Você recebe o atendimento no WhatsApp."],
           "etapas": [{"id": "e1", "titulo": "Cadastro", "texto": "Fichas dos usuários.", "figura": "fichas"}]})
}

pub(super) fn figura(id: &str) -> Value {
    json!({"id": id, "titulo": "Fichas", "impressao": "abc123", "codigo": "function (L) { return { svg: '<svg></svg>' }; }"})
}

pub(super) fn pasta(nome: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-apresentacao-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}
