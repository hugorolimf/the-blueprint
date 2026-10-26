//! O fluxo SSE de `GET /api/eventos`: entrega, em ordem e sem juntar, o texto já formatado (`id:`, `event:`, `data:`) que `Sincronia` põe no
//! canal da conexão. Diferente do `FluxoSse` do MCP, que junta avisos repetidos: aqui cada lote importa. Batimento a cada 25 s, como o do MCP.
use super::Guarda;
use std::io::{self, Read};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

const BATIMENTO: Duration = Duration::from_secs(25);

pub struct Fluxo {
    rx: Receiver<String>,
    pendente: Vec<u8>,
    pos: usize,
    aberto: bool,
    _guarda: Guarda,
}

impl Fluxo {
    pub fn novo(rx: Receiver<String>, guarda: Guarda) -> Fluxo {
        Fluxo { rx, pendente: Vec::new(), pos: 0, aberto: false, _guarda: guarda }
    }

    fn proximo_trecho(&mut self) -> Option<Vec<u8>> {
        if !self.aberto {
            self.aberto = true;
            return Some(b"retry: 3000\n: aberto\n\n".to_vec());
        }
        match self.rx.recv_timeout(BATIMENTO) {
            Ok(primeiro) => {
                let mut texto = primeiro;
                while let Ok(mais) = self.rx.try_recv() {
                    texto.push_str(&mais);
                }
                Some(texto.into_bytes())
            }
            Err(RecvTimeoutError::Timeout) => Some(b": batimento\n\n".to_vec()),
            Err(RecvTimeoutError::Disconnected) => None,
        }
    }
}

impl Read for Fluxo {
    fn read(&mut self, saida: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.pendente.len() {
            match self.proximo_trecho() {
                Some(t) => (self.pendente, self.pos) = (t, 0),
                None => return Ok(0),
            }
        }
        let n = saida.len().min(self.pendente.len() - self.pos);
        saida[..n].copy_from_slice(&self.pendente[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}
