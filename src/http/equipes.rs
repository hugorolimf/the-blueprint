//! As equipes a que este app já entrou (src/equipes.rs), para a tela de workspaces: listar, abrir a janela da equipe e esquecer. Só esta máquina (Host e Origin
//! em loopback), todo POST exige JSON, e só se abre uma origem que o app já lembra: nenhum corpo escolhe uma URL nova (o convite é o único caminho para isso).
use super::pedido::{Pedido, Resposta};
use super::Estado;
use crate::equipes;
use crate::mcp::permissao::so_loopback;
use serde_json::{json, Value};

fn erro(status: u16, codigo: &str, mensagem: String) -> Resposta {
    Resposta::json(status, &json!({"erro": mensagem, "codigo": codigo}))
}

pub fn rota(estado: &Estado, pedido: &Pedido) -> Resposta {
    if !so_loopback(&estado.mcp, pedido) {
        return erro(403, "so-local", crate::tr!("teams are only managed from this machine", "as equipes só se controlam nesta máquina"));
    }
    let pasta = super::config::pasta_do_app();
    match (pedido.metodo.as_str(), pedido.caminho.as_str()) {
        ("GET", "/api/equipes") => Resposta::json(200, &json!({"equipes": equipes::lista_json(&pasta)})).com("cache-control", "no-store"),
        ("POST", _) if !pedido.eh_json() => erro(415, "json", crate::tr!("send application/json", "envie application/json")),
        ("POST", "/api/equipes/abrir" | "/api/equipes/esquecer") => {
            let origem = serde_json::from_slice::<Value>(&pedido.corpo).ok().and_then(|v| v["origem"].as_str().map(String::from)).unwrap_or_default();
            if !equipes::conhecida(&pasta, &origem) {
                return erro(404, "equipe-desconhecida", crate::tr!("this team is not in the list", "esta equipe não está na lista"));
            }
            if pedido.caminho.ends_with("esquecer") {
                equipes::esquecer(&pasta, &origem);
                return Resposta::json(200, &json!({"equipes": equipes::lista_json(&pasta)}));
            }
            // a janela abre a equipe sem perguntar: o clique é do próprio usuário, na tela do app, e a origem já é conhecida
            match estado.ao_mostrar.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                Some(abrir) => {
                    abrir(Some(format!("{}{origem}", equipes::PREFIXO_LINK)));
                    Resposta::json(200, &json!({"ok": true}))
                }
                None => erro(404, "sem-janela", crate::tr!("there is no app window", "não há janela do app")),
            }
        }
        ("GET" | "POST", _) => Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado")),
        _ => Resposta::texto(405, "text/plain", crate::tr!("method not allowed", "método não permitido")),
    }
}
