//! `GET /api/exemplo`: o `data.exemplo.json` (ou o `.en`, pelo idioma do pedido), embutido no binário. É só o modelo: a página
//! (web/js/primeiro-uso.js) troca todos os ids, traz as datas para perto de hoje e grava como um workspace NOVO por `/api/mudancas`,
//! como qualquer workspace criado à mão. Nada aqui grava: os dados existentes não têm como ser tocados por esta rota.
use super::pedido::{falha, Pedido, Resposta};

const PT: &str = include_str!("../../data.exemplo.json");
const EN: &str = include_str!("../../data.exemplo.en.json");

pub fn rota(pedido: &Pedido) -> Resposta {
    if pedido.metodo != "GET" {
        return falha(405, crate::tr!("method not allowed", "método não permitido")).codigo("metodo").resposta();
    }
    let corpo = if crate::idioma::ingles() { EN } else { PT };
    Resposta::texto(200, "application/json; charset=utf-8", corpo).com("cache-control", "no-store")
}
