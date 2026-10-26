//! O atualizador: acha uma versão nova no GitHub, baixa o instalador em segundo plano, confere o SHA-256 e o deixa pronto; só instala quando o usuário
//! clica em Atualizar (na interface, na bandeja ou com `blueprint atualizar --instalar`). Nada baixado é executado sem esse clique.
//! O estado vive na memória e é reconstruído ao abrir o app (`ciclo::reconstruir`); a interface o recebe por GET /api/atualizacao e pelo SSE.
mod backup;
mod ciclo;
mod estado;
pub(crate) mod hash;
mod instalar;
mod preferencias;
pub(crate) mod rede;
mod release;
mod versao;

#[cfg(test)]
mod tests;

pub use backup::copiar_banco;
pub use estado::{Fase, Info};
pub use instalar::{lancar_instalador, Recusa};
pub use rede::Origem;
pub use release::Release;

use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// A versão deste executável. `BLUEPRINT_VERSAO_COMPILADA` na hora do `cargo build` (só testes: scripts/atualizacao-real-e2e.mjs) troca a do Cargo.toml.
pub const VERSAO: &str = match option_env!("BLUEPRINT_VERSAO_COMPILADA") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

type Ouvinte = Box<dyn Fn(&Value) + Send>;

struct Interno {
    atual: String,
    /// `%APPDATA%\Blueprint\atualizacao`: o instalador baixado, `pronta.json`, `verificacao.json` e a preferência
    pasta: PathBuf,
    origem: Origem,
    so_estaveis: bool,
    estado: Mutex<Info>,
    em_curso: AtomicBool,
    ouvintes: Mutex<Vec<Ouvinte>>,
}

/// Compartilhável entre threads (a verificação roda em segundo plano).
#[derive(Clone)]
pub struct Atualizador {
    i: Arc<Interno>,
}

impl Atualizador {
    pub fn novo(pasta: PathBuf, origem: Origem, atual: &str, so_estaveis: bool) -> Atualizador {
        Atualizador { i: Arc::new(Interno { atual: atual.to_string(), pasta, origem, so_estaveis, estado: Mutex::new(Info::default()), em_curso: AtomicBool::new(false), ouvintes: Mutex::new(Vec::new()) }) }
    }

    /// Do ambiente: a pasta fica ao lado do banco; `BLUEPRINT_ATUALIZACAO_URL` e `BLUEPRINT_ATUALIZACAO_SO_ESTAVEL=1` mudam a origem e o filtro.
    pub fn do_ambiente(banco: &std::path::Path) -> Atualizador {
        let pasta = banco.parent().filter(|p| !p.as_os_str().is_empty()).map_or_else(|| PathBuf::from("atualizacao"), |p| p.join("atualizacao"));
        let so_estaveis = std::env::var("BLUEPRINT_ATUALIZACAO_SO_ESTAVEL").as_deref() == Ok("1");
        // `BLUEPRINT_VERSAO_PARA_TESTE` (só testes): finge ser uma versão mais velha, para provar o ciclo contra uma release que já existe
        let atual = std::env::var("BLUEPRINT_VERSAO_PARA_TESTE").ok().filter(|v| versao::Versao::ler(v).is_some()).unwrap_or_else(|| VERSAO.to_string());
        Atualizador::novo(pasta, Origem::do_ambiente(), &atual, so_estaveis)
    }

    /// Quem quer saber de cada mudança de estado (o SSE da página, a bandeja).
    pub fn ouvir(&self, f: impl Fn(&Value) + Send + 'static) {
        self.i.ouvintes.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(f));
    }

    /// O estado como a interface o lê.
    pub fn json(&self) -> Value {
        self.i.estado.lock().unwrap_or_else(|e| e.into_inner()).json(&self.i.atual, self.automatica())
    }

    pub fn fase(&self) -> Fase {
        self.i.estado.lock().unwrap_or_else(|e| e.into_inner()).fase.clone()
    }

    pub fn versao_atual(&self) -> &str {
        &self.i.atual
    }

    /// A versão nova que está pronta para instalar, se houver.
    pub fn versao_pronta(&self) -> Option<String> {
        let e = self.i.estado.lock().unwrap_or_else(|e| e.into_inner());
        (e.fase == Fase::Pronta).then(|| e.versao.clone()).flatten()
    }

    /// Muda o estado e avisa os ouvintes (fora do cadeado do estado, para um ouvinte lento não o segurar).
    fn mudar(&self, f: impl FnOnce(&mut Info)) {
        let json = {
            let mut e = self.i.estado.lock().unwrap_or_else(|e| e.into_inner());
            f(&mut e);
            e.json(&self.i.atual, self.automatica())
        };
        for o in self.i.ouvintes.lock().unwrap_or_else(|e| e.into_inner()).iter() {
            o(&json);
        }
    }
}
