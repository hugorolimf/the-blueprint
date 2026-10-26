//! /api/dados: o data.json inteiro. A página lê tudo ao abrir e grava tudo ao salvar (com debounce, do lado dela).
//! A gravação é uma transação do SQLite: ou entra inteira ou nada muda.
//!
//! Várias janelas (e scripts) podem estar abertas ao mesmo tempo, então a rota controla a concorrência por versão: `GET` devolve a versão em
//! `x-dados-versao`; um `PUT` com `If-Match` de versão velha é recusado com 409 (e a versão atual), e a página junta as duas edições antes de
//! tentar de novo. Cada gravação avisa todas as janelas pelo SSE (`dados`), que se atualizam na hora. Sem `If-Match` o PUT continua valendo
//! como antes (importadores e scripts simples).
use super::estado::Estado;
use super::pedido::{Pedido, Resposta};
use crate::contexto::Contexto;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;

/// `"123"` e `123` valem igual: o cabeçalho pode vir com aspas.
fn versao_pedida(pedido: &Pedido) -> Option<u64> {
    pedido.cabecalho("if-match")?.trim().trim_matches('"').parse().ok()
}

pub fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta {
    match pedido.metodo.as_str() {
        // modo hospedado: cada um lê só o que o papel alcança (acesso::filtro), serializado por pedido; quem lê tudo usa a cópia de sempre
        "GET" if !crate::acesso::filtro::le_tudo(ctx) => {
            let mut dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
            let versao = estado.versao_dados.load(Ordering::SeqCst);
            match dados.ler_tudo(ctx).map(|tudo| serde_json::to_vec(&crate::acesso::filtro::filtrar(tudo, ctx))) {
                Ok(Ok(bytes)) => Resposta::texto(200, "application/json", bytes).com("x-dados-versao", &versao.to_string()),
                _ => Resposta::texto(500, "text/plain", "erro"),
            }
        }
        // o PUT troca o arquivo inteiro: com dados filtrados apagaria o que a pessoa não vê, então no modo hospedado é só do dono de todos
        "PUT" if !crate::acesso::filtro::le_tudo(ctx) => Resposta::json(403, &json!({"erro": crate::tr!("save by changes (POST /api/mudancas); replacing everything is only for the server owner", "grave por mudanças (POST /api/mudancas); trocar tudo é só do dono do servidor"), "codigo": "use-mudancas"})),
        "GET" => {
            let mut guardado = estado.dados_json.lock().unwrap_or_else(|e| e.into_inner());
            if guardado.is_none() {
                let mut dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
                match dados.ler_tudo(ctx).map(serde_json::to_vec) {
                    Ok(Ok(bytes)) => *guardado = Some(bytes),
                    _ => return Resposta::texto(500, "text/plain", "erro"),
                }
            }
            // a versão é lida com a cópia travada: nenhuma gravação passa entre o texto e o número
            let versao = estado.versao_dados.load(Ordering::SeqCst);
            Resposta::texto(200, "application/json", guardado.clone().unwrap_or_default()).com("x-dados-versao", &versao.to_string())
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
            let atual = estado.versao_dados.load(Ordering::SeqCst);
            if pedido.cabecalho("if-match").is_some() && versao_pedida(pedido) != Some(atual) {
                return Resposta::json(409, &json!({"erro": "desatualizado", "codigo": "desatualizado", "versao": atual})).com("x-dados-versao", &atual.to_string());
            }
            *guardado = None; // qualquer que seja o resultado, a próxima leitura refaz a cópia
            let mut dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
            match dados.gravar_tudo(ctx, &valor) {
                Ok(_) => {
                    let nova = estado.versao_dados.fetch_add(1, Ordering::SeqCst) + 1;
                    estado.mcp.notificar("dados", Some(json!({"versao": nova})));
                    Resposta::texto(200, "application/json", r#"{"ok":true}"#).com("x-dados-versao", &nova.to_string())
                }
                Err(crate::erro::Erro::Formato(m)) => Resposta::texto(400, "text/plain", m),
                Err(_) => Resposta::texto(500, "text/plain", "erro"),
            }
        }
        _ => Resposta::texto(405, "text/plain", "método não permitido"),
    }
}
