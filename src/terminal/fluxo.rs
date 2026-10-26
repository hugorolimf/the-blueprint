//! A saída de uma sessão como SSE: primeiro o histórico (evento `historico`: a página limpa a tela e o desenha), depois cada pedaço novo
//! (evento padrão) e, quando o processo sai, `fim` com o código. O texto vai como string JSON (o ESC vira \u001b). A thread da conexão dorme
//! até haver saída; pedaços que chegam juntos viram um evento só.
use super::Msg;
use serde_json::Value;
use std::io::{self, Read};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

/// Comentário periódico para o Windows e os proxies não derrubarem a conexão ociosa (o mesmo do SSE do MCP).
const BATIMENTO: Duration = Duration::from_secs(25);

fn evento(nome: Option<&str>, dados: &Value) -> String {
    nome.map_or(String::new(), |n| format!("event: {n}\n")) + &format!("data: {dados}\n\n")
}

fn fim(codigo: Option<u32>) -> String {
    evento(Some("fim"), &serde_json::json!({ "codigo": codigo }))
}

pub struct FluxoTerminal {
    rx: Option<Receiver<Msg>>,
    pendente: Vec<u8>,
    pos: usize,
}

impl FluxoTerminal {
    pub fn novo(historico: String, rx: Option<Receiver<Msg>>, codigo: Option<u32>) -> FluxoTerminal {
        let mut primeiro = String::from("retry: 3000\n") + &evento(Some("historico"), &Value::String(historico));
        if rx.is_none() {
            primeiro += &fim(codigo);
        }
        FluxoTerminal { rx, pendente: primeiro.into_bytes(), pos: 0 }
    }

    fn proximo(&mut self) -> Option<Vec<u8>> {
        let rx = self.rx.as_ref()?;
        let primeiro = match rx.recv_timeout(BATIMENTO) {
            Ok(m) => m,
            Err(RecvTimeoutError::Timeout) => return Some(b": batimento\n\n".to_vec()),
            Err(RecvTimeoutError::Disconnected) => return None, // desligado por lentidão: a página reconecta e recebe o histórico
        };
        let (mut texto, mut acabou) = (String::new(), None);
        for m in std::iter::once(primeiro).chain(std::iter::from_fn(|| rx.try_recv().ok()).take(64)) {
            match m {
                Msg::Dados(t) => texto.push_str(&t),
                Msg::Fim(c) => acabou = Some(c),
            }
        }
        let mut saida = if texto.is_empty() { String::new() } else { evento(None, &Value::String(texto)) };
        if let Some(c) = acabou {
            saida += &fim(c);
            self.rx = None;
        }
        Some(saida.into_bytes())
    }
}

impl Read for FluxoTerminal {
    fn read(&mut self, saida: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.pendente.len() {
            match self.proximo() {
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
