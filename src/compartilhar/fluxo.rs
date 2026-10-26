//! O SSE pela porta dos convidados. O quick tunnel da Cloudflare (trycloudflare.com) guarda a resposta inteira e só a entrega quando ela
//! termina (medido em 2026-10-26: 41 eventos, um por segundo, chegaram todos juntos aos 42 s; no loopback, um a cada segundo). Por isso o
//! fluxo de quem chega por essa porta termina logo depois do primeiro evento (ou do batimento de 25 s): o túnel entrega na hora e o
//! `EventSource` reconecta sozinho (`retry:` do fluxo) mandando `Last-Event-ID`, e o servidor reenvia o que faltou. É uma consulta longa
//! feita pelo próprio SSE, sem mudar a página. Só vale na porta dos convidados; a porta local continua com o fluxo aberto.
use crate::http::pedido::{Corpo, Resposta};
use std::io::{self, Read};

/// Troca o corpo de um SSE pelo que termina no primeiro evento; outras respostas passam iguais.
pub fn curto(mut r: Resposta) -> Resposta {
    if let Corpo::Fluxo(dentro) = r.corpo {
        r.corpo = Corpo::Fluxo(Box::new(UmEvento { dentro, cauda: Vec::new(), evento: false, fim: false }));
    }
    r
}

pub struct UmEvento<R> {
    dentro: R,
    /// os últimos bytes já passados, para achar `data:` dividido entre duas leituras
    cauda: Vec<u8>,
    evento: bool,
    fim: bool,
}

fn contem(texto: &[u8], trecho: &[u8]) -> bool {
    texto.windows(trecho.len()).any(|j| j == trecho)
}

impl<R: Read> Read for UmEvento<R> {
    fn read(&mut self, saida: &mut [u8]) -> io::Result<usize> {
        if self.fim {
            return Ok(0);
        }
        let n = self.dentro.read(saida)?;
        let mut janela = std::mem::take(&mut self.cauda);
        janela.extend_from_slice(&saida[..n]);
        // a abertura (`retry:` e `: aberto`) não conta; um evento (`data:`) ou o batimento, sim
        self.evento |= contem(&janela, b"\ndata:") || janela.starts_with(b"data:") || contem(&janela, b": batimento");
        // termina na fronteira de um bloco: o que vier depois chega na próxima conexão (Last-Event-ID)
        self.fim = self.evento && janela.ends_with(b"\n\n");
        self.cauda = janela[janela.len().saturating_sub(16)..].to_vec();
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Como os fluxos de verdade: cada leitura devolve no máximo o resto do trecho atual (nunca junta dois trechos).
    struct Trechos(Vec<Vec<u8>>);
    impl Read for Trechos {
        fn read(&mut self, saida: &mut [u8]) -> io::Result<usize> {
            let Some(atual) = self.0.first_mut() else { return Ok(0) };
            let n = saida.len().min(atual.len());
            saida[..n].copy_from_slice(&atual[..n]);
            atual.drain(..n);
            if atual.is_empty() {
                self.0.remove(0);
            }
            Ok(n)
        }
    }

    fn tudo(trechos: &[&str]) -> String {
        let r = Trechos(trechos.iter().map(|t| t.as_bytes().to_vec()).collect());
        let mut leitor = UmEvento { dentro: r, cauda: Vec::new(), evento: false, fim: false };
        let mut s = Vec::new();
        let mut buf = [0u8; 7]; // leituras pequenas: o `data:` cai dividido entre duas
        loop {
            let n = leitor.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            s.extend_from_slice(&buf[..n]);
        }
        String::from_utf8(s).unwrap()
    }

    #[test]
    fn termina_depois_do_primeiro_evento() {
        let lote = "id: 5\nevent: mudancas\ndata: {\"a\":1}\n\nid: 6\nevent: mudancas\ndata: {}\n\n"; // um trecho com dois lotes vai inteiro
        assert_eq!(tudo(&["retry: 3000\n: aberto\n\n", lote, "id: 7\nevent: mudancas\ndata: {}\n\n"]), format!("retry: 3000\n: aberto\n\n{lote}"));
    }

    #[test]
    fn termina_no_batimento_e_passa_inteiro_o_que_termina_antes() {
        assert_eq!(tudo(&["retry: 3000\n: aberto\n\n", ": batimento\n\n", "event: x\ndata: 1\n\n"]), "retry: 3000\n: aberto\n\n: batimento\n\n");
        assert_eq!(tudo(&["retry: 3000\n: aberto\n\n"]), "retry: 3000\n: aberto\n\n", "fluxo que acaba antes de um evento acaba igual");
    }

    #[test]
    fn outras_respostas_passam_iguais() {
        let r = curto(Resposta::texto(200, "text/plain", "oi"));
        assert_eq!(r.bytes(), b"oi");
    }
}
