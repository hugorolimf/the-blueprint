//! O pedido e a resposta HTTP em tipos próprios, sem depender do servidor: as rotas viram funções puras e testáveis.
use serde_json::Value;
use std::io::Read;

pub struct Pedido {
    pub metodo: String,
    /// só o caminho, sem a consulta
    pub caminho: String,
    pub consulta: String,
    pub cabecalhos: Vec<(String, String)>,
    pub corpo: Vec<u8>,
}

impl Pedido {
    pub fn novo(metodo: &str, url: &str, cabecalhos: Vec<(String, String)>, corpo: Vec<u8>) -> Pedido {
        let (caminho, consulta) = url.split_once('?').unwrap_or((url, ""));
        Pedido { metodo: metodo.to_uppercase(), caminho: caminho.to_string(), consulta: consulta.to_string(), cabecalhos, corpo }
    }

    pub fn cabecalho(&self, nome: &str) -> Option<&str> {
        self.cabecalhos.iter().find(|(k, _)| k.eq_ignore_ascii_case(nome)).map(|(_, v)| v.as_str())
    }

    /// `URLSearchParams.get`: decodifica %xx e `+`.
    pub fn parametro(&self, nome: &str) -> Option<String> {
        self.consulta.split('&').filter_map(|par| par.split_once('=').or(Some((par, "")))).find(|(k, _)| decodificar(k) == nome).map(|(_, v)| decodificar(v))
    }

    pub fn eh_json(&self) -> bool {
        self.cabecalho("content-type").is_some_and(|t| t.starts_with("application/json"))
    }
}

fn hex(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

fn decodificar(s: &str) -> String {
    let b = s.as_bytes();
    let (mut saida, mut i) = (Vec::with_capacity(b.len()), 0);
    while i < b.len() {
        match b[i] {
            b'+' => saida.push(b' '),
            b'%' if i + 2 < b.len() && hex(b[i + 1]).is_some() && hex(b[i + 2]).is_some() => {
                saida.push(hex(b[i + 1]).unwrap_or(0) * 16 + hex(b[i + 2]).unwrap_or(0));
                i += 2;
            }
            c => saida.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&saida).into_owned()
}

pub enum Corpo {
    Bytes(Vec<u8>),
    /// resposta que continua chegando (SSE); termina quando o leitor devolve 0 bytes
    Fluxo(Box<dyn Read + Send>),
}

pub struct Resposta {
    pub status: u16,
    pub cabecalhos: Vec<(String, String)>,
    pub corpo: Corpo,
}

impl Resposta {
    /// Como o `enviar` do servidor Node: tipo com charset utf-8 e sem cache.
    pub fn texto(status: u16, tipo: &str, corpo: impl Into<Vec<u8>>) -> Resposta {
        Resposta {
            status,
            cabecalhos: vec![("content-type".into(), format!("{tipo}; charset=utf-8")), ("cache-control".into(), "no-store".into())],
            corpo: Corpo::Bytes(corpo.into()),
        }
    }

    pub fn json(status: u16, valor: &Value) -> Resposta {
        Resposta::texto(status, "application/json", serde_json::to_vec(valor).unwrap_or_else(|_| b"{}".to_vec()))
    }

    pub fn com(mut self, nome: &str, valor: &str) -> Resposta {
        self.cabecalhos.retain(|(k, _)| !k.eq_ignore_ascii_case(nome));
        self.cabecalhos.push((nome.to_string(), valor.to_string()));
        self
    }

    pub fn bytes(&self) -> &[u8] {
        match &self.corpo {
            Corpo::Bytes(b) => b,
            Corpo::Fluxo(_) => &[],
        }
    }

    pub fn cabecalho(&self, nome: &str) -> Option<&str> {
        self.cabecalhos.iter().find(|(k, _)| k.eq_ignore_ascii_case(nome)).map(|(_, v)| v.as_str())
    }
}

/// Erro de uma rota com o status que ela devolve (o `falha(status, erro)` do mcp.mjs).
#[derive(Debug)]
pub struct ErroHttp {
    pub status: u16,
    pub mensagem: String,
}

pub fn falha(status: u16, mensagem: impl Into<String>) -> ErroHttp {
    ErroHttp { status, mensagem: mensagem.into() }
}

impl From<crate::erro::Erro> for ErroHttp {
    fn from(e: crate::erro::Erro) -> Self {
        use crate::erro::Erro;
        match e {
            Erro::Formato(m) => falha(400, m),
            Erro::NaoEncontrado(m) => falha(404, m),
            Erro::Proibido(m) => falha(403, m),
            outro => falha(500, outro.to_string()),
        }
    }
}
