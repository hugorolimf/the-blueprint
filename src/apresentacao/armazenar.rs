//! Uma apresentação por arquivo, `<pasta>/<id>.json`: `{id, workspaceId, blueprintId, titulo, publico, nivel, em, conteudo, figuras}`. O id é o mesmo formato dos
//! protótipos (16 hexadecimais), então nenhum caminho vindo de fora chega ao disco sem passar por `id_valido`.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub fn id_valido(id: &str) -> bool {
    crate::mcp::acoes::id_prototipo_valido(id)
}

fn arquivo(pasta: &Path, id: &str) -> PathBuf {
    pasta.join(format!("{id}.json"))
}

/// Grava num `.tmp` e renomeia: quem lê nunca vê meio arquivo. Cria a pasta se preciso.
pub fn gravar(pasta: &Path, id: &str, registro: &Value) -> std::io::Result<()> {
    std::fs::create_dir_all(pasta)?;
    let (tmp, final_) = (pasta.join(format!("{id}.tmp")), arquivo(pasta, id));
    std::fs::write(&tmp, registro.to_string())?;
    std::fs::rename(&tmp, &final_).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

pub fn ler(pasta: &Path, id: &str) -> Option<Value> {
    if !id_valido(id) {
        return None;
    }
    std::fs::read(arquivo(pasta, id)).ok().and_then(|b| serde_json::from_slice(&b).ok())
}

/// Apaga o arquivo (não falha se já não existe). Só mexe em id de formato válido.
pub fn remover(pasta: &Path, id: &str) {
    if id_valido(id) {
        let _ = std::fs::remove_file(arquivo(pasta, id));
    }
}

/// O alvo guardado no arquivo: (workspace, blueprint).
pub fn alvo(registro: &Value) -> (&str, &str) {
    let t = |k: &str| registro.get(k).and_then(Value::as_str).unwrap_or("");
    (t("workspaceId"), t("blueprintId"))
}

/// O que o agente e a página leem sem abrir o conteúdo: `{id, titulo, publico, nivel, em, editadoEm?, editadoPor?, figuras: [{id, impressao}]}`.
pub fn meta(registro: &Value) -> Value {
    let figuras: Vec<Value> = registro.get("figuras").and_then(Value::as_array).into_iter().flatten().map(|f| json!({"id": f.get("id"), "impressao": f.get("impressao").cloned().unwrap_or(Value::Null)})).collect();
    let mut m = json!({"id": registro["id"], "titulo": registro["titulo"], "publico": registro["publico"], "nivel": registro["nivel"], "em": registro["em"], "figuras": figuras});
    for k in ["editadoEm", "editadoPor"] {
        if let Some(v) = registro.get(k) {
            m[k] = v.clone(); // só existe depois de uma edição à mão ou do agente
        }
    }
    m
}

/// Os metadados das apresentações de um blueprint, da mais nova para a mais antiga.
pub fn listar(pasta: &Path, workspace: &str, blueprint: &str) -> Vec<Value> {
    let mut achadas: Vec<Value> = std::fs::read_dir(pasta)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from).filter(|id| id_valido(id) && e.path().extension().is_some_and(|x| x == "json")))
        .filter_map(|id| ler(pasta, &id))
        .filter(|r| alvo(r) == (workspace, blueprint))
        .map(|r| meta(&r))
        .collect();
    achadas.sort_by(|a, b| b["em"].as_str().cmp(&a["em"].as_str()));
    achadas
}
