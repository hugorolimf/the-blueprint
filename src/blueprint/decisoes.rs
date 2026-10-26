//! Decisões do blueprint (decisão de 2026-10-26): `blueprint.decisoes` = `[{id, titulo, escolha, motivo, estado, em}]`, sem tabela (vão em `extra`, como `projetos`).
//! Na proposta do agente: `blueprint.decisoes`. Item com `id` atualiza (o que não vier fica como está); sem `id` é novo. Nada é apagado por aqui.
use super::texto::{campo, lista, ou, str_de, truncar};
use serde_json::{json, Value};

const MAX_POR_PROPOSTA: usize = 12;
pub const ABERTA: &str = "aberta";
pub const DECIDIDA: &str = "decidida";

pub fn decisoes_do(p: &Value) -> &[Value] {
    lista(p.get("decisoes"))
}

/// Cada decisão proposta já completa (o que o agente omitiu vem da decisão existente), com `id` vazio nas novas.
pub fn decisoes_propostas(dados_blueprint: Option<&Value>, atual: Option<&Value>, erros: &mut Vec<String>) -> Vec<Value> {
    let existentes: &[Value] = atual.map(decisoes_do).unwrap_or(&[]);
    let enviadas = lista(dados_blueprint.and_then(|b| b.get("decisoes")));
    if enviadas.len() > MAX_POR_PROPOSTA {
        erros.push(crate::tr!("Too many decisions (at most {MAX_POR_PROPOSTA} per proposal).", "Decisões demais (máximo {MAX_POR_PROPOSTA} por proposta)."));
    }
    let mut saida = Vec::new();
    for (i, x) in enviadas.iter().take(MAX_POR_PROPOSTA).enumerate() {
        let rotulo = format!("`blueprint.decisoes[{i}]`");
        let id = campo(x, "id");
        let existente = if id.is_empty() { None } else { existentes.iter().find(|d| str_de(d, "id") == id) };
        if !id.is_empty() && existente.is_none() {
            erros.push(crate::tr!("{rotulo}: id \"{id}\" does not exist in this blueprint (leave it empty for a new decision; use only ids from ler_blueprint).", "{rotulo}: o id \"{id}\" não existe neste blueprint (deixe vazio numa decisão nova; use só ids de ler_blueprint)."));
            continue;
        }
        let antes = |k: &str| existente.map(|e| str_de(e, k)).unwrap_or_default();
        let titulo = ou(campo(x, "titulo"), antes("titulo"));
        if titulo.is_empty() {
            erros.push(crate::tr!("{rotulo}: the title is missing.", "{rotulo}: falta o título."));
            continue;
        }
        let escolha = ou(campo(x, "escolha"), antes("escolha"));
        let estado = match x.get("estado") {
            None | Some(Value::Null) if existente.is_some() => antes("estado"),
            None | Some(Value::Null) => (if escolha.is_empty() { ABERTA } else { DECIDIDA }).to_string(),
            Some(Value::String(s)) if s == ABERTA || s == DECIDIDA => s.clone(),
            Some(outro) => {
                erros.push(crate::tr!("{rotulo}: `estado` must be \"aberta\" or \"decidida\" (got {outro}).", "{rotulo}: `estado` deve ser \"aberta\" ou \"decidida\" (recebido {outro})."));
                continue;
            }
        };
        if estado == DECIDIDA && escolha.is_empty() {
            erros.push(crate::tr!("{rotulo}: a decided decision needs `escolha` (what was chosen); use `estado: \"aberta\"` while it is not decided.", "{rotulo}: uma decisão decidida precisa de `escolha` (o que foi escolhido); use `estado: \"aberta\"` enquanto não há decisão."));
            continue;
        }
        saida.push(json!({"id": id, "titulo": truncar(&titulo, 120), "escolha": truncar(&escolha, 2000), "motivo": truncar(&ou(campo(x, "motivo"), antes("motivo")), 3000), "estado": estado}));
    }
    saida
}

/// O que `ler_blueprint` mostra de cada decisão: o bastante para o agente não contradizer o que já foi decidido.
pub fn resumo_das_decisoes(p: &Value) -> Vec<Value> {
    decisoes_do(p).iter().map(|d| json!({"id": str_de(d, "id"), "titulo": str_de(d, "titulo"), "estado": str_de(d, "estado"), "escolha": truncar(&str_de(d, "escolha"), 200)})).collect()
}
