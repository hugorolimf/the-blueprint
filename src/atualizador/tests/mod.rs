//! Testes do atualizador. O GitHub é imitado por um servidor local (`Falso`); o download usa o curl.exe de verdade (como no app).
mod backup;
mod ciclo;
mod hash;
mod instalar;
mod release;
mod versao;

use super::{Atualizador, Origem};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tiny_http::{Response, Server};

/// Um servidor HTTP local que serve o que for registrado com `servir` e conta os pedidos.
pub struct Falso {
    pub base: String,
    pub pedidos: Arc<AtomicUsize>,
    rotas: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    servidor: Arc<Server>,
}

impl Falso {
    pub fn novo() -> Falso {
        let servidor = Arc::new(Server::http("127.0.0.1:0").unwrap());
        let base = format!("http://127.0.0.1:{}", servidor.server_addr().to_ip().unwrap().port());
        let (rotas, pedidos) = (Arc::new(Mutex::new(HashMap::<String, Vec<u8>>::new())), Arc::new(AtomicUsize::new(0)));
        let (s, r, p) = (servidor.clone(), rotas.clone(), pedidos.clone());
        std::thread::spawn(move || {
            for req in s.incoming_requests() {
                p.fetch_add(1, Ordering::SeqCst);
                let caminho = req.url().split('?').next().unwrap_or("").to_string();
                let corpo = r.lock().unwrap().get(&caminho).cloned();
                let _ = match corpo {
                    Some(c) => req.respond(Response::from_data(c)),
                    None => req.respond(Response::from_string("nada aqui").with_status_code(404)),
                };
            }
        });
        Falso { base, pedidos, rotas, servidor }
    }

    pub fn servir(&self, caminho: &str, corpo: impl Into<Vec<u8>>) {
        self.rotas.lock().unwrap().insert(caminho.to_string(), corpo.into());
    }

    pub fn origem(&self) -> Origem {
        Origem { api: self.base.clone(), teste: true }
    }

    /// Publica uma release `versao` com o instalador `conteudo` e o SHA256SUMS (`hash_no_sums`: o que vai escrito nele).
    pub fn publicar(&self, versao: &str, conteudo: &[u8], hash_no_sums: Option<&str>) {
        let nome = format!("Blueprint-{versao}-instalador.exe");
        let hash = hash_no_sums.map_or_else(|| sha256(conteudo), String::from);
        self.servir(&format!("/baixar/{nome}"), conteudo);
        self.servir("/baixar/SHA256SUMS", format!("{hash}  {nome}\n"));
        let release = json!([{"tag_name": format!("v{versao}"), "draft": false, "prerelease": true, "body": "Notas da versão", "assets": [
            {"name": nome, "size": conteudo.len(), "browser_download_url": format!("{}/baixar/{nome}", self.base)},
            {"name": "SHA256SUMS", "size": 90, "browser_download_url": format!("{}/baixar/SHA256SUMS", self.base)}]}]);
        self.servir("/releases", release.to_string());
    }
}

impl Drop for Falso {
    fn drop(&mut self) {
        self.servidor.unblock();
    }
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Uma pasta de teste vazia e só desta execução.
pub fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-atualizador-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

/// Um atualizador "instalado na 0.1.0" falando com o servidor falso.
pub fn atualizador(falso: &Falso, pasta: PathBuf) -> Atualizador {
    Atualizador::novo(pasta, falso.origem(), "0.1.0", false)
}
