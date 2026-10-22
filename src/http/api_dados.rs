//! /api/dados: o data.json inteiro. A página lê tudo ao abrir e grava tudo ao salvar (com debounce, do lado dela).
//! A gravação é uma transação do SQLite: ou entra inteira ou nada muda.
use super::estado::Estado;
use super::pedido::{Pedido, Resposta};
use crate::contexto::Contexto;
use serde_json::Value;

pub fn rota(estado: &Estado, pedido: &Pedido) -> Resposta {
    let ctx = Contexto::local();
    match pedido.metodo.as_str() {
        "GET" => {
            let mut guardado = estado.dados_json.lock().unwrap_or_else(|e| e.into_inner());
            if guardado.is_none() {
                let mut dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
                match dados.ler_tudo(&ctx).map(serde_json::to_vec) {
                    Ok(Ok(bytes)) => *guardado = Some(bytes),
                    _ => return Resposta::texto(500, "text/plain", "erro"),
                }
            }
            Resposta::texto(200, "application/json", guardado.clone().unwrap_or_default())
        }
        "PUT" => {
            if !pedido.eh_json() {
                return Resposta::texto(415, "text/plain", "envie application/json");
            }
            let Ok(valor) = serde_json::from_slice::<Value>(&pedido.corpo) else { return Resposta::texto(400, "text/plain", "erro") };
            if !valor.get("blueprints").is_some_and(Value::is_array) {
                return Resposta::texto(400, "text/plain", "formato inválido: falta blueprints[]");
            }
            let mut guardado = estado.dados_json.lock().unwrap_or_else(|e| e.into_inner());
            *guardado = None; // qualquer que seja o resultado, a próxima leitura refaz a cópia
            let mut dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
            match dados.gravar_tudo(&ctx, &valor) {
                Ok(_) => Resposta::texto(200, "application/json", r#"{"ok":true}"#),
                Err(crate::erro::Erro::Formato(m)) => Resposta::texto(400, "text/plain", m),
                Err(_) => Resposta::texto(500, "text/plain", "erro"),
            }
        }
        _ => Resposta::texto(405, "text/plain", "método não permitido"),
    }
}
