//! SSE: a página abre uma conexão e recebe um aviso a cada novidade (item na fila, mudança na presença dos agentes), em vez de perguntar
//! a cada 2,5 s. /api/mcp/fila e /api/mcp/agentes continuam sendo o contrato (e o plano B da página se a conexão cair).
//! Cada conexão ocupa uma thread, que dorme até haver aviso.
use std::io::{self, Read};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use serde_json::Value;

#[derive(Clone, PartialEq)]
pub struct Evento {
    pub nome: &'static str,
    pub dados: Option<Value>,
}

/// Aviso enviado pela página apenas depois de salvar: nomes e ids validados, nenhum texto livre no SSE.
pub(super) fn aviso(mcp: &super::Mcp, dados: &Value, corpo: &Value) -> Result<Value, crate::http::pedido::ErroHttp> {
    use crate::http::pedido::falha;
    let texto = |chave: &str| corpo[chave].as_str().unwrap_or("");
    let nome = match texto("tipo") {
        "proposta-aplicada" => "proposta-aplicada",
        "checkpoint-marcado" => "checkpoint-marcado",
        _ => return Err(falha(400, "tipo de aviso inválido")),
    };
    let (ws, bp) = (texto("workspaceId"), texto("blueprintId"));
    let estrutura = super::acoes::blueprint(dados, ws, bp, None)?;
    if nome == "checkpoint-marcado" {
        let acoes = estrutura["blueprint"]["acoes"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let existe = acoes.iter().flat_map(|a| a["checkpoints"].as_array().map(Vec::as_slice).unwrap_or(&[])).any(|c| c["id"] == texto("checkpointId") && c["feito"] == true);
        if !existe { return Err(falha(400, "checkpoint não está marcado no banco")); }
    }
    mcp.notificar(nome, Some(serde_json::json!({"workspaceId": ws, "blueprintId": bp})));
    Ok(serde_json::json!({"ok": true}))
}

/// De quanto em quanto tempo mandar um comentário, para o Windows e os proxies não derrubarem a conexão ociosa.
const BATIMENTO: Duration = Duration::from_secs(25);

pub struct FluxoSse {
    rx: Receiver<Evento>,
    pendente: Vec<u8>,
    pos: usize,
    aberto: bool,
}

impl FluxoSse {
    pub fn novo(rx: Receiver<Evento>) -> FluxoSse {
        FluxoSse { rx, pendente: Vec::new(), pos: 0, aberto: false }
    }

    fn proximo_trecho(&mut self) -> Option<Vec<u8>> {
        if !self.aberto {
            self.aberto = true;
            return Some(b"retry: 3000\n: aberto\n\n".to_vec());
        }
        match self.rx.recv_timeout(BATIMENTO) {
            Ok(primeiro) => {
                // vários avisos seguidos do mesmo tipo viram um só: a página busca a lista inteira
                let mut nomes = vec![primeiro];
                while let Ok(n) = self.rx.try_recv() {
                    if !nomes.contains(&n) {
                        nomes.push(n);
                    }
                }
                Some(nomes.iter().map(|n| format!("event: {}\ndata: {}\n\n", n.nome, n.dados.as_ref().map_or_else(|| "nova".into(), Value::to_string))).collect::<String>().into_bytes())
            }
            Err(RecvTimeoutError::Timeout) => Some(b": batimento\n\n".to_vec()),
            Err(RecvTimeoutError::Disconnected) => None,
        }
    }
}

impl Read for FluxoSse {
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
