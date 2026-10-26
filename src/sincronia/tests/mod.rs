//! Testes da sincronia, um arquivo por módulo: a leitura do lote, o buffer com a retomada e a presença. As rotas ficam em src/http/tests.
mod buffer;
mod lote;
mod papeis;
mod presenca;

use std::sync::mpsc::Receiver;
use std::time::Duration;

/// Tudo o que chegou no canal até agora (sem esperar).
pub fn recebido(rx: &Receiver<String>) -> String {
    let mut s = String::new();
    while let Ok(t) = rx.recv_timeout(Duration::from_millis(5)) {
        s.push_str(&t);
    }
    s
}
